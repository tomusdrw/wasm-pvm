// Control flow: branches, phi nodes, switch, return, unreachable.

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

use std::collections::BTreeMap;

use inkwell::IntPredicate;
use inkwell::basic_block::BasicBlock;
use inkwell::values::{AnyValue, BasicValueEnum, InstructionOpcode, InstructionValue, PhiValue};

use crate::pvm::Instruction;
use crate::{Error, Result, abi};

use super::emitter::{
    PvmEmitter, SCRATCH1, SCRATCH2, get_bb_operand, get_operand, has_phi_from, operand_reg,
    operand_reg_avoiding, result_slot, try_get_constant, val_key_basic, val_key_instr,
};
use crate::abi::{STACK_PTR_REG, TEMP_RESULT, TEMP1, TEMP2};

/// Lower a branch instruction (conditional or unconditional).
pub fn lower_br<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    current_bb: BasicBlock<'ctx>,
) -> Result<()> {
    let num_operands = instr.get_num_operands();

    if num_operands == 1 {
        // Unconditional: br label %dest
        let dest_bb = get_bb_operand(instr, 0)?;
        emit_phi_copies(e, current_bb, dest_bb)?;
        let label = *e
            .block_labels
            .get(&dest_bb)
            .ok_or_else(|| Error::Internal("branch to unknown basic block".into()))?;
        e.emit_jump_to_label(label);
    } else {
        // Conditional: br i1 %cond, label %then, label %else
        // LLVM internal operand order: [cond, false_bb, true_bb]
        let cond = get_operand(instr, 0)?;
        let else_bb = get_bb_operand(instr, 1)?;
        let then_bb = get_bb_operand(instr, 2)?;

        let then_label = *e
            .block_labels
            .get(&then_bb)
            .ok_or_else(|| Error::Internal("branch to unknown then block".into()))?;
        let else_label = *e
            .block_labels
            .get(&else_bb)
            .ok_or_else(|| Error::Internal("branch to unknown else block".into()))?;

        let then_has_phis = has_phi_from(current_bb, then_bb);
        let else_has_phis = has_phi_from(current_bb, else_bb);

        // Check for fused ICmp: emit a direct comparison+branch instead of
        // loading a boolean result and testing against 0.
        let fused = e.pending_fused_icmp.take();

        if !then_has_phis && !else_has_phis {
            if let Some(fused) = fused {
                // Emit fused comparison+branch to then_label, fallthrough to else.
                emit_fused_branch(e, &fused, then_label)?;
                e.emit_jump_to_label(else_label);
            } else {
                // Load-side coalescing for branch condition (no dst conflict — branches have no dest).
                let cond_reg = operand_reg(e, cond, TEMP1);
                if cond_reg == TEMP1 {
                    e.load_operand(cond, TEMP1)?;
                }
                e.emit_branch_ne_imm_to_label(cond_reg, 0, then_label);
                e.emit_jump_to_label(else_label);
            }
        } else {
            // Need per-edge phi copies. Create trampolines.
            let saved_next = e.next_block_label.take();
            let then_trampoline = e.alloc_label();
            if let Some(fused) = fused {
                emit_fused_branch(e, &fused, then_trampoline)?;
            } else {
                let cond_reg = operand_reg(e, cond, TEMP1);
                if cond_reg == TEMP1 {
                    e.load_operand(cond, TEMP1)?;
                }
                e.emit_branch_ne_imm_to_label(cond_reg, 0, then_trampoline);
            }

            // Else path: phi copies + jump to else.
            emit_phi_copies(e, current_bb, else_bb)?;
            e.emit_jump_to_label(else_label);

            // Then path: phi copies + jump to then.
            e.define_label(then_trampoline);
            emit_phi_copies(e, current_bb, then_bb)?;
            e.emit_jump_to_label(then_label);
            e.next_block_label = saved_next;
        }
    }
    Ok(())
}

/// Lower a switch instruction.
pub fn lower_switch<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    current_bb: BasicBlock<'ctx>,
) -> Result<()> {
    // switch i32 %val, label %default [i32 0, label %bb0  i32 1, label %bb1 ...]
    // Operands: [value, default_bb, (case_val, case_bb), ...]
    let val = get_operand(instr, 0)?;
    let default_bb = get_bb_operand(instr, 1)?;

    let default_label = *e
        .block_labels
        .get(&default_bb)
        .ok_or_else(|| Error::Internal("switch default to unknown block".into()))?;

    // Load-side coalescing for switch value (no dst conflict — branches have no dest).
    let val_reg = operand_reg(e, val, TEMP1);
    if val_reg == TEMP1 {
        e.load_operand(val, TEMP1)?;
    }

    // Collect cases. For each case that targets a block with phis, use a trampoline.
    let num_operands = instr.get_num_operands();
    let mut trampolines: Vec<(usize, BasicBlock<'ctx>)> = Vec::new();

    let mut i = 2;
    while i + 1 < num_operands {
        let case_val = get_operand(instr, i)?;
        let case_bb = get_bb_operand(instr, i + 1)?;

        if let BasicValueEnum::IntValue(iv) = case_val
            && let Some(c) = iv.get_zero_extended_constant()
        {
            if has_phi_from(current_bb, case_bb) {
                let trampoline = e.alloc_label();
                e.emit_branch_eq_imm_to_label(val_reg, c as i32, trampoline);
                trampolines.push((trampoline, case_bb));
            } else {
                let case_label = *e
                    .block_labels
                    .get(&case_bb)
                    .ok_or_else(|| Error::Internal("switch case to unknown block".into()))?;
                e.emit_branch_eq_imm_to_label(val_reg, c as i32, case_label);
            }
        }
        i += 2;
    }

    // Default: emit phi copies inline + jump.
    // Disable fallthrough optimization when trampolines follow the default Jump.
    let saved_next = if trampolines.is_empty() {
        None
    } else {
        e.next_block_label.take()
    };
    emit_phi_copies(e, current_bb, default_bb)?;
    e.emit_jump_to_label(default_label);

    // Emit trampolines for cases that need phi copies.
    for (trampoline_label, case_bb) in trampolines {
        let case_label = *e
            .block_labels
            .get(&case_bb)
            .ok_or_else(|| Error::Internal("switch case to unknown block".into()))?;
        e.define_label(trampoline_label);
        emit_phi_copies(e, current_bb, case_bb)?;
        e.emit_jump_to_label(case_label);
    }
    if let Some(saved) = saved_next {
        e.next_block_label = Some(saved);
    }

    Ok(())
}

/// Lower a return instruction.
#[allow(clippy::unnecessary_wraps)]
pub fn lower_return<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    is_main: bool,
) -> Result<()> {
    if is_main {
        if instr.get_num_operands() > 0 {
            // Unified entry convention: return value is a packed i64.
            // Lower 32 bits = WASM ptr, upper 32 bits = len.
            // JAM SPI result convention: r7 = start address, r8 = end address.
            let ret_val = get_operand(instr, 0)?;
            let wasm_memory_base = e.config.wasm_memory_base;
            e.load_operand(ret_val, TEMP1)?;
            // TEMP2 = packed >> 32 (length)
            e.emit(Instruction::LoadImm {
                reg: TEMP2,
                value: 32,
            });
            e.emit(Instruction::ShloR64 {
                dst: TEMP2,
                src1: TEMP1,
                src2: TEMP2,
            });
            // r7 = (packed & 0xFFFFFFFF) + wasm_memory_base (start address)
            // AddImm32 naturally truncates to 32 bits and adds the base.
            e.emit(Instruction::AddImm32 {
                dst: abi::ARGS_PTR_REG,
                src: TEMP1,
                value: wasm_memory_base,
            });
            // r8 = r7 + len (end address)
            e.emit(Instruction::Add64 {
                dst: abi::ARGS_LEN_REG,
                src1: abi::ARGS_PTR_REG,
                src2: TEMP2,
            });
        }
    } else {
        // Normal function: ret void | ret i64 %val → r7.
        if instr.get_num_operands() > 0 {
            let ret_val = get_operand(instr, 0)?;
            e.load_operand(ret_val, abi::RETURN_VALUE_REG)?;
        }
    }

    // Flush dirty callee-saved registers before epilogue restores them from stack.
    // (The epilogue will overwrite the physical registers with saved values.)
    if !is_main {
        e.spill_all_dirty_regs();
    }

    emit_epilogue(e, is_main);
    Ok(())
}

/// Emit function epilogue.
fn emit_epilogue(e: &mut PvmEmitter<'_>, is_main: bool) {
    if is_main {
        // For main, jump to exit address.
        e.emit(Instruction::LoadImm {
            reg: TEMP1,
            value: abi::EXIT_ADDRESS,
        });
        e.emit(Instruction::JumpInd {
            reg: TEMP1,
            offset: 0,
        });
    } else {
        // Restore callee-saved registers r9-r12 (only those actually saved).
        for i in 0..abi::MAX_LOCAL_REGS {
            if let Some(offset) = e.callee_save_offsets[i] {
                e.emit(Instruction::LoadIndU64 {
                    dst: abi::FIRST_LOCAL_REG + i as u8,
                    base: abi::STACK_PTR_REG,
                    offset,
                });
            }
        }

        // Restore return address (only if function makes calls).
        if e.has_calls {
            e.emit(Instruction::LoadIndU64 {
                dst: abi::RETURN_ADDR_REG,
                base: abi::STACK_PTR_REG,
                offset: 0,
            });
        }

        // Deallocate frame.
        e.emit(Instruction::AddImm64 {
            dst: abi::STACK_PTR_REG,
            src: abi::STACK_PTR_REG,
            value: e.frame_size,
        });

        // Return.
        e.emit(Instruction::JumpInd {
            reg: abi::RETURN_ADDR_REG,
            offset: 0,
        });
    }
}

/// Emit a fused comparison+branch for a deferred `ICmp`.
///
/// Loads the `ICmp` operands into TEMP1/TEMP2 and emits a single PVM branch
/// instruction to `true_label` if the predicate is satisfied.
/// Falls through otherwise.
fn emit_fused_branch<'a>(
    e: &mut PvmEmitter<'a>,
    fused: &super::emitter::FusedIcmp<'a>,
    true_label: usize,
) -> Result<()> {
    // Try immediate folding: branch-imm instructions avoid loading one operand.
    // BranchXxxImm { reg, value, offset } branches if reg <op> sign_extend(value).

    // Load-side coalescing for fused branches (no dst conflict — branches have no dest register).

    // RHS constant → load only LHS, use branch-imm directly.
    if let Some(rhs_const) = try_get_constant(fused.rhs)
        && i32::try_from(rhs_const).is_ok()
    {
        let imm = rhs_const as i32;
        let lhs_reg = operand_reg(e, fused.lhs, TEMP1);
        if lhs_reg == TEMP1 {
            e.load_operand(fused.lhs, TEMP1)?;
        }
        match fused.predicate {
            IntPredicate::EQ => e.emit_branch_eq_imm_to_label(lhs_reg, imm, true_label),
            IntPredicate::NE => e.emit_branch_ne_imm_to_label(lhs_reg, imm, true_label),
            IntPredicate::ULT => e.emit_branch_lt_u_imm_to_label(lhs_reg, imm, true_label),
            IntPredicate::ULE => e.emit_branch_le_u_imm_to_label(lhs_reg, imm, true_label),
            IntPredicate::UGT => e.emit_branch_gt_u_imm_to_label(lhs_reg, imm, true_label),
            IntPredicate::UGE => e.emit_branch_ge_u_imm_to_label(lhs_reg, imm, true_label),
            IntPredicate::SLT => e.emit_branch_lt_s_imm_to_label(lhs_reg, imm, true_label),
            IntPredicate::SLE => e.emit_branch_le_s_imm_to_label(lhs_reg, imm, true_label),
            IntPredicate::SGT => e.emit_branch_gt_s_imm_to_label(lhs_reg, imm, true_label),
            IntPredicate::SGE => e.emit_branch_ge_s_imm_to_label(lhs_reg, imm, true_label),
        }
        return Ok(());
    }

    // LHS constant → load only RHS, flip the predicate direction.
    if let Some(lhs_const) = try_get_constant(fused.lhs)
        && i32::try_from(lhs_const).is_ok()
    {
        let imm = lhs_const as i32;
        let rhs_reg = operand_reg(e, fused.rhs, TEMP1);
        if rhs_reg == TEMP1 {
            e.load_operand(fused.rhs, TEMP1)?;
        }
        match fused.predicate {
            IntPredicate::EQ => e.emit_branch_eq_imm_to_label(rhs_reg, imm, true_label),
            IntPredicate::NE => e.emit_branch_ne_imm_to_label(rhs_reg, imm, true_label),
            IntPredicate::ULT => e.emit_branch_gt_u_imm_to_label(rhs_reg, imm, true_label),
            IntPredicate::ULE => e.emit_branch_ge_u_imm_to_label(rhs_reg, imm, true_label),
            IntPredicate::UGT => e.emit_branch_lt_u_imm_to_label(rhs_reg, imm, true_label),
            IntPredicate::UGE => e.emit_branch_le_u_imm_to_label(rhs_reg, imm, true_label),
            IntPredicate::SLT => e.emit_branch_gt_s_imm_to_label(rhs_reg, imm, true_label),
            IntPredicate::SLE => e.emit_branch_ge_s_imm_to_label(rhs_reg, imm, true_label),
            IntPredicate::SGT => e.emit_branch_lt_s_imm_to_label(rhs_reg, imm, true_label),
            IntPredicate::SGE => e.emit_branch_le_s_imm_to_label(rhs_reg, imm, true_label),
        }
        return Ok(());
    }

    let lhs_reg = operand_reg_avoiding(e, fused.lhs, TEMP1, &[TEMP2]);
    let rhs_reg = operand_reg_avoiding(e, fused.rhs, TEMP2, &[TEMP1]);
    if lhs_reg == TEMP1 {
        e.load_operand(fused.lhs, TEMP1)?;
    }
    if rhs_reg == TEMP2 {
        e.load_operand(fused.rhs, TEMP2)?;
    }

    // PVM convention: Branch_op { reg1: a, reg2: b } branches if b op a.
    match fused.predicate {
        IntPredicate::EQ => {
            e.emit_branch_eq_to_label(lhs_reg, rhs_reg, true_label);
        }
        IntPredicate::NE => {
            e.emit_branch_ne_to_label(lhs_reg, rhs_reg, true_label);
        }
        IntPredicate::ULT => {
            e.emit_branch_lt_u_to_label(rhs_reg, lhs_reg, true_label);
        }
        IntPredicate::UGE => {
            e.emit_branch_ge_u_to_label(rhs_reg, lhs_reg, true_label);
        }
        IntPredicate::UGT => {
            e.emit_branch_lt_u_to_label(lhs_reg, rhs_reg, true_label);
        }
        IntPredicate::ULE => {
            e.emit_branch_ge_u_to_label(lhs_reg, rhs_reg, true_label);
        }
        IntPredicate::SLT => {
            e.emit_branch_lt_s_to_label(rhs_reg, lhs_reg, true_label);
        }
        IntPredicate::SGE => {
            e.emit_branch_ge_s_to_label(rhs_reg, lhs_reg, true_label);
        }
        IntPredicate::SGT => {
            e.emit_branch_lt_s_to_label(lhs_reg, rhs_reg, true_label);
        }
        IntPredicate::SLE => {
            e.emit_branch_ge_s_to_label(lhs_reg, rhs_reg, true_label);
        }
    }

    Ok(())
}

/// Information about a phi copy with register allocation details.
struct PhiCopy<'ctx> {
    phi_slot: i32,
    incoming_value: BasicValueEnum<'ctx>,
    /// Allocated register for the phi destination (if any).
    phi_reg: Option<u8>,
    /// Allocated register for the incoming value (if valid — i.e., the reg
    /// currently materializes the correct slot).
    incoming_reg: Option<u8>,
}

/// Emit copies for phi nodes in `target_bb` that have incoming values from `current_bb`.
///
/// With lazy spill enabled, uses register-aware phi resolution:
/// - reg→reg copies use `MoveReg` (or no-op if same register)
/// - reg→stack and stack→reg copies avoid unnecessary round-trips
/// - Parallel move resolver handles cycles in register-to-register copies
///
/// Without lazy spill, falls back to the two-pass temp-register approach.
pub fn emit_phi_copies<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    current_bb: BasicBlock<'ctx>,
    target_bb: BasicBlock<'ctx>,
) -> Result<()> {
    // Collect (phi_slot, incoming_value) pairs along with allocation info.
    let mut copies: Vec<PhiCopy<'ctx>> = Vec::new();

    for instr in target_bb.get_instructions() {
        if instr.get_opcode() != InstructionOpcode::Phi {
            break;
        }
        let phi_slot = result_slot(e, instr)?;
        let phi_key = val_key_instr(instr);
        let phi_reg = e.regalloc.val_to_reg.get(&phi_key).copied();

        let phi: PhiValue<'ctx> = instr
            .try_into()
            .map_err(|()| Error::Internal("expected Phi instruction".into()))?;
        let num_incomings = phi.count_incoming();
        for i in 0..num_incomings {
            if let Some((value, block)) = phi.get_incoming(i)
                && block == current_bb
            {
                // Check if incoming value has an allocated register that's valid.
                let incoming_reg = get_valid_alloc_reg(e, value);
                copies.push(PhiCopy {
                    phi_slot,
                    incoming_value: value,
                    phi_reg,
                    incoming_reg,
                });
                break;
            }
        }
    }

    if copies.is_empty() {
        return Ok(());
    }

    // Without lazy spill, use the original two-pass approach.
    if !e.config.lazy_spill_enabled {
        return emit_phi_copies_legacy(e, &copies);
    }

    // With lazy spill: register-aware phi resolution.
    emit_phi_copies_regaware(e, &copies)
}

/// Get the valid allocated register for an incoming value, if any.
/// Returns None if the value is a constant, has no allocation, or the
/// register doesn't currently hold the right slot.
fn get_valid_alloc_reg(e: &PvmEmitter<'_>, value: BasicValueEnum<'_>) -> Option<u8> {
    if let BasicValueEnum::IntValue(iv) = value {
        // Constants don't have allocated registers.
        if iv.get_sign_extended_constant().is_some() || iv.get_zero_extended_constant().is_some() {
            return None;
        }
        if iv.is_poison() || iv.is_undef() {
            return None;
        }
        let key = val_key_basic(value);
        if let Some(&alloc_reg) = e.regalloc.val_to_reg.get(&key) {
            let slot = e.get_slot(key)?;
            if e.is_alloc_reg_valid(alloc_reg, slot) {
                return Some(alloc_reg);
            }
        }
    }
    None
}

/// Legacy two-pass phi copy (used when lazy spill is disabled).
fn emit_phi_copies_legacy<'ctx>(e: &mut PvmEmitter<'ctx>, copies: &[PhiCopy<'ctx>]) -> Result<()> {
    e.spill_all_dirty_regs();

    if copies.len() == 1 {
        let copy = &copies[0];
        e.load_operand(copy.incoming_value, TEMP1)?;
        e.store_to_slot(copy.phi_slot, TEMP1);
    } else {
        // Fast path: when the copy count fits in the temp-register pool, load
        // every incoming value into a distinct temp register and then write to
        // destinations. This emits the same `2N` PVM instructions the
        // slot-based fallback would, but skips the topological / cycle
        // bookkeeping. The 5-temp limit comes from the available scratch
        // registers (TEMP1/TEMP2/TEMP_RESULT plus SCRATCH1/SCRATCH2 when no
        // bulk memory or funnel-shift uses them in this function).
        let temp_regs = [TEMP1, TEMP2, TEMP_RESULT, SCRATCH1, SCRATCH2];
        if copies.len() <= temp_regs.len() {
            // When using SCRATCH1/SCRATCH2 as temp registers (4+ copies), spill
            // any dirty values and invalidate alloc state so that load_operand
            // will reload from the stack instead of using the (about to be
            // clobbered) register.
            if copies.len() > 3 {
                e.reload_allocated_regs_after_scratch_clobber();
            }
            for (i, copy) in copies.iter().enumerate() {
                e.load_operand(copy.incoming_value, temp_regs[i])?;
            }
            for (i, copy) in copies.iter().enumerate() {
                e.store_to_slot(copy.phi_slot, temp_regs[i]);
            }
        } else {
            // More copies than the temp-register snapshot can hold: fall back
            // to a slot-based parallel-move resolver that uses TEMP1/TEMP2 and
            // handles arbitrary copy counts (incl. permutation cycles).
            emit_phi_copies_via_slots(e, copies)?;
        }
    }

    e.spill_all_dirty_regs();
    Ok(())
}

/// Register-aware phi copy resolution with parallel move handling.
///
/// Uses a unified approach: ALL copies (reg→reg, reg→stack, stack→reg, stack→stack)
/// are handled together using a two-phase strategy:
/// 1. Load ALL incoming values into temporaries (temp regs or save allocated regs)
/// 2. Store all values to destinations
///
/// This avoids ordering issues where reg→reg copies clobber sources needed by other copies.
fn emit_phi_copies_regaware<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    copies: &[PhiCopy<'ctx>],
) -> Result<()> {
    // Check if any incoming value needs to be loaded from stack.
    let needs_stack = copies.iter().any(|c| c.incoming_reg.is_none());
    if needs_stack {
        // Flush dirty regs so the stack is authoritative for stack loads.
        e.spill_all_dirty_regs();
    }

    // Filter no-op copies: when incoming_reg == phi_reg AND the register
    // currently holds the incoming value, the phi copy is a no-op. This
    // happens when the linear scan assigns the same register to both via
    // early interval expiration for loop phi destinations. Only update the
    // emitter's alloc_reg_slot state — no data movement needed.
    //
    // Guard: don't treat SCRATCH1/SCRATCH2 as no-ops when the total copy
    // count might require them as temp registers (4+ copies). The temp
    // phase would clobber the SCRATCH register, invalidating the no-op.
    let scratch_might_be_temps = copies.len() > 3;
    let mut active_copies: Vec<usize> = Vec::new();
    for (i, copy) in copies.iter().enumerate() {
        if let (Some(src_reg), Some(phi_reg)) = (copy.incoming_reg, copy.phi_reg)
            && src_reg == phi_reg
            && !(scratch_might_be_temps && (src_reg == SCRATCH1 || src_reg == SCRATCH2))
        {
            let key = val_key_basic(copy.incoming_value);
            if let Some(slot) = e.get_slot(key)
                && e.is_alloc_reg_valid(src_reg, slot)
            {
                // No-op: value is already in the right register.
                e.spill_dirty_reg_pub(phi_reg);
                e.set_alloc_reg_for_slot(phi_reg, copy.phi_slot);
                continue;
            }
        }
        active_copies.push(i);
    }

    // Phase 1: Snapshot all ACTIVE incoming values.
    // For incoming values in allocated registers, we need to save them before
    // any destination writes can clobber them.
    //
    // Strategy: use temp registers to hold ALL incoming values, then write
    // them to destinations. This is simple and handles all dependency cases.
    // Threshold of 5 = the available scratch pool. For larger phi shapes we
    // delegate to `emit_phi_copies_via_slots`, which handles arbitrary
    // counts and permutation cycles using only TEMP1/TEMP2.
    let temp_regs = [TEMP1, TEMP2, TEMP_RESULT, SCRATCH1, SCRATCH2];

    // When using SCRATCH1/SCRATCH2 as temp registers (4+ active copies), we
    // must spill any dirty values in those registers and invalidate alloc
    // state. Additionally, any copy whose incoming_reg is SCRATCH1/SCRATCH2
    // must be forced through load_operand (reload from stack) since the
    // register will be clobbered by an earlier temp write.
    let scratch_clobbered = active_copies.len() > 3;
    if scratch_clobbered {
        e.reload_allocated_regs_after_scratch_clobber();
    }

    if active_copies.len() <= temp_regs.len() {
        // Phase 1: Load all incoming values into temp registers.
        for (ti, &ci) in active_copies.iter().enumerate() {
            let copy = &copies[ci];
            // If SCRATCH1/SCRATCH2 are being used as temps, any copy whose
            // incoming value was in one of those registers can no longer use
            // it directly — force a stack reload instead.
            let effective_incoming_reg = if scratch_clobbered {
                copy.incoming_reg
                    .filter(|&r| r != SCRATCH1 && r != SCRATCH2)
            } else {
                copy.incoming_reg
            };
            if let Some(src_reg) = effective_incoming_reg {
                if src_reg != temp_regs[ti] {
                    e.emit(Instruction::MoveReg {
                        dst: temp_regs[ti],
                        src: src_reg,
                    });
                }
            } else {
                e.load_operand(copy.incoming_value, temp_regs[ti])?;
            }
        }

        // Phase 2: Store all values to destinations.
        for (ti, &ci) in active_copies.iter().enumerate() {
            let copy = &copies[ci];
            if let Some(phi_reg) = copy.phi_reg {
                // Destination is an allocated register.
                e.spill_dirty_reg_pub(phi_reg);
                if phi_reg != temp_regs[ti] {
                    e.emit_raw_move(phi_reg, temp_regs[ti]);
                }
                e.set_alloc_reg_for_slot(phi_reg, copy.phi_slot);
            } else {
                // Destination is stack only.
                e.store_to_slot(copy.phi_slot, temp_regs[ti]);
            }
        }
    } else {
        // More active copies than the temp-register snapshot can hold: fall
        // back to slot-based parallel-move resolution. Build a sub-list of
        // PhiCopy values for the active subset.
        let active: Vec<PhiCopy<'ctx>> = active_copies
            .iter()
            .map(|&ci| PhiCopy {
                phi_slot: copies[ci].phi_slot,
                incoming_value: copies[ci].incoming_value,
                phi_reg: copies[ci].phi_reg,
                incoming_reg: copies[ci].incoming_reg,
            })
            .collect();
        emit_phi_copies_via_slots(e, &active)?;
    }

    Ok(())
}

/// Slot-based parallel-move resolver for phi copies.
///
/// Handles arbitrary numbers of copies using only TEMP1/TEMP2. Operates
/// entirely on stack slots after spilling all dirty allocated registers, so
/// it is correct regardless of the lazy-spill / register-cache state.
///
/// Algorithm:
/// 1. Spill all dirty regs so each value's canonical home is on the stack.
/// 2. Split copies into:
///    - constant copies (no source slot — emitted last via `LoadImm` + store);
///    - slot copies `(dst_slot, src_slot)`.
/// 3. Topologically resolve slot copies: a copy whose destination slot is not
///    used as a source by any unfinished copy can be emitted directly with
///    `load TEMP1; store TEMP1`.
/// 4. Remaining copies form one or more cycles. For each cycle of length k:
///    - save the original value of `cycle[0].dst` to TEMP1 (`k`th copy will
///      write to that slot but the value is needed by the last write);
///    - emit copies 0..k-1 normally via TEMP2;
///    - emit final copy by storing TEMP1 to `cycle[k-1].dst`.
///    - Total: `2k` PVM instructions per cycle.
///
/// Cache invalidation: after every direct slot store, any general or allocated
/// register cache entry pointing to the destination slot is invalidated so
/// subsequent loads reload from the (now-canonical) stack value.
fn emit_phi_copies_via_slots<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    copies: &[PhiCopy<'ctx>],
) -> Result<()> {
    e.spill_all_dirty_regs();

    // Bucket copies into slot-to-slot moves and constant materializations.
    // Skip self-copies (dst_slot == src_slot for the same SSA value).
    let mut slot_copies: Vec<(i32, i32)> = Vec::new();
    let mut const_copies: Vec<(i32, BasicValueEnum<'ctx>)> = Vec::new();

    for copy in copies {
        if is_constant_or_undef(copy.incoming_value) {
            const_copies.push((copy.phi_slot, copy.incoming_value));
            continue;
        }
        let key = val_key_basic(copy.incoming_value);
        let src_slot = e
            .get_slot(key)
            .ok_or_else(|| Error::Internal("phi incoming value has no slot".into()))?;
        if src_slot != copy.phi_slot {
            slot_copies.push((copy.phi_slot, src_slot));
        }
    }

    resolve_slot_parallel_move(e, &slot_copies);

    // Constants are emitted last: they don't depend on any source slot, but
    // their destination might have been a source for a slot copy that just ran.
    for (dst_slot, val) in const_copies {
        e.load_operand(val, TEMP1)?;
        e.emit(Instruction::StoreIndU64 {
            base: STACK_PTR_REG,
            src: TEMP1,
            offset: dst_slot,
        });
        e.invalidate_cache_for_slot(dst_slot);
    }

    // Reload phi destinations into their allocated registers (if any).
    //
    // The parallel-move resolver writes values to stack slots via raw
    // `StoreIndU64`, but never touches the register file. When the target
    // block begins, `restore_phi_alloc_reg_slots` will tell the emitter
    // that each phi_reg "owns" its phi_slot — under that invariant the
    // physical register must already hold the correct value, otherwise
    // future reads via `operand_reg(phi)` see stale data. Load each
    // phi_reg from its (now canonical) slot here.
    for copy in copies {
        if let Some(phi_reg) = copy.phi_reg {
            e.emit(Instruction::LoadIndU64 {
                dst: phi_reg,
                base: STACK_PTR_REG,
                offset: copy.phi_slot,
            });
        }
    }

    Ok(())
}

fn is_constant_or_undef(val: BasicValueEnum<'_>) -> bool {
    if let BasicValueEnum::IntValue(iv) = val {
        iv.get_sign_extended_constant().is_some()
            || iv.get_zero_extended_constant().is_some()
            || iv.is_poison()
            || iv.is_undef()
    } else {
        false
    }
}

/// Emit a slot-to-slot move using TEMP1 as the staging register.
fn emit_slot_move(e: &mut PvmEmitter<'_>, dst: i32, src: i32, via: u8) {
    e.emit(Instruction::LoadIndU64 {
        dst: via,
        base: STACK_PTR_REG,
        offset: src,
    });
    e.emit(Instruction::StoreIndU64 {
        base: STACK_PTR_REG,
        src: via,
        offset: dst,
    });
    e.invalidate_cache_for_slot(dst);
}

/// Resolve a parallel-move on stack slots.
///
/// The input is a list of `(dst, src)` slot pairs representing simultaneous
/// copies (`for each pair: slot[dst] := slot[src]`). The implementation
/// emits a serialized sequence that achieves the same effect.
///
/// Uses a topological pass for the non-cyclic part, then breaks any
/// remaining cycles by saving the head of the cycle to TEMP1 once.
fn resolve_slot_parallel_move(e: &mut PvmEmitter<'_>, slot_copies: &[(i32, i32)]) {
    if slot_copies.is_empty() {
        return;
    }

    // Track how many remaining copies still use each slot as a source.
    // Once a slot is no longer anyone's source, any copy writing it is "free".
    let mut src_use_count: BTreeMap<i32, usize> = BTreeMap::new();
    for &(_, src) in slot_copies {
        *src_use_count.entry(src).or_default() += 1;
    }

    let mut remaining: Vec<(i32, i32)> = slot_copies.to_vec();

    // Phase 1: emit any copy whose destination slot isn't a source of an
    // unfinished copy. Repeat until no more such copies exist.
    loop {
        let leaf = remaining
            .iter()
            .position(|&(dst, _)| src_use_count.get(&dst).is_none_or(|&c| c == 0));
        match leaf {
            Some(i) => {
                let (dst, src) = remaining.swap_remove(i);
                emit_slot_move(e, dst, src, TEMP1);
                if let Some(c) = src_use_count.get_mut(&src) {
                    *c -= 1;
                }
            }
            None => break,
        }
    }

    // Phase 2: anything left forms one or more cycles. Resolve each.
    while !remaining.is_empty() {
        let cycle = extract_cycle(&mut remaining);
        emit_slot_cycle(e, &cycle);
    }
}

/// Extract one cycle from `remaining`, returning the chain of copies.
///
/// Walks forward following "next dst becomes prev src" links until the loop
/// closes. For a true permutation the chain always closes; if input is
/// malformed (broken cycle) the function returns whatever path it walked,
/// which is harmless — `emit_slot_cycle` still produces a correct sequence.
fn extract_cycle(remaining: &mut Vec<(i32, i32)>) -> Vec<(i32, i32)> {
    // Pick any remaining copy as the start.
    let first = remaining.swap_remove(0);
    let head_dst = first.0;
    let mut chain = vec![first];
    let mut current_src = chain[0].1;

    while current_src != head_dst {
        let Some(idx) = remaining.iter().position(|(d, _)| *d == current_src) else {
            // Shouldn't happen if the input was a clean permutation; bail
            // gracefully — `emit_slot_cycle` handles partial chains too.
            break;
        };
        let copy = remaining.swap_remove(idx);
        current_src = copy.1;
        chain.push(copy);
    }

    chain
}

/// Emit a cyclic chain of copies using TEMP1 for the saved head value and
/// TEMP2 for the per-step staging register.
///
/// `chain` is `[(d_0, s_0), ..., (d_{k-1}, s_{k-1})]`, where for each
/// `i in 0..k-1`, `s_i == d_{i+1}` (next link), and `s_{k-1} == d_0` for a
/// closed cycle. For a chain that didn't close (defensive path), the last
/// copy's source is read from the stack like any other source.
fn emit_slot_cycle(e: &mut PvmEmitter<'_>, chain: &[(i32, i32)]) {
    if chain.is_empty() {
        return;
    }
    if chain.len() == 1 {
        // Self-loop or stray single-element chain: emit as a normal move.
        let (dst, src) = chain[0];
        emit_slot_move(e, dst, src, TEMP1);
        return;
    }

    let head_dst = chain[0].0;
    let last_src = chain[chain.len() - 1].1;
    let closed = last_src == head_dst;

    if closed {
        // Save head's original value (which is `last_src` == `head_dst`).
        e.emit(Instruction::LoadIndU64 {
            dst: TEMP1,
            base: STACK_PTR_REG,
            offset: head_dst,
        });

        // Emit all but the last copy via TEMP2.
        for &(dst, src) in &chain[..chain.len() - 1] {
            e.emit(Instruction::LoadIndU64 {
                dst: TEMP2,
                base: STACK_PTR_REG,
                offset: src,
            });
            e.emit(Instruction::StoreIndU64 {
                base: STACK_PTR_REG,
                src: TEMP2,
                offset: dst,
            });
            e.invalidate_cache_for_slot(dst);
        }

        // Final copy reads from saved TEMP1 (slot was overwritten by the first
        // step, so we can't read it from the stack any more).
        let last_dst = chain[chain.len() - 1].0;
        e.emit(Instruction::StoreIndU64 {
            base: STACK_PTR_REG,
            src: TEMP1,
            offset: last_dst,
        });
        e.invalidate_cache_for_slot(last_dst);
    } else {
        // Defensive: chain didn't close. Emit each copy directly.
        for &(dst, src) in chain {
            emit_slot_move(e, dst, src, TEMP1);
        }
    }
}
