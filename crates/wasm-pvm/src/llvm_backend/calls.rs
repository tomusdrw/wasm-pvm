// Function calls: direct WASM calls, indirect calls, and import stubs.

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

use super::emitter::{
    LlvmCallFixup, LlvmIndirectCallFixup, LoweringContext, PvmEmitter, get_operand, result_slot,
};
use crate::abi::{TEMP_RESULT, TEMP1, TEMP2};

/// Lower a WASM function call.
pub fn lower_wasm_call<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    fn_name: &str,
    ctx: &LoweringContext,
) -> Result<()> {
    // Parse function name to get global function index: "wasm_func_N" → N
    let global_func_idx: u32 = fn_name
        .strip_prefix("wasm_func_")
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| Error::Internal(format!("cannot parse function name: {fn_name}")))?;

    // Check if this is an imported function (stub with Trap or dummy return).
    if (global_func_idx as usize) < ctx.num_imported_funcs {
        return lower_import_call(e, instr, ctx, global_func_idx);
    }

    // Convert to local function index for the call fixup.
    let local_func_idx = global_func_idx - ctx.num_imported_funcs as u32;

    // Get signature info.
    let (_num_params, has_return) = ctx
        .function_signatures
        .get(global_func_idx as usize)
        .copied()
        .ok_or_else(|| Error::Internal(format!("unknown function index: {global_func_idx}")))?;

    // Load arguments from LLVM call operands into r9-r12 (first 4) and
    // PARAM_OVERFLOW_BASE (5th+). The last operand is the function pointer.
    let num_args = (instr.get_num_operands() - 1) as usize;

    for i in 0..num_args {
        let arg = get_operand(instr, i as u32)?;
        if i < abi::MAX_LOCAL_REGS {
            e.load_operand(arg, abi::FIRST_LOCAL_REG + i as u8)?;
        } else {
            e.load_operand(arg, TEMP1)?;
            let overflow_offset = abi::PARAM_OVERFLOW_BASE + ((i - abi::MAX_LOCAL_REGS) * 8) as i32;
            e.emit(Instruction::LoadImm {
                reg: TEMP2,
                value: overflow_offset,
            });
            e.emit(Instruction::StoreIndU64 {
                base: TEMP2,
                src: TEMP1,
                offset: 0,
            });
        }
    }

    // Emit call fixup: LoadImm64 for return address + Jump to callee.
    let return_addr_instr = e.instructions.len();
    e.emit(Instruction::LoadImm64 {
        reg: abi::RETURN_ADDR_REG,
        value: 0, // patched during fixup resolution
    });
    let jump_instr = e.instructions.len();
    e.emit(Instruction::Jump {
        offset: 0, // patched during fixup resolution
    });

    // Return point.
    e.emit(Instruction::Fallthrough);

    e.call_fixups.push(LlvmCallFixup {
        return_addr_instr,
        jump_instr,
        target_func: local_func_idx,
    });

    // If function returns a value, store r7 to result slot.
    if has_return && let Ok(slot) = result_slot(e, instr) {
        e.store_to_slot(slot, abi::RETURN_VALUE_REG);
    }

    Ok(())
}

/// Emit a stub for calling an imported function.
/// Imported functions are not available at PVM level — emit Trap (all import
/// stubs trap since the actual function is not available in PVM).
#[allow(clippy::unnecessary_wraps)]
pub fn lower_import_call<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
    global_func_idx: u32,
) -> Result<()> {
    let (_num_params, has_return) = ctx
        .function_signatures
        .get(global_func_idx as usize)
        .copied()
        .ok_or_else(|| {
            Error::Internal(format!(
                "unknown import function signature for index {global_func_idx}"
            ))
        })?;

    // All import stubs emit Trap (the function is not available).
    e.emit(Instruction::Trap);

    // If the import has a return value, push a dummy 0 so the rest of the code
    // can still type-check (dead code after the trap).
    if has_return && let Ok(slot) = result_slot(e, instr) {
        e.emit(Instruction::LoadImm {
            reg: TEMP_RESULT,
            value: 0,
        });
        e.store_to_slot(slot, TEMP_RESULT);
    }

    Ok(())
}

/// Lower an indirect call via the PVM dispatch table.
pub fn lower_pvm_call_indirect<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    _ctx: &LoweringContext,
) -> Result<()> {
    // __pvm_call_indirect(type_idx, table_entry, arg0, arg1, ...)
    // Operands: [type_idx, table_entry, arg0, ..., argN-1, fn_ptr]
    let num_operands = instr.get_num_operands();

    // Prevent underflow when calculating num_args.
    if num_operands < 3 {
        return Err(Error::Internal(format!(
            "__pvm_call_indirect requires at least 3 operands, got {num_operands}"
        )));
    }

    let num_args = (num_operands - 3) as usize; // subtract type_idx, table_entry, fn_ptr

    let type_idx_val = get_operand(instr, 0)?;
    let table_entry_val = get_operand(instr, 1)?;

    // Validate type_idx is a constant integer (required for signature validation).
    let expected_type_idx = match type_idx_val {
        BasicValueEnum::IntValue(iv) => iv.get_zero_extended_constant().ok_or_else(|| {
            Error::Internal("__pvm_call_indirect type_idx must be a constant".into())
        })? as u32,
        _ => {
            return Err(Error::Internal(
                "__pvm_call_indirect type_idx must be an integer".into(),
            ));
        }
    };

    // Load table entry index into ARGS_LEN_REG and save it in the spill area.
    // Using OPERAND_SPILL_BASE ensures we have reserved space in the frame.
    e.load_operand(table_entry_val, abi::ARGS_LEN_REG)?;
    e.emit(Instruction::StoreIndU64 {
        base: abi::STACK_PTR_REG,
        src: abi::ARGS_LEN_REG,
        offset: abi::OPERAND_SPILL_BASE, // Use documented spill area instead of hardcoded -8
    });

    // Load function arguments into r9-r12 and overflow area.
    for i in 0..num_args {
        let arg = get_operand(instr, (i + 2) as u32)?;
        if i < abi::MAX_LOCAL_REGS {
            e.load_operand(arg, abi::FIRST_LOCAL_REG + i as u8)?;
        } else {
            e.load_operand(arg, TEMP1)?;
            let overflow_offset = abi::PARAM_OVERFLOW_BASE + ((i - abi::MAX_LOCAL_REGS) * 8) as i32;
            e.emit(Instruction::LoadImm {
                reg: TEMP2,
                value: overflow_offset,
            });
            e.emit(Instruction::StoreIndU64 {
                base: TEMP2,
                src: TEMP1,
                offset: 0,
            });
        }
    }

    // Restore table index from saved location.
    e.emit(Instruction::LoadIndU64 {
        dst: abi::ARGS_LEN_REG, // r8, used as SAVED_TABLE_IDX_REG
        base: abi::STACK_PTR_REG,
        offset: abi::OPERAND_SPILL_BASE, // Use documented spill area
    });

    // Dispatch table lookup: each entry is 8 bytes (4-byte jump ref + 4-byte type index).
    // Multiply table index by 8 (entry size) via 3 doublings: idx * 2 * 2 * 2
    // table_addr = RO_DATA_BASE + table_idx * 8
    e.emit(Instruction::Add32 {
        dst: abi::ARGS_LEN_REG,
        src1: abi::ARGS_LEN_REG,
        src2: abi::ARGS_LEN_REG,
    });
    e.emit(Instruction::Add32 {
        dst: abi::ARGS_LEN_REG,
        src1: abi::ARGS_LEN_REG,
        src2: abi::ARGS_LEN_REG,
    });
    e.emit(Instruction::Add32 {
        dst: abi::ARGS_LEN_REG,
        src1: abi::ARGS_LEN_REG,
        src2: abi::ARGS_LEN_REG,
    });
    e.emit(Instruction::AddImm32 {
        dst: abi::ARGS_LEN_REG,
        src: abi::ARGS_LEN_REG,
        value: abi::RO_DATA_BASE,
    });

    // Load and validate type signature.
    e.emit(Instruction::LoadIndU32 {
        dst: TEMP1,
        base: abi::ARGS_LEN_REG,
        offset: 4, // type index at offset 4
    });

    let sig_ok_label = e.alloc_label();
    e.emit_branch_eq_imm_to_label(TEMP1, expected_type_idx as i32, sig_ok_label);
    e.emit(Instruction::Trap);
    e.define_label(sig_ok_label);

    // Load jump address from dispatch table (at offset 0).
    e.emit(Instruction::LoadIndU32 {
        dst: abi::ARGS_LEN_REG,
        base: abi::ARGS_LEN_REG,
        offset: 0,
    });

    // Emit indirect call: LoadImm64 for return address + JumpInd.
    let return_addr_instr = e.instructions.len();
    e.emit(Instruction::LoadImm64 {
        reg: abi::RETURN_ADDR_REG,
        value: 0, // patched during fixup resolution
    });
    let jump_ind_instr = e.instructions.len();
    e.emit(Instruction::JumpInd {
        reg: abi::ARGS_LEN_REG,
        offset: 0,
    });

    e.emit(Instruction::Fallthrough);

    e.indirect_call_fixups.push(LlvmIndirectCallFixup {
        return_addr_instr,
        jump_ind_instr,
    });

    // Store return value if the call produces one.
    if let Ok(slot) = result_slot(e, instr) {
        e.store_to_slot(slot, abi::RETURN_VALUE_REG);
    }

    Ok(())
}

/// Lower a call instruction (dispatches to intrinsic, wasm call, or import).
pub fn lower_call<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
) -> Result<()> {
    // Extract the called function name using CallSiteValue API for robustness.
    // This handles direct calls reliably, even through bitcasts or aliases.
    let call_site: inkwell::values::CallSiteValue = instr.try_into().map_err(|()| {
        Error::Internal("expected call instruction to convert to CallSiteValue".into())
    })?;

    let fn_name = if let Some(fn_val) = call_site.get_called_fn_value() {
        fn_val.get_name().to_string_lossy().to_string()
    } else {
        // Indirect call without a statically known callee.
        return Err(Error::Internal(
            "indirect call not supported (no static callee)".into(),
        ));
    };

    if fn_name.starts_with("__pvm_") {
        return super::intrinsics::lower_pvm_intrinsic(e, instr, &fn_name, ctx);
    }

    if fn_name.starts_with("llvm.") {
        return super::intrinsics::lower_llvm_intrinsic(e, instr, &fn_name);
    }

    // Regular WASM function call.
    lower_wasm_call(e, instr, &fn_name, ctx)
}
