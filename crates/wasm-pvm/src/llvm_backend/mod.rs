// LLVM IR backend: lowers LLVM IR → PVM bytecode.
//
// This module is organized into submodules:
// - `emitter`: Core PvmEmitter struct and value management
// - `alu`: Arithmetic, logic, comparison, conversion, and select operations
// - `memory`: Load/store and memory intrinsics (size, grow, fill, copy)
// - `control_flow`: Branches, phi nodes, switch, return
// - `calls`: Direct calls, indirect calls, import stubs
// - `intrinsics`: PVM and LLVM intrinsic lowering

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

mod alu;
mod calls;
mod control_flow;
mod emitter;
mod intrinsics;
mod memory;

pub use emitter::{LlvmCallFixup, LlvmFunctionTranslation, LlvmIndirectCallFixup, LoweringContext};

use inkwell::values::{FunctionValue, InstructionOpcode};

use crate::pvm::Instruction;
use crate::{Error, Result, abi};

use abi::{TEMP1, TEMP2};
use emitter::{PvmEmitter, pre_scan_function};

/// Lower a single LLVM function to PVM bytecode.
///
/// `result_globals`: For entry functions using the legacy globals convention,
/// pass `Some((ptr_global_idx, len_global_idx))` so the epilogue loads them
/// into r7/r8 before exiting.
pub fn lower_function(
    function: FunctionValue<'_>,
    ctx: &LoweringContext,
    is_main: bool,
    _func_idx: usize,
    result_globals: Option<(u32, u32)>,
    entry_returns_ptr_len: bool,
) -> Result<LlvmFunctionTranslation> {
    let mut emitter = PvmEmitter::new();
    emitter.result_globals = result_globals;
    emitter.entry_returns_ptr_len = entry_returns_ptr_len;
    emitter.wasm_memory_base = ctx.wasm_memory_base;

    // Phase 1: Pre-scan — allocate labels for blocks and slots for all SSA values.
    pre_scan_function(&mut emitter, function);
    emitter.frame_size = emitter.next_slot_offset;

    // Phase 2: Emit prologue.
    emit_prologue(&mut emitter, function, ctx, is_main)?;

    // Phase 3: Lower each basic block.
    for bb in function.get_basic_blocks() {
        let label = emitter.block_labels[&bb];
        emitter.define_label(label);

        for instruction in bb.get_instructions() {
            lower_instruction(&mut emitter, instruction, bb, ctx, is_main)?;
        }
    }

    // Peephole optimization: remove redundant instructions before fixup resolution.
    crate::pvm::peephole::optimize(
        &mut emitter.instructions,
        &mut emitter.fixups,
        &mut emitter.call_fixups,
        &mut emitter.indirect_call_fixups,
        &mut emitter.labels,
    );

    emitter.resolve_fixups()?;

    Ok(LlvmFunctionTranslation {
        instructions: emitter.instructions,
        call_fixups: emitter.call_fixups,
        indirect_call_fixups: emitter.indirect_call_fixups,
    })
}

/// Emit function prologue.
fn emit_prologue<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    function: FunctionValue<'ctx>,
    ctx: &LoweringContext,
    is_main: bool,
) -> Result<()> {
    if !is_main {
        // Stack overflow check: verify SP - frame_size >= stack_limit.
        let limit = abi::stack_limit(ctx.stack_size);
        let continue_label = e.alloc_label();

        // Must use LoadImm64 (not LoadImm) because the limit is in the 0xFExx_xxxx
        // range which is negative as i32. LoadImm sign-extends to i64, producing
        // 0xFFFFFFFF_FExx_xxxx which breaks unsigned comparison.
        e.emit(Instruction::LoadImm64 {
            reg: TEMP1,
            value: u64::from(limit as u32),
        });
        e.emit(Instruction::AddImm64 {
            dst: TEMP2,
            src: abi::STACK_PTR_REG,
            value: -e.frame_size,
        });
        // Branch to continue if new_sp >= limit.
        // BranchGeU { reg1, reg2 } branches if reg2 >= reg1.
        let fixup_idx = e.instructions.len();
        e.fixups.push((fixup_idx, continue_label));
        e.emit(Instruction::BranchGeU {
            reg1: TEMP1,
            reg2: TEMP2,
            offset: 0,
        });
        e.emit(Instruction::Trap);
        e.define_label(continue_label);
    }

    // Allocate stack frame (needed for SSA slot storage in all functions).
    e.emit(Instruction::AddImm64 {
        dst: abi::STACK_PTR_REG,
        src: abi::STACK_PTR_REG,
        value: -e.frame_size,
    });

    if !is_main {
        // Save return address.
        e.emit(Instruction::StoreIndU64 {
            base: abi::STACK_PTR_REG,
            src: abi::RETURN_ADDR_REG,
            offset: 0,
        });

        // Save callee-saved registers r9-r12.
        for i in 0..abi::MAX_LOCAL_REGS {
            e.emit(Instruction::StoreIndU64 {
                base: abi::STACK_PTR_REG,
                src: abi::FIRST_LOCAL_REG + i as u8,
                offset: (8 + i * 8) as i32,
            });
        }
    }

    // Copy parameters to their SSA slots.
    let params = function.get_params();
    for (i, param) in params.iter().enumerate() {
        let key = emitter::val_key_basic(*param);
        let slot = e
            .get_slot(key)
            .ok_or_else(|| Error::Internal(format!("no slot for parameter {i} (key {key:?})")))?;

        if is_main {
            // For main, SPI passes r7=args_ptr, r8=args_len.
            // Adjust args_ptr by subtracting wasm_memory_base.
            if i == 0 {
                e.emit(Instruction::AddImm64 {
                    dst: abi::ARGS_PTR_REG,
                    src: abi::ARGS_PTR_REG,
                    value: -ctx.wasm_memory_base,
                });
                e.store_to_slot(slot, abi::ARGS_PTR_REG);
            } else if i == 1 {
                e.store_to_slot(slot, abi::ARGS_LEN_REG);
            }
        } else if i < abi::MAX_LOCAL_REGS {
            // First 4 params come in r9-r12.
            e.store_to_slot(slot, abi::FIRST_LOCAL_REG + i as u8);
        } else {
            // Overflow params from PARAM_OVERFLOW_BASE.
            let overflow_offset = abi::PARAM_OVERFLOW_BASE + ((i - abi::MAX_LOCAL_REGS) * 8) as i32;
            e.emit(Instruction::LoadImm {
                reg: TEMP1,
                value: overflow_offset,
            });
            e.emit(Instruction::LoadIndU64 {
                dst: TEMP1,
                base: TEMP1,
                offset: 0,
            });
            e.store_to_slot(slot, TEMP1);
        }
    }

    Ok(())
}

/// Lower a single LLVM instruction.
fn lower_instruction<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: inkwell::values::InstructionValue<'ctx>,
    current_bb: inkwell::basic_block::BasicBlock<'ctx>,
    ctx: &LoweringContext,
    is_main: bool,
) -> Result<()> {
    use alu::{
        BinaryOp, lower_binary_arith, lower_icmp, lower_select, lower_sext, lower_trunc, lower_zext,
    };
    use calls::lower_call;
    use control_flow::{lower_br, lower_return, lower_switch};
    use memory::{lower_wasm_global_load, lower_wasm_global_store};

    match instr.get_opcode() {
        // Binary arithmetic
        InstructionOpcode::Add => lower_binary_arith(e, instr, BinaryOp::Add),
        InstructionOpcode::Sub => lower_binary_arith(e, instr, BinaryOp::Sub),
        InstructionOpcode::Mul => lower_binary_arith(e, instr, BinaryOp::Mul),
        InstructionOpcode::UDiv => lower_binary_arith(e, instr, BinaryOp::UDiv),
        InstructionOpcode::SDiv => lower_binary_arith(e, instr, BinaryOp::SDiv),
        InstructionOpcode::URem => lower_binary_arith(e, instr, BinaryOp::URem),
        InstructionOpcode::SRem => lower_binary_arith(e, instr, BinaryOp::SRem),

        // Bitwise
        InstructionOpcode::And => lower_binary_arith(e, instr, BinaryOp::And),
        InstructionOpcode::Or => lower_binary_arith(e, instr, BinaryOp::Or),
        InstructionOpcode::Xor => lower_binary_arith(e, instr, BinaryOp::Xor),
        InstructionOpcode::Shl => lower_binary_arith(e, instr, BinaryOp::Shl),
        InstructionOpcode::LShr => lower_binary_arith(e, instr, BinaryOp::LShr),
        InstructionOpcode::AShr => lower_binary_arith(e, instr, BinaryOp::AShr),

        // Comparisons
        InstructionOpcode::ICmp => lower_icmp(e, instr),

        // Conversions
        InstructionOpcode::ZExt => lower_zext(e, instr),
        InstructionOpcode::SExt => lower_sext(e, instr),
        InstructionOpcode::Trunc => lower_trunc(e, instr),

        // Select
        InstructionOpcode::Select => lower_select(e, instr),

        // Control flow (pass current_bb for phi elimination)
        InstructionOpcode::Br => lower_br(e, instr, current_bb),
        InstructionOpcode::Switch => lower_switch(e, instr, current_bb),
        InstructionOpcode::Return => lower_return(e, instr, is_main),
        InstructionOpcode::Unreachable => {
            e.emit(Instruction::Trap);
            Ok(())
        }

        // Load/Store (globals after mem2reg)
        InstructionOpcode::Load => lower_wasm_global_load(e, instr, ctx),
        InstructionOpcode::Store => lower_wasm_global_store(e, instr, ctx),

        // Calls (intrinsics + wasm functions)
        InstructionOpcode::Call => lower_call(e, instr, ctx),

        // Phi nodes — copies emitted by terminators via emit_phi_copies()
        InstructionOpcode::Phi => Ok(()),

        _ => Err(Error::Unsupported(format!(
            "LLVM opcode {:?}",
            instr.get_opcode()
        ))),
    }
}
