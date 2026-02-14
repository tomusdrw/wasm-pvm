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
    PvmLoadKind, PvmStoreKind, emit_pvm_load, emit_pvm_memory_copy, emit_pvm_memory_fill,
    emit_pvm_memory_grow, emit_pvm_memory_size, emit_pvm_store,
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

        // ── Indirect calls ──
        "__pvm_call_indirect" => super::calls::lower_pvm_call_indirect(e, instr, ctx),

        _ => Err(crate::Error::Unsupported(format!(
            "unknown PVM intrinsic: {name}"
        ))),
    }
}

/// Lower an LLVM intrinsic call (ctlz, cttz, ctpop, fshl, fshr).
pub fn lower_llvm_intrinsic<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    name: &str,
) -> Result<()> {
    use crate::abi::{SCRATCH1, SCRATCH2};

    let slot = result_slot(e, instr)?;

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
