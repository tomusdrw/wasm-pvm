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

use super::emitter::{
    LoweringContext, PvmEmitter, get_operand, operand_bit_width, operand_reg, operand_reg_avoiding,
    prepare_operand, prepare_operand_avoiding, result_reg, result_slot, try_get_constant,
    val_key_basic,
};
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
        // memory_size doesn't use SCRATCH1/SCRATCH2, no spill needed.
        "__pvm_memory_size" => emit_pvm_memory_size(e, instr, ctx),
        // These intrinsics use abi::SCRATCH1/SCRATCH2 (r5/r6) internally.
        // Spill/reload register-allocated values around them.
        "__pvm_memory_grow" => {
            e.spill_allocated_regs();
            let result = emit_pvm_memory_grow(e, instr, ctx);
            e.reload_allocated_regs_after_scratch_clobber();
            result
        }
        "__pvm_memory_fill" => {
            e.spill_allocated_regs();
            let result = emit_pvm_memory_fill(e, instr, ctx);
            e.reload_allocated_regs_after_scratch_clobber();
            result
        }
        "__pvm_memory_copy" => {
            e.spill_allocated_regs();
            let result = emit_pvm_memory_copy(e, instr, ctx);
            e.reload_allocated_regs_after_scratch_clobber();
            result
        }
        "__pvm_memory_init" => {
            e.spill_allocated_regs();
            let result = emit_pvm_memory_init(e, instr, ctx);
            e.reload_allocated_regs_after_scratch_clobber();
            result
        }
        "__pvm_data_drop" => emit_pvm_data_drop(e, instr, ctx),

        // ── Wide multiply ──
        // Upper 64 bits of unsigned 64×64→128 product. Used by the synthesized
        // `__multi3` body in `llvm_frontend::libcall_recognition`.
        "__pvm_mul_upper_uu" => emit_pvm_mul_upper_uu(e, instr),

        // ── Indirect calls ──
        "__pvm_call_indirect" => super::calls::lower_pvm_call_indirect(e, instr, ctx),

        _ => Err(crate::Error::Unsupported(format!(
            "unknown PVM intrinsic: {name}"
        ))),
    }
}

/// Emit `MulUpperUU dst, a, b` for `__pvm_mul_upper_uu(a, b)`.
///
/// Mirrors the operand-coalescing dance of `Mul64` in `alu.rs`: prefer each
/// operand's allocated register, fall back to `TEMP1`/`TEMP2` when the alias
/// would clobber an allocated dst. The `prepare_operand_avoiding` helper
/// keeps `dst == TEMP_RESULT` aliases intact (PVM 3-operand instructions
/// read both sources before writing dst) and avoids returning the sibling
/// operand's load-target register.
fn emit_pvm_mul_upper_uu<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
) -> Result<()> {
    let a = get_operand(instr, 0)?;
    let b = get_operand(instr, 1)?;
    let slot = result_slot(e, instr)?;
    let dst = result_reg(e, instr);

    // Use `prepare_operand_avoiding` for both: if `a` is cached at TEMP2, the
    // sibling `b` load below would clobber it (and vice versa). Original code
    // used plain `operand_reg`, a latent miscompile on the rare path where the
    // per-block cache put one operand in the other's load-target register.
    let a_reg = prepare_operand_avoiding(e, a, TEMP1, &[TEMP2], dst)?;
    let b_reg = prepare_operand_avoiding(e, b, TEMP2, &[TEMP1], dst)?;

    e.emit(Instruction::MulUpperUU {
        dst,
        src1: a_reg,
        src2: b_reg,
    });
    e.store_to_slot(slot, dst);
    Ok(())
}

/// Lower an LLVM intrinsic call (smax, smin, umax, umin, bitreverse, bswap, abs, ctlz, cttz, ctpop, fshl, fshr, assume).
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

    // llvm.trap — emit a real trap. Used by `--trap-floats` mode in place of
    // bare `unreachable` (which simplifycfg folds away as UB).
    if name == "llvm.trap" {
        e.emit(Instruction::Trap);
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
        let dst = result_reg(e, instr);

        let is_signed = name.contains("smax") || name.contains("smin");
        let is_max = name.contains("max");
        let bits = operand_bit_width(instr);

        // For i32 signed operations, sign-extend operands. Use operand_reg for the
        // source of the sign extension, but the min/max itself uses TEMP1/TEMP2
        // (which hold the sign-extended values).
        let (a_reg, b_reg) = if is_signed && bits == 32 {
            let a_src = operand_reg_avoiding(e, a, TEMP1, &[TEMP2]);
            let b_src = operand_reg_avoiding(e, b, TEMP2, &[TEMP1]);
            if a_src == TEMP1 {
                e.load_operand(a, TEMP1)?;
            }
            if b_src == TEMP2 {
                e.load_operand(b, TEMP2)?;
            }
            e.emit(Instruction::AddImm32 {
                dst: TEMP1,
                src: a_src,
                value: 0,
            });
            e.emit(Instruction::AddImm32 {
                dst: TEMP2,
                src: b_src,
                value: 0,
            });
            (TEMP1, TEMP2)
        } else {
            let a_reg = prepare_operand_avoiding(e, a, TEMP1, &[TEMP2], dst)?;
            let b_reg = prepare_operand_avoiding(e, b, TEMP2, &[TEMP1], dst)?;
            (a_reg, b_reg)
        };

        match (is_max, is_signed) {
            (true, true) => e.emit(Instruction::Max {
                dst,
                src1: a_reg,
                src2: b_reg,
            }),
            (true, false) => e.emit(Instruction::MaxU {
                dst,
                src1: a_reg,
                src2: b_reg,
            }),
            (false, true) => e.emit(Instruction::Min {
                dst,
                src1: a_reg,
                src2: b_reg,
            }),
            (false, false) => e.emit(Instruction::MinU {
                dst,
                src1: a_reg,
                src2: b_reg,
            }),
        }

        e.store_to_slot(slot, dst);
        return Ok(());
    }

    // llvm.bitreverse — reverse bit order within the value.
    //
    // Distinct from llvm.bswap (which reverses byte order). PVM has no native
    // bit-reverse instruction, so this is implemented in software via the
    // standard "swap odd/even bits, then pairs, then nibbles, then bytes"
    // algorithm; the byte-swap step (i16/i32/i64) uses PVM's ReverseBytes.
    // Supports i8 (no byte-swap step), i16, i32, and i64.
    if name.contains("bitreverse") {
        let val = get_operand(instr, 0)?;
        let dst = result_reg(e, instr);
        let bits = operand_bit_width(instr);

        // Use TEMP1 as the running value across the three mask phases.
        // Phase 0 reads from the operand register (allocated reg, or TEMP1
        // after load_operand); phases 1+ read from TEMP1.
        //
        // Cannot use `apply_dst_conflict_fallback` here: the i64 bitreverse
        // path emits `LoadImm64 TEMP_RESULT, mask` mid-sequence and would
        // clobber `val_reg` if it equals `TEMP_RESULT`. The conservative
        // `val_reg != TEMP1 && val_reg == dst → TEMP1` fallback handles the
        // `dst == TEMP_RESULT` case correctly by forcing a fresh TEMP1 load.
        let mut val_reg = operand_reg(e, val, TEMP1);
        if val_reg != TEMP1 && val_reg == dst {
            val_reg = TEMP1;
        }
        if val_reg == TEMP1 {
            e.load_operand(val, TEMP1)?;
        }

        if bits == 8 || bits == 16 || bits == 32 {
            // i8/i16/i32 share one shape: 3 mask phases via `AndImm` +
            // `ShloLImm32`/`ShloRImm32` + `Or`. i8 skips the byte-swap
            // step because a single byte doesn't need one. All masks fit
            // in positive i32 (max 0x5555_5555 < 2^31). See
            // `docs/src/learnings.md` for the per-width recipe and the
            // 32-bit-shift sign-extension correctness note.
            let masks: &[i32] = match bits {
                8 => &[0x55, 0x33, 0x0F],
                16 => &[0x5555, 0x3333, 0x0F0F],
                32 => &[0x5555_5555, 0x3333_3333, 0x0F0F_0F0F],
                _ => unreachable!(),
            };
            for (i, &mask) in masks.iter().enumerate() {
                let shift = 1_i32 << i;
                let src = if i == 0 { val_reg } else { TEMP1 };

                e.emit(Instruction::AndImm {
                    dst: TEMP2,
                    src,
                    value: mask,
                });
                e.emit(Instruction::ShloLImm32 {
                    dst: TEMP2,
                    src: TEMP2,
                    value: shift,
                });
                e.emit(Instruction::ShloRImm32 {
                    dst: TEMP1,
                    src,
                    value: shift,
                });
                e.emit(Instruction::AndImm {
                    dst: TEMP1,
                    src: TEMP1,
                    value: mask,
                });
                e.emit(Instruction::Or {
                    dst: TEMP1,
                    src1: TEMP1,
                    src2: TEMP2,
                });
            }

            if bits == 8 {
                // No byte-swap step for i8; TEMP1 already holds the
                // bit-reversed value in its low 8 bits with upper bits zero.
                // Store directly from TEMP1 to skip a redundant MoveReg
                // when `dst == TEMP_RESULT` and there's no allocated reg.
                e.store_to_slot(slot, TEMP1);
                return Ok(());
            }

            // i16/i32: ReverseBytes leaves the result in the top `bits` bits
            // of the 64-bit register; shift right by (64 - bits) to recover
            // (mirrors the bswap path).
            e.emit(Instruction::ReverseBytes { dst, src: TEMP1 });
            e.emit(Instruction::ShloRImm64 {
                dst,
                src: dst,
                value: if bits == 16 { 48 } else { 32 },
            });
        } else if bits == 64 {
            // 64-bit masks don't fit in AndImm's i32 immediate; materialize
            // each into TEMP_RESULT via LoadImm64 and use the register-form
            // And.
            for (i, &mask) in [
                0x5555_5555_5555_5555_u64,
                0x3333_3333_3333_3333_u64,
                0x0F0F_0F0F_0F0F_0F0F_u64,
            ]
            .iter()
            .enumerate()
            {
                let shift = 1_i32 << i;
                let src = if i == 0 { val_reg } else { TEMP1 };

                e.emit(Instruction::LoadImm64 {
                    reg: TEMP_RESULT,
                    value: mask,
                });
                e.emit(Instruction::And {
                    dst: TEMP2,
                    src1: src,
                    src2: TEMP_RESULT,
                });
                e.emit(Instruction::ShloLImm64 {
                    dst: TEMP2,
                    src: TEMP2,
                    value: shift,
                });
                e.emit(Instruction::ShloRImm64 {
                    dst: TEMP1,
                    src,
                    value: shift,
                });
                e.emit(Instruction::And {
                    dst: TEMP1,
                    src1: TEMP1,
                    src2: TEMP_RESULT,
                });
                e.emit(Instruction::Or {
                    dst: TEMP1,
                    src1: TEMP1,
                    src2: TEMP2,
                });
            }

            // i64 byte-reverse needs no post-shift.
            e.emit(Instruction::ReverseBytes { dst, src: TEMP1 });
        } else {
            return Err(crate::Error::Unsupported(format!(
                "bitreverse with unsupported bit width: {bits}"
            )));
        }

        e.store_to_slot(slot, dst);
        return Ok(());
    }

    // llvm.bswap — byte swap (reverse byte order).
    if name.contains("bswap") {
        let val = get_operand(instr, 0)?;
        let dst = result_reg(e, instr);
        let val_reg = prepare_operand(e, val, TEMP1, dst)?;
        let bits = operand_bit_width(instr);

        // Validate the width and choose the post-shift up front. PVM's
        // `ReverseBytes` is always a 64-bit byte reversal, so for narrower
        // widths we shift the result down from the high end of the register.
        let post_shift = match bits {
            16 => 48,
            32 => 32,
            64 => 0,
            _ => {
                return Err(crate::Error::Unsupported(format!(
                    "bswap with unsupported bit width: {bits}"
                )));
            }
        };

        e.emit(Instruction::ReverseBytes { dst, src: val_reg });
        if post_shift > 0 {
            e.emit(Instruction::ShloRImm64 {
                dst,
                src: dst,
                value: post_shift,
            });
        }

        e.store_to_slot(slot, dst);
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
        let dst = result_reg(e, instr);
        let val_reg = prepare_operand(e, val, TEMP1, dst)?;
        let bits = operand_bit_width(instr);
        if bits == 32 {
            e.emit(Instruction::LeadingZeroBits32 { dst, src: val_reg });
        } else {
            e.emit(Instruction::LeadingZeroBits64 { dst, src: val_reg });
        }
        e.store_to_slot(slot, dst);
        return Ok(());
    }

    if name.contains("cttz") {
        let val = get_operand(instr, 0)?;
        let dst = result_reg(e, instr);
        let val_reg = prepare_operand(e, val, TEMP1, dst)?;
        let bits = operand_bit_width(instr);
        if bits == 32 {
            e.emit(Instruction::TrailingZeroBits32 { dst, src: val_reg });
        } else {
            e.emit(Instruction::TrailingZeroBits64 { dst, src: val_reg });
        }
        e.store_to_slot(slot, dst);
        return Ok(());
    }

    if name.contains("ctpop") {
        let val = get_operand(instr, 0)?;
        let dst = result_reg(e, instr);
        let val_reg = prepare_operand(e, val, TEMP1, dst)?;
        let bits = operand_bit_width(instr);
        if bits == 32 {
            e.emit(Instruction::CountSetBits32 { dst, src: val_reg });
        } else {
            e.emit(Instruction::CountSetBits64 { dst, src: val_reg });
        }
        e.store_to_slot(slot, dst);
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

        // Rotation detection: when a and b are the same SSA value, use RotL/RotR.
        if val_key_basic(a) == val_key_basic(b) {
            let dst = result_reg(e, instr);
            // `amt_reg` is resolved lazily below (only on the variable-amount
            // path), so `a_reg` only needs to avoid TEMP2 — the future load
            // target for `amt` if the constant fast-path doesn't fire.
            let a_reg = prepare_operand_avoiding(e, a, TEMP1, &[TEMP2], dst)?;

            // Constant rotation amount: use the *Imm variants and drop the
            // `LoadImm` for the amount + the TEMP2 dependency. PVM has no
            // `RotLImm*` opcodes, so rewrite `rotl(x, n)` as `rotr(x, bits-n)`
            // when n is known (the two are equivalent modulo the rotation width).
            if let Some(amt_const) = try_get_constant(amt) {
                let masked = (amt_const as u64) & u64::from(bits - 1);
                let is_rotr = name.contains("fshr");
                let rotr_amt = if is_rotr {
                    masked
                } else {
                    (u64::from(bits) - masked) & u64::from(bits - 1)
                };
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                let rotr_amt_i32 = rotr_amt as i32;
                let rot = if is_32 {
                    Instruction::RotRImm32 {
                        dst,
                        src: a_reg,
                        value: rotr_amt_i32,
                    }
                } else {
                    Instruction::RotRImm64 {
                        dst,
                        src: a_reg,
                        value: rotr_amt_i32,
                    }
                };
                e.emit(rot);
                e.store_to_slot(slot, dst);
                return Ok(());
            }

            // Variable rotation amount: use the register-form rotates.
            let amt_reg = prepare_operand_avoiding(e, amt, TEMP2, &[TEMP1], dst)?;
            let rot = if name.contains("fshl") {
                if is_32 {
                    Instruction::RotL32 {
                        dst,
                        src1: a_reg,
                        src2: amt_reg,
                    }
                } else {
                    Instruction::RotL64 {
                        dst,
                        src1: a_reg,
                        src2: amt_reg,
                    }
                }
            } else if is_32 {
                Instruction::RotR32 {
                    dst,
                    src1: a_reg,
                    src2: amt_reg,
                }
            } else {
                Instruction::RotR64 {
                    dst,
                    src1: a_reg,
                    src2: amt_reg,
                }
            };
            e.emit(rot);
            e.store_to_slot(slot, dst);
            return Ok(());
        }

        // Non-rotation funnel shift uses abi::SCRATCH1/SCRATCH2 (r5/r6).
        // Spill any register-allocated values in those registers.
        e.spill_allocated_regs();

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

        let dst = result_reg(e, instr);
        e.emit(Instruction::Or {
            dst,
            src1: TEMP1,
            src2: TEMP2,
        });

        e.store_to_slot(slot, dst);

        // Reload register-allocated values after using SCRATCH1/SCRATCH2.
        e.reload_allocated_regs_after_scratch_clobber();

        return Ok(());
    }

    // Saturating arithmetic — see lower_{u,s}{add,sub}_sat helpers below.
    if name.contains("usub.sat") {
        return lower_usub_sat(e, instr);
    }
    if name.contains("uadd.sat") {
        return lower_uadd_sat(e, instr);
    }
    if name.contains("ssub.sat") {
        return lower_ssub_sat(e, instr);
    }
    if name.contains("sadd.sat") {
        return lower_sadd_sat(e, instr);
    }

    Err(crate::Error::Unsupported(format!(
        "unsupported LLVM intrinsic: {name}"
    )))
}

/// Lower `@llvm.usub.sat.iN(a, b)`.
///
/// Result form: zero-extended (`u8`/`u16`/`u32`/`u64`).
/// Algorithm: `result = (a < b) ? 0 : a - b` with unsigned compare.
///   - i8/i16: `AndImm` mask, `SetLtU`, `Sub64`, `CmovNzImm dst, cond, 0`
///   - i32: shift-shift zero-extend (mask `0xFFFFFFFF` doesn't fit `AndImm`),
///     then same shape
///   - i64: `SetLtU`, `Sub64`, `CmovNzImm dst, cond, 0`
fn lower_usub_sat<'ctx>(e: &mut PvmEmitter<'ctx>, instr: InstructionValue<'ctx>) -> Result<()> {
    let slot = result_slot(e, instr)?;
    let a = get_operand(instr, 0)?;
    let b = get_operand(instr, 1)?;
    let dst = result_reg(e, instr);
    let bits = operand_bit_width(instr);

    // Force-load both operands into TEMP1/TEMP2 so we can clobber freely.
    e.load_operand(a, TEMP1)?;
    e.load_operand(b, TEMP2)?;

    match bits {
        8 => {
            e.emit(Instruction::AndImm {
                dst: TEMP1,
                src: TEMP1,
                value: 0xFF,
            });
            e.emit(Instruction::AndImm {
                dst: TEMP2,
                src: TEMP2,
                value: 0xFF,
            });
        }
        16 => {
            e.emit(Instruction::AndImm {
                dst: TEMP1,
                src: TEMP1,
                value: 0xFFFF,
            });
            e.emit(Instruction::AndImm {
                dst: TEMP2,
                src: TEMP2,
                value: 0xFFFF,
            });
        }
        32 => {
            // Zero-extend via shl 32 + shr 32 (0xFFFFFFFF doesn't fit AndImm).
            e.emit(Instruction::ShloLImm64 {
                dst: TEMP1,
                src: TEMP1,
                value: 32,
            });
            e.emit(Instruction::ShloRImm64 {
                dst: TEMP1,
                src: TEMP1,
                value: 32,
            });
            e.emit(Instruction::ShloLImm64 {
                dst: TEMP2,
                src: TEMP2,
                value: 32,
            });
            e.emit(Instruction::ShloRImm64 {
                dst: TEMP2,
                src: TEMP2,
                value: 32,
            });
        }
        64 => {
            // Operands are already canonical i64; no masking needed.
        }
        _ => {
            return Err(crate::Error::Unsupported(format!(
                "usub.sat with unsupported bit width: {bits}"
            )));
        }
    }

    // Compute result = a - b, then cond = (a < b unsigned), then zero dst if cond.
    //
    // Ordering is critical: `dst` may alias `TEMP_RESULT` (the fallback when no
    // register is allocated for this instruction's result).  If we emitted
    // `SetLtU { dst: TEMP_RESULT }` first and then `Sub64 { dst: TEMP_RESULT }`,
    // the subtraction would silently overwrite the condition before `CmovNzImm`
    // reads it, producing the wrong result.
    //
    // Safe ordering:
    //   1. Sub64   → writes *dst* (may be TEMP_RESULT), leaves TEMP1/TEMP2 intact
    //   2. SetLtU  → writes TEMP1 (clobbers TEMP1=a, which we no longer need),
    //                reads TEMP2=b which is still valid
    //   3. CmovNzImm → reads *cond* from TEMP1 (never aliases dst), writes *dst*
    //
    // TEMP1 is safe to reuse for cond because `dst` is never TEMP1 — it is either
    // an allocator-assigned register or the TEMP_RESULT fallback (r4).
    e.emit(Instruction::Sub64 {
        dst,
        src1: TEMP1,
        src2: TEMP2,
    });
    e.emit(Instruction::SetLtU {
        dst: TEMP1,
        src1: TEMP1,
        src2: TEMP2,
    });
    e.emit(Instruction::CmovNzImm {
        dst,
        cond: TEMP1,
        value: 0,
    });

    e.store_to_slot(slot, dst);
    Ok(())
}

/// Lower `@llvm.uadd.sat.iN(a, b)`.
///
/// Result form: zero-extended.
/// Algorithm:
///   - i8/i16/i32: zero-extend operands, do 64-bit add (cannot overflow
///     because both fit in 32 bits), `MinU` against the width's UMAX.
///   - i64: `Add64`, detect wrap via `SetLtU sum, a`, `CmovNz` to `UINT64_MAX`.
fn lower_uadd_sat<'ctx>(e: &mut PvmEmitter<'ctx>, instr: InstructionValue<'ctx>) -> Result<()> {
    let slot = result_slot(e, instr)?;
    let a = get_operand(instr, 0)?;
    let b = get_operand(instr, 1)?;
    let dst = result_reg(e, instr);
    let bits = operand_bit_width(instr);

    e.load_operand(a, TEMP1)?;
    e.load_operand(b, TEMP2)?;

    // After `Add64 dst, TEMP1, TEMP2`, both TEMP1 and TEMP2 are free —
    // we use TEMP1 for the umax constant. Avoiding TEMP_RESULT here is
    // critical: `result_reg` may return TEMP_RESULT under register pressure,
    // so loading the constant into TEMP_RESULT would clobber `dst`'s sum.
    match bits {
        8 => {
            e.emit(Instruction::AndImm {
                dst: TEMP1,
                src: TEMP1,
                value: 0xFF,
            });
            e.emit(Instruction::AndImm {
                dst: TEMP2,
                src: TEMP2,
                value: 0xFF,
            });
            e.emit(Instruction::Add64 {
                dst,
                src1: TEMP1,
                src2: TEMP2,
            });
            e.emit(Instruction::LoadImm {
                reg: TEMP1,
                value: 0xFF,
            });
            e.emit(Instruction::MinU {
                dst,
                src1: dst,
                src2: TEMP1,
            });
        }
        16 => {
            e.emit(Instruction::AndImm {
                dst: TEMP1,
                src: TEMP1,
                value: 0xFFFF,
            });
            e.emit(Instruction::AndImm {
                dst: TEMP2,
                src: TEMP2,
                value: 0xFFFF,
            });
            e.emit(Instruction::Add64 {
                dst,
                src1: TEMP1,
                src2: TEMP2,
            });
            e.emit(Instruction::LoadImm {
                reg: TEMP1,
                value: 0xFFFF,
            });
            e.emit(Instruction::MinU {
                dst,
                src1: dst,
                src2: TEMP1,
            });
        }
        32 => {
            // Zero-extend via shl 32 + shr 32 (0xFFFFFFFF doesn't fit AndImm).
            e.emit(Instruction::ShloLImm64 {
                dst: TEMP1,
                src: TEMP1,
                value: 32,
            });
            e.emit(Instruction::ShloRImm64 {
                dst: TEMP1,
                src: TEMP1,
                value: 32,
            });
            e.emit(Instruction::ShloLImm64 {
                dst: TEMP2,
                src: TEMP2,
                value: 32,
            });
            e.emit(Instruction::ShloRImm64 {
                dst: TEMP2,
                src: TEMP2,
                value: 32,
            });
            e.emit(Instruction::Add64 {
                dst,
                src1: TEMP1,
                src2: TEMP2,
            });
            // umax = 0xFFFFFFFF: load -1 (all 1s) then logical-shift right 32.
            e.emit(Instruction::LoadImm {
                reg: TEMP1,
                value: -1,
            });
            e.emit(Instruction::ShloRImm64 {
                dst: TEMP1,
                src: TEMP1,
                value: 32,
            });
            e.emit(Instruction::MinU {
                dst,
                src1: dst,
                src2: TEMP1,
            });
        }
        64 => {
            e.emit(Instruction::Add64 {
                dst,
                src1: TEMP1,
                src2: TEMP2,
            });
            // Overflow iff sum < a (unsigned). Use TEMP2 for the flag (TEMP2
            // held `b`, no longer needed). Using TEMP_RESULT here would
            // clobber `dst` when `result_reg` returned TEMP_RESULT.
            e.emit(Instruction::SetLtU {
                dst: TEMP2,
                src1: dst,
                src2: TEMP1,
            });
            // umax_64 = -1 sign-extended = 0xFFFF_FFFF_FFFF_FFFF.
            e.emit(Instruction::LoadImm {
                reg: TEMP1,
                value: -1,
            });
            e.emit(Instruction::CmovNz {
                dst,
                src: TEMP1,
                cond: TEMP2,
            });
        }
        _ => {
            return Err(crate::Error::Unsupported(format!(
                "uadd.sat with unsupported bit width: {bits}"
            )));
        }
    }

    e.store_to_slot(slot, dst);
    Ok(())
}

/// Lower `@llvm.ssub.sat.iN(a, b)`.
///
/// Result form: sign-extended.
/// Algorithm:
///   - i8/i16/i32: sign-extend operands, do 64-bit subtract (true difference
///     fits in i64 because two iN values differ by at most 2^N), then clamp
///     to [`INT_MIN`, `INT_MAX`] via signed Max/Min.
///   - i64: Hacker's Delight overflow detection. Uses SCRATCH1/SCRATCH2,
///     so brackets the sequence with `spill_allocated_regs()` /
///     `reload_allocated_regs_after_scratch_clobber()`.
fn lower_ssub_sat<'ctx>(e: &mut PvmEmitter<'ctx>, instr: InstructionValue<'ctx>) -> Result<()> {
    use crate::abi::{SCRATCH1, SCRATCH2};

    let slot = result_slot(e, instr)?;
    let a = get_operand(instr, 0)?;
    let b = get_operand(instr, 1)?;
    let dst = result_reg(e, instr);
    let bits = operand_bit_width(instr);

    match bits {
        8 | 16 | 32 => {
            e.load_operand(a, TEMP1)?;
            e.load_operand(b, TEMP2)?;
            match bits {
                8 => {
                    e.emit(Instruction::SignExtend8 {
                        dst: TEMP1,
                        src: TEMP1,
                    });
                    e.emit(Instruction::SignExtend8 {
                        dst: TEMP2,
                        src: TEMP2,
                    });
                }
                16 => {
                    e.emit(Instruction::SignExtend16 {
                        dst: TEMP1,
                        src: TEMP1,
                    });
                    e.emit(Instruction::SignExtend16 {
                        dst: TEMP2,
                        src: TEMP2,
                    });
                }
                _ => {
                    // i32: AddImm32 _, _, 0 sign-extends low 32 bits to 64.
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
            }
            e.emit(Instruction::Sub64 {
                dst,
                src1: TEMP1,
                src2: TEMP2,
            });

            // After Sub64, both TEMP1 and TEMP2 are dead — use TEMP1 for the
            // imin/imax constants (NOT TEMP_RESULT, which may alias `dst`
            // when result_reg returns TEMP_RESULT under register pressure).
            let (imin, imax): (i32, i32) = match bits {
                8 => (-128, 127),
                16 => (-32_768, 32_767),
                _ => (i32::MIN, i32::MAX),
            };
            e.emit(Instruction::LoadImm {
                reg: TEMP1,
                value: imin,
            });
            e.emit(Instruction::Max {
                dst,
                src1: dst,
                src2: TEMP1,
            });
            e.emit(Instruction::LoadImm {
                reg: TEMP1,
                value: imax,
            });
            e.emit(Instruction::Min {
                dst,
                src1: dst,
                src2: TEMP1,
            });
        }
        64 => {
            // Uses SCRATCH1/SCRATCH2 — bracket with spill/reload.
            e.spill_allocated_regs();
            e.load_operand(a, TEMP1)?;
            e.load_operand(b, TEMP2)?;
            // sum = a - b (wrapping)
            e.emit(Instruction::Sub64 {
                dst,
                src1: TEMP1,
                src2: TEMP2,
            });
            // t1 = a ^ b ; t2 = a ^ sum ; t1 &= t2 ; cond = arith-shift t1 right 63
            e.emit(Instruction::Xor {
                dst: SCRATCH1,
                src1: TEMP1,
                src2: TEMP2,
            });
            e.emit(Instruction::Xor {
                dst: SCRATCH2,
                src1: TEMP1,
                src2: dst,
            });
            e.emit(Instruction::And {
                dst: SCRATCH1,
                src1: SCRATCH1,
                src2: SCRATCH2,
            });
            e.emit(Instruction::SharRImm64 {
                dst: SCRATCH1,
                src: SCRATCH1,
                value: 63,
            });
            // sign_a = arith-shift a right 63 (0 or -1)
            e.emit(Instruction::SharRImm64 {
                dst: SCRATCH2,
                src: TEMP1,
                value: 63,
            });
            // imax = INT64_MAX = (-1 logical-shift-right 1)
            e.emit(Instruction::LoadImm {
                reg: TEMP1,
                value: -1,
            });
            e.emit(Instruction::ShloRImm64 {
                dst: TEMP1,
                src: TEMP1,
                value: 1,
            });
            // sat = sign_a ^ INT64_MAX
            e.emit(Instruction::Xor {
                dst: SCRATCH2,
                src1: SCRATCH2,
                src2: TEMP1,
            });
            // if cond, dst = sat
            e.emit(Instruction::CmovNz {
                dst,
                src: SCRATCH2,
                cond: SCRATCH1,
            });
            e.reload_allocated_regs_after_scratch_clobber();
        }
        _ => {
            return Err(crate::Error::Unsupported(format!(
                "ssub.sat with unsupported bit width: {bits}"
            )));
        }
    }

    e.store_to_slot(slot, dst);
    Ok(())
}

/// Lower `@llvm.sadd.sat.iN(a, b)`.
///
/// Result form: sign-extended.
/// Same shape as `ssub.sat` — only differences are `Add64` vs `Sub64` and the
/// i64 overflow XOR pair `(a^sum) & (b^sum)` vs `(a^b) & (a^sum)`.
fn lower_sadd_sat<'ctx>(e: &mut PvmEmitter<'ctx>, instr: InstructionValue<'ctx>) -> Result<()> {
    use crate::abi::{SCRATCH1, SCRATCH2};

    let slot = result_slot(e, instr)?;
    let a = get_operand(instr, 0)?;
    let b = get_operand(instr, 1)?;
    let dst = result_reg(e, instr);
    let bits = operand_bit_width(instr);

    match bits {
        8 | 16 | 32 => {
            e.load_operand(a, TEMP1)?;
            e.load_operand(b, TEMP2)?;
            match bits {
                8 => {
                    e.emit(Instruction::SignExtend8 {
                        dst: TEMP1,
                        src: TEMP1,
                    });
                    e.emit(Instruction::SignExtend8 {
                        dst: TEMP2,
                        src: TEMP2,
                    });
                }
                16 => {
                    e.emit(Instruction::SignExtend16 {
                        dst: TEMP1,
                        src: TEMP1,
                    });
                    e.emit(Instruction::SignExtend16 {
                        dst: TEMP2,
                        src: TEMP2,
                    });
                }
                _ => {
                    // i32: AddImm32 _, _, 0 sign-extends low 32 bits to 64.
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
            }
            e.emit(Instruction::Add64 {
                dst,
                src1: TEMP1,
                src2: TEMP2,
            });
            // After Add64, both TEMP1 and TEMP2 are dead — use TEMP1 for
            // the imin/imax constants (NOT TEMP_RESULT, which may alias `dst`).
            let (imin, imax): (i32, i32) = match bits {
                8 => (-128, 127),
                16 => (-32_768, 32_767),
                _ => (i32::MIN, i32::MAX),
            };
            e.emit(Instruction::LoadImm {
                reg: TEMP1,
                value: imin,
            });
            e.emit(Instruction::Max {
                dst,
                src1: dst,
                src2: TEMP1,
            });
            e.emit(Instruction::LoadImm {
                reg: TEMP1,
                value: imax,
            });
            e.emit(Instruction::Min {
                dst,
                src1: dst,
                src2: TEMP1,
            });
        }
        64 => {
            // Uses SCRATCH1/SCRATCH2 — bracket with spill/reload.
            e.spill_allocated_regs();
            e.load_operand(a, TEMP1)?;
            e.load_operand(b, TEMP2)?;
            // sum = a + b (wrapping)
            e.emit(Instruction::Add64 {
                dst,
                src1: TEMP1,
                src2: TEMP2,
            });
            // sadd overflow test: (a^sum) & (b^sum) negative ⟺ same-sign-input + diff-sign-output.
            e.emit(Instruction::Xor {
                dst: SCRATCH1,
                src1: TEMP1,
                src2: dst,
            });
            e.emit(Instruction::Xor {
                dst: SCRATCH2,
                src1: TEMP2,
                src2: dst,
            });
            e.emit(Instruction::And {
                dst: SCRATCH1,
                src1: SCRATCH1,
                src2: SCRATCH2,
            });
            e.emit(Instruction::SharRImm64 {
                dst: SCRATCH1,
                src: SCRATCH1,
                value: 63,
            });
            // sign_a = arith-shift a right 63 (0 or -1)
            e.emit(Instruction::SharRImm64 {
                dst: SCRATCH2,
                src: TEMP1,
                value: 63,
            });
            // imax = INT64_MAX = (-1 logical-shift-right 1)
            e.emit(Instruction::LoadImm {
                reg: TEMP1,
                value: -1,
            });
            e.emit(Instruction::ShloRImm64 {
                dst: TEMP1,
                src: TEMP1,
                value: 1,
            });
            // sat = sign_a ^ INT64_MAX
            e.emit(Instruction::Xor {
                dst: SCRATCH2,
                src1: SCRATCH2,
                src2: TEMP1,
            });
            // if cond, dst = sat
            e.emit(Instruction::CmovNz {
                dst,
                src: SCRATCH2,
                cond: SCRATCH1,
            });
            e.reload_allocated_regs_after_scratch_clobber();
        }
        _ => {
            return Err(crate::Error::Unsupported(format!(
                "sadd.sat with unsupported bit width: {bits}"
            )));
        }
    }

    e.store_to_slot(slot, dst);
    Ok(())
}
