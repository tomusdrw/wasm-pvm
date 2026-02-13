// LLVM IR → PVM bytecode lowering.
//
// Each SSA value gets a dedicated stack slot (correctness-first, no register allocation).
// Pattern: load operands from slots → temp regs, compute, store result to slot.

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

use std::collections::HashMap;

use inkwell::IntPredicate;
use inkwell::basic_block::BasicBlock;
use inkwell::values::{
    AnyValue, AnyValueEnum, AsValueRef, BasicValueEnum, FunctionValue, InstructionOpcode,
    InstructionValue, IntValue, PhiValue,
};

use crate::pvm::Instruction;
use crate::{abi, Error, Result};

// ── Register assignments ──

use crate::abi::{
    ARGS_LEN_REG, ARGS_PTR_REG, FIRST_LOCAL_REG, FRAME_HEADER_SIZE, MAX_LOCAL_REGS,
    RETURN_ADDR_REG, RETURN_VALUE_REG, SCRATCH1, SCRATCH2, STACK_PTR_REG, TEMP1, TEMP2,
    TEMP_RESULT,
};

// ── Public types ──

/// Context for lowering functions from a single WASM module.
pub struct LoweringContext {
    pub wasm_memory_base: i32,
    pub num_globals: usize,
    pub function_signatures: Vec<(usize, bool)>,
    pub type_signatures: Vec<(usize, usize)>,
    pub function_table: Vec<u32>,
    pub num_imported_funcs: usize,
    pub initial_memory_pages: u32,
    pub max_memory_pages: u32,
    pub stack_size: u32,
}

/// Result of lowering one LLVM function to PVM instructions.
pub struct LlvmFunctionTranslation {
    pub instructions: Vec<Instruction>,
    pub call_fixups: Vec<LlvmCallFixup>,
    pub indirect_call_fixups: Vec<LlvmIndirectCallFixup>,
}

#[derive(Debug, Clone)]
pub struct LlvmCallFixup {
    pub return_addr_instr: usize,
    pub jump_instr: usize,
    pub target_func: u32,
}

#[derive(Debug, Clone)]
pub struct LlvmIndirectCallFixup {
    pub return_addr_instr: usize,
    pub jump_ind_instr: usize,
}

// ── PVM Emitter ──

struct PvmEmitter<'ctx> {
    instructions: Vec<Instruction>,
    labels: Vec<Option<usize>>,
    fixups: Vec<(usize, usize)>,

    /// LLVM basic block → PVM label.
    block_labels: HashMap<BasicBlock<'ctx>, usize>,

    /// LLVM int values (params + instruction results) → stack slot offset from SP.
    value_slots: HashMap<ValKey, i32>,

    /// Next available slot offset (bump allocator, starts after frame header).
    next_slot_offset: i32,

    /// Total frame size (set after pre-scan).
    frame_size: i32,

    call_fixups: Vec<LlvmCallFixup>,
    indirect_call_fixups: Vec<LlvmIndirectCallFixup>,

    /// For entry functions: (`result_ptr_global`, `result_len_global`) indices.
    /// When set, the epilogue loads these globals into r7/r8 before exiting.
    result_globals: Option<(u32, u32)>,

    /// When true, the entry function's return value is a packed i64:
    /// lower 32 bits = ptr (WASM address), upper 32 bits = len.
    entry_returns_ptr_len: bool,

    /// WASM memory base address (for converting WASM addresses to PVM addresses).
    wasm_memory_base: i32,
}

/// Wrapper key for LLVM values in the slot map.
/// Uses the raw LLVM value pointer cast to usize for hashing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ValKey(usize);

fn val_key_int(val: IntValue<'_>) -> ValKey {
    ValKey(val.as_value_ref() as usize)
}

fn val_key_basic(val: BasicValueEnum<'_>) -> ValKey {
    ValKey(val.as_value_ref() as usize)
}

fn val_key_instr(val: InstructionValue<'_>) -> ValKey {
    ValKey(val.as_value_ref() as usize)
}

impl<'ctx> PvmEmitter<'ctx> {
    fn new() -> Self {
        Self {
            instructions: Vec::new(),
            labels: Vec::new(),
            fixups: Vec::new(),
            block_labels: HashMap::new(),
            value_slots: HashMap::new(),
            next_slot_offset: FRAME_HEADER_SIZE,
            frame_size: 0,
            call_fixups: Vec::new(),
            indirect_call_fixups: Vec::new(),
            result_globals: None,
            entry_returns_ptr_len: false,
            wasm_memory_base: 0,
        }
    }

    fn alloc_label(&mut self) -> usize {
        let id = self.labels.len();
        self.labels.push(None);
        id
    }

    fn define_label(&mut self, label: usize) {
        if self
            .instructions
            .last()
            .is_some_and(|last| !last.is_terminating())
        {
            self.emit(Instruction::Fallthrough);
        }
        self.labels[label] = Some(self.current_offset());
    }

    fn current_offset(&self) -> usize {
        self.instructions.iter().map(|i| i.encode().len()).sum()
    }

    fn emit(&mut self, instr: Instruction) {
        self.instructions.push(instr);
    }

    fn emit_jump_to_label(&mut self, label: usize) {
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, label));
        self.emit(Instruction::Jump { offset: 0 });
    }

    fn emit_branch_ne_imm_to_label(&mut self, reg: u8, value: i32, label: usize) {
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, label));
        self.emit(Instruction::BranchNeImm {
            reg,
            value,
            offset: 0,
        });
    }

    fn emit_branch_eq_imm_to_label(&mut self, reg: u8, value: i32, label: usize) {
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, label));
        self.emit(Instruction::BranchEqImm {
            reg,
            value,
            offset: 0,
        });
    }

    // ── Slot allocation ──

    fn alloc_slot_for_key(&mut self, key: ValKey) -> i32 {
        let offset = self.next_slot_offset;
        self.value_slots.insert(key, offset);
        self.next_slot_offset += 8;
        offset
    }

    fn get_slot(&self, key: ValKey) -> Option<i32> {
        self.value_slots.get(&key).copied()
    }

    // ── Value load / store ──

    /// Load a value into a temp register. Constants are inlined; SSA values are loaded from slots.
    fn load_operand(&mut self, val: BasicValueEnum<'ctx>, temp_reg: u8) {
        match val {
            BasicValueEnum::IntValue(iv) => {
                if let Some(const_val) = iv.get_zero_extended_constant() {
                    self.emit_load_const(temp_reg, const_val);
                } else if let Some(slot) = self.get_slot(val_key_int(iv)) {
                    self.emit(Instruction::LoadIndU64 {
                        dst: temp_reg,
                        base: STACK_PTR_REG,
                        offset: slot,
                    });
                } else {
                    // Unknown value — shouldn't happen. Load 0 as fallback.
                    self.emit(Instruction::LoadImm {
                        reg: temp_reg,
                        value: 0,
                    });
                }
            }
            _ => {
                // Non-int values shouldn't appear in our IR (all i64).
                self.emit(Instruction::LoadImm {
                    reg: temp_reg,
                    value: 0,
                });
            }
        }
    }

    fn emit_load_const(&mut self, reg: u8, val: u64) {
        if val == 0 {
            self.emit(Instruction::LoadImm { reg, value: 0 });
        } else if i32::try_from(val).is_ok() {
            self.emit(Instruction::LoadImm {
                reg,
                value: val as i32,
            });
        } else {
            self.emit(Instruction::LoadImm64 { reg, value: val });
        }
    }

    fn store_to_slot(&mut self, slot_offset: i32, src_reg: u8) {
        self.emit(Instruction::StoreIndU64 {
            base: STACK_PTR_REG,
            src: src_reg,
            offset: slot_offset,
        });
    }

    // ── Fixup resolution ──

    fn resolve_fixups(&mut self) -> Result<()> {
        for &(instr_idx, label_id) in &self.fixups {
            let target_offset = self.labels[label_id]
                .ok_or_else(|| Error::Unsupported("unresolved label".to_string()))?;

            let instr_start: usize = self.instructions[..instr_idx]
                .iter()
                .map(|i| i.encode().len())
                .sum();

            let relative_offset = (target_offset as i32) - (instr_start as i32);

            match &mut self.instructions[instr_idx] {
                Instruction::Jump { offset }
                | Instruction::BranchNeImm { offset, .. }
                | Instruction::BranchEqImm { offset, .. }
                | Instruction::BranchGeSImm { offset, .. }
                | Instruction::BranchGeU { offset, .. }
                | Instruction::BranchLtU { offset, .. } => {
                    *offset = relative_offset;
                }
                _ => {
                    return Err(Error::Unsupported(
                        "cannot fixup non-jump instruction".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}

// ── Public API ──

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
    emit_prologue(&mut emitter, function, ctx, is_main);

    // Phase 3: Lower each basic block.
    for bb in function.get_basic_blocks() {
        let label = emitter.block_labels[&bb];
        emitter.define_label(label);

        for instruction in bb.get_instructions() {
            lower_instruction(&mut emitter, instruction, bb, ctx, is_main)?;
        }
    }

    emitter.resolve_fixups()?;

    Ok(LlvmFunctionTranslation {
        instructions: emitter.instructions,
        call_fixups: emitter.call_fixups,
        indirect_call_fixups: emitter.indirect_call_fixups,
    })
}

// ── Pre-scan ──

fn pre_scan_function<'ctx>(emitter: &mut PvmEmitter<'ctx>, function: FunctionValue<'ctx>) {
    // Allocate slots for function parameters.
    for param in function.get_params() {
        let key = val_key_basic(param);
        emitter.alloc_slot_for_key(key);
    }

    // Allocate labels for all basic blocks.
    for bb in function.get_basic_blocks() {
        let label = emitter.alloc_label();
        emitter.block_labels.insert(bb, label);
    }

    // Allocate slots for instruction results that produce integer values.
    for bb in function.get_basic_blocks() {
        for instr in bb.get_instructions() {
            if instruction_produces_value(instr) {
                let key = val_key_instr(instr);
                emitter.alloc_slot_for_key(key);
            }
        }
    }
}

fn instruction_produces_value(instr: InstructionValue<'_>) -> bool {
    matches!(
        instr.get_opcode(),
        InstructionOpcode::Add
            | InstructionOpcode::Sub
            | InstructionOpcode::Mul
            | InstructionOpcode::UDiv
            | InstructionOpcode::SDiv
            | InstructionOpcode::URem
            | InstructionOpcode::SRem
            | InstructionOpcode::And
            | InstructionOpcode::Or
            | InstructionOpcode::Xor
            | InstructionOpcode::Shl
            | InstructionOpcode::LShr
            | InstructionOpcode::AShr
            | InstructionOpcode::ICmp
            | InstructionOpcode::ZExt
            | InstructionOpcode::SExt
            | InstructionOpcode::Trunc
            | InstructionOpcode::Select
            | InstructionOpcode::Phi
            | InstructionOpcode::Load
            | InstructionOpcode::Call
    )
}

// ── Prologue / Epilogue ──

fn emit_prologue<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    function: FunctionValue<'ctx>,
    ctx: &LoweringContext,
    is_main: bool,
) {
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
            src: STACK_PTR_REG,
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
        dst: STACK_PTR_REG,
        src: STACK_PTR_REG,
        value: -e.frame_size,
    });

    if !is_main {
        // Save return address.
        e.emit(Instruction::StoreIndU64 {
            base: STACK_PTR_REG,
            src: RETURN_ADDR_REG,
            offset: 0,
        });

        // Save callee-saved registers r9-r12.
        for i in 0..MAX_LOCAL_REGS {
            e.emit(Instruction::StoreIndU64 {
                base: STACK_PTR_REG,
                src: FIRST_LOCAL_REG + i as u8,
                offset: (8 + i * 8) as i32,
            });
        }
    }

    // Copy parameters to their SSA slots.
    let params = function.get_params();
    for (i, param) in params.iter().enumerate() {
        let key = val_key_basic(*param);
        let slot = e.get_slot(key).unwrap();

        if is_main {
            // For main, SPI passes r7=args_ptr, r8=args_len.
            // Adjust args_ptr by subtracting wasm_memory_base.
            if i == 0 {
                e.emit(Instruction::AddImm64 {
                    dst: ARGS_PTR_REG,
                    src: ARGS_PTR_REG,
                    value: -ctx.wasm_memory_base,
                });
                e.store_to_slot(slot, ARGS_PTR_REG);
            } else if i == 1 {
                e.store_to_slot(slot, ARGS_LEN_REG);
            }
        } else if i < MAX_LOCAL_REGS {
            // First 4 params come in r9-r12.
            e.store_to_slot(slot, FIRST_LOCAL_REG + i as u8);
        } else {
            // Overflow params from PARAM_OVERFLOW_BASE.
            let overflow_offset =
                abi::PARAM_OVERFLOW_BASE + ((i - MAX_LOCAL_REGS) * 8) as i32;
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
}

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
        // Restore callee-saved registers r9-r12.
        for i in 0..MAX_LOCAL_REGS {
            e.emit(Instruction::LoadIndU64 {
                dst: FIRST_LOCAL_REG + i as u8,
                base: STACK_PTR_REG,
                offset: (8 + i * 8) as i32,
            });
        }

        // Restore return address.
        e.emit(Instruction::LoadIndU64 {
            dst: RETURN_ADDR_REG,
            base: STACK_PTR_REG,
            offset: 0,
        });

        // Deallocate frame.
        e.emit(Instruction::AddImm64 {
            dst: STACK_PTR_REG,
            src: STACK_PTR_REG,
            value: e.frame_size,
        });

        // Return.
        e.emit(Instruction::JumpInd {
            reg: RETURN_ADDR_REG,
            offset: 0,
        });
    }
}

// ── Instruction Lowering ──

fn lower_instruction<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    current_bb: BasicBlock<'ctx>,
    ctx: &LoweringContext,
    is_main: bool,
) -> Result<()> {
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
        InstructionOpcode::Load => lower_load(e, instr, ctx),
        InstructionOpcode::Store => lower_store(e, instr, ctx),

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

// ── Helpers to get operands ──

/// Get the i-th operand of an instruction as a `BasicValueEnum`.
fn get_operand(instr: InstructionValue<'_>, i: u32) -> Result<BasicValueEnum<'_>> {
    instr
        .get_operand(i)
        .and_then(inkwell::values::Operand::value)
        .ok_or_else(|| Error::Internal(format!("missing operand {i} in {:?}", instr.get_opcode())))
}

/// Get the i-th operand of an instruction as a `BasicBlock`.
fn get_bb_operand(instr: InstructionValue<'_>, i: u32) -> Result<BasicBlock<'_>> {
    instr
        .get_operand(i)
        .and_then(inkwell::values::Operand::block)
        .ok_or_else(|| {
            Error::Internal(format!(
                "missing bb operand {i} in {:?}",
                instr.get_opcode()
            ))
        })
}

/// Get the slot offset for an instruction's result.
fn result_slot(e: &PvmEmitter<'_>, instr: InstructionValue<'_>) -> Result<i32> {
    let key = val_key_instr(instr);
    e.get_slot(key)
        .ok_or_else(|| Error::Internal(format!("no slot for {:?} result", instr.get_opcode())))
}

/// Detect the bit width of an instruction's result or first operand.
fn operand_bit_width(instr: InstructionValue<'_>) -> u32 {
    // For most instructions, check the operand type.
    if let Some(op) = instr
        .get_operand(0)
        .and_then(inkwell::values::Operand::value)
        && let BasicValueEnum::IntValue(iv) = op
    {
        return iv.get_type().get_bit_width();
    }
    64 // default
}

// ── Binary operations ──

#[derive(Clone, Copy)]
enum BinaryOp {
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

fn lower_binary_arith<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    op: BinaryOp,
) -> Result<()> {
    let lhs = get_operand(instr, 0)?;
    let rhs = get_operand(instr, 1)?;
    let slot = result_slot(e, instr)?;
    let bits = operand_bit_width(instr);

    e.load_operand(lhs, TEMP1);
    e.load_operand(rhs, TEMP2);

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
        (BinaryOp::UDiv, true) => e.emit(Instruction::DivU32 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::UDiv, false) => e.emit(Instruction::DivU64 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::SDiv, true) => e.emit(Instruction::DivS32 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::SDiv, false) => e.emit(Instruction::DivS64 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::URem, true) => e.emit(Instruction::RemU32 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::URem, false) => e.emit(Instruction::RemU64 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::SRem, true) => e.emit(Instruction::RemS32 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
        (BinaryOp::SRem, false) => e.emit(Instruction::RemS64 {
            dst: TEMP_RESULT,
            src1: TEMP1,
            src2: TEMP2,
        }),
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

// ── Comparisons ──

fn lower_icmp<'ctx>(e: &mut PvmEmitter<'ctx>, instr: InstructionValue<'ctx>) -> Result<()> {
    let lhs = get_operand(instr, 0)?;
    let rhs = get_operand(instr, 1)?;
    let slot = result_slot(e, instr)?;

    let pred = instr
        .get_icmp_predicate()
        .ok_or_else(|| Error::Internal("ICmp without predicate".into()))?;

    e.load_operand(lhs, TEMP1);
    e.load_operand(rhs, TEMP2);

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

// ── Conversions ──

fn lower_zext<'ctx>(e: &mut PvmEmitter<'ctx>, instr: InstructionValue<'ctx>) -> Result<()> {
    let src = get_operand(instr, 0)?;
    let slot = result_slot(e, instr)?;
    let from_bits = operand_bit_width(instr);

    e.load_operand(src, TEMP1);

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

fn lower_sext<'ctx>(e: &mut PvmEmitter<'ctx>, instr: InstructionValue<'ctx>) -> Result<()> {
    let src = get_operand(instr, 0)?;
    let slot = result_slot(e, instr)?;
    let from_bits = operand_bit_width(instr);

    e.load_operand(src, TEMP1);

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

fn lower_trunc<'ctx>(e: &mut PvmEmitter<'ctx>, instr: InstructionValue<'ctx>) -> Result<()> {
    let src = get_operand(instr, 0)?;
    let slot = result_slot(e, instr)?;

    e.load_operand(src, TEMP1);

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

// ── Select ──

fn lower_select<'ctx>(e: &mut PvmEmitter<'ctx>, instr: InstructionValue<'ctx>) -> Result<()> {
    // select i1 %cond, i64 %true_val, i64 %false_val
    let cond = get_operand(instr, 0)?;
    let true_val = get_operand(instr, 1)?;
    let false_val = get_operand(instr, 2)?;
    let slot = result_slot(e, instr)?;

    // Start with false_val in result slot.
    e.load_operand(false_val, TEMP_RESULT);
    e.store_to_slot(slot, TEMP_RESULT);

    // If cond != 0, overwrite with true_val.
    e.load_operand(cond, TEMP1);
    let skip_label = e.alloc_label();
    e.emit_branch_eq_imm_to_label(TEMP1, 0, skip_label);

    e.load_operand(true_val, TEMP_RESULT);
    e.store_to_slot(slot, TEMP_RESULT);

    e.define_label(skip_label);
    Ok(())
}

// ── Phi node elimination ──

/// Emit copies for phi nodes in `target_bb` that have incoming values from `current_bb`.
///
/// Uses a two-pass approach to handle potential phi cycles: first loads all
/// incoming values into temp registers (or temp stack slots), then stores them
/// to the phi node slots.
fn emit_phi_copies<'ctx>(
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
        e.load_operand(value, TEMP1);
        e.store_to_slot(slot, TEMP1);
    } else {
        // Multiple phis — use two-pass to avoid clobbering.
        let temp_regs = [TEMP1, TEMP2, TEMP_RESULT, SCRATCH1, SCRATCH2];

        if copies.len() <= temp_regs.len() {
            // All fit in temp registers: load all first, then store all.
            for (i, (_, value)) in copies.iter().enumerate() {
                e.load_operand(*value, temp_regs[i]);
            }
            for (i, (slot, _)) in copies.iter().enumerate() {
                e.store_to_slot(*slot, temp_regs[i]);
            }
        } else {
            // Fallback: use temp stack space below the frame (negative offsets from SP).
            for (i, (_, value)) in copies.iter().enumerate() {
                e.load_operand(*value, TEMP1);
                e.emit(Instruction::StoreIndU64 {
                    base: STACK_PTR_REG,
                    src: TEMP1,
                    offset: -(8 + i as i32 * 8),
                });
            }
            for (i, (slot, _)) in copies.iter().enumerate() {
                e.emit(Instruction::LoadIndU64 {
                    dst: TEMP1,
                    base: STACK_PTR_REG,
                    offset: -(8 + i as i32 * 8),
                });
                e.store_to_slot(*slot, TEMP1);
            }
        }
    }

    Ok(())
}

/// Check whether `target_bb` has any phi nodes with incomings from `current_bb`.
fn has_phi_from<'ctx>(current_bb: BasicBlock<'ctx>, target_bb: BasicBlock<'ctx>) -> bool {
    for instr in target_bb.get_instructions() {
        if instr.get_opcode() != InstructionOpcode::Phi {
            break;
        }
        let Ok(phi): std::result::Result<PhiValue<'ctx>, _> = instr.try_into() else {
            break;
        };
        let num_incomings = phi.count_incoming();
        for i in 0..num_incomings {
            if let Some((_, block)) = phi.get_incoming(i)
                && block == current_bb
            {
                return true;
            }
        }
    }
    false
}

// ── Control flow ──

fn lower_br<'ctx>(
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

        e.load_operand(cond, TEMP1);

        if !then_has_phis && !else_has_phis {
            // No phi copies needed — simple branch.
            e.emit_branch_ne_imm_to_label(TEMP1, 0, then_label);
            e.emit_jump_to_label(else_label);
        } else {
            // Need per-edge phi copies. Create trampolines.
            let then_trampoline = e.alloc_label();
            e.emit_branch_ne_imm_to_label(TEMP1, 0, then_trampoline);

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

fn lower_switch<'ctx>(
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

    e.load_operand(val, TEMP1);

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

#[allow(clippy::unnecessary_wraps)]
fn lower_return<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    is_main: bool,
) -> Result<()> {
    if is_main {
        if let Some((ptr_global, len_global)) = e.result_globals {
            // Globals convention: load result_ptr and result_len from WASM globals.
            // JAM SPI result convention: r7 = start address, r8 = end address.
            let wasm_memory_base = e.wasm_memory_base;
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
                dst: ARGS_PTR_REG,
                src: TEMP1,
                value: wasm_memory_base,
            });
            // r8 = r7 + len (end PVM address)
            e.emit(Instruction::Add64 {
                dst: ARGS_LEN_REG,
                src1: ARGS_PTR_REG,
                src2: TEMP2,
            });
        } else if e.entry_returns_ptr_len && instr.get_num_operands() > 0 {
            // Packed (ptr, len) convention: return value is packed i64.
            // Lower 32 bits = WASM ptr, upper 32 bits = len.
            // JAM SPI result convention: r7 = start address, r8 = end address.
            if let Ok(ret_val) = get_operand(instr, 0) {
                let wasm_memory_base = e.wasm_memory_base;
                e.load_operand(ret_val, TEMP1);
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
                    dst: ARGS_PTR_REG,
                    src: TEMP1,
                    value: wasm_memory_base,
                });
                // r8 = r7 + len (end address)
                e.emit(Instruction::Add64 {
                    dst: ARGS_LEN_REG,
                    src1: ARGS_PTR_REG,
                    src2: TEMP2,
                });
            }
        } else if instr.get_num_operands() > 0 {
            // Entry function returns a value → r7.
            if let Ok(ret_val) = get_operand(instr, 0) {
                e.load_operand(ret_val, RETURN_VALUE_REG);
            }
        }
    } else {
        // Normal function: ret void | ret i64 %val → r7.
        if instr.get_num_operands() > 0
            && let Ok(ret_val) = get_operand(instr, 0)
        {
            e.load_operand(ret_val, RETURN_VALUE_REG);
        }
    }

    emit_epilogue(e, is_main);
    Ok(())
}

// ── Load / Store (globals) ──

fn lower_load<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    _ctx: &LoweringContext,
) -> Result<()> {
    // After mem2reg, remaining loads are from LLVM globals (WASM globals).
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

fn lower_store<'ctx>(
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
            e.load_operand(val, TEMP1);
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

// ── Calls (intrinsic recognition) ──

fn lower_call<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
) -> Result<()> {
    // Get the called function name from the last operand (the function pointer).
    let num_operands = instr.get_num_operands();
    let fn_operand = instr
        .get_operand(num_operands - 1)
        .and_then(inkwell::values::Operand::value)
        .ok_or_else(|| Error::Internal("call without function operand".into()))?;
    let fn_name = match fn_operand {
        BasicValueEnum::PointerValue(pv) => pv.get_name().to_string_lossy().to_string(),
        _ => return Err(Error::Internal("call operand is not a pointer".into())),
    };

    if fn_name.starts_with("__pvm_") {
        return lower_pvm_intrinsic(e, instr, &fn_name, ctx);
    }

    if fn_name.starts_with("llvm.") {
        return lower_llvm_intrinsic(e, instr, &fn_name);
    }

    // Regular WASM function call.
    lower_wasm_call(e, instr, &fn_name, ctx)
}

// ── WASM function calls ──

fn lower_wasm_call<'ctx>(
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
        if i < MAX_LOCAL_REGS {
            e.load_operand(arg, FIRST_LOCAL_REG + i as u8);
        } else {
            e.load_operand(arg, TEMP1);
            let overflow_offset =
                abi::PARAM_OVERFLOW_BASE + ((i - MAX_LOCAL_REGS) * 8) as i32;
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
        reg: RETURN_ADDR_REG,
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
        e.store_to_slot(slot, RETURN_VALUE_REG);
    }

    Ok(())
}

/// Emit a stub for calling an imported function.
/// Imported functions are not available at PVM level — emit Trap for abort-like
/// functions and a dummy return value (0) for others.
#[allow(clippy::unnecessary_wraps)]
fn lower_import_call<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
    global_func_idx: u32,
) -> Result<()> {
    let (_num_params, has_return) = ctx
        .function_signatures
        .get(global_func_idx as usize)
        .copied()
        .unwrap_or((0, false));

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

fn lower_pvm_intrinsic<'ctx>(
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
        "__pvm_call_indirect" => lower_pvm_call_indirect(e, instr, ctx),

        _ => Err(Error::Unsupported(format!("unknown PVM intrinsic: {name}"))),
    }
}

#[derive(Clone, Copy)]
enum PvmLoadKind {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32S,
    U64,
}

#[derive(Clone, Copy)]
enum PvmStoreKind {
    U8,
    U16,
    U32,
    U64,
}

fn emit_pvm_load<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
    kind: PvmLoadKind,
) -> Result<()> {
    let addr = get_operand(instr, 0)?;
    let slot = result_slot(e, instr)?;

    e.load_operand(addr, TEMP1);

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

fn emit_pvm_store<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
    kind: PvmStoreKind,
) -> Result<()> {
    // Intrinsic: __pvm_store_*(addr, val)
    let addr = get_operand(instr, 0)?;
    let val = get_operand(instr, 1)?;

    e.load_operand(addr, TEMP1);
    e.load_operand(val, TEMP2);

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

fn emit_pvm_memory_size<'ctx>(
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

fn emit_pvm_memory_grow<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
) -> Result<()> {
    let delta = get_operand(instr, 0)?;
    let slot = result_slot(e, instr)?;
    let global_addr = abi::memory_size_global_offset(ctx.num_globals);

    // Load delta into SCRATCH1.
    e.load_operand(delta, SCRATCH1);

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

    // Check new_size > max_pages → fail.
    let fail_label = e.alloc_label();
    let end_label = e.alloc_label();

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

fn emit_pvm_memory_fill<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
) -> Result<()> {
    // __pvm_memory_fill(dst, val, len)
    let dst_addr = get_operand(instr, 0)?;
    let val = get_operand(instr, 1)?;
    let len = get_operand(instr, 2)?;

    e.load_operand(dst_addr, TEMP1); // dest
    e.load_operand(val, TEMP2); // value
    e.load_operand(len, TEMP_RESULT); // size (counter)

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

fn emit_pvm_memory_copy<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    ctx: &LoweringContext,
) -> Result<()> {
    // __pvm_memory_copy(dst, src, len)
    let dst_addr = get_operand(instr, 0)?;
    let src_addr = get_operand(instr, 1)?;
    let len = get_operand(instr, 2)?;

    e.load_operand(dst_addr, TEMP1); // dest
    e.load_operand(src_addr, TEMP2); // src
    e.load_operand(len, TEMP_RESULT); // size (counter)

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

// ── Indirect call ──

fn lower_pvm_call_indirect<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    _ctx: &LoweringContext,
) -> Result<()> {
    // __pvm_call_indirect(type_idx, table_entry, arg0, arg1, ...)
    // Operands: [type_idx, table_entry, arg0, ..., argN-1, fn_ptr]
    let num_operands = instr.get_num_operands();
    let num_args = (num_operands - 3) as usize; // subtract type_idx, table_entry, fn_ptr

    let type_idx_val = get_operand(instr, 0)?;
    let table_entry_val = get_operand(instr, 1)?;

    // Load type_idx as an immediate (it's always a constant).
    let expected_type_idx = match type_idx_val {
        BasicValueEnum::IntValue(iv) => iv.get_zero_extended_constant().unwrap_or(0) as u32,
        _ => 0,
    };

    // Load table entry index into SCRATCH2 and save it early.
    e.load_operand(table_entry_val, SCRATCH2);
    // Save table index below the frame where it won't be clobbered by arg loading.
    e.emit(Instruction::StoreIndU64 {
        base: STACK_PTR_REG,
        src: SCRATCH2,
        offset: -8,
    });

    // Load function arguments into r9-r12 and overflow area.
    for i in 0..num_args {
        let arg = get_operand(instr, (i + 2) as u32)?;
        if i < MAX_LOCAL_REGS {
            e.load_operand(arg, FIRST_LOCAL_REG + i as u8);
        } else {
            e.load_operand(arg, TEMP1);
            let overflow_offset =
                abi::PARAM_OVERFLOW_BASE + ((i - MAX_LOCAL_REGS) * 8) as i32;
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
        dst: ARGS_LEN_REG, // r8, used as SAVED_TABLE_IDX_REG
        base: STACK_PTR_REG,
        offset: -8,
    });

    // Dispatch table lookup: each entry is 8 bytes (4-byte jump ref + 4-byte type index).
    // Multiply table index by 8 (entry size) via 3 doublings: idx * 2 * 2 * 2
    // table_addr = RO_DATA_BASE + table_idx * 8
    e.emit(Instruction::Add32 {
        dst: ARGS_LEN_REG,
        src1: ARGS_LEN_REG,
        src2: ARGS_LEN_REG,
    });
    e.emit(Instruction::Add32 {
        dst: ARGS_LEN_REG,
        src1: ARGS_LEN_REG,
        src2: ARGS_LEN_REG,
    });
    e.emit(Instruction::Add32 {
        dst: ARGS_LEN_REG,
        src1: ARGS_LEN_REG,
        src2: ARGS_LEN_REG,
    });
    e.emit(Instruction::AddImm32 {
        dst: ARGS_LEN_REG,
        src: ARGS_LEN_REG,
        value: abi::RO_DATA_BASE,
    });

    // Load and validate type signature.
    e.emit(Instruction::LoadIndU32 {
        dst: TEMP1,
        base: ARGS_LEN_REG,
        offset: 4, // type index at offset 4
    });

    let sig_ok_label = e.alloc_label();
    e.emit_branch_eq_imm_to_label(TEMP1, expected_type_idx as i32, sig_ok_label);
    e.emit(Instruction::Trap);
    e.define_label(sig_ok_label);

    // Load jump address from dispatch table (at offset 0).
    e.emit(Instruction::LoadIndU32 {
        dst: ARGS_LEN_REG,
        base: ARGS_LEN_REG,
        offset: 0,
    });

    // Emit indirect call: LoadImm64 for return address + JumpInd.
    let return_addr_instr = e.instructions.len();
    e.emit(Instruction::LoadImm64 {
        reg: RETURN_ADDR_REG,
        value: 0, // patched during fixup resolution
    });
    let jump_ind_instr = e.instructions.len();
    e.emit(Instruction::JumpInd {
        reg: ARGS_LEN_REG,
        offset: 0,
    });

    e.emit(Instruction::Fallthrough);

    e.indirect_call_fixups.push(LlvmIndirectCallFixup {
        return_addr_instr,
        jump_ind_instr,
    });

    // Store return value if the call produces one.
    if let Ok(slot) = result_slot(e, instr) {
        e.store_to_slot(slot, RETURN_VALUE_REG);
    }

    Ok(())
}

// ── LLVM intrinsics (ctlz, cttz, ctpop, fshl, fshr) ──

fn lower_llvm_intrinsic<'ctx>(
    e: &mut PvmEmitter<'ctx>,
    instr: InstructionValue<'ctx>,
    name: &str,
) -> Result<()> {
    let slot = result_slot(e, instr)?;

    if name.contains("ctlz") {
        let val = get_operand(instr, 0)?;
        e.load_operand(val, TEMP1);
        if name.contains("i32") {
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
        e.load_operand(val, TEMP1);
        if name.contains("i32") {
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
        e.load_operand(val, TEMP1);
        if name.contains("i32") {
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
        let is_32 = name.contains("i32");
        let bits = if is_32 { 32 } else { 64 };

        e.load_operand(a, TEMP1);
        e.load_operand(b, TEMP2);
        e.load_operand(amt, SCRATCH1);

        // Mask shift amount to valid range.
        e.emit(Instruction::LoadImm {
            reg: SCRATCH2,
            value: bits - 1,
        });
        e.emit(Instruction::And {
            dst: SCRATCH1,
            src1: SCRATCH1,
            src2: SCRATCH2,
        });

        // Compute bits - amt into SCRATCH2.
        e.emit(Instruction::LoadImm {
            reg: SCRATCH2,
            value: bits,
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

    Err(Error::Unsupported(format!(
        "unsupported LLVM intrinsic: {name}"
    )))
}
