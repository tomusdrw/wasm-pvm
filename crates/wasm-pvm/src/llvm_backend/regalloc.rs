// Linear-scan register allocator for the PVM backend.
//
// Allocates long-lived SSA values to physical registers so they persist across
// basic block boundaries — particularly loop back-edges where the per-block
// register cache is cleared.
//
// Allocatable registers:
//   - `BASE_ALLOCATABLE_REGS` is currently empty: we intentionally avoid global
//     allocation of r5/r6 (`abi::SCRATCH1`/`SCRATCH2`) because several lowering
//     paths reuse them as scratch registers.
//   - r9-r12 (`abi::FIRST_LOCAL_REG`..+4) are available beyond parameter count
//     in both leaf and non-leaf functions. Non-leaf functions invalidate
//     allocated callee regs after calls since call argument setup reuses r9-r12.
//
// The allocator operates on LLVM IR (before PVM lowering) and produces a
// mapping from `ValKey` → physical register. The emitter then uses this mapping
// in `load_operand`/`store_to_slot` to avoid redundant memory traffic.
//
// Functions without loops are skipped entirely — the per-block register cache
// already handles within-block forwarding, and adding spill/reload around calls
// would be a net negative.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use std::collections::{BTreeSet, HashMap};

use inkwell::values::{FunctionValue, PhiValue};

use super::emitter::{ValKey, val_key_basic, val_key_instr};
use super::successors::collect_successors;

/// Base registers available for allocation.
///
/// We currently avoid allocating r5/r6 globally because they are reused as
/// scratch registers by several lowering paths. Callee-saved allocation
/// (r9-r12, when available) is configured below.
const BASE_ALLOCATABLE_REGS: &[u8] = &[];
/// Minimum dynamic use count required before a value is considered for allocation.
const MIN_USES_FOR_ALLOCATION: usize = 3;

/// A live interval for an SSA value.
#[derive(Debug, Clone)]
struct LiveInterval {
    val_key: ValKey,
    slot: i32,
    /// Start position (linearized instruction index).
    start: usize,
    /// End position (last use, inclusive).
    end: usize,
    /// Number of uses (for spill weight heuristic).
    use_count: usize,
}

/// Result of register allocation for one function.
#[derive(Debug, Clone, Default)]
pub struct RegAllocResult {
    /// For each `ValKey`, the allocated physical register.
    pub val_to_reg: HashMap<ValKey, u8>,
    /// For each stack slot offset, the allocated physical register.
    pub slot_to_reg: HashMap<i32, u8>,
    /// Reverse: physical register → stack slot offset (for spill/reload).
    pub reg_to_slot: HashMap<u8, i32>,
    /// Instrumentation stats for this function's allocation run.
    pub stats: RegAllocStats,
}

/// Instrumentation counters for register allocation.
#[derive(Debug, Clone, Default)]
pub struct RegAllocStats {
    /// Count of stack-slotted values seen by regalloc.
    pub total_values: usize,
    /// Number of candidate live intervals after filtering.
    pub total_intervals: usize,
    /// Whether a back-edge loop was detected.
    pub has_loops: bool,
    /// Number of physical registers made available to linear scan.
    pub allocatable_regs: usize,
    /// Maximum number of direct call arguments used by this function.
    pub max_call_args: usize,
    /// Number of values that received a final register assignment.
    pub allocated_values: usize,
    /// Why allocation was skipped, if applicable.
    pub skipped_reason: Option<&'static str>,
}

/// Run register allocation for a function.
///
/// `value_slots` maps `ValKey` → stack slot offset (from the pre-scan bump allocator).
/// `num_params` is the number of function parameters (for determining available callee-saved regs).
pub fn run(
    function: FunctionValue<'_>,
    value_slots: &HashMap<ValKey, i32>,
    is_leaf: bool,
    num_params: usize,
) -> RegAllocResult {
    let fn_name = function.get_name().to_string_lossy().to_string();
    let mut stats = RegAllocStats {
        total_values: value_slots.len(),
        ..RegAllocStats::default()
    };

    let blocks = function.get_basic_blocks();
    if blocks.is_empty() {
        stats.skipped_reason = Some("no_blocks");
        tracing::debug!(
            target: "wasm_pvm::regalloc",
            function = %fn_name,
            is_leaf,
            num_params,
            total_values = stats.total_values,
            skipped_reason = stats.skipped_reason,
            "regalloc skipped"
        );
        return RegAllocResult {
            stats,
            ..RegAllocResult::default()
        };
    }

    // Phase 1: Linearize instructions and compute block index ranges.
    let (instr_index, block_ranges) = linearize(&blocks);
    let max_call_args = max_call_args(function);
    stats.max_call_args = max_call_args;

    // Phase 2: Compute live intervals + detect loops.
    let (intervals, has_loops) =
        compute_live_intervals(&blocks, &instr_index, &block_ranges, value_slots);
    stats.has_loops = has_loops;
    stats.total_intervals = intervals.len();

    // Skip allocation for functions without loops — the per-block register cache
    // already handles within-block forwarding, and spill/reload around calls
    // would be a net cost.
    if !has_loops {
        stats.skipped_reason = Some("no_loops");
        tracing::debug!(
            target: "wasm_pvm::regalloc",
            function = %fn_name,
            is_leaf,
            num_params,
            total_values = stats.total_values,
            total_intervals = stats.total_intervals,
            has_loops = stats.has_loops,
            skipped_reason = stats.skipped_reason,
            "regalloc skipped"
        );
        return RegAllocResult {
            stats,
            ..RegAllocResult::default()
        };
    }
    if intervals.is_empty() {
        stats.skipped_reason = Some("no_candidate_intervals");
        tracing::debug!(
            target: "wasm_pvm::regalloc",
            function = %fn_name,
            is_leaf,
            num_params,
            total_values = stats.total_values,
            total_intervals = stats.total_intervals,
            has_loops = stats.has_loops,
            skipped_reason = stats.skipped_reason,
            "regalloc skipped"
        );
        return RegAllocResult {
            stats,
            ..RegAllocResult::default()
        };
    }

    // Phase 3: Build the allocatable register set.
    let mut allocatable_regs: Vec<u8> = BASE_ALLOCATABLE_REGS.to_vec();

    // Add callee-saved registers (r9-r12) beyond parameter count.
    // For non-leaf functions, reserve outgoing argument registers (r9..)
    // based on the function's max call arity, since call setup writes them.
    let first_alloc_idx = if is_leaf {
        num_params
    } else {
        num_params.max(max_call_args.min(crate::abi::MAX_LOCAL_REGS))
    };
    for i in first_alloc_idx..crate::abi::MAX_LOCAL_REGS {
        allocatable_regs.push(crate::abi::FIRST_LOCAL_REG + i as u8);
    }
    stats.allocatable_regs = allocatable_regs.len();

    // Phase 4: Linear scan allocation.
    let mut result = linear_scan(intervals, &allocatable_regs);
    stats.allocated_values = result.val_to_reg.len();
    result.stats = stats;

    tracing::debug!(
        target: "wasm_pvm::regalloc",
        function = %fn_name,
        is_leaf,
        num_params,
        total_values = result.stats.total_values,
        total_intervals = result.stats.total_intervals,
        has_loops = result.stats.has_loops,
        allocatable_regs = result.stats.allocatable_regs,
        max_call_args = result.stats.max_call_args,
        allocated_values = result.stats.allocated_values,
        "regalloc completed"
    );

    result
}

/// Maps each LLVM instruction to a linearized index.
/// Also returns the (start, end) index range for each basic block.
fn linearize<'ctx>(
    blocks: &[inkwell::basic_block::BasicBlock<'ctx>],
) -> (
    HashMap<ValKey, usize>,
    HashMap<inkwell::basic_block::BasicBlock<'ctx>, (usize, usize)>,
) {
    let mut instr_index = HashMap::new();
    let mut block_ranges = HashMap::new();
    let mut idx = 0usize;

    for &bb in blocks {
        let start = idx;
        for instr in bb.get_instructions() {
            let key = val_key_instr(instr);
            instr_index.insert(key, idx);
            idx += 1;
        }
        let end = if idx > start { idx - 1 } else { start };
        block_ranges.insert(bb, (start, end));
    }

    (instr_index, block_ranges)
}

/// Returns the maximum direct call argument count used in this function.
fn max_call_args(function: FunctionValue<'_>) -> usize {
    let mut max_args = 0usize;
    for bb in function.get_basic_blocks() {
        for instr in bb.get_instructions() {
            if instr.get_opcode() == inkwell::values::InstructionOpcode::Call {
                // LLVM call operands include the callee as the final operand.
                let num_args = instr.get_num_operands().saturating_sub(1) as usize;
                if num_args > max_args {
                    max_args = num_args;
                }
            }
        }
    }
    max_args
}

/// Compute live intervals for all SSA values (parameters and instruction results).
/// Also returns whether any loops were detected (back-edges exist).
fn compute_live_intervals<'ctx>(
    blocks: &[inkwell::basic_block::BasicBlock<'ctx>],
    instr_index: &HashMap<ValKey, usize>,
    block_ranges: &HashMap<inkwell::basic_block::BasicBlock<'ctx>, (usize, usize)>,
    value_slots: &HashMap<ValKey, i32>,
) -> (Vec<LiveInterval>, bool) {
    use inkwell::values::InstructionOpcode;

    let mut def_point: HashMap<ValKey, usize> = HashMap::new();
    let mut last_use: HashMap<ValKey, usize> = HashMap::new();
    let mut use_count: HashMap<ValKey, usize> = HashMap::new();

    // Collect block index mapping for back-edge detection.
    let block_order: HashMap<inkwell::basic_block::BasicBlock<'_>, usize> =
        blocks.iter().enumerate().map(|(i, &bb)| (bb, i)).collect();

    // Detect loop back-edges: successor has a lower block index than the source.
    let mut loop_headers: HashMap<inkwell::basic_block::BasicBlock<'_>, usize> = HashMap::new();
    for &bb in blocks {
        if let Some(term) = bb.get_terminator() {
            let successors = collect_successors(term);
            let bb_idx = block_order[&bb];
            let (_, bb_end) = block_ranges[&bb];
            for succ in successors {
                if let Some(&succ_idx) = block_order.get(&succ)
                    && succ_idx <= bb_idx
                {
                    // Back-edge: bb -> succ (succ is a loop header).
                    let entry = loop_headers.entry(succ).or_insert(0);
                    *entry = (*entry).max(bb_end);
                }
            }
        }
    }

    let has_loops = !loop_headers.is_empty();

    // Walk all instructions to find defs and uses.
    for &bb in blocks {
        for instr in bb.get_instructions() {
            let instr_key = val_key_instr(instr);
            let instr_idx = instr_index[&instr_key];

            // This instruction defines instr_key (if it produces a value).
            if value_slots.contains_key(&instr_key) {
                def_point.entry(instr_key).or_insert(instr_idx);
            }

            // Check all operands for uses.
            let num_ops = instr.get_num_operands();
            match instr.get_opcode() {
                InstructionOpcode::Phi => {
                    // Phi node: each incoming value is "used" at the end of the
                    // corresponding predecessor block, not at the phi itself.
                    if let Ok(phi) = TryInto::<PhiValue<'_>>::try_into(instr) {
                        let num_incomings = phi.count_incoming();
                        for i in 0..num_incomings {
                            if let Some((val, pred_bb)) = phi.get_incoming(i) {
                                let vk = val_key_basic(val);
                                if value_slots.contains_key(&vk) {
                                    let (_, pred_end) = block_ranges[&pred_bb];
                                    update_use(&mut last_use, &mut use_count, vk, pred_end);
                                }
                            }
                        }
                    }
                }
                _ => {
                    for i in 0..num_ops {
                        if let Some(inkwell::values::Operand::Value(val)) = instr.get_operand(i) {
                            let vk = val_key_basic(val);
                            if value_slots.contains_key(&vk) {
                                update_use(&mut last_use, &mut use_count, vk, instr_idx);
                            }
                        }
                    }
                }
            }
        }
    }

    // Parameters defined at position 0 (before first instruction).
    for &vk in value_slots.keys() {
        def_point.entry(vk).or_insert(0);
    }

    // Build intervals, extending for loops.
    let mut intervals = Vec::new();
    for (&vk, &slot) in value_slots {
        let start = def_point.get(&vk).copied().unwrap_or(0);
        let mut end = last_use.get(&vk).copied().unwrap_or(start);
        let uses = use_count.get(&vk).copied().unwrap_or(0);

        // Skip low-use values — not worth allocating a register.
        if uses < MIN_USES_FOR_ALLOCATION {
            continue;
        }

        // Values defined inside loop bodies are typically updated every
        // iteration. With write-through slots this tends to add move traffic
        // without enough load savings, so we only allocate loop-carried values
        // that originate before the loop.
        let mut defined_in_loop = false;
        for (&header_bb, &back_edge_end) in &loop_headers {
            let (header_start, _) = block_ranges[&header_bb];
            if start >= header_start && start <= back_edge_end {
                defined_in_loop = true;
                break;
            }
        }
        if defined_in_loop {
            continue;
        }

        // Loop extension: if this value is live at a loop header and the loop's
        // back-edge source is beyond the current end, extend the range.
        for (&header_bb, &back_edge_end) in &loop_headers {
            let (header_start, _) = block_ranges[&header_bb];
            if start <= header_start && end >= header_start {
                end = end.max(back_edge_end);
            }
            if start >= header_start && start <= back_edge_end && end >= header_start {
                end = end.max(back_edge_end);
            }
        }

        intervals.push(LiveInterval {
            val_key: vk,
            slot,
            start,
            end,
            use_count: uses,
        });
    }

    (intervals, has_loops)
}

fn update_use(
    last_use: &mut HashMap<ValKey, usize>,
    use_count: &mut HashMap<ValKey, usize>,
    vk: ValKey,
    idx: usize,
) {
    let entry = last_use.entry(vk).or_insert(0);
    *entry = (*entry).max(idx);
    *use_count.entry(vk).or_insert(0) += 1;
}

/// Standard linear-scan register allocation.
fn linear_scan(mut intervals: Vec<LiveInterval>, allocatable_regs: &[u8]) -> RegAllocResult {
    // Sort by start point (ascending), then by use_count descending (prefer allocating
    // heavily-used values when two intervals start at the same point).
    intervals.sort_by(|a, b| {
        a.start
            .cmp(&b.start)
            .then_with(|| b.use_count.cmp(&a.use_count))
    });

    let mut result = RegAllocResult::default();

    // Active intervals sorted by end point (using BTreeSet of (end, index)).
    let mut active: BTreeSet<(usize, usize)> = BTreeSet::new();
    let mut free_regs: Vec<u8> = allocatable_regs.to_vec();
    // Register assignment for currently active intervals.
    let mut active_assigned: HashMap<usize, u8> = HashMap::new();
    // Final register assignment for intervals that were allocated and never evicted.
    // Naturally expired intervals stay here so their earlier uses can still benefit.
    let mut final_assigned: HashMap<usize, u8> = HashMap::new();

    for (i, interval) in intervals.iter().enumerate() {
        // Expire old intervals.
        let expired: Vec<_> = active
            .iter()
            .take_while(|(end, _)| *end < interval.start)
            .copied()
            .collect();
        for (end, idx) in expired {
            active.remove(&(end, idx));
            if let Some(reg) = active_assigned.remove(&idx) {
                free_regs.push(reg);
            }
        }

        if let Some(reg) = free_regs.pop() {
            active_assigned.insert(i, reg);
            final_assigned.insert(i, reg);
            active.insert((interval.end, i));
        } else if let Some(&(furthest_end, furthest_idx)) = active.iter().next_back()
            && furthest_end > interval.end
        {
            // Evict the interval with the furthest end, give its register to us.
            let reg = active_assigned
                .remove(&furthest_idx)
                .expect("active interval must have an assigned register");
            active.remove(&(furthest_end, furthest_idx));
            // Evicted interval no longer has a stable whole-interval assignment.
            final_assigned.remove(&furthest_idx);

            active_assigned.insert(i, reg);
            final_assigned.insert(i, reg);
            active.insert((interval.end, i));
        }
        // else: no free register and current interval ends further — spill it.
    }

    // Build result from all non-evicted assignments (single pass).
    for (&idx, &reg) in &final_assigned {
        let interval = &intervals[idx];
        result.val_to_reg.insert(interval.val_key, reg);
        result.slot_to_reg.insert(interval.slot, reg);
        result.reg_to_slot.insert(reg, interval.slot);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_scan_keeps_non_overlapping_assignments() {
        let intervals = vec![
            LiveInterval {
                val_key: ValKey(1),
                slot: 8,
                start: 0,
                end: 1,
                use_count: 3,
            },
            LiveInterval {
                val_key: ValKey(2),
                slot: 16,
                start: 2,
                end: 3,
                use_count: 3,
            },
        ];

        let result = linear_scan(intervals, &[9]);

        assert_eq!(result.val_to_reg.get(&ValKey(1)), Some(&9));
        assert_eq!(result.val_to_reg.get(&ValKey(2)), Some(&9));
        assert_eq!(result.slot_to_reg.get(&8), Some(&9));
        assert_eq!(result.slot_to_reg.get(&16), Some(&9));
        assert!(result.reg_to_slot.contains_key(&9));
    }

    #[test]
    fn linear_scan_drops_evicted_interval_assignment() {
        let intervals = vec![
            LiveInterval {
                val_key: ValKey(1),
                slot: 8,
                start: 0,
                end: 10,
                use_count: 3,
            },
            LiveInterval {
                val_key: ValKey(2),
                slot: 16,
                start: 1,
                end: 4,
                use_count: 3,
            },
        ];

        let result = linear_scan(intervals, &[9]);

        assert!(!result.val_to_reg.contains_key(&ValKey(1)));
        assert_eq!(result.val_to_reg.get(&ValKey(2)), Some(&9));
    }
}
