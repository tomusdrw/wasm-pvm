// Linear-scan register allocator for the PVM backend.
//
// Allocates SSA values to physical registers so they persist across basic
// block boundaries. Benefits both loop-heavy code (avoiding cache clears at
// back-edges) and straight-line code (reducing LoadIndU64 traffic).
//
// Allocatable registers:
//   - r5/r6 (`abi::SCRATCH1`/`SCRATCH2`) are available in all functions whose
//     LLVM IR contains no operations that clobber them (bulk memory ops,
//     non-rotation funnel shifts). Detected via `scratch_regs_safe` parameter.
//     In non-leaf functions, these are caller-saved and spilled/reloaded around
//     calls automatically via spill_allocated_regs + clear_reg_cache.
//   - r7/r8 (`RETURN_VALUE_REG`/`ARGS_LEN_REG`) are available in all functions.
//     In non-leaf functions, they are caller-saved and handled the same way as
//     r5/r6. Lowering paths that use them as scratch trigger `invalidate_reg`
//     on emit, forcing lazy reload from the stack slot.
//   - r9-r12 (`abi::FIRST_LOCAL_REG`..+4) are available beyond parameter count
//     in both leaf and non-leaf functions. Non-leaf functions invalidate
//     allocated callee regs after calls since call argument setup reuses r9-r12.
//
// The allocator operates on LLVM IR (before PVM lowering) and produces a
// mapping from `ValKey` → physical register. The emitter then uses this mapping
// in `load_operand`/`store_to_slot` to avoid redundant memory traffic.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use std::collections::{BTreeSet, HashMap};

use inkwell::values::{FunctionValue, PhiValue};

use super::emitter::{ValKey, val_key_basic, val_key_instr};
use super::successors::collect_successors;

/// Base registers always available for allocation (empty — all allocatable
/// registers are added conditionally in `run()` based on leaf/scratch analysis).
const BASE_ALLOCATABLE_REGS: &[u8] = &[];
/// Minimum dynamic use count required before a value is considered for allocation.
const MIN_USES_FOR_ALLOCATION: usize = 2;
/// Lower threshold when aggressive register allocation is enabled.
const MIN_USES_FOR_ALLOCATION_AGGRESSIVE: usize = 1;

/// A live interval for an SSA value.
#[derive(Debug, Clone)]
struct LiveInterval {
    val_key: ValKey,
    slot: i32,
    /// Start position (linearized instruction index).
    start: usize,
    /// End position (last use, inclusive).
    end: usize,
    /// Spill weight: sum of loop-depth-weighted uses. Higher weight → more
    /// expensive to spill (the value is used frequently in hot code).
    spill_weight: f64,
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
/// `aggressive` lowers the minimum-use threshold from 3 to 2, capturing more candidates.
/// `scratch_regs_safe` indicates that this function never clobbers r5/r6
/// (`abi::SCRATCH1`/`SCRATCH2`), making them available for allocation.
/// `allocate_caller_saved` enables r7/r8 allocation in leaf functions.
pub fn run(
    function: FunctionValue<'_>,
    value_slots: &HashMap<ValKey, i32>,
    is_leaf: bool,
    num_params: usize,
    aggressive: bool,
    scratch_regs_safe: bool,
    allocate_caller_saved: bool,
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

    // Phase 1b: Detect loop headers (used by both live interval computation
    // and the calls-in-loops heuristic).
    let loop_headers = detect_loop_headers(&blocks, &block_ranges);

    // Phase 1c: Compute per-position loop nesting depth for spill weight.
    let max_position = instr_index.values().copied().max().unwrap_or(0);
    let loop_depths = compute_loop_depths(&block_ranges, &loop_headers, max_position);

    // Phase 2: Compute live intervals + detect loops.
    let min_uses = if aggressive {
        MIN_USES_FOR_ALLOCATION_AGGRESSIVE
    } else {
        MIN_USES_FOR_ALLOCATION
    };
    let (intervals, has_loops) = compute_live_intervals(
        &blocks,
        &instr_index,
        &block_ranges,
        value_slots,
        &loop_headers,
        &loop_depths,
        min_uses,
    );
    stats.has_loops = has_loops;
    stats.total_intervals = intervals.len();

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

    // Add r5/r6 (abi::SCRATCH1/SCRATCH2) when the function doesn't clobber them.
    // In leaf functions, these are always available.
    // In non-leaf functions, they are caller-saved and clobbered by calls.
    // The call lowering (spill_allocated_regs + clear_reg_cache) handles
    // spill/reload: dirty values are flushed before the call, alloc_reg_slot
    // is cleared after, and load_operand lazily reloads on next use.
    if scratch_regs_safe {
        allocatable_regs.push(crate::abi::SCRATCH1);
        allocatable_regs.push(crate::abi::SCRATCH2);
    }

    // Add r7/r8 (RETURN_VALUE_REG/ARGS_LEN_REG) in all functions.
    // In leaf functions, these are idle after the prologue.
    // In non-leaf functions, r7 holds the return value after calls and r8 is
    // used as scratch in indirect call dispatch. The call lowering handles
    // spill/reload via the same mechanism as r5/r6 above.
    // Lowering paths that use r7/r8 as scratch (alu.rs signed div, NE compare,
    // control_flow.rs multi-phi) will trigger invalidate_reg via emit(), forcing
    // a lazy reload from the write-through stack slot on next use.
    if allocate_caller_saved {
        allocatable_regs.push(crate::abi::ARGS_LEN_REG); // r8
        allocatable_regs.push(crate::abi::RETURN_VALUE_REG); // r7
    }

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

    // Non-leaf functions need at least one allocatable register to proceed.
    if !is_leaf && allocatable_regs.is_empty() {
        stats.skipped_reason = Some("insufficient_nonleaf_regs");
        tracing::debug!(
            target: "wasm_pvm::regalloc",
            function = %fn_name,
            is_leaf,
            num_params,
            total_values = stats.total_values,
            total_intervals = stats.total_intervals,
            has_loops = stats.has_loops,
            allocatable_regs = stats.allocatable_regs,
            skipped_reason = stats.skipped_reason,
            "regalloc skipped"
        );
        return RegAllocResult {
            stats,
            ..RegAllocResult::default()
        };
    }

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

/// Detect loop back-edges and return a map from loop header → back-edge end position.
fn detect_loop_headers<'ctx>(
    blocks: &[inkwell::basic_block::BasicBlock<'ctx>],
    block_ranges: &HashMap<inkwell::basic_block::BasicBlock<'ctx>, (usize, usize)>,
) -> HashMap<inkwell::basic_block::BasicBlock<'ctx>, usize> {
    let block_order: HashMap<inkwell::basic_block::BasicBlock<'_>, usize> =
        blocks.iter().enumerate().map(|(i, &bb)| (bb, i)).collect();

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
                    let entry = loop_headers.entry(succ).or_insert(0);
                    *entry = (*entry).max(bb_end);
                }
            }
        }
    }
    loop_headers
}

/// Compute loop nesting depth for each instruction position.
///
/// For each loop (identified by a back-edge to a header), all positions within
/// [header_start, back_edge_end] have their depth incremented. Nested loops
/// contribute additively.
fn compute_loop_depths<'ctx>(
    block_ranges: &HashMap<inkwell::basic_block::BasicBlock<'ctx>, (usize, usize)>,
    loop_headers: &HashMap<inkwell::basic_block::BasicBlock<'ctx>, usize>,
    max_position: usize,
) -> Vec<u32> {
    let mut depths = vec![0u32; max_position + 1];
    for (&header_bb, &back_edge_end) in loop_headers {
        let (header_start, _) = block_ranges[&header_bb];
        for d in &mut depths[header_start..=back_edge_end.min(max_position)] {
            *d += 1;
        }
    }
    depths
}

/// Compute live intervals for all SSA values (parameters and instruction results).
/// Also returns whether any loops were detected (back-edges exist).
fn compute_live_intervals<'ctx>(
    blocks: &[inkwell::basic_block::BasicBlock<'ctx>],
    instr_index: &HashMap<ValKey, usize>,
    block_ranges: &HashMap<inkwell::basic_block::BasicBlock<'ctx>, (usize, usize)>,
    value_slots: &HashMap<ValKey, i32>,
    loop_headers: &HashMap<inkwell::basic_block::BasicBlock<'ctx>, usize>,
    loop_depths: &[u32],
    min_uses: usize,
) -> (Vec<LiveInterval>, bool) {
    use inkwell::values::InstructionOpcode;

    let mut def_point: HashMap<ValKey, usize> = HashMap::new();
    let mut last_use: HashMap<ValKey, usize> = HashMap::new();
    let mut use_count: HashMap<ValKey, usize> = HashMap::new();
    // Accumulated loop-depth-weighted use count per value.
    let mut weighted_uses: HashMap<ValKey, f64> = HashMap::new();

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
                                    update_use(
                                        &mut last_use,
                                        &mut use_count,
                                        &mut weighted_uses,
                                        loop_depths,
                                        vk,
                                        pred_end,
                                    );
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
                                update_use(
                                    &mut last_use,
                                    &mut use_count,
                                    &mut weighted_uses,
                                    loop_depths,
                                    vk,
                                    instr_idx,
                                );
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
        let weight = weighted_uses.get(&vk).copied().unwrap_or(0.0);

        // Skip low-use values — not worth allocating a register.
        if uses < min_uses {
            continue;
        }

        // Loop extension: if this value is live at a loop header and the loop's
        // back-edge source is beyond the current end, extend the range.
        for (&header_bb, &back_edge_end) in loop_headers {
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
            spill_weight: weight,
        });
    }

    (intervals, has_loops)
}

/// Loop depth multiplier: each nesting level multiplies cost by 10.
fn depth_weight(depth: u32) -> f64 {
    10.0f64.powi(depth as i32)
}

fn update_use(
    last_use: &mut HashMap<ValKey, usize>,
    use_count: &mut HashMap<ValKey, usize>,
    weighted_uses: &mut HashMap<ValKey, f64>,
    loop_depths: &[u32],
    vk: ValKey,
    idx: usize,
) {
    let entry = last_use.entry(vk).or_insert(0);
    *entry = (*entry).max(idx);
    *use_count.entry(vk).or_insert(0) += 1;
    let depth = loop_depths.get(idx).copied().unwrap_or(0);
    *weighted_uses.entry(vk).or_insert(0.0) += depth_weight(depth);
}

/// Standard linear-scan register allocation with spill-weight eviction.
fn linear_scan(mut intervals: Vec<LiveInterval>, allocatable_regs: &[u8]) -> RegAllocResult {
    // Sort by start point (ascending), then by spill_weight descending (prefer
    // allocating high-weight values when two intervals start at the same point).
    intervals.sort_by(|a, b| {
        a.start.cmp(&b.start).then_with(|| {
            b.spill_weight
                .partial_cmp(&a.spill_weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
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
        } else {
            // No free register: evict the active interval with the LOWEST
            // spill weight (cheapest to reload), but only if the current
            // interval has a higher weight (more valuable to keep in a register).
            let evict_candidate = active
                .iter()
                .map(|&(end, idx)| (idx, end, intervals[idx].spill_weight))
                .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

            if let Some((evict_idx, evict_end, evict_weight)) = evict_candidate
                && interval.spill_weight > evict_weight
            {
                if let Some(reg) = active_assigned.remove(&evict_idx) {
                    active.remove(&(evict_end, evict_idx));
                    // Evicted interval no longer has a stable whole-interval assignment.
                    final_assigned.remove(&evict_idx);

                    active_assigned.insert(i, reg);
                    final_assigned.insert(i, reg);
                    active.insert((interval.end, i));
                } else {
                    active.remove(&(evict_end, evict_idx));
                    final_assigned.remove(&evict_idx);
                }
            }
            // else: current interval has lower/equal weight — spill it.
        }
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
                spill_weight: 3.0,
            },
            LiveInterval {
                val_key: ValKey(2),
                slot: 16,
                start: 2,
                end: 3,
                spill_weight: 3.0,
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
    fn linear_scan_evicts_lowest_weight() {
        // Two overlapping intervals: ValKey(1) has lower weight, so it gets evicted.
        let intervals = vec![
            LiveInterval {
                val_key: ValKey(1),
                slot: 8,
                start: 0,
                end: 10,
                spill_weight: 2.0, // low weight — eviction candidate
            },
            LiveInterval {
                val_key: ValKey(2),
                slot: 16,
                start: 1,
                end: 4,
                spill_weight: 50.0, // high weight (loop-hot) — wins
            },
        ];

        let result = linear_scan(intervals, &[9]);

        assert!(!result.val_to_reg.contains_key(&ValKey(1)));
        assert_eq!(result.val_to_reg.get(&ValKey(2)), Some(&9));
    }

    #[test]
    fn linear_scan_keeps_higher_weight_active() {
        // Current interval has lower weight than active — no eviction.
        let intervals = vec![
            LiveInterval {
                val_key: ValKey(1),
                slot: 8,
                start: 0,
                end: 10,
                spill_weight: 50.0, // high weight — should stay
            },
            LiveInterval {
                val_key: ValKey(2),
                slot: 16,
                start: 1,
                end: 4,
                spill_weight: 2.0, // low weight — gets spilled
            },
        ];

        let result = linear_scan(intervals, &[9]);

        assert_eq!(result.val_to_reg.get(&ValKey(1)), Some(&9));
        assert!(!result.val_to_reg.contains_key(&ValKey(2)));
    }

    #[test]
    fn linear_scan_equal_weight_no_eviction() {
        // When weights are equal, no eviction occurs — the new interval is spilled.
        let intervals = vec![
            LiveInterval {
                val_key: ValKey(1),
                slot: 8,
                start: 0,
                end: 10,
                spill_weight: 5.0,
            },
            LiveInterval {
                val_key: ValKey(2),
                slot: 16,
                start: 1,
                end: 4,
                spill_weight: 5.0, // same weight → no eviction
            },
        ];

        let result = linear_scan(intervals, &[9]);

        assert_eq!(result.val_to_reg.get(&ValKey(1)), Some(&9));
        assert!(!result.val_to_reg.contains_key(&ValKey(2)));
    }

    #[test]
    fn depth_weight_scales_exponentially() {
        assert_eq!(depth_weight(0), 1.0);
        assert_eq!(depth_weight(1), 10.0);
        assert_eq!(depth_weight(2), 100.0);
        assert_eq!(depth_weight(3), 1000.0);
    }
}
