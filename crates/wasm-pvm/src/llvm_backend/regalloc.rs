// Linear-scan register allocator for the PVM backend.
//
// Allocates long-lived SSA values to physical registers so they persist across
// basic block boundaries — particularly loop back-edges where the per-block
// register cache is cleared.
//
// Allocatable registers:
//   - r5, r6 (`abi::SCRATCH1`/`SCRATCH2`) — always available, spilled/reloaded
//     around memory intrinsics and funnel shifts that clobber them.
//   - r9-r12 (`abi::FIRST_LOCAL_REG`..+4) — available in leaf functions only,
//     for registers beyond the parameter count.
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

/// Base registers available for allocation (r5 and r6).
/// These are always allocatable and spilled/reloaded around clobbering ops.
const BASE_ALLOCATABLE_REGS: &[u8] = &[5, 6];

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
}

/// Run register allocation for a function.
///
/// `value_slots` maps `ValKey` → stack slot offset (from the pre-scan bump allocator).
/// `is_leaf` indicates whether the function contains no call instructions.
/// `num_params` is the number of function parameters (for determining available callee-saved regs).
pub fn run(
    function: FunctionValue<'_>,
    value_slots: &HashMap<ValKey, i32>,
    is_leaf: bool,
    num_params: usize,
) -> RegAllocResult {
    let blocks = function.get_basic_blocks();
    if blocks.is_empty() {
        return RegAllocResult::default();
    }

    // Phase 1: Linearize instructions and compute block index ranges.
    let (instr_index, block_ranges) = linearize(&blocks);

    // Phase 2: Compute live intervals + detect loops.
    let (intervals, has_loops) =
        compute_live_intervals(&blocks, &instr_index, &block_ranges, value_slots);

    // Skip allocation for functions without loops — the per-block register cache
    // already handles within-block forwarding, and spill/reload around calls
    // would be a net cost.
    if !has_loops || intervals.is_empty() {
        return RegAllocResult::default();
    }

    // Phase 3: Build the allocatable register set.
    let mut allocatable_regs: Vec<u8> = BASE_ALLOCATABLE_REGS.to_vec();

    // For leaf functions, add unused callee-saved registers (r9-r12) beyond
    // parameter count. These are saved/restored in prologue/epilogue anyway,
    // so using them for allocation is free.
    if is_leaf {
        for i in num_params..crate::abi::MAX_LOCAL_REGS {
            allocatable_regs.push(crate::abi::FIRST_LOCAL_REG + i as u8);
        }
    }

    // Phase 4: Linear scan allocation.
    linear_scan(intervals, &allocatable_regs)
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

        // Skip values with zero or one use — not worth allocating a register.
        if uses <= 1 {
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
    let mut assigned: HashMap<usize, u8> = HashMap::new();

    for (i, interval) in intervals.iter().enumerate() {
        // Expire old intervals.
        let expired: Vec<_> = active
            .iter()
            .take_while(|(end, _)| *end < interval.start)
            .copied()
            .collect();
        for (end, idx) in expired {
            active.remove(&(end, idx));
            if let Some(reg) = assigned.get(&idx) {
                free_regs.push(*reg);
            }
        }

        if let Some(reg) = free_regs.pop() {
            assigned.insert(i, reg);
            active.insert((interval.end, i));
        } else if let Some(&(furthest_end, furthest_idx)) = active.iter().next_back()
            && furthest_end > interval.end
        {
            // Evict the interval with the furthest end, give its register to us.
            let reg = assigned
                .remove(&furthest_idx)
                .expect("active interval must have an assigned register");
            active.remove(&(furthest_end, furthest_idx));

            assigned.insert(i, reg);
            active.insert((interval.end, i));
        }
        // else: no free register and current interval ends further — spill it.
    }

    // Build result from assigned intervals (single pass).
    for (&idx, &reg) in &assigned {
        let interval = &intervals[idx];
        result.val_to_reg.insert(interval.val_key, reg);
        result.slot_to_reg.insert(interval.slot, reg);
        result.reg_to_slot.insert(reg, interval.slot);
    }

    result
}
