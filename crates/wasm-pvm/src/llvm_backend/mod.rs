// LLVM IR backend: lowers LLVM IR → PVM bytecode.
//
// This module is organized into submodules:
// - `emitter`: Core PvmEmitter struct and value management
// - `alu`: Arithmetic, logic, comparison, conversion, and select operations
// - `memory`: Load/store and memory intrinsics (size, grow, fill, copy)
// - `control_flow`: Branches, phi nodes, switch, return
// - `calls`: Direct calls, indirect calls, import stubs
// - `intrinsics`: PVM and LLVM intrinsic lowering

// We use 'as' casts extensively for:
// - PVM register indices (u8) from iterators
// - Address offsets (i32) from pointers
// - Immediate values (i32/i64) from LLVM constants
// These are intentional truncations/wraps where we know the range is safe or valid for PVM.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

mod alu;
mod calls;
mod control_flow;
mod emitter;
mod intrinsics;
mod memory;
pub(crate) mod regalloc;
mod successors;

pub use emitter::{
    EmitterConfig, LlvmCallFixup, LlvmFunctionTranslation, LlvmIndirectCallFixup, LoweringContext,
};

use std::collections::HashMap;

use inkwell::basic_block::BasicBlock;
use inkwell::values::{FunctionValue, InstructionOpcode};

use crate::pvm::Instruction;
use crate::{Error, Result, abi};

use abi::{TEMP1, TEMP2};
use emitter::{PvmEmitter, pre_scan_function, val_key_instr};

/// Lower a single LLVM function to PVM bytecode.
pub fn lower_function(
    function: FunctionValue<'_>,
    ctx: &LoweringContext,
    is_main: bool,
    _func_idx: usize,
    call_return_base: usize,
) -> Result<LlvmFunctionTranslation> {
    let config = EmitterConfig {
        wasm_memory_base: ctx.wasm_memory_base,
        register_cache_enabled: ctx.optimizations.register_cache,
        icmp_fusion_enabled: ctx.optimizations.icmp_branch_fusion,
        shrink_wrap_enabled: ctx.optimizations.shrink_wrap_callee_saves,
        constant_propagation_enabled: ctx.optimizations.constant_propagation,
        cross_block_cache_enabled: ctx.optimizations.cross_block_cache,
        register_allocation_enabled: ctx.optimizations.register_allocation,
        fallthrough_jumps_enabled: ctx.optimizations.fallthrough_jumps,
        lazy_spill_enabled: ctx.optimizations.lazy_spill,
    };
    let mut emitter = PvmEmitter::new(config, call_return_base);

    // Phase 1: Pre-scan — allocate labels for blocks and slots for all SSA values.
    pre_scan_function(&mut emitter, function, is_main);
    emitter.frame_size = emitter.next_slot_offset;

    // Phase 1b: Register allocation — assign long-lived values to physical registers.
    if emitter.config.register_allocation_enabled {
        let is_leaf = !emitter.has_calls;
        let scratch_safe =
            ctx.optimizations.allocate_scratch_regs && emitter::scratch_regs_safe(function);
        emitter.regalloc = regalloc::run(
            function,
            &emitter.value_slots,
            is_leaf,
            function.count_params() as usize,
            ctx.optimizations.aggressive_register_allocation,
            scratch_safe,
            ctx.optimizations.allocate_caller_saved_regs,
        );

        // If regalloc allocated any callee-saved registers (r9-r12), mark them
        // as used so shrink wrapping saves/restores them in prologue/epilogue.
        for &reg in emitter.regalloc.reg_to_slot.keys() {
            if reg >= crate::abi::FIRST_LOCAL_REG
                && reg < crate::abi::FIRST_LOCAL_REG + crate::abi::MAX_LOCAL_REGS as u8
            {
                let idx = (reg - crate::abi::FIRST_LOCAL_REG) as usize;
                if !emitter.used_callee_regs[idx] {
                    emitter.used_callee_regs[idx] = true;
                    // Assign a frame offset for this newly-used callee-save reg.
                    emitter.callee_save_offsets[idx] = Some(emitter.next_slot_offset);
                    emitter.next_slot_offset += 8;
                    emitter.frame_size = emitter.next_slot_offset;
                }
            }
        }
    }

    // Phase 2: Emit prologue.
    emit_prologue(&mut emitter, function, ctx, is_main)?;

    // Flush dirty registers from the prologue's store_to_slot calls.
    // With lazy spill, the prologue stores parameters to allocated registers
    // without writing to the stack. We must flush before the first block so
    // the stack is authoritative at block boundaries.
    if emitter.config.lazy_spill_enabled {
        emitter.spill_all_dirty_regs();
    }

    // Phase 3: Lower each basic block.
    let use_cross_block_cache =
        emitter.config.register_cache_enabled && emitter.config.cross_block_cache_enabled;
    let has_regalloc = !emitter.regalloc.val_to_reg.is_empty();
    let is_leaf = !emitter.has_calls;
    let mut block_exit_cache: HashMap<BasicBlock<'_>, emitter::CacheSnapshot> = HashMap::new();

    // For functions with regalloc that need alloc_reg_slot propagation, build
    // predecessor map. Needed for non-leaf functions (intersection at merge points
    // + dominator propagation at loop headers) and leaf functions with lazy spill
    // (dominator propagation at loop headers).
    let pred_map: HashMap<BasicBlock<'_>, Vec<BasicBlock<'_>>> =
        if has_regalloc && (!is_leaf || emitter.config.lazy_spill_enabled) {
        let mut map: HashMap<BasicBlock<'_>, Vec<BasicBlock<'_>>> = HashMap::new();
        for bb in function.get_basic_blocks() {
            if let Some(term) = bb.get_terminator() {
                let successors = successors::collect_successors(term);
                let unique_succs: std::collections::HashSet<_> = successors.into_iter().collect();
                for succ in unique_succs {
                    map.entry(succ).or_default().push(bb);
                }
            }
        }
        map
    } else {
        HashMap::new()
    };

    let basic_blocks = function.get_basic_blocks();
    for (block_idx, bb) in basic_blocks.iter().enumerate() {
        let bb = *bb;
        let label = emitter.block_labels[&bb];
        let pred_info = emitter.block_single_pred.get(&bb).copied();

        // Set next_block_label so emit_jump_to_label can skip jumps to the next block.
        emitter.next_block_label = basic_blocks
            .get(block_idx + 1)
            .and_then(|next_bb| emitter.block_labels.get(next_bb).copied());

        let mut propagated = false;
        if use_cross_block_cache
            && let Some(pred_bb) = pred_info
            && !emitter::block_has_phis(bb)
            && let Some(snapshot) = block_exit_cache.get(&pred_bb).cloned()
        {
            emitter.define_label_preserving_cache(label);
            emitter.restore_cache(&snapshot);
            propagated = true;
        }

        if !propagated {
            if has_regalloc && is_leaf && !emitter.config.lazy_spill_enabled {
                // Leaf function without lazy spill: write-through ensures the
                // stack is always authoritative, so alloc_reg_slot from any
                // predecessor is safe to inherit.
                emitter.define_label_preserving_alloc(label);
            } else if has_regalloc && (!is_leaf || emitter.config.lazy_spill_enabled) {
                // Non-leaf function or leaf with lazy spill: use predecessor
                // intersection for forward merge points where ALL predecessors
                // have been processed, and dominator propagation at loop headers
                // (where back-edge predecessors are not yet processed).
                emitter.define_label(label);
                if let Some(preds) = pred_map.get(&bb) {
                    let all_processed = preds.iter().all(|p| block_exit_cache.contains_key(p));
                    if all_processed && !preds.is_empty() {
                        // All predecessors processed: intersect all alloc states.
                        let first_snap = &block_exit_cache[&preds[0]];
                        emitter.set_alloc_reg_slot_from(&first_snap.alloc_reg_slot);
                        for pred in &preds[1..] {
                            let snap = &block_exit_cache[pred];
                            emitter.intersect_alloc_reg_slot(&snap.alloc_reg_slot);
                        }
                    } else if !preds.is_empty() {
                        // Back-edge present: propagate alloc state from processed
                        // predecessors (typically the dominator). For non-leaf
                        // functions, only callee-saved registers beyond
                        // max_call_args are safe (never clobbered by call argument
                        // setup or caller-save convention). For leaf functions,
                        // all registers are safe (no calls to clobber them).
                        let mut processed = preds
                            .iter()
                            .filter(|p| block_exit_cache.contains_key(p));
                        if let Some(first) = processed.next() {
                            let first_snap = &block_exit_cache[first];
                            if is_leaf {
                                emitter
                                    .set_alloc_reg_slot_from(&first_snap.alloc_reg_slot);
                            } else {
                                let clobbered_locals = emitter
                                    .regalloc
                                    .stats
                                    .max_call_args
                                    .min(crate::abi::MAX_LOCAL_REGS);
                                let first_safe = crate::abi::FIRST_LOCAL_REG
                                    + clobbered_locals as u8;
                                let last_safe = crate::abi::FIRST_LOCAL_REG
                                    + crate::abi::MAX_LOCAL_REGS as u8;
                                emitter.set_alloc_reg_slot_filtered(
                                    &first_snap.alloc_reg_slot,
                                    |r| r >= first_safe && r < last_safe,
                                );
                            }
                            for pred in processed {
                                let snap = &block_exit_cache[pred];
                                emitter
                                    .intersect_alloc_reg_slot(&snap.alloc_reg_slot);
                            }
                        }
                    }
                }
            } else {
                emitter.define_label(label);
            }

            // With lazy spill: after define_label clears alloc state, restore
            // alloc_reg_slot for phi destinations that have allocated registers.
            // The phi copy code (emit_phi_copies_regaware) wrote the values into
            // these registers and marked them dirty. The define_label cleared it,
            // so we re-establish ownership here so load_operand knows the register
            // holds the correct value.
            if has_regalloc && emitter.config.lazy_spill_enabled && emitter::block_has_phis(bb) {
                restore_phi_alloc_reg_slots(&mut emitter, bb);
            }
        }

        // Process instructions, saving cache snapshot before the terminator.
        // The terminator (branch/switch) may emit path-specific phi copies that
        // corrupt the cache for other successors. By snapshotting before the
        // terminator and invalidating temp registers it may clobber, we get a
        // cache that's valid for ALL successors.
        let instructions: Vec<_> = bb.get_instructions().collect();
        if use_cross_block_cache && !instructions.is_empty() {
            let term_idx = instructions.len() - 1;
            for &instruction in &instructions[..term_idx] {
                lower_instruction(&mut emitter, instruction, bb, ctx, is_main)?;
            }
            // Lazy spill: flush dirty registers before snapshotting.
            // With register-aware phi copies (Phase 5), the snapshot captures
            // dirty state that successors will inherit. Successors with phi
            // nodes restore alloc_reg_slot from their phi destinations.
            // Non-phi successors restore from the snapshot (with dirty flags),
            // and auto-spill ensures correctness if registers are clobbered.
            // We keep the spill here for safety: non-phi successors that DON'T
            // use cross-block cache will clear alloc state and reload from stack,
            // requiring authoritative stack values.
            if emitter.config.lazy_spill_enabled {
                emitter.spill_all_dirty_regs();
            }
            // Snapshot before terminator, then invalidate temp registers that
            // the terminator's operand loads may overwrite (TEMP1/TEMP2 for
            // branch conditions, switch values, fused ICmp operands).
            // CacheSnapshot now includes allocated-register slot ownership too.
            let mut snap = emitter.snapshot_cache();
            snap.invalidate_reg(TEMP1);
            snap.invalidate_reg(TEMP2);
            block_exit_cache.insert(bb, snap);
            // Now lower the terminator.
            lower_instruction(&mut emitter, instructions[term_idx], bb, ctx, is_main)?;
        } else if !instructions.is_empty() && emitter.config.lazy_spill_enabled {
            let term_idx = instructions.len() - 1;
            for &instruction in &instructions[..term_idx] {
                lower_instruction(&mut emitter, instruction, bb, ctx, is_main)?;
            }
            // Flush dirty registers before the terminator.
            emitter.spill_all_dirty_regs();
            lower_instruction(&mut emitter, instructions[term_idx], bb, ctx, is_main)?;
        } else {
            for &instruction in &instructions {
                lower_instruction(&mut emitter, instruction, bb, ctx, is_main)?;
            }
        }
    }
    emitter.next_block_label = None;

    // Collect pre-DSE instruction count for stats.
    let pre_dse_instructions = emitter.instructions.len();

    // Dead store elimination: remove SP-relative stores that are never loaded from.
    // With register-aware phi resolution (Phase 5), phi destination values are
    // read from registers (via alloc_reg_slot), not from the stack. The spill
    // stores from the before-terminator flush can be eliminated by DSE if no
    // other code path loads from those slots. We no longer protect allocated
    // slot offsets unconditionally — DSE can now remove truly dead spill stores.
    if ctx.optimizations.dead_store_elimination {
        let protected_offsets: std::collections::HashSet<i32> = std::collections::HashSet::new();
        crate::pvm::peephole::eliminate_dead_stores(
            &mut emitter.instructions,
            &mut emitter.fixups,
            &mut emitter.call_fixups,
            &mut emitter.indirect_call_fixups,
            &mut emitter.labels,
            &protected_offsets,
        );
    }

    // Collect pre-peephole instruction count for stats.
    let pre_peephole_instructions = emitter.instructions.len();

    // Peephole optimization: remove redundant instructions before fixup resolution.
    if ctx.optimizations.peephole {
        crate::pvm::peephole::optimize(
            &mut emitter.instructions,
            &mut emitter.fixups,
            &mut emitter.call_fixups,
            &mut emitter.indirect_call_fixups,
            &mut emitter.labels,
        );
    }

    emitter.resolve_fixups()?;

    if emitter.config.register_allocation_enabled {
        let fn_name = function.get_name().to_string_lossy().to_string();
        let stats = &emitter.regalloc.stats;
        let usage = &emitter.regalloc_usage;
        tracing::info!(
            target: "wasm_pvm::regalloc",
            function = %fn_name,
            is_leaf = !emitter.has_calls,
            params = function.count_params(),
            total_values = stats.total_values,
            total_intervals = stats.total_intervals,
            has_loops = stats.has_loops,
            allocatable_regs = stats.allocatable_regs,
            allocated_values = stats.allocated_values,
            skipped_reason = ?stats.skipped_reason,
            alloc_load_hits = usage.load_hits,
            alloc_load_reloads = usage.load_reloads,
            alloc_load_moves = usage.load_moves,
            alloc_store_hits = usage.store_hits,
            alloc_store_moves = usage.store_moves,
            emitted_instructions = emitter.instructions.len(),
            "regalloc lowering summary"
        );
    }

    // Build per-function lowering stats.
    let regalloc_stats = &emitter.regalloc.stats;
    let regalloc_usage = &emitter.regalloc_usage;
    let lowering_stats = emitter::FunctionLoweringStats {
        frame_size: emitter.frame_size,
        is_leaf: !emitter.has_calls,
        pre_dse_instructions,
        pre_peephole_instructions,
        regalloc_total_values: regalloc_stats.total_values,
        regalloc_allocated_values: regalloc_stats.allocated_values,
        regalloc_registers_used: emitter.regalloc.reg_to_slot.keys().copied().collect(),
        regalloc_skipped_reason: regalloc_stats.skipped_reason,
        regalloc_load_hits: regalloc_usage.load_hits,
        regalloc_load_reloads: regalloc_usage.load_reloads,
        regalloc_load_moves: regalloc_usage.load_moves,
        regalloc_store_hits: regalloc_usage.store_hits,
        regalloc_store_moves: regalloc_usage.store_moves,
    };

    let num_call_returns = emitter.num_call_returns();
    Ok(LlvmFunctionTranslation {
        instructions: emitter.instructions,
        call_fixups: emitter.call_fixups,
        indirect_call_fixups: emitter.indirect_call_fixups,
        num_call_returns,
        lowering_stats,
    })
}

/// Restore alloc_reg_slot for phi destinations at the start of a block.
///
/// After `define_label` clears all alloc state, this re-establishes ownership
/// for phi destinations that have allocated registers. The phi copy code
/// (register-aware path) wrote the values into these registers before the
/// block boundary, so they are physically correct.
fn restore_phi_alloc_reg_slots(e: &mut PvmEmitter<'_>, bb: BasicBlock<'_>) {
    for instr in bb.get_instructions() {
        if instr.get_opcode() != InstructionOpcode::Phi {
            break;
        }
        let phi_key = val_key_instr(instr);
        if let Some(&phi_reg) = e.regalloc.val_to_reg.get(&phi_key)
            && let Some(phi_slot) = e.get_slot(phi_key)
        {
            e.set_alloc_reg_for_slot(phi_reg, phi_slot);
        }
    }
}

/// Emit function prologue.
fn emit_prologue<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    function: FunctionValue<'ctx>,
    ctx: &LoweringContext,
    is_main: bool,
) -> Result<()> {
    if !is_main {
        // Stack overflow check: verify SP - frame_size >= stack_limit.
        let limit = abi::stack_limit(ctx.stack_size);
        let continue_label = e.alloc_label();

        // Must use LoadImm64 (not LoadImm) because the limit is in the 0xFExx_xxxx
        // range which is negative as i32. LoadImm sign-extends to i64, producing
        // 0xFFFFFFFF_FExx_xxxx which breaks unsigned comparison.
        e.emit(Instruction::LoadImm64 {
            reg: TEMP1,
            value: u64::from(limit as u32),
        });
        e.emit(Instruction::AddImm64 {
            dst: TEMP2,
            src: abi::STACK_PTR_REG,
            value: -e.frame_size,
        });
        // Branch to continue if new_sp >= limit.
        // BranchGeU { reg1, reg2 } branches if reg2 >= reg1.
        let fixup_idx = e.instructions.len();
        e.fixups.push((fixup_idx, continue_label));
        e.emit(Instruction::BranchGeU {
            reg1: TEMP1,
            reg2: TEMP2,
            offset: 0,
        });
        e.emit(Instruction::Trap);
        e.define_label(continue_label);
    }

    // Allocate stack frame (needed for SSA slot storage in all functions).
    e.emit(Instruction::AddImm64 {
        dst: abi::STACK_PTR_REG,
        src: abi::STACK_PTR_REG,
        value: -e.frame_size,
    });

    if !is_main {
        // Save return address (only if function makes calls).
        if e.has_calls {
            e.emit(Instruction::StoreIndU64 {
                base: abi::STACK_PTR_REG,
                src: abi::RETURN_ADDR_REG,
                offset: 0,
            });
        }

        // Save callee-saved registers r9-r12 (only those actually used).
        for i in 0..abi::MAX_LOCAL_REGS {
            if let Some(offset) = e.callee_save_offsets[i] {
                e.emit(Instruction::StoreIndU64 {
                    base: abi::STACK_PTR_REG,
                    src: abi::FIRST_LOCAL_REG + i as u8,
                    offset,
                });
            }
        }
    }

    // Copy parameters to their SSA slots.
    let params = function.get_params();
    for (i, param) in params.iter().enumerate() {
        let key = emitter::val_key_basic(*param);
        let slot = e
            .get_slot(key)
            .ok_or_else(|| Error::Internal(format!("no slot for parameter {i} (key {key:?})")))?;

        if is_main {
            // For main, SPI passes r7=args_ptr, r8=args_len.
            // Adjust args_ptr by subtracting wasm_memory_base.
            if i == 0 {
                e.emit(Instruction::AddImm64 {
                    dst: abi::ARGS_PTR_REG,
                    src: abi::ARGS_PTR_REG,
                    value: -e.config.wasm_memory_base,
                });
                e.store_to_slot(slot, abi::ARGS_PTR_REG);
            } else if i == 1 {
                e.store_to_slot(slot, abi::ARGS_LEN_REG);
            }
        } else if i < abi::MAX_LOCAL_REGS {
            // First 4 params come in r9-r12.
            e.store_to_slot(slot, abi::FIRST_LOCAL_REG + i as u8);
        } else {
            // Overflow params from PARAM_OVERFLOW_BASE.
            let overflow_offset = abi::PARAM_OVERFLOW_BASE + ((i - abi::MAX_LOCAL_REGS) * 8) as i32;
            e.emit(Instruction::LoadImm {
                reg: TEMP1,
                value: overflow_offset,
            });
            e.emit(Instruction::LoadIndU64 {
                dst: TEMP1,
                base: TEMP1,
                offset: 0,
            });
            e.store_to_slot(slot, TEMP1);
        }
    }

    Ok(())
}

/// Lower a single LLVM instruction.
fn lower_instruction<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: inkwell::values::InstructionValue<'ctx>,
    current_bb: inkwell::basic_block::BasicBlock<'ctx>,
    ctx: &LoweringContext,
    is_main: bool,
) -> Result<()> {
    use alu::{
        BinaryOp, lower_binary_arith, lower_icmp, lower_select, lower_sext, lower_trunc, lower_zext,
    };
    use calls::lower_call;
    use control_flow::{lower_br, lower_return, lower_switch};
    use memory::{lower_wasm_global_load, lower_wasm_global_store};

    match instr.get_opcode() {
        // Binary arithmetic
        InstructionOpcode::Add => lower_binary_arith(e, instr, BinaryOp::Add),
        InstructionOpcode::Sub => lower_binary_arith(e, instr, BinaryOp::Sub),
        InstructionOpcode::Mul => lower_binary_arith(e, instr, BinaryOp::Mul),
        InstructionOpcode::UDiv => lower_binary_arith(e, instr, BinaryOp::UDiv),
        InstructionOpcode::SDiv => lower_binary_arith(e, instr, BinaryOp::SDiv),
        InstructionOpcode::URem => lower_binary_arith(e, instr, BinaryOp::URem),
        InstructionOpcode::SRem => lower_binary_arith(e, instr, BinaryOp::SRem),

        // Bitwise
        InstructionOpcode::And => lower_binary_arith(e, instr, BinaryOp::And),
        InstructionOpcode::Or => lower_binary_arith(e, instr, BinaryOp::Or),
        InstructionOpcode::Xor => lower_binary_arith(e, instr, BinaryOp::Xor),
        InstructionOpcode::Shl => lower_binary_arith(e, instr, BinaryOp::Shl),
        InstructionOpcode::LShr => lower_binary_arith(e, instr, BinaryOp::LShr),
        InstructionOpcode::AShr => lower_binary_arith(e, instr, BinaryOp::AShr),

        // Comparisons
        InstructionOpcode::ICmp => lower_icmp(e, instr),

        // Conversions
        InstructionOpcode::ZExt => lower_zext(e, instr),
        InstructionOpcode::SExt => lower_sext(e, instr),
        InstructionOpcode::Trunc => lower_trunc(e, instr),

        // Select
        InstructionOpcode::Select => lower_select(e, instr),

        // Control flow (pass current_bb for phi elimination)
        InstructionOpcode::Br => lower_br(e, instr, current_bb),
        InstructionOpcode::Switch => lower_switch(e, instr, current_bb),
        InstructionOpcode::Return => lower_return(e, instr, is_main),
        InstructionOpcode::Unreachable => {
            e.emit(Instruction::Trap);
            Ok(())
        }

        // Load/Store (globals after mem2reg)
        InstructionOpcode::Load => lower_wasm_global_load(e, instr, ctx),
        InstructionOpcode::Store => lower_wasm_global_store(e, instr, ctx),

        // Calls (intrinsics + wasm functions)
        InstructionOpcode::Call => lower_call(e, instr, ctx),

        // Phi nodes — copies emitted by terminators via emit_phi_copies()
        InstructionOpcode::Phi => Ok(()),

        _ => Err(Error::Unsupported(format!(
            "LLVM opcode {:?}",
            instr.get_opcode()
        ))),
    }
}
