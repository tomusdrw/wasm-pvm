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

use super::emitter::{LoweringContext, PvmEmitter, get_operand, result_slot};
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
pub fn emit_pvm_store<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
    kind: PvmStoreKind,
) -> Result<()> {
    // Intrinsic: __pvm_store_*(addr, val)
    let addr = get_operand(instr, 0)?;
    let val = get_operand(instr, 1)?;

    e.load_operand(addr, TEMP1)?;
    e.load_operand(val, TEMP2)?;

    let offset = ctx.wasm_memory_base;
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

/// Emit memory.fill operation.
pub fn emit_pvm_memory_fill<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
) -> Result<()> {
    // __pvm_memory_fill(dst, val, len)
    let dst_addr = get_operand(instr, 0)?;
    let val = get_operand(instr, 1)?;
    let len = get_operand(instr, 2)?;

    e.load_operand(dst_addr, TEMP1)?; // dest
    e.load_operand(val, TEMP2)?; // value
    e.load_operand(len, TEMP_RESULT)?; // size (counter)

    // Add wasm_memory_base to dest.
    e.emit(Instruction::AddImm32 {
        dst: TEMP1,
        src: TEMP1,
        value: ctx.wasm_memory_base,
    });

    // Loop: while size > 0, store byte and advance.
    let loop_start = e.alloc_label();
    let loop_end = e.alloc_label();

    e.emit_branch_eq_imm_to_label(TEMP_RESULT, 0, loop_end);
    e.define_label(loop_start);

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
    e.emit_branch_ne_imm_to_label(TEMP_RESULT, 0, loop_start);

    e.define_label(loop_end);
    Ok(())
}

/// Emit memory.copy with memmove semantics (handles overlapping regions).
///
/// When `dst > src`, a naive forward copy corrupts overlapping bytes before
/// they are read. We detect this case and copy backward (from high to low
/// addresses) so that source data is always read before being overwritten.
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
    // BranchLtU(reg1: dst, reg2: src) checks if src < dst (i.e. dst > src).
    let backward_label = e.alloc_label();
    let forward_label = e.alloc_label();
    let loop_end = e.alloc_label();

    // Check if length is 0 (optimization).
    e.emit_branch_eq_imm_to_label(TEMP_RESULT, 0, loop_end);

    // If src < dst (i.e. dst > src), use backward copy.
    let fixup_idx = e.instructions.len();
    e.fixups.push((fixup_idx, backward_label));
    e.emit(Instruction::BranchLtU {
        reg1: TEMP1, // dst
        reg2: TEMP2, // src
        offset: 0,
    });

    // ── Forward Copy (dst <= src or no overlap) ──
    // while size > 0: dst++ = src++
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
    // src += len - 1; dst += len - 1;
    // while size > 0: dst-- = src--
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

/// Emit memory.init operation.
pub fn emit_pvm_memory_init<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
) -> Result<()> {
    use crate::abi::{RO_DATA_BASE, SCRATCH1};

    // __pvm_memory_init(segment_idx, dst, src_offset, len)
    let seg_idx_val = get_operand(instr, 0)?;
    let dst_addr = get_operand(instr, 1)?;
    let src_offset_val = get_operand(instr, 2)?;
    let len_val = get_operand(instr, 3)?;

    let seg_idx = match seg_idx_val {
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

    let ro_offset = *ctx
        .data_segment_offsets
        .get(&seg_idx)
        .ok_or_else(|| Error::Internal(format!("unknown passive data segment index {seg_idx}")))?;

    // Load operands
    e.load_operand(dst_addr, TEMP1)?; // dst (in RAM)
    e.load_operand(src_offset_val, TEMP2)?; // src offset (relative to segment start)
    e.load_operand(len_val, TEMP_RESULT)?; // len (counter)

    // Bounds checks: trap if src_offset + len > segment_length or dst + len > memory_size
    let bounds_ok_label = e.alloc_label();

    // Get segment length
    let seg_len = *ctx
        .data_segment_lengths
        .get(&seg_idx)
        .ok_or_else(|| Error::Internal(format!("no length for data segment {seg_idx}")))?;

    // Check 1: src_offset + len <= segment_length
    // Calculate src_end = src_offset + len (use SCRATCH1)
    e.emit(Instruction::Add64 {
        dst: SCRATCH1,
        src1: TEMP2,
        src2: TEMP_RESULT,
    });
    // If src_end > seg_len, trap
    e.emit(Instruction::LoadImm {
        reg: TEMP2,
        value: seg_len as i32,
    });
    // Use SetLtU: src_end > seg_len  ⟺  seg_len < src_end
    e.emit(Instruction::SetLtU {
        dst: SCRATCH1,
        src1: TEMP2,
        src2: SCRATCH1,
    });
    // If result != 0, trap
    e.emit_branch_eq_imm_to_label(SCRATCH1, 0, bounds_ok_label);
    e.emit(Instruction::Trap);
    e.define_label(bounds_ok_label);

    // Check 2: dst + len <= memory_size
    let bounds_ok_label2 = e.alloc_label();
    // Calculate dst_end = dst + len (use SCRATCH1)
    e.emit(Instruction::Add64 {
        dst: SCRATCH1,
        src1: TEMP1,
        src2: TEMP_RESULT,
    });
    // Memory size = initial_memory_pages * 64KB
    let memory_size = i64::from(ctx.initial_memory_pages) * 64 * 1024;
    e.emit(Instruction::LoadImm64 {
        reg: TEMP2,
        value: memory_size as u64,
    });
    // If dst_end > memory_size, trap
    e.emit(Instruction::SetLtU {
        dst: SCRATCH1,
        src1: TEMP2,
        src2: SCRATCH1,
    });
    e.emit_branch_eq_imm_to_label(SCRATCH1, 0, bounds_ok_label2);
    e.emit(Instruction::Trap);
    e.define_label(bounds_ok_label2);

    // Reload src_offset and len (clobbered by bounds checks)
    e.load_operand(src_offset_val, TEMP2)?;
    e.load_operand(len_val, TEMP_RESULT)?;

    // Calculate src_addr = RO_DATA_BASE + ro_offset + src_offset
    // Use TEMP2 for source address
    e.emit(Instruction::AddImm32 {
        dst: TEMP2,
        src: TEMP2,
        value: (RO_DATA_BASE as u32 + ro_offset) as i32,
    });

    // Calculate dst_addr = wasm_memory_base + dst
    // Use TEMP1 for dest address
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
