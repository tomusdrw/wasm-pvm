// Memory operations: global load/store and PVM memory intrinsics.

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

use inkwell::values::{BasicValueEnum, InstructionValue};

use crate::pvm::Instruction;
use crate::{Error, Result, abi};

use super::emitter::{LoweringContext, PvmEmitter, get_operand, result_slot, try_get_constant};
use crate::abi::{TEMP_RESULT, TEMP1, TEMP2};

/// Lower a load from a WASM global variable.
///
/// After mem2reg optimization, remaining loads in LLVM IR typically access
/// WASM global variables (represented as LLVM globals with names like `wasm_global_N`).
pub fn lower_wasm_global_load<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    _ctx: &LoweringContext,
) -> Result<()> {
    let slot = result_slot(e, instr)?;
    let ptr = get_operand(instr, 0)?;

    // Try to extract the global index from the name.
    if let BasicValueEnum::PointerValue(pv) = ptr {
        let name = pv.get_name().to_string_lossy().to_string();
        if let Some(idx) = name
            .strip_prefix("wasm_global_")
            .and_then(|s| s.parse::<u32>().ok())
        {
            let global_addr = abi::global_addr(idx);
            e.emit(Instruction::LoadImm {
                reg: TEMP1,
                value: global_addr,
            });
            e.emit(Instruction::LoadIndU32 {
                dst: TEMP_RESULT,
                base: TEMP1,
                offset: 0,
            });
            e.store_to_slot(slot, TEMP_RESULT);
            return Ok(());
        }
    }

    Err(Error::Internal(format!(
        "unexpected load operand in LLVM IR: {instr:?}"
    )))
}

/// Lower a store to a WASM global variable.
///
/// When the value is a compile-time constant that fits in i32, uses `StoreImmIndU32`
/// to avoid loading the value into a register.
pub fn lower_wasm_global_store<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    _ctx: &LoweringContext,
) -> Result<()> {
    // store i64 %val, ptr @wasm_global_N
    let val = get_operand(instr, 0)?;
    let ptr = get_operand(instr, 1)?;

    if let BasicValueEnum::PointerValue(pv) = ptr {
        let name = pv.get_name().to_string_lossy().to_string();
        if let Some(idx) = name
            .strip_prefix("wasm_global_")
            .and_then(|s| s.parse::<u32>().ok())
        {
            let global_addr = abi::global_addr(idx);

            // If value is a compile-time constant that fits in i32, use StoreImm.
            if let Some(val_const) = try_get_constant(val)
                && i32::try_from(val_const).is_ok()
            {
                e.emit(Instruction::StoreImmU32 {
                    address: global_addr,
                    value: val_const as i32,
                });
                return Ok(());
            }

            e.load_operand(val, TEMP1)?;
            e.emit(Instruction::LoadImm {
                reg: TEMP2,
                value: global_addr,
            });
            e.emit(Instruction::StoreIndU32 {
                base: TEMP2,
                src: TEMP1,
                offset: 0,
            });
            return Ok(());
        }
    }

    Err(Error::Internal(format!(
        "unexpected store operand in LLVM IR: {instr:?}"
    )))
}

/// Kinds of PVM load operations.
#[derive(Clone, Copy)]
pub enum PvmLoadKind {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32S,
    U64,
}

/// Kinds of PVM store operations.
#[derive(Clone, Copy)]
pub enum PvmStoreKind {
    U8,
    U16,
    U32,
    U64,
}

/// Emit a PVM load intrinsic.
pub fn emit_pvm_load<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
    kind: PvmLoadKind,
) -> Result<()> {
    let addr = get_operand(instr, 0)?;
    let slot = result_slot(e, instr)?;

    e.load_operand(addr, TEMP1)?;

    // Emit the PVM load with wasm_memory_base as the offset.
    let offset = ctx.wasm_memory_base;
    match kind {
        PvmLoadKind::U8 => e.emit(Instruction::LoadIndU8 {
            dst: TEMP_RESULT,
            base: TEMP1,
            offset,
        }),
        PvmLoadKind::I8 => e.emit(Instruction::LoadIndI8 {
            dst: TEMP_RESULT,
            base: TEMP1,
            offset,
        }),
        PvmLoadKind::U16 => e.emit(Instruction::LoadIndU16 {
            dst: TEMP_RESULT,
            base: TEMP1,
            offset,
        }),
        PvmLoadKind::I16 => e.emit(Instruction::LoadIndI16 {
            dst: TEMP_RESULT,
            base: TEMP1,
            offset,
        }),
        PvmLoadKind::U32 => e.emit(Instruction::LoadIndU32 {
            dst: TEMP_RESULT,
            base: TEMP1,
            offset,
        }),
        PvmLoadKind::I32S => {
            // Load as u32 then sign-extend from 32 bits.
            e.emit(Instruction::LoadIndU32 {
                dst: TEMP_RESULT,
                base: TEMP1,
                offset,
            });
            e.emit(Instruction::AddImm32 {
                dst: TEMP_RESULT,
                src: TEMP_RESULT,
                value: 0,
            });
        }
        PvmLoadKind::U64 => e.emit(Instruction::LoadIndU64 {
            dst: TEMP_RESULT,
            base: TEMP1,
            offset,
        }),
    }

    e.store_to_slot(slot, TEMP_RESULT);
    Ok(())
}

/// Emit a PVM store intrinsic.
///
/// When the value being stored is a compile-time constant that fits in i32,
/// uses `StoreImmInd*` to avoid an extra `LoadImm` instruction.
pub fn emit_pvm_store<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
    kind: PvmStoreKind,
) -> Result<()> {
    // Intrinsic: __pvm_store_*(addr, val)
    let addr = get_operand(instr, 0)?;
    let val = get_operand(instr, 1)?;

    let offset = ctx.wasm_memory_base;

    // Try immediate store: if value is a constant that fits in i32, use StoreImmInd*.
    if let Some(val_const) = try_get_constant(val)
        && i32::try_from(val_const).is_ok()
    {
        let imm = val_const as i32;
        e.load_operand(addr, TEMP1)?;
        match kind {
            PvmStoreKind::U8 => e.emit(Instruction::StoreImmIndU8 {
                base: TEMP1,
                offset,
                value: imm,
            }),
            PvmStoreKind::U16 => e.emit(Instruction::StoreImmIndU16 {
                base: TEMP1,
                offset,
                value: imm,
            }),
            PvmStoreKind::U32 => e.emit(Instruction::StoreImmIndU32 {
                base: TEMP1,
                offset,
                value: imm,
            }),
            PvmStoreKind::U64 => e.emit(Instruction::StoreImmIndU64 {
                base: TEMP1,
                offset,
                value: imm,
            }),
        }
        return Ok(());
    }

    e.load_operand(addr, TEMP1)?;
    e.load_operand(val, TEMP2)?;

    match kind {
        PvmStoreKind::U8 => e.emit(Instruction::StoreIndU8 {
            base: TEMP1,
            src: TEMP2,
            offset,
        }),
        PvmStoreKind::U16 => e.emit(Instruction::StoreIndU16 {
            base: TEMP1,
            src: TEMP2,
            offset,
        }),
        PvmStoreKind::U32 => e.emit(Instruction::StoreIndU32 {
            base: TEMP1,
            src: TEMP2,
            offset,
        }),
        PvmStoreKind::U64 => e.emit(Instruction::StoreIndU64 {
            base: TEMP1,
            src: TEMP2,
            offset,
        }),
    }

    Ok(())
}

/// Emit memory.size operation.
pub fn emit_pvm_memory_size<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
) -> Result<()> {
    let slot = result_slot(e, instr)?;
    let global_addr = abi::memory_size_global_offset(ctx.num_globals);

    e.emit(Instruction::LoadImm {
        reg: TEMP1,
        value: global_addr,
    });
    e.emit(Instruction::LoadIndU32 {
        dst: TEMP_RESULT,
        base: TEMP1,
        offset: 0,
    });
    e.store_to_slot(slot, TEMP_RESULT);
    Ok(())
}

/// Emit memory.grow operation.
pub fn emit_pvm_memory_grow<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
) -> Result<()> {
    use crate::abi::{SCRATCH1, SCRATCH2};

    let delta = get_operand(instr, 0)?;
    let slot = result_slot(e, instr)?;
    let global_addr = abi::memory_size_global_offset(ctx.num_globals);

    // Load delta into SCRATCH1.
    e.load_operand(delta, SCRATCH1)?;

    // Load current memory size into TEMP_RESULT (this will be the return value on success).
    e.emit(Instruction::LoadImm {
        reg: TEMP1,
        value: global_addr,
    });
    e.emit(Instruction::LoadIndU32 {
        dst: TEMP_RESULT,
        base: TEMP1,
        offset: 0,
    });

    // new_size = current + delta
    e.emit(Instruction::Add32 {
        dst: SCRATCH2,
        src1: TEMP_RESULT,
        src2: SCRATCH1,
    });

    // Check for overflow: if new_size < current, overflow occurred.
    let fail_label = e.alloc_label();
    let end_label = e.alloc_label();

    // Branch to fail if new_size < current (overflow).
    // PVM BranchLtU branches when reg2 < reg1, so reg1=TEMP_RESULT, reg2=SCRATCH2
    // means "branch if SCRATCH2(new_size) < TEMP_RESULT(current)".
    let fixup_idx = e.instructions.len();
    e.fixups.push((fixup_idx, fail_label));
    e.emit(Instruction::BranchLtU {
        reg1: TEMP_RESULT,
        reg2: SCRATCH2,
        offset: 0,
    });

    // Check new_size > max_pages → fail.
    e.emit(Instruction::LoadImm {
        reg: SCRATCH1,
        value: ctx.max_memory_pages as i32,
    });
    // Branch to fail if SCRATCH2 > SCRATCH1 (i.e. new_size > max).
    // BranchLtU branches if reg2 < reg1, so we want SCRATCH1 < SCRATCH2.
    let fixup_idx = e.instructions.len();
    e.fixups.push((fixup_idx, fail_label));
    e.emit(Instruction::BranchLtU {
        reg1: SCRATCH2,
        reg2: SCRATCH1,
        offset: 0,
    });

    // Success: store new_size.
    e.emit(Instruction::LoadImm {
        reg: SCRATCH1,
        value: global_addr,
    });
    e.emit(Instruction::StoreIndU32 {
        base: SCRATCH1,
        src: SCRATCH2,
        offset: 0,
    });

    // SBRK: grow PVM memory by (delta * 65536) bytes.
    e.emit(Instruction::Sub32 {
        dst: SCRATCH1,
        src1: SCRATCH2,
        src2: TEMP_RESULT,
    });
    e.emit(Instruction::LoadImm {
        reg: SCRATCH2,
        value: 16,
    });
    e.emit(Instruction::ShloL32 {
        dst: SCRATCH1,
        src1: SCRATCH1,
        src2: SCRATCH2,
    });
    e.emit(Instruction::Sbrk {
        dst: SCRATCH1,
        src: SCRATCH1,
    });

    // TEMP_RESULT has old size — jump to end.
    e.emit_jump_to_label(end_label);

    // Failure: return -1.
    e.define_label(fail_label);
    e.emit(Instruction::LoadImm {
        reg: TEMP_RESULT,
        value: -1,
    });

    e.define_label(end_label);
    e.store_to_slot(slot, TEMP_RESULT);
    Ok(())
}

/// Emit memory.fill operation using word-sized (64-bit) stores for the bulk,
/// with a byte-by-byte tail for the remaining 0-7 bytes.
pub fn emit_pvm_memory_fill<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
) -> Result<()> {
    use crate::abi::{SCRATCH1, SCRATCH2};

    // __pvm_memory_fill(dst, val, len)
    let dst_addr = get_operand(instr, 0)?;
    let val = get_operand(instr, 1)?;
    let len = get_operand(instr, 2)?;

    e.load_operand(dst_addr, TEMP1)?; // dest
    e.load_operand(val, TEMP2)?; // value (single byte)
    e.load_operand(len, TEMP_RESULT)?; // size (counter)

    // Add wasm_memory_base to dest.
    e.emit(Instruction::AddImm32 {
        dst: TEMP1,
        src: TEMP1,
        value: ctx.wasm_memory_base,
    });

    let word_loop = e.alloc_label();
    let byte_loop = e.alloc_label();
    let end_label = e.alloc_label();

    // Skip everything if len == 0.
    e.emit_branch_eq_imm_to_label(TEMP_RESULT, 0, end_label);

    // Replicate the byte value across all 8 bytes of a 64-bit word.
    // First mask to low byte (WASM spec: memory.fill uses val & 0xFF),
    // then multiply by 0x0101010101010101 to broadcast to all byte lanes.
    e.emit(Instruction::LoadImm {
        reg: SCRATCH1,
        value: 0xFF,
    });
    e.emit(Instruction::And {
        dst: SCRATCH1,
        src1: TEMP2,
        src2: SCRATCH1,
    });
    e.emit(Instruction::LoadImm64 {
        reg: SCRATCH2,
        value: 0x0101_0101_0101_0101,
    });
    e.emit(Instruction::Mul64 {
        dst: SCRATCH1,
        src1: SCRATCH1,
        src2: SCRATCH2,
    });
    // SCRATCH1 now holds the 64-bit fill pattern.

    // SCRATCH2 = len >> 3 (number of 8-byte words).
    e.emit(Instruction::LoadImm {
        reg: SCRATCH2,
        value: 3,
    });
    e.emit(Instruction::ShloR64 {
        dst: SCRATCH2,
        src1: TEMP_RESULT,
        src2: SCRATCH2,
    });

    // Skip word loop if no full words.
    e.emit_branch_eq_imm_to_label(SCRATCH2, 0, byte_loop);

    // ── Word loop: store 8 bytes at a time ──
    e.define_label(word_loop);
    e.emit(Instruction::StoreIndU64 {
        base: TEMP1,
        src: SCRATCH1,
        offset: 0,
    });
    e.emit(Instruction::AddImm64 {
        dst: TEMP1,
        src: TEMP1,
        value: 8,
    });
    e.emit(Instruction::AddImm64 {
        dst: SCRATCH2,
        src: SCRATCH2,
        value: -1,
    });
    e.emit_branch_ne_imm_to_label(SCRATCH2, 0, word_loop);

    // ── Byte tail: TEMP_RESULT = len & 7 (remaining bytes) ──
    e.define_label(byte_loop);
    e.emit(Instruction::LoadImm {
        reg: SCRATCH2,
        value: 7,
    });
    e.emit(Instruction::And {
        dst: TEMP_RESULT,
        src1: TEMP_RESULT,
        src2: SCRATCH2,
    });
    e.emit_branch_eq_imm_to_label(TEMP_RESULT, 0, end_label);

    let byte_loop_body = e.alloc_label();
    e.define_label(byte_loop_body);
    e.emit(Instruction::StoreIndU8 {
        base: TEMP1,
        src: TEMP2,
        offset: 0,
    });
    e.emit(Instruction::AddImm64 {
        dst: TEMP1,
        src: TEMP1,
        value: 1,
    });
    e.emit(Instruction::AddImm64 {
        dst: TEMP_RESULT,
        src: TEMP_RESULT,
        value: -1,
    });
    e.emit_branch_ne_imm_to_label(TEMP_RESULT, 0, byte_loop_body);

    e.define_label(end_label);
    Ok(())
}

/// Emit memory.copy with memmove semantics (handles overlapping regions).
///
/// When `dst > src`, a naive forward copy corrupts overlapping bytes before
/// they are read. We detect this case and copy backward (from high to low
/// addresses) so that source data is always read before being overwritten.
///
/// Uses word-sized (64-bit) loads/stores for the bulk of forward copies,
/// with byte-by-byte handling for the tail and backward copies.
pub fn emit_pvm_memory_copy<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
) -> Result<()> {
    use crate::abi::{SCRATCH1, SCRATCH2};

    // __pvm_memory_copy(dst, src, len)
    let dst_addr = get_operand(instr, 0)?;
    let src_addr = get_operand(instr, 1)?;
    let len = get_operand(instr, 2)?;

    e.load_operand(dst_addr, TEMP1)?; // dest
    e.load_operand(src_addr, TEMP2)?; // src
    e.load_operand(len, TEMP_RESULT)?; // size (counter)

    // Add wasm_memory_base to both addresses.
    e.emit(Instruction::AddImm32 {
        dst: TEMP1,
        src: TEMP1,
        value: ctx.wasm_memory_base,
    });
    e.emit(Instruction::AddImm32 {
        dst: TEMP2,
        src: TEMP2,
        value: ctx.wasm_memory_base,
    });

    // Check for overlap and direction: if dst > src, copy backwards.
    // PVM branch semantics: BranchLtU{reg1: a, reg2: b} branches if b < a.
    // So BranchLtU(reg1: dst, reg2: src) branches if src < dst (i.e. dst > src).
    let backward_label = e.alloc_label();
    let forward_label = e.alloc_label();
    let loop_end = e.alloc_label();

    // Check if length is 0 (optimization).
    e.emit_branch_eq_imm_to_label(TEMP_RESULT, 0, loop_end);

    // If src < dst (i.e. dst > src), use backward copy.
    e.emit_branch_lt_u_to_label(TEMP1, TEMP2, backward_label);

    // ── Forward Copy (dst <= src or no overlap) ──
    // Word loop: copy 8 bytes at a time.
    // SCRATCH2 = len >> 3 (number of full 8-byte words).
    e.emit(Instruction::LoadImm {
        reg: SCRATCH2,
        value: 3,
    });
    e.emit(Instruction::ShloR64 {
        dst: SCRATCH2,
        src1: TEMP_RESULT,
        src2: SCRATCH2,
    });

    let fwd_word_loop = e.alloc_label();
    let fwd_byte_tail = e.alloc_label();

    e.emit_branch_eq_imm_to_label(SCRATCH2, 0, fwd_byte_tail);
    e.define_label(fwd_word_loop);

    e.emit(Instruction::LoadIndU64 {
        dst: SCRATCH1,
        base: TEMP2,
        offset: 0,
    });
    e.emit(Instruction::StoreIndU64 {
        base: TEMP1,
        src: SCRATCH1,
        offset: 0,
    });
    e.emit(Instruction::AddImm64 {
        dst: TEMP1,
        src: TEMP1,
        value: 8,
    });
    e.emit(Instruction::AddImm64 {
        dst: TEMP2,
        src: TEMP2,
        value: 8,
    });
    e.emit(Instruction::AddImm64 {
        dst: SCRATCH2,
        src: SCRATCH2,
        value: -1,
    });
    e.emit_branch_ne_imm_to_label(SCRATCH2, 0, fwd_word_loop);

    // Byte tail: remaining = len & 7.
    e.define_label(fwd_byte_tail);
    e.emit(Instruction::LoadImm {
        reg: SCRATCH2,
        value: 7,
    });
    e.emit(Instruction::And {
        dst: TEMP_RESULT,
        src1: TEMP_RESULT,
        src2: SCRATCH2,
    });
    e.emit_branch_eq_imm_to_label(TEMP_RESULT, 0, loop_end);

    e.define_label(forward_label);
    e.emit(Instruction::LoadIndU8 {
        dst: SCRATCH1,
        base: TEMP2,
        offset: 0,
    });
    e.emit(Instruction::StoreIndU8 {
        base: TEMP1,
        src: SCRATCH1,
        offset: 0,
    });
    e.emit(Instruction::AddImm64 {
        dst: TEMP1,
        src: TEMP1,
        value: 1,
    });
    e.emit(Instruction::AddImm64 {
        dst: TEMP2,
        src: TEMP2,
        value: 1,
    });
    e.emit(Instruction::AddImm64 {
        dst: TEMP_RESULT,
        src: TEMP_RESULT,
        value: -1,
    });
    e.emit_branch_ne_imm_to_label(TEMP_RESULT, 0, forward_label);
    e.emit_jump_to_label(loop_end);

    // ── Backward Copy (dst > src) ──
    // Byte-by-byte backward copy for correctness with overlapping regions.
    // (Word-sized backward copy is avoided because it increases code size
    // significantly for large programs, and backward copies are rare in practice.)
    e.define_label(backward_label);

    // Decrement size once for the offset calculation (len - 1).
    e.emit(Instruction::AddImm64 {
        dst: SCRATCH2,
        src: TEMP_RESULT,
        value: -1,
    });
    // Adjust pointers to the end.
    e.emit(Instruction::Add64 {
        dst: TEMP1,
        src1: TEMP1,
        src2: SCRATCH2,
    });
    e.emit(Instruction::Add64 {
        dst: TEMP2,
        src1: TEMP2,
        src2: SCRATCH2,
    });

    let backward_loop = e.alloc_label();
    e.define_label(backward_loop);

    e.emit(Instruction::LoadIndU8 {
        dst: SCRATCH1,
        base: TEMP2,
        offset: 0,
    });
    e.emit(Instruction::StoreIndU8 {
        base: TEMP1,
        src: SCRATCH1,
        offset: 0,
    });
    e.emit(Instruction::AddImm64 {
        dst: TEMP1,
        src: TEMP1,
        value: -1,
    });
    e.emit(Instruction::AddImm64 {
        dst: TEMP2,
        src: TEMP2,
        value: -1,
    });
    e.emit(Instruction::AddImm64 {
        dst: TEMP_RESULT,
        src: TEMP_RESULT,
        value: -1,
    });
    e.emit_branch_ne_imm_to_label(TEMP_RESULT, 0, backward_loop);

    e.define_label(loop_end);
    Ok(())
}

/// Emit data.drop operation (zero the segment's effective length).
pub fn emit_pvm_data_drop<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
) -> Result<()> {
    let seg_idx_val = get_operand(instr, 0)?;
    let seg_idx = match seg_idx_val {
        BasicValueEnum::IntValue(iv) => iv
            .get_zero_extended_constant()
            .map(|v| v as u32)
            .ok_or_else(|| Error::Internal("data.drop segment index must be constant".into()))?,
        _ => {
            return Err(Error::Internal(
                "data.drop segment index must be int".into(),
            ));
        }
    };

    let length_addr = *ctx
        .data_segment_length_addrs
        .get(&seg_idx)
        .ok_or_else(|| Error::Internal(format!("unknown passive data segment index {seg_idx}")))?;

    // Store 0 to the segment's effective length address.
    e.emit(Instruction::StoreImmU32 {
        address: length_addr,
        value: 0,
    });

    Ok(())
}

/// Emit memory.init operation with runtime bounds checking.
///
/// Traps if:
/// - `src_offset + len > effective_segment_length` (out of bounds read from passive segment)
/// - `dst + len > memory_size_bytes` (out of bounds write to WASM memory)
pub fn emit_pvm_memory_init<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
) -> Result<()> {
    use crate::abi::{RO_DATA_BASE, SCRATCH1, SCRATCH2};

    // __pvm_memory_init(segment_idx, dst, src_offset, len)
    let wasm_seg_idx_val = get_operand(instr, 0)?;
    let wasm_dst = get_operand(instr, 1)?;
    let wasm_src_offset = get_operand(instr, 2)?;
    let wasm_len = get_operand(instr, 3)?;

    let wasm_seg_idx = match wasm_seg_idx_val {
        BasicValueEnum::IntValue(iv) => {
            if let Some(val) = iv.get_zero_extended_constant() {
                val as u32
            } else {
                return Err(Error::Internal(
                    "memory.init segment index must be constant".into(),
                ));
            }
        }
        _ => {
            return Err(Error::Internal(
                "memory.init segment index must be int".into(),
            ));
        }
    };

    let ro_offset = *ctx.data_segment_offsets.get(&wasm_seg_idx).ok_or_else(|| {
        Error::Internal(format!("unknown passive data segment index {wasm_seg_idx}"))
    })?;

    let length_addr = *ctx
        .data_segment_length_addrs
        .get(&wasm_seg_idx)
        .ok_or_else(|| {
            Error::Internal(format!(
                "unknown passive data segment length addr {wasm_seg_idx}"
            ))
        })?;

    // Load operands: TEMP1=dst, TEMP2=src_offset, TEMP_RESULT=len
    e.load_operand(wasm_dst, TEMP1)?;
    e.load_operand(wasm_src_offset, TEMP2)?;
    e.load_operand(wasm_len, TEMP_RESULT)?;

    // ── Bounds check 1: src_offset + len <= effective_segment_length ──
    // SCRATCH1 = src_offset + len
    e.emit(Instruction::Add32 {
        dst: SCRATCH1,
        src1: TEMP2,
        src2: TEMP_RESULT,
    });
    // SCRATCH2 = effective_segment_length (from runtime address)
    e.emit(Instruction::LoadImm {
        reg: SCRATCH2,
        value: length_addr,
    });
    e.emit(Instruction::LoadIndU32 {
        dst: SCRATCH2,
        base: SCRATCH2,
        offset: 0,
    });
    // Trap if SCRATCH1 > SCRATCH2 (i.e. src_offset + len > effective_length)
    // BranchGeU branches when reg2 >= reg1, so we want "branch to ok if SCRATCH1 <= SCRATCH2"
    // which is "branch if SCRATCH1 <= SCRATCH2" i.e. BranchGeU(reg1=SCRATCH1, reg2=SCRATCH2)
    let src_ok_label = e.alloc_label();
    let fixup_idx = e.instructions.len();
    e.fixups.push((fixup_idx, src_ok_label));
    e.emit(Instruction::BranchGeU {
        reg1: SCRATCH1,
        reg2: SCRATCH2,
        offset: 0,
    });
    e.emit(Instruction::Trap);
    e.define_label(src_ok_label);

    // ── Bounds check 2: dst + len <= memory_size * 65536 ──
    // SCRATCH1 = dst + len
    e.emit(Instruction::Add32 {
        dst: SCRATCH1,
        src1: TEMP1,
        src2: TEMP_RESULT,
    });
    // SCRATCH2 = memory_size (in pages)
    let mem_size_addr = crate::abi::memory_size_global_offset(ctx.num_globals);
    e.emit(Instruction::LoadImm {
        reg: SCRATCH2,
        value: mem_size_addr,
    });
    e.emit(Instruction::LoadIndU32 {
        dst: SCRATCH2,
        base: SCRATCH2,
        offset: 0,
    });
    // SCRATCH2 = memory_size * 65536 (shift left by 16)
    e.emit(Instruction::LoadImm {
        reg: TEMP2, // temporarily reuse TEMP2 for shift amount
        value: 16,
    });
    e.emit(Instruction::ShloL32 {
        dst: SCRATCH2,
        src1: SCRATCH2,
        src2: TEMP2,
    });
    // Trap if SCRATCH1 > SCRATCH2 (dst + len > memory_size_bytes)
    let dst_ok_label = e.alloc_label();
    let fixup_idx = e.instructions.len();
    e.fixups.push((fixup_idx, dst_ok_label));
    e.emit(Instruction::BranchGeU {
        reg1: SCRATCH1,
        reg2: SCRATCH2,
        offset: 0,
    });
    e.emit(Instruction::Trap);
    e.define_label(dst_ok_label);

    // ── Reload src_offset into TEMP2 (was clobbered by shift amount) ──
    e.load_operand(wasm_src_offset, TEMP2)?;

    // Calculate src_addr = RO_DATA_BASE + ro_offset + src_offset
    e.emit(Instruction::AddImm32 {
        dst: TEMP2,
        src: TEMP2,
        value: (RO_DATA_BASE as u32 + ro_offset) as i32,
    });

    // Calculate dst_addr = wasm_memory_base + dst
    e.emit(Instruction::AddImm32 {
        dst: TEMP1,
        src: TEMP1,
        value: ctx.wasm_memory_base,
    });

    // Loop: while size > 0: dst++ = src++
    let loop_start = e.alloc_label();
    let loop_end = e.alloc_label();

    e.emit_branch_eq_imm_to_label(TEMP_RESULT, 0, loop_end);
    e.define_label(loop_start);

    e.emit(Instruction::LoadIndU8 {
        dst: SCRATCH1,
        base: TEMP2,
        offset: 0,
    });
    e.emit(Instruction::StoreIndU8 {
        base: TEMP1,
        src: SCRATCH1,
        offset: 0,
    });
    e.emit(Instruction::AddImm64 {
        dst: TEMP1,
        src: TEMP1,
        value: 1,
    });
    e.emit(Instruction::AddImm64 {
        dst: TEMP2,
        src: TEMP2,
        value: 1,
    });
    e.emit(Instruction::AddImm64 {
        dst: TEMP_RESULT,
        src: TEMP_RESULT,
        value: -1,
    });
    e.emit_branch_ne_imm_to_label(TEMP_RESULT, 0, loop_start);

    e.define_label(loop_end);

    Ok(())
}
