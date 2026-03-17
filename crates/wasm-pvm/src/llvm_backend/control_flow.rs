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

use inkwell::IntPredicate;
use inkwell::basic_block::BasicBlock;
use inkwell::values::{AnyValue, BasicValueEnum, InstructionOpcode, InstructionValue, PhiValue};

use crate::pvm::Instruction;
use crate::{Error, Result, abi};

use super::emitter::{
    PvmEmitter, SCRATCH1, SCRATCH2, get_bb_operand, get_operand, has_phi_from, operand_reg,
    result_slot, try_get_constant, val_key_basic, val_key_instr,
};
use crate::abi::{TEMP_RESULT, TEMP1, TEMP2};

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

    let lhs_reg = operand_reg(e, fused.lhs, TEMP1);
    let rhs_reg = operand_reg(e, fused.rhs, TEMP2);
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
/// - reg→reg copies use MoveReg (or no-op if same register)
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
fn emit_phi_copies_legacy<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    copies: &[PhiCopy<'ctx>],
) -> Result<()> {
    e.spill_all_dirty_regs();

    if copies.len() == 1 {
        let copy = &copies[0];
        e.load_operand(copy.incoming_value, TEMP1)?;
        e.store_to_slot(copy.phi_slot, TEMP1);
    } else {
        let temp_regs = [TEMP1, TEMP2, TEMP_RESULT, SCRATCH1, SCRATCH2];
        if copies.len() <= temp_regs.len() {
            for (i, copy) in copies.iter().enumerate() {
                e.load_operand(copy.incoming_value, temp_regs[i])?;
            }
            for (i, copy) in copies.iter().enumerate() {
                e.store_to_slot(copy.phi_slot, temp_regs[i]);
            }
        } else {
            return Err(Error::Unsupported(
                "too many phi values for available temp registers".to_string(),
            ));
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

    // Phase 1: Snapshot all incoming values.
    // For incoming values in allocated registers, we need to save them before
    // any destination writes can clobber them.
    //
    // Strategy: use temp registers to hold ALL incoming values, then write
    // them to destinations. This is simple and handles all dependency cases.
    let temp_regs = [TEMP1, TEMP2, TEMP_RESULT, SCRATCH1, SCRATCH2];

    if copies.len() <= temp_regs.len() {
        // Phase 1: Load all incoming values into temp registers.
        for (i, copy) in copies.iter().enumerate() {
            if let Some(src_reg) = copy.incoming_reg {
                if src_reg != temp_regs[i] {
                    e.emit(Instruction::MoveReg {
                        dst: temp_regs[i],
                        src: src_reg,
                    });
                }
            } else {
                e.load_operand(copy.incoming_value, temp_regs[i])?;
            }
        }

        // Phase 2: Store all values to destinations.
        for (i, copy) in copies.iter().enumerate() {
            if let Some(phi_reg) = copy.phi_reg {
                // Destination is an allocated register.
                e.spill_dirty_reg_pub(phi_reg);
                if phi_reg != temp_regs[i] {
                    e.emit_raw_move(phi_reg, temp_regs[i]);
                }
                e.set_alloc_reg_for_slot(phi_reg, copy.phi_slot);
            } else {
                // Destination is stack only.
                e.store_to_slot(copy.phi_slot, temp_regs[i]);
            }
        }
    } else {
        return Err(Error::Unsupported(
            "too many phi values for available temp registers".to_string(),
        ));
    }

    Ok(())
}

