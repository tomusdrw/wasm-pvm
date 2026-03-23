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

use super::emitter::{ValKey, is_real_call, val_key_basic, val_key_instr};
use super::successors::collect_successors;

/// Base registers always available for allocation (empty — all allocatable
/// registers are added conditionally in `run()` based on leaf/scratch analysis).
const BASE_ALLOCATABLE_REGS: &[u8] = &[];
/// Minimum dynamic use count required before a value is considered for allocation.
const MIN_USES_FOR_ALLOCATION: usize = 2;
/// Lower threshold when aggressive register allocation is enabled.
const MIN_USES_FOR_ALLOCATION_AGGRESSIVE: usize = 1;
/// Spill weight penalty per call that falls within a value's live range.
/// Each spanning call costs a spill+reload pair, so values with many spanning
/// calls are less profitable to keep in registers.
const CALL_SPANNING_PENALTY: f64 = 2.0;

/// A live interval for an SSA value.
#[derive(Debug, Clone)]
struct LiveInterval {
    val_key: ValKey,
    slot: i32,
    /// Start position (linearized instruction index).
    start: usize,
    /// End position (last use, inclusive).
    end: usize,
    /// Expiration point used for linear scan active set expiration.
    /// For loop phi destinations, this is the actual last use before loop
    /// extension (so the register becomes available earlier when the value
    /// is no longer truly live). For all other values, equals `end`.
    expiration: usize,
    /// Spill weight: sum of loop-depth-weighted uses. Higher weight → more
    /// expensive to spill (the value is used frequently in hot code).
    spill_weight: f64,
    /// Preferred register hint. If set, the allocator tries to assign this
    /// register first (e.g., r7 for call return values to avoid `MoveReg`).
    preferred_reg: Option<u8>,
    /// Whether this interval's live range contains at least one real call.
    spans_calls: bool,
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
/// `aggressive` lowers the minimum-use threshold from 2 to 1, capturing more candidates.
/// `scratch_regs_safe` indicates that this function never clobbers r5/r6
/// (`abi::SCRATCH1`/`SCRATCH2`), making them available for allocation.
/// `allocate_caller_saved` enables r7/r8 allocation in leaf functions.
#[allow(clippy::fn_params_excessive_bools)]
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

    // Phase 1d: Collect real call positions for spill weight refinement.
    let call_positions = collect_call_positions(&blocks, &instr_index);

    // Phase 2: Compute live intervals + detect loops.
    let min_uses = if aggressive {
        MIN_USES_FOR_ALLOCATION_AGGRESSIVE
    } else {
        MIN_USES_FOR_ALLOCATION
    };
    let (mut intervals, has_loops) = compute_live_intervals(
        &blocks,
        &instr_index,
        &block_ranges,
        value_slots,
        &loop_headers,
        &loop_depths,
        &call_positions,
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

    // Add r7/r8 (RETURN_VALUE_REG/ARGS_LEN_REG) in leaf functions.
    // In leaf functions, these are idle after the prologue.
    // In non-leaf functions, r7 holds the return value after calls and r8 is
    // used as scratch in indirect call dispatch. The call lowering handles
    // spill/reload via the same mechanism as r5/r6 above.
    // Lowering paths that use r7/r8 as scratch (alu.rs signed div, NE compare,
    // control_flow.rs multi-phi) will trigger invalidate_reg via emit(), forcing
    // a lazy reload from the write-through stack slot on next use.
    // Non-leaf r7/r8 allocation is not feasible: operand_reg() returns the
    // allocated register directly as a source operand, and address computation
    // (adding wasm_memory_base) modifies the register in-place, clobbering the
    // value before it can be used. Same root cause as callee-saved state
    // preservation — see docs/src/learnings.md.
    if allocate_caller_saved && is_leaf {
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

    // Pressure guard: disable early expiration when register pressure is high.
    // Freed phi registers get taken by unrelated values, causing reload traffic.
    if intervals.len() > allocatable_regs.len() * 2 {
        for iv in &mut intervals {
            iv.expiration = iv.end;
        }
    }

    // Phase 4: Linear scan allocation.
    let mut result = linear_scan(intervals, &allocatable_regs, is_leaf);
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
/// Only counts real calls (`wasm_func_*`, `__pvm_call_indirect`), not intrinsics
/// (`__pvm_load/store/memory_*`, `llvm.*`) and import function declarations
/// (`host_call_N`, `ecalli:N`, etc.) which don't use outgoing argument registers
/// (r9+). Import calls are lowered by `lower_import_call` which loads args
/// into r7+ independently.
fn max_call_args(function: FunctionValue<'_>) -> usize {
    let mut max_args = 0usize;
    for bb in function.get_basic_blocks() {
        for instr in bb.get_instructions() {
            if instr.get_opcode() == inkwell::values::InstructionOpcode::Call {
                let call_site: std::result::Result<inkwell::values::CallSiteValue, _> =
                    instr.try_into();
                if let Ok(cs) = call_site
                    && let Some(fn_val) = cs.get_called_fn_value()
                {
                    let name = fn_val.get_name().to_string_lossy();
                    // Skip PVM intrinsics (except call_indirect) and LLVM intrinsics.
                    if (name.starts_with("__pvm_") && name != "__pvm_call_indirect")
                        || name.starts_with("llvm.")
                    {
                        continue;
                    }
                    // Skip import function declarations (no body). These are
                    // lowered by lower_import_call which handles register
                    // loading into r7+ (host_call_N, ecalli:N, trap, nop),
                    // NOT via the standard r9+ calling convention.
                    // __pvm_call_indirect is also a declaration but uses the
                    // standard convention, so exclude it from this check.
                    if fn_val.count_basic_blocks() == 0 && !name.starts_with("__pvm_") {
                        continue;
                    }
                }
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
/// `[header_start, back_edge_end]` have their depth incremented. Nested loops
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

/// Collect linearized positions of real call instructions (sorted ascending).
/// Used for spill weight refinement: values spanning many calls are penalized.
fn collect_call_positions(
    blocks: &[inkwell::basic_block::BasicBlock<'_>],
    instr_index: &HashMap<ValKey, usize>,
) -> Vec<usize> {
    let mut positions = Vec::new();
    for &bb in blocks {
        for instr in bb.get_instructions() {
            if instr.get_opcode() == inkwell::values::InstructionOpcode::Call && is_real_call(instr)
            {
                let key = val_key_instr(instr);
                if let Some(&idx) = instr_index.get(&key) {
                    positions.push(idx);
                }
            }
        }
    }
    positions.sort_unstable();
    positions
}

/// Count how many call positions fall strictly within the range (start, end).
/// Excludes the defining call (at start) and the consuming call (at end) since
/// those don't require a spill+reload pair.
fn count_spanning_calls(call_positions: &[usize], start: usize, end: usize) -> usize {
    let lo = call_positions.partition_point(|&p| p <= start); // first p > start
    let hi = call_positions.partition_point(|&p| p < end); // first p >= end
    hi.saturating_sub(lo)
}

/// Compute live intervals for all SSA values (parameters and instruction results).
/// Also returns whether any loops were detected (back-edges exist).
#[allow(clippy::too_many_arguments)]
fn compute_live_intervals<'ctx>(
    blocks: &[inkwell::basic_block::BasicBlock<'ctx>],
    instr_index: &HashMap<ValKey, usize>,
    block_ranges: &HashMap<inkwell::basic_block::BasicBlock<'ctx>, (usize, usize)>,
    value_slots: &HashMap<ValKey, i32>,
    loop_headers: &HashMap<inkwell::basic_block::BasicBlock<'ctx>, usize>,
    loop_depths: &[u32],
    call_positions: &[usize],
    min_uses: usize,
) -> (Vec<LiveInterval>, bool) {
    use inkwell::values::InstructionOpcode;

    let mut def_point: HashMap<ValKey, usize> = HashMap::new();
    let mut last_use: HashMap<ValKey, usize> = HashMap::new();
    let mut use_count: HashMap<ValKey, usize> = HashMap::new();
    // Accumulated loop-depth-weighted use count per value.
    let mut weighted_uses: HashMap<ValKey, f64> = HashMap::new();
    // Values defined by real call instructions (prefer r7 allocation).
    let mut call_defined: HashMap<ValKey, bool> = HashMap::new();
    // Values defined by phi instructions at loop headers (for early expiration).
    let mut is_loop_phi: std::collections::HashSet<ValKey> = std::collections::HashSet::new();

    let has_loops = !loop_headers.is_empty();

    // Walk all instructions to find defs and uses.
    for &bb in blocks {
        let at_loop_header = loop_headers.contains_key(&bb);
        for instr in bb.get_instructions() {
            let instr_key = val_key_instr(instr);
            let instr_idx = instr_index[&instr_key];

            // This instruction defines instr_key (if it produces a value).
            if value_slots.contains_key(&instr_key) {
                def_point.entry(instr_key).or_insert(instr_idx);
                // Track call-defined values for register preference hints.
                if instr.get_opcode() == InstructionOpcode::Call && is_real_call(instr) {
                    call_defined.insert(instr_key, true);
                }
                // Track phi instructions at loop headers for early expiration.
                if at_loop_header && instr.get_opcode() == InstructionOpcode::Phi {
                    is_loop_phi.insert(instr_key);
                }
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
    // Sort by key for deterministic iteration — HashMap order depends on
    // LLVM pointer addresses which vary with ASLR, causing different register
    // assignments across runs.
    let mut sorted_slots: Vec<_> = value_slots.iter().map(|(&k, &v)| (k, v)).collect();
    sorted_slots.sort_by_key(|(k, _)| *k);
    let mut intervals = Vec::new();
    for &(vk, slot) in &sorted_slots {
        let start = def_point.get(&vk).copied().unwrap_or(0);
        let mut end = last_use.get(&vk).copied().unwrap_or(start);
        let uses = use_count.get(&vk).copied().unwrap_or(0);
        let weight = weighted_uses.get(&vk).copied().unwrap_or(0.0);

        // Skip low-use values — not worth allocating a register.
        if uses < min_uses {
            continue;
        }

        // Capture end before loop extension for early expiration of loop phis.
        let pre_extension_end = end;

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

        // Spill weight refinement: penalize values whose live range spans calls.
        // Each spanning call costs a spill+reload pair when the value is allocated.
        let spanning_calls = count_spanning_calls(call_positions, start, end);
        #[allow(clippy::cast_precision_loss)]
        let adjusted_weight = weight - (spanning_calls as f64 * CALL_SPANNING_PENALTY);

        // Call return value hint: prefer r7 for values defined by call instructions.
        let preferred_reg = if call_defined.contains_key(&vk) {
            Some(crate::abi::RETURN_VALUE_REG)
        } else {
            None
        };

        // For loop phi destinations, use pre-extension end as expiration so the
        // register is freed earlier in linear scan when the value is no longer
        // truly live (the extension only keeps the interval alive for correctness).
        let expiration = if is_loop_phi.contains(&vk) && pre_extension_end < end {
            pre_extension_end
        } else {
            end
        };

        intervals.push(LiveInterval {
            val_key: vk,
            slot,
            start,
            end,
            expiration,
            spill_weight: adjusted_weight,
            preferred_reg,
            spans_calls: spanning_calls > 0,
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
///
/// `is_leaf` controls register class preference: in non-leaf functions,
/// call-spanning intervals prefer callee-saved registers and non-call-spanning
/// intervals prefer caller-saved registers. In leaf functions, all registers
/// are equal (no calls to invalidate them).
fn linear_scan(
    mut intervals: Vec<LiveInterval>,
    allocatable_regs: &[u8],
    is_leaf: bool,
) -> RegAllocResult {
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

    // Active intervals sorted by expiration point (using BTreeSet of (expiration, index)).
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

        // Prefer the hinted register if available (e.g., r7 for call return values).
        // For non-leaf functions: call-spanning intervals prefer callee-saved
        // registers (survive calls without invalidation), non-call-spanning
        // intervals prefer caller-saved (leave callee-saved for others).
        let reg = if let Some(pref) = interval.preferred_reg {
            if let Some(pos) = free_regs.iter().position(|&r| r == pref) {
                Some(free_regs.swap_remove(pos))
            } else {
                free_regs.pop()
            }
        } else if !is_leaf && interval.spans_calls {
            // Call-spanning: prefer callee-saved (r9-r12 beyond call args).
            let callee_pos = free_regs.iter().position(|&r| {
                r >= crate::abi::FIRST_LOCAL_REG
                    && r < crate::abi::FIRST_LOCAL_REG + crate::abi::MAX_LOCAL_REGS as u8
            });
            if let Some(pos) = callee_pos {
                Some(free_regs.swap_remove(pos))
            } else {
                free_regs.pop()
            }
        } else if !is_leaf {
            // Non-call-spanning in non-leaf: prefer caller-saved (r5-r8).
            let caller_pos = free_regs.iter().position(|&r| {
                r == crate::abi::SCRATCH1
                    || r == crate::abi::SCRATCH2
                    || r == crate::abi::RETURN_VALUE_REG
                    || r == crate::abi::ARGS_LEN_REG
            });
            if let Some(pos) = caller_pos {
                Some(free_regs.swap_remove(pos))
            } else {
                free_regs.pop()
            }
        } else {
            free_regs.pop()
        };
        if let Some(reg) = reg {
            active_assigned.insert(i, reg);
            final_assigned.insert(i, reg);
            active.insert((interval.expiration, i));
        } else {
            // No free register: evict the active interval with the LOWEST
            // spill weight (cheapest to reload), but only if the current
            // interval has a higher weight (more valuable to keep in a register).
            let evict_candidate = active
                .iter()
                .map(|&(exp, idx)| (idx, exp, intervals[idx].spill_weight))
                .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

            if let Some((evict_idx, evict_exp, evict_weight)) = evict_candidate
                && interval.spill_weight > evict_weight
            {
                if let Some(reg) = active_assigned.remove(&evict_idx) {
                    active.remove(&(evict_exp, evict_idx));
                    // Evicted interval no longer has a stable whole-interval assignment.
                    final_assigned.remove(&evict_idx);

                    active_assigned.insert(i, reg);
                    final_assigned.insert(i, reg);
                    active.insert((interval.expiration, i));
                } else {
                    active.remove(&(evict_exp, evict_idx));
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
                expiration: 1,
                spill_weight: 3.0,
                preferred_reg: None,
                spans_calls: false,
            },
            LiveInterval {
                val_key: ValKey(2),
                slot: 16,
                start: 2,
                end: 3,
                expiration: 3,
                spill_weight: 3.0,
                preferred_reg: None,
                spans_calls: false,
            },
        ];

        let result = linear_scan(intervals, &[9], true);

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
                expiration: 10,
                spill_weight: 2.0, // low weight — eviction candidate
                preferred_reg: None,
                spans_calls: false,
            },
            LiveInterval {
                val_key: ValKey(2),
                slot: 16,
                start: 1,
                end: 4,
                expiration: 4,
                spill_weight: 50.0, // high weight (loop-hot) — wins
                preferred_reg: None,
                spans_calls: false,
            },
        ];

        let result = linear_scan(intervals, &[9], true);

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
                expiration: 10,
                spill_weight: 50.0, // high weight — should stay
                preferred_reg: None,
                spans_calls: false,
            },
            LiveInterval {
                val_key: ValKey(2),
                slot: 16,
                start: 1,
                end: 4,
                expiration: 4,
                spill_weight: 2.0, // low weight — gets spilled
                preferred_reg: None,
                spans_calls: false,
            },
        ];

        let result = linear_scan(intervals, &[9], true);

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
                expiration: 10,
                spill_weight: 5.0,
                preferred_reg: None,
                spans_calls: false,
            },
            LiveInterval {
                val_key: ValKey(2),
                slot: 16,
                start: 1,
                end: 4,
                expiration: 4,
                spill_weight: 5.0, // same weight → no eviction
                preferred_reg: None,
                spans_calls: false,
            },
        ];

        let result = linear_scan(intervals, &[9], true);

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

    #[test]
    fn preferred_reg_hint_selects_preferred_when_available() {
        // ValKey(1) prefers register 7; both r7 and r9 are free.
        let intervals = vec![LiveInterval {
            val_key: ValKey(1),
            slot: 8,
            start: 0,
            end: 5,
            expiration: 5,
            spill_weight: 3.0,
            preferred_reg: Some(7),
            spans_calls: false,
        }];

        let result = linear_scan(intervals, &[9, 7], true);

        assert_eq!(result.val_to_reg.get(&ValKey(1)), Some(&7));
    }

    #[test]
    fn preferred_reg_falls_back_when_unavailable() {
        // ValKey(1) prefers r7 but r7 is not allocatable.
        let intervals = vec![LiveInterval {
            val_key: ValKey(1),
            slot: 8,
            start: 0,
            end: 5,
            expiration: 5,
            spill_weight: 3.0,
            preferred_reg: Some(7),
            spans_calls: false,
        }];

        let result = linear_scan(intervals, &[9], true);

        assert_eq!(result.val_to_reg.get(&ValKey(1)), Some(&9));
    }

    #[test]
    fn count_spanning_calls_basic() {
        let call_positions = vec![2, 5, 8, 12];
        // Open interval (start, end) — excludes calls at exactly start or end.
        assert_eq!(count_spanning_calls(&call_positions, 0, 1), 0);
        assert_eq!(count_spanning_calls(&call_positions, 0, 3), 1); // call at 2
        assert_eq!(count_spanning_calls(&call_positions, 0, 10), 3); // calls at 2, 5, 8
        assert_eq!(count_spanning_calls(&call_positions, 5, 8), 0); // endpoints excluded
        assert_eq!(count_spanning_calls(&call_positions, 4, 9), 2); // calls at 5, 8
        assert_eq!(count_spanning_calls(&call_positions, 0, 20), 4); // all calls
    }

    #[test]
    fn early_expiration_frees_register_for_reuse() {
        // Simulates a loop phi destination (expiration=3, end=10) and an incoming
        // value starting at position 4. Without early expiration, the phi would
        // occupy the register until position 10, blocking the incoming value.
        // With early expiration, the phi frees the register at position 3.
        let intervals = vec![
            LiveInterval {
                val_key: ValKey(1),
                slot: 8,
                start: 0,
                end: 10,       // loop-extended
                expiration: 3, // actual last use
                spill_weight: 10.0,
                preferred_reg: None,
                spans_calls: false,
            },
            LiveInterval {
                val_key: ValKey(2),
                slot: 16,
                start: 4,
                end: 10,
                expiration: 10,
                spill_weight: 5.0,
                preferred_reg: None,
                spans_calls: false,
            },
        ];

        // Only 1 register: without early expiration, ValKey(2) would be evicted
        // or spilled. With early expiration, ValKey(1) frees the register at 3,
        // and ValKey(2) gets it at 4.
        let result = linear_scan(intervals, &[9], true);
        assert_eq!(result.val_to_reg.get(&ValKey(1)), Some(&9));
        assert_eq!(result.val_to_reg.get(&ValKey(2)), Some(&9));
    }

    #[test]
    fn no_early_expiration_without_gap() {
        // When expiration == end (no loop extension), standard behavior applies.
        // Two overlapping intervals with 1 register: second gets evicted.
        let intervals = vec![
            LiveInterval {
                val_key: ValKey(1),
                slot: 8,
                start: 0,
                end: 10,
                expiration: 10, // no early expiration
                spill_weight: 10.0,
                preferred_reg: None,
                spans_calls: false,
            },
            LiveInterval {
                val_key: ValKey(2),
                slot: 16,
                start: 4,
                end: 10,
                expiration: 10,
                spill_weight: 5.0,
                preferred_reg: None,
                spans_calls: false,
            },
        ];

        let result = linear_scan(intervals, &[9], true);
        assert_eq!(result.val_to_reg.get(&ValKey(1)), Some(&9));
        // ValKey(2) not allocated — ValKey(1) still active (higher weight)
        assert!(!result.val_to_reg.contains_key(&ValKey(2)));
    }

    #[test]
    fn call_spanning_prefers_callee_saved_in_nonleaf() {
        // In a non-leaf function, a call-spanning interval should prefer
        // callee-saved (r9) over caller-saved (r5).
        let intervals = vec![LiveInterval {
            val_key: ValKey(1),
            slot: 8,
            start: 0,
            end: 5,
            expiration: 5,
            spill_weight: 3.0,
            preferred_reg: None,
            spans_calls: true,
        }];

        // free_regs = [9, 5]: pop() would give r5 (caller-saved), but the
        // callee-saved preference should seek out r9 instead.
        let result = linear_scan(intervals, &[9, 5], false);
        assert_eq!(result.val_to_reg.get(&ValKey(1)), Some(&9));
    }

    #[test]
    fn non_call_spanning_prefers_caller_saved_in_nonleaf() {
        // In a non-leaf function, a non-call-spanning interval should prefer
        // caller-saved (r5) over callee-saved (r9), leaving callee-saved
        // registers available for call-spanning values.
        let intervals = vec![LiveInterval {
            val_key: ValKey(1),
            slot: 8,
            start: 0,
            end: 5,
            expiration: 5,
            spill_weight: 3.0,
            preferred_reg: None,
            spans_calls: false,
        }];

        // free_regs = [5, 9]: pop() would give 9, but the caller-saved
        // preference explicitly seeks r5.
        let result = linear_scan(intervals, &[5, 9], false);
        assert_eq!(result.val_to_reg.get(&ValKey(1)), Some(&5));
    }

    #[test]
    fn leaf_function_ignores_call_spanning_preference() {
        // In a leaf function, spans_calls is always false, but even if set,
        // no preference should apply — all registers are equal.
        let intervals = vec![LiveInterval {
            val_key: ValKey(1),
            slot: 8,
            start: 0,
            end: 5,
            expiration: 5,
            spill_weight: 3.0,
            preferred_reg: None,
            spans_calls: true, // would be unusual in leaf, but test the guard
        }];

        // In leaf, pop() gives 9 (last element), no preference applied.
        let result = linear_scan(intervals, &[5, 9], true);
        assert_eq!(result.val_to_reg.get(&ValKey(1)), Some(&9));
    }

    #[test]
    fn call_spanning_and_non_spanning_get_complementary_classes() {
        // Two overlapping intervals in a non-leaf function: the call-spanning
        // one should get callee-saved (r11), the non-call-spanning one should
        // get caller-saved (r5). Without the preference, both would use
        // default pop() order, potentially assigning backwards.
        let intervals = vec![
            LiveInterval {
                val_key: ValKey(1),
                slot: 8,
                start: 0,
                end: 10,
                expiration: 10,
                spill_weight: 5.0,
                preferred_reg: None,
                spans_calls: false, // should get caller-saved (r5)
            },
            LiveInterval {
                val_key: ValKey(2),
                slot: 16,
                start: 0,
                end: 10,
                expiration: 10,
                spill_weight: 4.0,
                preferred_reg: None,
                spans_calls: true, // should get callee-saved (r11)
            },
        ];

        let result = linear_scan(intervals, &[11, 5], false);
        assert_eq!(result.val_to_reg.get(&ValKey(1)), Some(&5));
        assert_eq!(result.val_to_reg.get(&ValKey(2)), Some(&11));
    }

    #[test]
    fn non_spanning_falls_back_to_callee_saved_when_no_caller_saved() {
        // When only callee-saved registers are available, a non-call-spanning
        // interval should still get allocated (fallback to pop()).
        let intervals = vec![LiveInterval {
            val_key: ValKey(1),
            slot: 8,
            start: 0,
            end: 5,
            expiration: 5,
            spill_weight: 3.0,
            preferred_reg: None,
            spans_calls: false,
        }];

        // Only callee-saved available — preference can't find caller-saved,
        // so it falls back to pop() and gets r11.
        let result = linear_scan(intervals, &[11], false);
        assert_eq!(result.val_to_reg.get(&ValKey(1)), Some(&11));
    }

    #[test]
    fn preferred_reg_overrides_callee_saved_preference() {
        // preferred_reg (r7) takes priority over the callee-saved preference,
        // even for call-spanning intervals in non-leaf functions.
        let intervals = vec![LiveInterval {
            val_key: ValKey(1),
            slot: 8,
            start: 0,
            end: 5,
            expiration: 5,
            spill_weight: 3.0,
            preferred_reg: Some(7),
            spans_calls: true,
        }];

        let result = linear_scan(intervals, &[9, 7], false);
        assert_eq!(result.val_to_reg.get(&ValKey(1)), Some(&7));
    }
}
