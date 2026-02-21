// PVM and LLVM intrinsic function lowering.

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

use inkwell::values::InstructionValue;

use crate::Result;
use crate::pvm::Instruction;

use super::emitter::{LoweringContext, PvmEmitter, get_operand, operand_bit_width, result_slot};
use super::memory::{
    PvmLoadKind, PvmStoreKind, emit_pvm_data_drop, emit_pvm_load, emit_pvm_memory_copy,
    emit_pvm_memory_fill, emit_pvm_memory_grow, emit_pvm_memory_init, emit_pvm_memory_size,
    emit_pvm_store,
};
use crate::abi::{TEMP_RESULT, TEMP1, TEMP2};

/// Lower a PVM intrinsic call.
pub fn lower_pvm_intrinsic<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    name: &str,
    ctx: &LoweringContext,
) -> Result<()> {
    match name {
        // ── Loads ──
        "__pvm_load_i32" | "__pvm_load_i32u_64" => emit_pvm_load(e, instr, ctx, PvmLoadKind::U32),
        "__pvm_load_i64" => emit_pvm_load(e, instr, ctx, PvmLoadKind::U64),
        "__pvm_load_i8u" | "__pvm_load_i8u_64" => emit_pvm_load(e, instr, ctx, PvmLoadKind::U8),
        "__pvm_load_i8s" | "__pvm_load_i8s_64" => emit_pvm_load(e, instr, ctx, PvmLoadKind::I8),
        "__pvm_load_i16u" | "__pvm_load_i16u_64" => emit_pvm_load(e, instr, ctx, PvmLoadKind::U16),
        "__pvm_load_i16s" | "__pvm_load_i16s_64" => emit_pvm_load(e, instr, ctx, PvmLoadKind::I16),
        "__pvm_load_i32s_64" => emit_pvm_load(e, instr, ctx, PvmLoadKind::I32S),

        // ── Stores ──
        "__pvm_store_i32" | "__pvm_store_i32_64" => {
            emit_pvm_store(e, instr, ctx, PvmStoreKind::U32)
        }
        "__pvm_store_i64" => emit_pvm_store(e, instr, ctx, PvmStoreKind::U64),
        "__pvm_store_i8" | "__pvm_store_i8_64" => emit_pvm_store(e, instr, ctx, PvmStoreKind::U8),
        "__pvm_store_i16" | "__pvm_store_i16_64" => {
            emit_pvm_store(e, instr, ctx, PvmStoreKind::U16)
        }

        // ── Memory management ──
        "__pvm_memory_size" => emit_pvm_memory_size(e, instr, ctx),
        "__pvm_memory_grow" => emit_pvm_memory_grow(e, instr, ctx),
        "__pvm_memory_fill" => emit_pvm_memory_fill(e, instr, ctx),
        "__pvm_memory_copy" => emit_pvm_memory_copy(e, instr, ctx),
        "__pvm_memory_init" => emit_pvm_memory_init(e, instr, ctx),
        "__pvm_data_drop" => emit_pvm_data_drop(e, instr, ctx),

        // ── Indirect calls ──
        "__pvm_call_indirect" => super::calls::lower_pvm_call_indirect(e, instr, ctx),

        _ => Err(crate::Error::Unsupported(format!(
            "unknown PVM intrinsic: {name}"
        ))),
    }
}

/// Lower an LLVM intrinsic call (ctlz, cttz, ctpop, fshl, fshr, assume).
pub fn lower_llvm_intrinsic<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    name: &str,
) -> Result<()> {
    use crate::abi::{SCRATCH1, SCRATCH2};

    // llvm.assume is a no-op hint inserted by instcombine; safe to ignore.
    if name.contains("assume") {
        return Ok(());
    }

    let slot = result_slot(e, instr)?;

    // llvm.smax / llvm.smin / llvm.umax / llvm.umin — integer min/max intrinsics.
    // Lowered as: compare + conditional select via branch.
    if name.contains("smax")
        || name.contains("smin")
        || name.contains("umax")
        || name.contains("umin")
    {
        let a = get_operand(instr, 0)?;
        let b = get_operand(instr, 1)?;
        e.load_operand(a, TEMP1)?;
        e.load_operand(b, TEMP2)?;

        // Determine which comparison to use and which operand is the "winner".
        // For smax: result = a >= b ? a : b → if a < b (signed), result = b, else a
        // For smin: result = a <= b ? a : b → if b < a (signed), result = b, else a
        // For umax: result = a >= b ? a : b → if a < b (unsigned), result = b, else a
        // For umin: result = a <= b ? a : b → if b < a (unsigned), result = b, else a
        //
        // Strategy: SetLt(S/U) dst, src1, src2 → dst = (src1 < src2) ? 1 : 0
        // Then branch on result to select.
        let is_signed = name.contains("smax") || name.contains("smin");
        let is_max = name.contains("max");
        let bits = operand_bit_width(instr);

        // For i32 signed comparisons, load_operand zero-extends, so negative
        // i32 values appear as large positive i64 values. We must sign-extend
        // before using SetLtS. AddImm32 with value 0 sign-extends in PVM.
        if is_signed && bits == 32 {
            e.emit(Instruction::AddImm32 {
                dst: TEMP1,
                src: TEMP1,
                value: 0,
            });
            e.emit(Instruction::AddImm32 {
                dst: TEMP2,
                src: TEMP2,
                value: 0,
            });
        }

        // For max: SetLt(a, b) → 1 if a < b → pick b; else pick a
        // For min: SetLt(b, a) → 1 if b < a → pick b; else pick a (which is smaller)
        // Wait, for min: we want the smaller one. SetLt(a, b) → 1 if a < b → pick a; else pick b
        // Let's use: SetLt(a, b) and then:
        //   max: if a < b → result = b, else result = a
        //   min: if a < b → result = a, else result = b

        if is_signed {
            e.emit(Instruction::SetLtS {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        } else {
            e.emit(Instruction::SetLtU {
                dst: TEMP_RESULT,
                src1: TEMP1,
                src2: TEMP2,
            });
        }

        // TEMP_RESULT = 1 if a < b, 0 otherwise
        let done_label = e.alloc_label();
        let pick_second_label = e.alloc_label();

        // Branch if a < b (TEMP_RESULT != 0)
        e.emit_branch_ne_imm_to_label(TEMP_RESULT, 0, pick_second_label);

        if is_max {
            // a >= b: pick a
            e.store_to_slot(slot, TEMP1);
        } else {
            // a >= b: pick b (for min)
            e.store_to_slot(slot, TEMP2);
        }
        e.emit_jump_to_label(done_label);

        e.define_label(pick_second_label);
        if is_max {
            // a < b: pick b
            e.store_to_slot(slot, TEMP2);
        } else {
            // a < b: pick a (for min)
            e.store_to_slot(slot, TEMP1);
        }

        e.define_label(done_label);
        return Ok(());
    }

    // llvm.bswap — byte swap (reverse byte order).
    if name.contains("bswap") {
        let val = get_operand(instr, 0)?;
        e.load_operand(val, TEMP1)?;
        let bits = operand_bit_width(instr);

        // We'll build the result byte-by-byte using shifts and masks.
        // Use TEMP_RESULT as accumulator, TEMP2 as scratch for each byte.
        e.emit(Instruction::LoadImm {
            reg: TEMP_RESULT,
            value: 0,
        });

        if bits == 16 {
            // i16: swap 2 bytes
            emit_extract_and_place_byte(e, TEMP1, TEMP2, SCRATCH1, TEMP_RESULT, 0, 8);
            emit_extract_and_place_byte(e, TEMP1, TEMP2, SCRATCH1, TEMP_RESULT, 8, 0);
        } else if bits == 32 {
            // Byte 0 (bits 0-7) → bits 24-31
            emit_extract_and_place_byte(e, TEMP1, TEMP2, SCRATCH1, TEMP_RESULT, 0, 24);
            // Byte 1 (bits 8-15) → bits 16-23
            emit_extract_and_place_byte(e, TEMP1, TEMP2, SCRATCH1, TEMP_RESULT, 8, 16);
            // Byte 2 (bits 16-23) → bits 8-15
            emit_extract_and_place_byte(e, TEMP1, TEMP2, SCRATCH1, TEMP_RESULT, 16, 8);
            // Byte 3 (bits 24-31) → bits 0-7
            emit_extract_and_place_byte(e, TEMP1, TEMP2, SCRATCH1, TEMP_RESULT, 24, 0);
        } else if bits == 64 {
            // i64: 8 bytes to swap
            for i in 0..8u32 {
                let src_shift = i * 8;
                let dst_shift = (7 - i) * 8;
                emit_extract_and_place_byte(
                    e,
                    TEMP1,
                    TEMP2,
                    SCRATCH1,
                    TEMP_RESULT,
                    src_shift,
                    dst_shift,
                );
            }
        } else {
            return Err(crate::Error::Unsupported(format!(
                "bswap with unsupported bit width: {bits}"
            )));
        }

        e.store_to_slot(slot, TEMP_RESULT);
        return Ok(());
    }

    // llvm.abs — absolute value intrinsic.
    // Lowered as: if x < 0 then 0 - x else x.
    if name.contains("abs") {
        let val = get_operand(instr, 0)?;
        e.load_operand(val, TEMP1)?;
        let bits = operand_bit_width(instr);

        // For i32, sign-extend to i64 for correct signed comparison.
        if bits == 32 {
            e.emit(Instruction::AddImm32 {
                dst: TEMP1,
                src: TEMP1,
                value: 0,
            });
        }

        // Branch if TEMP1 >= 0 (signed): skip negation.
        let done_label = e.alloc_label();
        let nonneg_label = e.alloc_label();
        e.emit(Instruction::LoadImm {
            reg: TEMP2,
            value: 0,
        });
        // BranchGeS { reg1, reg2 } branches if reg2 >= reg1 (signed).
        e.emit_branch_ge_s_to_label(TEMP2, TEMP1, nonneg_label);

        // Negative path: result = 0 - x.
        if bits == 32 {
            e.emit(Instruction::Sub32 {
                dst: TEMP_RESULT,
                src1: TEMP2,
                src2: TEMP1,
            });
        } else {
            e.emit(Instruction::Sub64 {
                dst: TEMP_RESULT,
                src1: TEMP2,
                src2: TEMP1,
            });
        }
        e.store_to_slot(slot, TEMP_RESULT);
        e.emit_jump_to_label(done_label);

        // Non-negative path: result = x.
        e.define_label(nonneg_label);
        e.store_to_slot(slot, TEMP1);

        e.define_label(done_label);
        return Ok(());
    }

    if name.contains("ctlz") {
        let val = get_operand(instr, 0)?;
        e.load_operand(val, TEMP1)?;
        let bits = operand_bit_width(instr);
        if bits == 32 {
            e.emit(Instruction::LeadingZeroBits32 {
                dst: TEMP_RESULT,
                src: TEMP1,
            });
        } else {
            e.emit(Instruction::LeadingZeroBits64 {
                dst: TEMP_RESULT,
                src: TEMP1,
            });
        }
        e.store_to_slot(slot, TEMP_RESULT);
        return Ok(());
    }

    if name.contains("cttz") {
        let val = get_operand(instr, 0)?;
        e.load_operand(val, TEMP1)?;
        let bits = operand_bit_width(instr);
        if bits == 32 {
            e.emit(Instruction::TrailingZeroBits32 {
                dst: TEMP_RESULT,
                src: TEMP1,
            });
        } else {
            e.emit(Instruction::TrailingZeroBits64 {
                dst: TEMP_RESULT,
                src: TEMP1,
            });
        }
        e.store_to_slot(slot, TEMP_RESULT);
        return Ok(());
    }

    if name.contains("ctpop") {
        let val = get_operand(instr, 0)?;
        e.load_operand(val, TEMP1)?;
        let bits = operand_bit_width(instr);
        if bits == 32 {
            e.emit(Instruction::CountSetBits32 {
                dst: TEMP_RESULT,
                src: TEMP1,
            });
        } else {
            e.emit(Instruction::CountSetBits64 {
                dst: TEMP_RESULT,
                src: TEMP1,
            });
        }
        e.store_to_slot(slot, TEMP_RESULT);
        return Ok(());
    }

    if name.contains("fshl") || name.contains("fshr") {
        // Funnel shift: fshl(a, b, amt) = (a << amt) | (b >> (bits - amt))
        //               fshr(a, b, amt) = (a << (bits - amt)) | (b >> amt)
        // For rotation (a == b): fshl = rotl, fshr = rotr
        let a = get_operand(instr, 0)?;
        let b = get_operand(instr, 1)?;
        let amt = get_operand(instr, 2)?;
        let bits = operand_bit_width(instr);
        let is_32 = bits == 32;

        e.load_operand(a, TEMP1)?;
        e.load_operand(b, TEMP2)?;
        e.load_operand(amt, SCRATCH1)?;

        // Mask shift amount to valid range.
        e.emit(Instruction::LoadImm {
            reg: SCRATCH2,
            value: (bits - 1) as i32,
        });
        e.emit(Instruction::And {
            dst: SCRATCH1,
            src1: SCRATCH1,
            src2: SCRATCH2,
        });

        // Compute bits - amt into SCRATCH2.
        e.emit(Instruction::LoadImm {
            reg: SCRATCH2,
            value: bits as i32,
        });
        if is_32 {
            e.emit(Instruction::Sub32 {
                dst: SCRATCH2,
                src1: SCRATCH2,
                src2: SCRATCH1,
            });
        } else {
            e.emit(Instruction::Sub64 {
                dst: SCRATCH2,
                src1: SCRATCH2,
                src2: SCRATCH1,
            });
        }

        if name.contains("fshl") {
            // (a << amt) | (b >> (bits - amt))
            if is_32 {
                e.emit(Instruction::ShloL32 {
                    dst: TEMP1,
                    src1: TEMP1,
                    src2: SCRATCH1,
                });
                e.emit(Instruction::ShloR32 {
                    dst: TEMP2,
                    src1: TEMP2,
                    src2: SCRATCH2,
                });
            } else {
                e.emit(Instruction::ShloL64 {
                    dst: TEMP1,
                    src1: TEMP1,
                    src2: SCRATCH1,
                });
                e.emit(Instruction::ShloR64 {
                    dst: TEMP2,
                    src1: TEMP2,
                    src2: SCRATCH2,
                });
            }
        } else {
            // (a << (bits - amt)) | (b >> amt)
            if is_32 {
                e.emit(Instruction::ShloL32 {
                    dst: TEMP1,
                    src1: TEMP1,
                    src2: SCRATCH2,
                });
                e.emit(Instruction::ShloR32 {
                    dst: TEMP2,
                    src1: TEMP2,
                    src2: SCRATCH1,
                });
            } else {
                e.emit(Instruction::ShloL64 {
                    dst: TEMP1,
                    src1: TEMP1,
                    src2: SCRATCH2,
                });
                e.emit(Instruction::ShloR64 {
                    dst: TEMP2,
                    src1: TEMP2,
                    src2: SCRATCH1,
                });
            }
        }

        e.emit(Instruction::Or {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        });

        e.store_to_slot(slot, TEMP_RESULT);
        return Ok(());
    }

    Err(crate::Error::Unsupported(format!(
        "unsupported LLVM intrinsic: {name}"
    )))
}

/// Extract a byte at `src_shift` bit position from `src_reg`, shift it to `dst_shift`,
/// and OR it into `acc_reg`. Uses `tmp_reg` and `mask_reg` as scratch.
fn emit_extract_and_place_byte(
    e: &mut PvmEmitter<'_>,
    src_reg: u8,
    tmp_reg: u8,
    mask_reg: u8,
    acc_reg: u8,
    src_shift: u32,
    dst_shift: u32,
) {
    // tmp = src >> src_shift
    if src_shift > 0 {
        e.emit(Instruction::LoadImm {
            reg: mask_reg,
            value: src_shift as i32,
        });
        e.emit(Instruction::ShloR64 {
            dst: tmp_reg,
            src1: src_reg,
            src2: mask_reg,
        });
    } else {
        // Just copy
        e.emit(Instruction::AddImm64 {
            dst: tmp_reg,
            src: src_reg,
            value: 0,
        });
    }
    // tmp &= 0xFF
    e.emit(Instruction::LoadImm {
        reg: mask_reg,
        value: 0xFF,
    });
    e.emit(Instruction::And {
        dst: tmp_reg,
        src1: tmp_reg,
        src2: mask_reg,
    });
    // tmp <<= dst_shift
    if dst_shift > 0 {
        e.emit(Instruction::LoadImm {
            reg: mask_reg,
            value: dst_shift as i32,
        });
        e.emit(Instruction::ShloL64 {
            dst: tmp_reg,
            src1: tmp_reg,
            src2: mask_reg,
        });
    }
    // acc |= tmp
    e.emit(Instruction::Or {
        dst: acc_reg,
        src1: acc_reg,
        src2: tmp_reg,
    });
}
