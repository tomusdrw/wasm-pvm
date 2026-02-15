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
use inkwell::values::{AnyValue, AnyValueEnum, InstructionValue};

use crate::Result;
use crate::pvm::Instruction;

use super::emitter::{PvmEmitter, SCRATCH1, get_operand, operand_bit_width, result_slot};
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

fn emit_div_zero_check(e: &mut PvmEmitter, rhs_reg: u8) {
    let ok_label = e.alloc_label();
    e.emit_branch_ne_imm_to_label(rhs_reg, 0, ok_label);
    e.emit(Instruction::Trap);
    e.define_label(ok_label);
}

fn emit_signed_overflow_check(e: &mut PvmEmitter, lhs_reg: u8, rhs_reg: u8, is_32bit: bool) {
    let ok_label = e.alloc_label();

    // 1. Check rhs != -1
    e.emit_branch_ne_imm_to_label(rhs_reg, -1, ok_label);

    // 2. Check lhs == INT_MIN
    if is_32bit {
        // INT_MIN_32 = -2147483648 (fits in i32)
        // If lhs != INT_MIN, jump to ok
        e.emit_branch_ne_imm_to_label(lhs_reg, i32::MIN, ok_label);
    } else {
        // INT_MIN_64 = 0x8000000000000000
        // Load INT_MIN_64 to SCRATCH1
        e.emit(Instruction::LoadImm64 {
            reg: SCRATCH1,
            value: i64::MIN as u64,
        });
        // Check if lhs != SCRATCH1
        // Xor lhs, SCRATCH1 -> SCRATCH1 (clobbers SCRATCH1)
        e.emit(Instruction::Xor {
            dst: SCRATCH1,
            src1: lhs_reg,
            src2: SCRATCH1,
        });
        // If result != 0, then lhs != INT_MIN
        e.emit_branch_ne_imm_to_label(SCRATCH1, 0, ok_label);
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
            emit_div_zero_check(e, TEMP2);
            e.emit(Instruction::DivU32 {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        }
        (BinaryOp::UDiv, false) => {
            emit_div_zero_check(e, TEMP2);
            e.emit(Instruction::DivU64 {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        }
        (BinaryOp::SDiv, true) => {
            emit_div_zero_check(e, TEMP2);
            emit_signed_overflow_check(e, TEMP1, TEMP2, true);
            e.emit(Instruction::DivS32 {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        }
        (BinaryOp::SDiv, false) => {
            emit_div_zero_check(e, TEMP2);
            emit_signed_overflow_check(e, TEMP1, TEMP2, false);
            e.emit(Instruction::DivS64 {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        }
        (BinaryOp::URem, true) => {
            emit_div_zero_check(e, TEMP2);
            e.emit(Instruction::RemU32 {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        }
        (BinaryOp::URem, false) => {
            emit_div_zero_check(e, TEMP2);
            e.emit(Instruction::RemU64 {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        }
        (BinaryOp::SRem, true) => {
            emit_div_zero_check(e, TEMP2);
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
            emit_div_zero_check(e, TEMP2);
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

pub fn lower_icmp<'ctx>(e: &mut PvmEmitter<'ctx>, instr: InstructionValue<'ctx>) -> Result<()> {
    let lhs = get_operand(instr, 0)?;
    let rhs = get_operand(instr, 1)?;
    let slot = result_slot(e, instr)?;

    let pred = instr
        .get_icmp_predicate()
        .ok_or_else(|| crate::Error::Internal("ICmp without predicate".into()))?;

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
    let from_bits = operand_bit_width(instr);

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
    let from_bits = operand_bit_width(instr);

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
