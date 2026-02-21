// Arithmetic, logic, comparison, conversion, and select operations.

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
use inkwell::values::{AnyValue, AnyValueEnum, InstructionOpcode, InstructionValue};

use crate::Result;
use crate::pvm::Instruction;

use super::emitter::{
    FusedIcmp, PvmEmitter, SCRATCH1, get_operand, operand_bit_width, result_slot,
    source_bit_width, try_get_constant,
};
use crate::abi::{TEMP_RESULT, TEMP1, TEMP2};

#[derive(Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    UDiv,
    SDiv,
    URem,
    SRem,
    And,
    Or,
    Xor,
    Shl,
    LShr,
    AShr,
}

/// Emit a WASM-style trap if the divisor is zero.
fn emit_wasm_div_zero_trap(e: &mut PvmEmitter, rhs_reg: u8) {
    let ok_label = e.alloc_label();
    e.emit_branch_ne_imm_to_label(rhs_reg, 0, ok_label);
    e.emit(Instruction::Trap);
    e.define_label(ok_label);
}

/// Emit a WASM-style trap for signed overflow (`INT_MIN` / -1).
/// For 32-bit operations, operands are first sign-extended from 32 to 64 bits
/// because `load_operand` uses unsigned loads which zero-extend.
fn emit_wasm_signed_overflow_trap(e: &mut PvmEmitter, lhs_reg: u8, rhs_reg: u8, is_32bit: bool) {
    let ok_label = e.alloc_label();

    // For 32-bit operands, we need to sign-extend them first because
    // load_operand uses LoadIndU64 which zero-extends.
    let lhs = if is_32bit {
        // Sign-extend lhs from 32 to 64 bits
        e.emit(Instruction::AddImm32 {
            dst: TEMP_RESULT,
            src: lhs_reg,
            value: 0,
        });
        TEMP_RESULT
    } else {
        lhs_reg
    };

    let rhs = if is_32bit {
        // Sign-extend rhs from 32 to 64 bits into SCRATCH1
        e.emit(Instruction::AddImm32 {
            dst: SCRATCH1,
            src: rhs_reg,
            value: 0,
        });
        SCRATCH1
    } else {
        rhs_reg
    };

    // 1. Check rhs != -1
    e.emit_branch_ne_imm_to_label(rhs, -1, ok_label);

    // 2. Check lhs == INT_MIN
    if is_32bit {
        // INT_MIN_32 = -2147483648 (already sign-extended in 64-bit register)
        e.emit_branch_ne_imm_to_label(lhs, i32::MIN, ok_label);
    } else {
        // INT_MIN_64 = 0x8000000000000000
        // Load INT_MIN_64 to TEMP_RESULT
        e.emit(Instruction::LoadImm64 {
            reg: TEMP_RESULT,
            value: i64::MIN as u64,
        });
        // Check if lhs != TEMP_RESULT
        // Xor lhs, TEMP_RESULT -> TEMP_RESULT (clobbers TEMP_RESULT)
        e.emit(Instruction::Xor {
            dst: TEMP_RESULT,
            src1: lhs,
            src2: TEMP_RESULT,
        });
        // If result != 0, then lhs != INT_MIN
        e.emit_branch_ne_imm_to_label(TEMP_RESULT, 0, ok_label);
    }

    // 3. Trap if we are here (rhs == -1 AND lhs == INT_MIN)
    e.emit(Instruction::Trap);

    // 4. Label
    e.define_label(ok_label);
}

pub fn lower_binary_arith<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    op: BinaryOp,
) -> Result<()> {
    let lhs = get_operand(instr, 0)?;
    let rhs = get_operand(instr, 1)?;
    let slot = result_slot(e, instr)?;
    let bits = operand_bit_width(instr);

    // Try immediate folding for Add/Sub with constant RHS.
    if let Some(rhs_const) = try_get_constant(rhs) {
        match op {
            BinaryOp::Add if i32::try_from(rhs_const).is_ok() => {
                e.load_operand(lhs, TEMP1)?;
                let imm = rhs_const as i32;
                if bits <= 32 {
                    e.emit(Instruction::AddImm32 {
                        dst: TEMP_RESULT,
                        src: TEMP1,
                        value: imm,
                    });
                } else {
                    e.emit(Instruction::AddImm64 {
                        dst: TEMP_RESULT,
                        src: TEMP1,
                        value: imm,
                    });
                }
                e.store_to_slot(slot, TEMP_RESULT);
                return Ok(());
            }
            BinaryOp::Sub
                if i32::try_from(rhs_const).is_ok() && rhs_const != i64::from(i32::MIN) =>
            {
                e.load_operand(lhs, TEMP1)?;
                let imm = -(rhs_const as i32);
                if bits <= 32 {
                    e.emit(Instruction::AddImm32 {
                        dst: TEMP_RESULT,
                        src: TEMP1,
                        value: imm,
                    });
                } else {
                    e.emit(Instruction::AddImm64 {
                        dst: TEMP_RESULT,
                        src: TEMP1,
                        value: imm,
                    });
                }
                e.store_to_slot(slot, TEMP_RESULT);
                return Ok(());
            }
            _ => {} // Fall through to two-register path.
        }
    }

    e.load_operand(lhs, TEMP1)?;
    e.load_operand(rhs, TEMP2)?;

    match (op, bits <= 32) {
        (BinaryOp::Add, true) => e.emit(Instruction::Add32 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::Add, false) => e.emit(Instruction::Add64 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::Sub, true) => e.emit(Instruction::Sub32 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::Sub, false) => e.emit(Instruction::Sub64 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::Mul, true) => e.emit(Instruction::Mul32 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::Mul, false) => e.emit(Instruction::Mul64 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::UDiv, true) => {
            emit_wasm_div_zero_trap(e, TEMP2);
            e.emit(Instruction::DivU32 {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        }
        (BinaryOp::UDiv, false) => {
            emit_wasm_div_zero_trap(e, TEMP2);
            e.emit(Instruction::DivU64 {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        }
        (BinaryOp::SDiv, true) => {
            emit_wasm_div_zero_trap(e, TEMP2);
            emit_wasm_signed_overflow_trap(e, TEMP1, TEMP2, true);
            e.emit(Instruction::DivS32 {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        }
        (BinaryOp::SDiv, false) => {
            emit_wasm_div_zero_trap(e, TEMP2);
            emit_wasm_signed_overflow_trap(e, TEMP1, TEMP2, false);
            e.emit(Instruction::DivS64 {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        }
        (BinaryOp::URem, true) => {
            emit_wasm_div_zero_trap(e, TEMP2);
            e.emit(Instruction::RemU32 {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        }
        (BinaryOp::URem, false) => {
            emit_wasm_div_zero_trap(e, TEMP2);
            e.emit(Instruction::RemU64 {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        }
        (BinaryOp::SRem, true) => {
            emit_wasm_div_zero_trap(e, TEMP2);
            // SRem overflow check?
            // "The sign of the result equals the sign of the dividend."
            // "If the divisor is 0, then a trap occurs."
            // "If the dividend is the most negative value and the divisor is -1, then the result is 0." (No trap for rem)
            // WASM spec: "Signed remainder ... trap if divisor is 0."
            // "Overflow: If n1 is the minimum signed integer and n2 is -1, the result is 0." (No trap)
            // So NO signed overflow check for Rem.
            e.emit(Instruction::RemS32 {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        }
        (BinaryOp::SRem, false) => {
            emit_wasm_div_zero_trap(e, TEMP2);
            e.emit(Instruction::RemS64 {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        }
        (BinaryOp::And, _) => e.emit(Instruction::And {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::Or, _) => e.emit(Instruction::Or {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::Xor, _) => e.emit(Instruction::Xor {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::Shl, true) => e.emit(Instruction::ShloL32 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::Shl, false) => e.emit(Instruction::ShloL64 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::LShr, true) => e.emit(Instruction::ShloR32 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::LShr, false) => e.emit(Instruction::ShloR64 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::AShr, true) => e.emit(Instruction::SharR32 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::AShr, false) => e.emit(Instruction::SharR64 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
    }

    e.store_to_slot(slot, TEMP_RESULT);
    Ok(())
}

/// Check if an instruction has exactly one use and that use is a branch instruction.
fn is_single_use_by_branch(instr: InstructionValue<'_>) -> bool {
    let first_use = instr.get_first_use();
    let Some(use_val) = first_use else {
        return false;
    };
    // Must be single use.
    if use_val.get_next_use().is_some() {
        return false;
    }
    // The user must be a Br instruction.
    if let inkwell::values::AnyValueEnum::InstructionValue(user) = use_val.get_user() {
        return user.get_opcode() == InstructionOpcode::Br;
    }
    false
}

pub fn lower_icmp<'ctx>(e: &mut PvmEmitter<'ctx>, instr: InstructionValue<'ctx>) -> Result<()> {
    let lhs = get_operand(instr, 0)?;
    let rhs = get_operand(instr, 1)?;
    let slot = result_slot(e, instr)?;

    let pred = instr
        .get_icmp_predicate()
        .ok_or_else(|| crate::Error::Internal("ICmp without predicate".into()))?;

    // Optimization: if this ICmp is only used by a single branch instruction,
    // defer it for fusion — the branch will emit a single fused PVM branch
    // instead of computing a boolean and branching on it.
    if e.config.icmp_fusion_enabled && is_single_use_by_branch(instr) {
        e.pending_fused_icmp = Some(FusedIcmp {
            predicate: pred,
            lhs,
            rhs,
        });
        return Ok(());
    }

    // Try immediate folding for ULT/SLT with constant RHS.
    if let Some(rhs_const) = try_get_constant(rhs)
        && i32::try_from(rhs_const).is_ok()
    {
        let imm = rhs_const as i32;
        match pred {
            IntPredicate::ULT => {
                e.load_operand(lhs, TEMP1)?;
                e.emit(Instruction::SetLtUImm {
                    dst: TEMP_RESULT,
                    src: TEMP1,
                    value: imm,
                });
                e.store_to_slot(slot, TEMP_RESULT);
                return Ok(());
            }
            IntPredicate::SLT => {
                e.load_operand(lhs, TEMP1)?;
                e.emit(Instruction::SetLtSImm {
                    dst: TEMP_RESULT,
                    src: TEMP1,
                    value: imm,
                });
                e.store_to_slot(slot, TEMP_RESULT);
                return Ok(());
            }
            _ => {} // Fall through to two-register path.
        }
    }

    e.load_operand(lhs, TEMP1)?;
    e.load_operand(rhs, TEMP2)?;

    match pred {
        IntPredicate::EQ => {
            // xor + setltuimm(result, 1) → (a == b)
            e.emit(Instruction::Xor {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
            e.emit(Instruction::SetLtUImm {
                dst: TEMP_RESULT,
                src: TEMP_RESULT,
                value: 1,
            });
        }
        IntPredicate::NE => {
            // xor, then check nonzero: loadimm 0 → SCRATCH1, setltu(SCRATCH1, result)
            e.emit(Instruction::Xor {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
            e.emit(Instruction::LoadImm {
                reg: SCRATCH1,
                value: 0,
            });
            e.emit(Instruction::SetLtU {
                dst: TEMP_RESULT,
                src1: SCRATCH1,
                src2: TEMP_RESULT,
            });
        }
        IntPredicate::ULT => {
            e.emit(Instruction::SetLtU {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        }
        IntPredicate::SLT => {
            e.emit(Instruction::SetLtS {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        }
        IntPredicate::UGT => {
            // a > b ⟺ b < a
            e.emit(Instruction::SetLtU {
                dst: TEMP_RESULT,
                src1: TEMP2,
                src2: TEMP1,
            });
        }
        IntPredicate::SGT => {
            e.emit(Instruction::SetLtS {
                dst: TEMP_RESULT,
                src1: TEMP2,
                src2: TEMP1,
            });
        }
        IntPredicate::ULE => {
            // a <= b ⟺ !(b < a)
            e.emit(Instruction::SetLtU {
                dst: TEMP_RESULT,
                src1: TEMP2,
                src2: TEMP1,
            });
            e.emit(Instruction::SetLtUImm {
                dst: TEMP_RESULT,
                src: TEMP_RESULT,
                value: 1,
            });
        }
        IntPredicate::SLE => {
            e.emit(Instruction::SetLtS {
                dst: TEMP_RESULT,
                src1: TEMP2,
                src2: TEMP1,
            });
            e.emit(Instruction::SetLtUImm {
                dst: TEMP_RESULT,
                src: TEMP_RESULT,
                value: 1,
            });
        }
        IntPredicate::UGE => {
            // a >= b ⟺ !(a < b)
            e.emit(Instruction::SetLtU {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
            e.emit(Instruction::SetLtUImm {
                dst: TEMP_RESULT,
                src: TEMP_RESULT,
                value: 1,
            });
        }
        IntPredicate::SGE => {
            e.emit(Instruction::SetLtS {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
            e.emit(Instruction::SetLtUImm {
                dst: TEMP_RESULT,
                src: TEMP_RESULT,
                value: 1,
            });
        }
    }

    e.store_to_slot(slot, TEMP_RESULT);
    Ok(())
}

pub fn lower_zext<'ctx>(e: &mut PvmEmitter<'ctx>, instr: InstructionValue<'ctx>) -> Result<()> {
    let src = get_operand(instr, 0)?;
    let slot = result_slot(e, instr)?;
    let from_bits = source_bit_width(instr);

    e.load_operand(src, TEMP1)?;

    if from_bits == 1 {
        // i1 → i32/i64: value is already 0 or 1, just copy.
        // (no-op, TEMP1 already has the value)
    } else if from_bits == 32 {
        // i32 → i64: clear upper 32 bits.
        // shift left 32, logical shift right 32.
        e.emit(Instruction::LoadImm {
            reg: TEMP2,
            value: 32,
        });
        e.emit(Instruction::ShloL64 {
            dst: TEMP1,
            src1: TEMP1,
            src2: TEMP2,
        });
        e.emit(Instruction::ShloR64 {
            dst: TEMP1,
            src1: TEMP1,
            src2: TEMP2,
        });
    }
    // Other widths: just copy (the value is already in the register).

    e.store_to_slot(slot, TEMP1);
    Ok(())
}

pub fn lower_sext<'ctx>(e: &mut PvmEmitter<'ctx>, instr: InstructionValue<'ctx>) -> Result<()> {
    let src = get_operand(instr, 0)?;
    let slot = result_slot(e, instr)?;
    let from_bits = source_bit_width(instr);

    e.load_operand(src, TEMP1)?;

    if from_bits == 1 {
        // i1 → i64: 0→0, 1→-1 (all ones).
        // negate: 0 - val
        e.emit(Instruction::LoadImm {
            reg: TEMP2,
            value: 0,
        });
        e.emit(Instruction::Sub64 {
            dst: TEMP1,
            src1: TEMP2,
            src2: TEMP1,
        });
    } else if from_bits == 8 {
        e.emit(Instruction::SignExtend8 {
            dst: TEMP1,
            src: TEMP1,
        });
    } else if from_bits == 16 {
        e.emit(Instruction::SignExtend16 {
            dst: TEMP1,
            src: TEMP1,
        });
    } else if from_bits == 32 {
        // Sign-extend from 32 to 64: AddImm32 with value 0 sign-extends in PVM.
        e.emit(Instruction::AddImm32 {
            dst: TEMP1,
            src: TEMP1,
            value: 0,
        });
    }

    e.store_to_slot(slot, TEMP1);
    Ok(())
}

pub fn lower_trunc<'ctx>(e: &mut PvmEmitter<'ctx>, instr: InstructionValue<'ctx>) -> Result<()> {
    let src = get_operand(instr, 0)?;
    let slot = result_slot(e, instr)?;

    e.load_operand(src, TEMP1)?;

    // Check the result type to determine target bit width.
    // For trunc i64 to i32: AddImm32 truncates and sign-extends.
    // For trunc to i1: mask with 1.
    let result_bits = match instr.as_any_value_enum() {
        AnyValueEnum::IntValue(iv) => iv.get_type().get_bit_width(),
        _ => 32, // default fallback
    };

    if result_bits == 1 {
        // Mask to i1: and with 1.
        e.emit(Instruction::LoadImm {
            reg: TEMP2,
            value: 1,
        });
        e.emit(Instruction::And {
            dst: TEMP1,
            src1: TEMP1,
            src2: TEMP2,
        });
    } else if result_bits <= 32 {
        // Truncate to 32 bits (sign-extends in PVM).
        e.emit(Instruction::AddImm32 {
            dst: TEMP1,
            src: TEMP1,
            value: 0,
        });
    }
    // i64 → i64 would be a no-op.

    e.store_to_slot(slot, TEMP1);
    Ok(())
}

pub fn lower_select<'ctx>(e: &mut PvmEmitter<'ctx>, instr: InstructionValue<'ctx>) -> Result<()> {
    // select i1 %cond, i64 %true_val, i64 %false_val
    let cond = get_operand(instr, 0)?;
    let true_val = get_operand(instr, 1)?;
    let false_val = get_operand(instr, 2)?;
    let slot = result_slot(e, instr)?;

    // Start with false_val in result slot.
    e.load_operand(false_val, TEMP_RESULT)?;
    e.store_to_slot(slot, TEMP_RESULT);

    // If cond != 0, overwrite with true_val.
    e.load_operand(cond, TEMP1)?;
    let skip_label = e.alloc_label();
    e.emit_branch_eq_imm_to_label(TEMP1, 0, skip_label);

    e.load_operand(true_val, TEMP_RESULT)?;
    e.store_to_slot(slot, TEMP_RESULT);

    e.define_label(skip_label);
    Ok(())
}
