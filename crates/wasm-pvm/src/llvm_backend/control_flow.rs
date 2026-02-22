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
use inkwell::values::{BasicValueEnum, InstructionOpcode, InstructionValue, PhiValue};

use crate::pvm::Instruction;
use crate::{Error, Result, abi};

use super::emitter::{
    PvmEmitter, SCRATCH1, SCRATCH2, get_bb_operand, get_operand, has_phi_from, result_slot,
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
                e.load_operand(cond, TEMP1)?;
                e.emit_branch_ne_imm_to_label(TEMP1, 0, then_label);
                e.emit_jump_to_label(else_label);
            }
        } else {
            // Need per-edge phi copies. Create trampolines.
            let then_trampoline = e.alloc_label();
            if let Some(fused) = fused {
                emit_fused_branch(e, &fused, then_trampoline)?;
            } else {
                e.load_operand(cond, TEMP1)?;
                e.emit_branch_ne_imm_to_label(TEMP1, 0, then_trampoline);
            }

            // Else path: phi copies + jump to else.
            emit_phi_copies(e, current_bb, else_bb)?;
            e.emit_jump_to_label(else_label);

            // Then path: phi copies + jump to then.
            e.define_label(then_trampoline);
            emit_phi_copies(e, current_bb, then_bb)?;
            e.emit_jump_to_label(then_label);
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

    e.load_operand(val, TEMP1)?;

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
                // Needs a trampoline for phi copies.
                let trampoline = e.alloc_label();
                e.emit_branch_eq_imm_to_label(TEMP1, c as i32, trampoline);
                trampolines.push((trampoline, case_bb));
            } else {
                let case_label = *e
                    .block_labels
                    .get(&case_bb)
                    .ok_or_else(|| Error::Internal("switch case to unknown block".into()))?;
                e.emit_branch_eq_imm_to_label(TEMP1, c as i32, case_label);
            }
        }
        i += 2;
    }

    // Default: emit phi copies inline + jump.
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
        if let Some((ptr_global, len_global)) = e.config.result_globals {
            // Globals convention: load result_ptr and result_len from WASM globals.
            // JAM SPI result convention: r7 = start address, r8 = end address.
            let wasm_memory_base = e.config.wasm_memory_base;
            let ptr_addr = abi::global_addr(ptr_global);
            e.emit(Instruction::LoadImm {
                reg: TEMP1,
                value: ptr_addr,
            });
            e.emit(Instruction::LoadIndU32 {
                dst: TEMP1,
                base: TEMP1,
                offset: 0,
            });
            let len_addr = abi::global_addr(len_global);
            e.emit(Instruction::LoadImm {
                reg: TEMP2,
                value: len_addr,
            });
            e.emit(Instruction::LoadIndU32 {
                dst: TEMP2,
                base: TEMP2,
                offset: 0,
            });
            // r7 = wasm_ptr + wasm_memory_base (start PVM address)
            e.emit(Instruction::AddImm32 {
                dst: abi::ARGS_PTR_REG,
                src: TEMP1,
                value: wasm_memory_base,
            });
            // r8 = r7 + len (end PVM address)
            e.emit(Instruction::Add64 {
                dst: abi::ARGS_LEN_REG,
                src1: abi::ARGS_PTR_REG,
                src2: TEMP2,
            });
        } else if e.config.entry_returns_ptr_len && instr.get_num_operands() > 0 {
            // Packed (ptr, len) convention: return value is packed i64.
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
        } else if instr.get_num_operands() > 0 {
            // Entry function returns a value → r7.
            let ret_val = get_operand(instr, 0)?;
            e.load_operand(ret_val, abi::RETURN_VALUE_REG)?;
        }
    } else {
        // Normal function: ret void | ret i64 %val → r7.
        if instr.get_num_operands() > 0 {
            let ret_val = get_operand(instr, 0)?;
            e.load_operand(ret_val, abi::RETURN_VALUE_REG)?;
        }
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
    e.load_operand(fused.lhs, TEMP1)?;
    e.load_operand(fused.rhs, TEMP2)?;

    // PVM convention: Branch_op { reg1: a, reg2: b } branches if b op a.
    // So to test "TEMP1 op TEMP2" we pass reg1=TEMP2, reg2=TEMP1.
    match fused.predicate {
        // EQ: branch if TEMP1 == TEMP2 (symmetric)
        IntPredicate::EQ => {
            e.emit_branch_eq_to_label(TEMP1, TEMP2, true_label);
        }
        // NE: branch if TEMP1 != TEMP2 (symmetric)
        IntPredicate::NE => {
            e.emit_branch_ne_to_label(TEMP1, TEMP2, true_label);
        }
        // ULT: branch if lhs < rhs → TEMP1 < TEMP2
        // Need: reg2 < reg1, so reg2=TEMP1, reg1=TEMP2
        IntPredicate::ULT => {
            e.emit_branch_lt_u_to_label(TEMP2, TEMP1, true_label);
        }
        // UGE: branch if lhs >= rhs → TEMP1 >= TEMP2
        // Need: reg2 >= reg1, so reg2=TEMP1, reg1=TEMP2
        IntPredicate::UGE => {
            e.emit_branch_ge_u_to_label(TEMP2, TEMP1, true_label);
        }
        // UGT: branch if lhs > rhs → TEMP2 < TEMP1
        // Need: reg2 < reg1, so reg2=TEMP2, reg1=TEMP1
        IntPredicate::UGT => {
            e.emit_branch_lt_u_to_label(TEMP1, TEMP2, true_label);
        }
        // ULE: branch if lhs <= rhs → TEMP2 >= TEMP1
        // Need: reg2 >= reg1, so reg2=TEMP2, reg1=TEMP1
        IntPredicate::ULE => {
            e.emit_branch_ge_u_to_label(TEMP1, TEMP2, true_label);
        }
        // SLT: branch if lhs < rhs (signed) → TEMP1 < TEMP2
        IntPredicate::SLT => {
            e.emit_branch_lt_s_to_label(TEMP2, TEMP1, true_label);
        }
        // SGE: branch if lhs >= rhs (signed) → TEMP1 >= TEMP2
        IntPredicate::SGE => {
            e.emit_branch_ge_s_to_label(TEMP2, TEMP1, true_label);
        }
        // SGT: branch if lhs > rhs (signed) → TEMP2 < TEMP1
        IntPredicate::SGT => {
            e.emit_branch_lt_s_to_label(TEMP1, TEMP2, true_label);
        }
        // SLE: branch if lhs <= rhs (signed) → TEMP2 >= TEMP1
        IntPredicate::SLE => {
            e.emit_branch_ge_s_to_label(TEMP1, TEMP2, true_label);
        }
    }

    Ok(())
}

/// Emit copies for phi nodes in `target_bb` that have incoming values from `current_bb`.
///
/// Uses a two-pass approach to handle potential phi cycles: first loads all
/// incoming values into temp registers (or temp stack slots), then stores them
/// to the phi node slots.
pub fn emit_phi_copies<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    current_bb: BasicBlock<'ctx>,
    target_bb: BasicBlock<'ctx>,
) -> Result<()> {
    // Collect (phi_slot, incoming_value) pairs.
    let mut copies: Vec<(i32, BasicValueEnum<'ctx>)> = Vec::new();

    for instr in target_bb.get_instructions() {
        if instr.get_opcode() != InstructionOpcode::Phi {
            break; // Phi nodes are always at the start of a block.
        }
        let phi_slot = result_slot(e, instr)?;
        // Use PhiValue API to properly access incoming (value, block) pairs.
        // InstructionValue::get_num_operands() only counts values, not blocks,
        // so the old `get_num_operands() / 2` approach was wrong.
        let phi: PhiValue<'ctx> = instr
            .try_into()
            .map_err(|()| Error::Internal("expected Phi instruction".into()))?;
        let num_incomings = phi.count_incoming();
        for i in 0..num_incomings {
            if let Some((value, block)) = phi.get_incoming(i)
                && block == current_bb
            {
                copies.push((phi_slot, value));
                break;
            }
        }
    }

    if copies.is_empty() {
        return Ok(());
    }

    if copies.len() == 1 {
        // Single phi — no cycle possible, direct copy.
        let (slot, value) = copies[0];
        e.load_operand(value, TEMP1)?;
        e.store_to_slot(slot, TEMP1);
    } else {
        // Multiple phis — use two-pass to avoid clobbering.
        let temp_regs = [TEMP1, TEMP2, TEMP_RESULT, SCRATCH1, SCRATCH2];

        if copies.len() <= temp_regs.len() {
            // All fit in temp registers: load all first, then store all.
            for (i, (_, value)) in copies.iter().enumerate() {
                e.load_operand(*value, temp_regs[i])?;
            }
            for (i, (slot, _)) in copies.iter().enumerate() {
                e.store_to_slot(*slot, temp_regs[i]);
            }
        } else {
            // Too many phi values to fit in temp registers.
            // This requires spill space in the frame, which is not currently reserved.
            return Err(Error::Unsupported(
                "too many phi values for available temp registers".to_string(),
            ));
        }
    }

    Ok(())
}
