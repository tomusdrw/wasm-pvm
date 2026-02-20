// Core PVM emitter and value management.
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
    AnyValue, AsValueRef, BasicValueEnum, FunctionValue, InstructionValue, IntValue, PhiValue,
};

use crate::pvm::Instruction;
use crate::{Error, Result};

use crate::abi::{FRAME_HEADER_SIZE, STACK_PTR_REG};

/// Context for lowering functions from a single WASM module.
pub struct LoweringContext {
    pub wasm_memory_base: i32,
    pub num_globals: usize,
    pub function_signatures: Vec<(usize, bool)>,
    pub type_signatures: Vec<(usize, usize)>,
    pub function_table: Vec<u32>,
    pub num_imported_funcs: usize,
    pub imported_func_names: Vec<String>,
    pub initial_memory_pages: u32,
    pub max_memory_pages: u32,
    pub stack_size: u32,
    /// Map from data segment index to offset in `RO_DATA` (for passive segments).
    pub data_segment_offsets: HashMap<u32, u32>,
    /// Map from data segment index to byte length (for passive segments bounds checking).
    pub data_segment_lengths: HashMap<u32, u32>,
    /// Map from data segment index to PVM address storing the effective length at runtime.
    /// Used by `memory.init` for bounds checking and by `data.drop` to zero the length.
    pub data_segment_length_addrs: HashMap<u32, i32>,
    /// User-provided mapping from WASM import names to actions (trap, nop).
    pub wasm_import_map: Option<HashMap<String, crate::translate::ImportAction>>,
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

/// PVM code emitter for a single function.
pub struct PvmEmitter<'ctx> {
    pub(crate) instructions: Vec<Instruction>,
    pub(crate) labels: Vec<Option<usize>>,
    pub(crate) fixups: Vec<(usize, usize)>,

    /// LLVM basic block → PVM label.
    pub(crate) block_labels: HashMap<BasicBlock<'ctx>, usize>,

    /// LLVM int values (params + instruction results) → stack slot offset from SP.
    pub(crate) value_slots: HashMap<ValKey, i32>,

    /// Next available slot offset (bump allocator, starts after frame header).
    pub(crate) next_slot_offset: i32,

    /// Total frame size (set after pre-scan).
    pub(crate) frame_size: i32,

    pub(crate) call_fixups: Vec<LlvmCallFixup>,
    pub(crate) indirect_call_fixups: Vec<LlvmIndirectCallFixup>,

    /// For entry functions: (`result_ptr_global`, `result_len_global`) indices.
    /// When set, the epilogue loads these globals into r7/r8 before exiting.
    pub(crate) result_globals: Option<(u32, u32)>,

    /// When true, the entry function's return value is a packed i64:
    /// lower 32 bits = ptr (WASM address), upper 32 bits = len.
    pub(crate) entry_returns_ptr_len: bool,

    /// WASM memory base address (for converting WASM addresses to PVM addresses).
    pub(crate) wasm_memory_base: i32,

    /// Current byte offset of the emitted code (for O(1) offset calculations).
    pub(crate) byte_offset: usize,

    /// Maps stack slot offset → register that currently holds this slot's value.
    slot_cache: HashMap<i32, u8>,
    /// Reverse: register → slot offset it holds (for fast invalidation).
    reg_to_slot: [Option<i32>; 13],

    /// Pending fused `ICmp`: when an `ICmp` has a single use (by a branch), we defer
    /// it and store the predicate + operands here. The branch will emit a fused
    /// branch instruction instead of loading the boolean result.
    pub(crate) pending_fused_icmp: Option<FusedIcmp<'ctx>>,
}

/// Deferred `ICmp` info for branch fusion.
pub struct FusedIcmp<'ctx> {
    pub predicate: IntPredicate,
    pub lhs: BasicValueEnum<'ctx>,
    pub rhs: BasicValueEnum<'ctx>,
}

/// Wrapper key for LLVM values in the slot map.
/// Uses the raw LLVM value pointer cast to usize for hashing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ValKey(pub(crate) usize);

pub fn val_key_int(val: IntValue<'_>) -> ValKey {
    ValKey(val.as_value_ref() as usize)
}

pub fn val_key_basic(val: BasicValueEnum<'_>) -> ValKey {
    ValKey(val.as_value_ref() as usize)
}

pub fn val_key_instr(val: InstructionValue<'_>) -> ValKey {
    ValKey(val.as_value_ref() as usize)
}

impl<'ctx> PvmEmitter<'ctx> {
    pub fn new() -> Self {
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
            byte_offset: 0,
            slot_cache: HashMap::new(),
            reg_to_slot: [None; 13],
            pending_fused_icmp: None,
        }
    }

    pub fn alloc_label(&mut self) -> usize {
        let id = self.labels.len();
        self.labels.push(None);
        id
    }

    pub fn define_label(&mut self, label: usize) {
        if self
            .instructions
            .last()
            .is_some_and(|last| !last.is_terminating())
        {
            self.emit(Instruction::Fallthrough);
        }
        self.labels[label] = Some(self.current_offset());
        self.clear_reg_cache();
    }

    pub fn current_offset(&self) -> usize {
        self.byte_offset
    }

    pub fn emit(&mut self, instr: Instruction) {
        if let Some(reg) = instr.dest_reg() {
            self.invalidate_reg(reg);
        }
        self.byte_offset += instr.encode().len();
        self.instructions.push(instr);
    }

    pub fn emit_jump_to_label(&mut self, label: usize) {
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, label));
        self.emit(Instruction::Jump { offset: 0 });
    }

    pub fn emit_branch_ne_imm_to_label(&mut self, reg: u8, value: i32, label: usize) {
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, label));
        self.emit(Instruction::BranchNeImm {
            reg,
            value,
            offset: 0,
        });
    }

    pub fn emit_branch_eq_imm_to_label(&mut self, reg: u8, value: i32, label: usize) {
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, label));
        self.emit(Instruction::BranchEqImm {
            reg,
            value,
            offset: 0,
        });
    }

    pub fn emit_branch_lt_u_to_label(&mut self, reg1: u8, reg2: u8, label: usize) {
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, label));
        self.emit(Instruction::BranchLtU {
            reg1,
            reg2,
            offset: 0,
        });
    }

    pub fn emit_branch_ge_u_to_label(&mut self, reg1: u8, reg2: u8, label: usize) {
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, label));
        self.emit(Instruction::BranchGeU {
            reg1,
            reg2,
            offset: 0,
        });
    }

    pub fn emit_branch_eq_to_label(&mut self, reg1: u8, reg2: u8, label: usize) {
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, label));
        self.emit(Instruction::BranchEq {
            reg1,
            reg2,
            offset: 0,
        });
    }

    pub fn emit_branch_ne_to_label(&mut self, reg1: u8, reg2: u8, label: usize) {
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, label));
        self.emit(Instruction::BranchNe {
            reg1,
            reg2,
            offset: 0,
        });
    }

    pub fn emit_branch_lt_s_to_label(&mut self, reg1: u8, reg2: u8, label: usize) {
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, label));
        self.emit(Instruction::BranchLtS {
            reg1,
            reg2,
            offset: 0,
        });
    }

    pub fn emit_branch_ge_s_to_label(&mut self, reg1: u8, reg2: u8, label: usize) {
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, label));
        self.emit(Instruction::BranchGeS {
            reg1,
            reg2,
            offset: 0,
        });
    }

    // ── Slot allocation ──

    pub fn alloc_slot_for_key(&mut self, key: ValKey) -> i32 {
        let offset = self.next_slot_offset;
        self.value_slots.insert(key, offset);
        self.next_slot_offset += 8;
        offset
    }

    pub fn get_slot(&self, key: ValKey) -> Option<i32> {
        self.value_slots.get(&key).copied()
    }

    // ── Value load / store ──

    /// Load a value into a temp register. Constants are inlined; SSA values are loaded from slots.
    /// Poison/undef values are materialized as 0 (any value is valid for undefined behavior).
    /// Returns an error for truly unknown values.
    pub fn load_operand(&mut self, val: BasicValueEnum<'ctx>, temp_reg: u8) -> Result<()> {
        match val {
            BasicValueEnum::IntValue(iv) => {
                if let Some(signed_val) = iv.get_sign_extended_constant() {
                    // Try sign-extended first: negative i32 values emit compact
                    // LoadImm instead of LoadImm64.
                    if let Ok(v32) = i32::try_from(signed_val) {
                        self.emit(Instruction::LoadImm {
                            reg: temp_reg,
                            value: v32,
                        });
                    } else {
                        self.emit(Instruction::LoadImm64 {
                            reg: temp_reg,
                            value: signed_val as u64,
                        });
                    }
                } else if let Some(const_val) = iv.get_zero_extended_constant() {
                    // Fallback for unsigned constants.
                    self.emit_load_const(temp_reg, const_val);
                } else if let Some(slot) = self.get_slot(val_key_int(iv)) {
                    // Check register cache: skip load if value is already in a register.
                    if let Some(&cached_reg) = self.slot_cache.get(&slot) {
                        if cached_reg != temp_reg {
                            // Emit a register copy (cheaper than memory load).
                            self.emit(Instruction::MoveReg {
                                dst: temp_reg,
                                src: cached_reg,
                            });
                        }
                        // If cached_reg == temp_reg, skip entirely (0 instructions).
                    } else {
                        self.emit(Instruction::LoadIndU64 {
                            dst: temp_reg,
                            base: STACK_PTR_REG,
                            offset: slot,
                        });
                        self.cache_slot(slot, temp_reg);
                    }
                } else if iv.is_poison() || iv.is_undef() {
                    // Poison/undef values can be materialized as any value; use 0.
                    self.emit(Instruction::LoadImm {
                        reg: temp_reg,
                        value: 0,
                    });
                } else {
                    return Err(Error::Internal(format!(
                        "no slot for int value {:?} (opcode: {:?})",
                        val_key_int(iv),
                        iv.as_instruction().map(InstructionValue::get_opcode),
                    )));
                }
            }
            _ => {
                return Err(Error::Internal(format!(
                    "cannot load non-integer value type {:?}",
                    val.get_type()
                )));
            }
        }
        Ok(())
    }

    pub fn emit_load_const(&mut self, reg: u8, val: u64) {
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

    pub fn store_to_slot(&mut self, slot_offset: i32, src_reg: u8) {
        self.emit(Instruction::StoreIndU64 {
            base: STACK_PTR_REG,
            src: src_reg,
            offset: slot_offset,
        });
        self.cache_slot(slot_offset, src_reg);
    }

    // ── Register cache ──

    /// Record that `reg` now holds the value of `slot`.
    fn cache_slot(&mut self, slot: i32, reg: u8) {
        // Remove any previous slot cached in this register.
        if let Some(old_slot) = self.reg_to_slot[reg as usize].take() {
            self.slot_cache.remove(&old_slot);
        }
        // Remove any previous register caching this slot.
        if let Some(old_reg) = self.slot_cache.insert(slot, reg) {
            self.reg_to_slot[old_reg as usize] = None;
        }
        self.reg_to_slot[reg as usize] = Some(slot);
    }

    /// Invalidate a register's cache entry (called when the register is overwritten).
    fn invalidate_reg(&mut self, reg: u8) {
        if let Some(slot) = self.reg_to_slot[reg as usize].take() {
            self.slot_cache.remove(&slot);
        }
    }

    /// Clear the entire register cache (at block boundaries and after calls).
    pub fn clear_reg_cache(&mut self) {
        self.slot_cache.clear();
        self.reg_to_slot = [None; 13];
    }

    // ── Fixup resolution ──

    pub fn resolve_fixups(&mut self) -> Result<()> {
        // Precompute byte offsets for each instruction to avoid O(n²) re-scanning.
        let mut offsets = Vec::with_capacity(self.instructions.len());
        let mut running = 0usize;
        for instr in &self.instructions {
            offsets.push(running);
            running += instr.encode().len();
        }

        for &(instr_idx, label_id) in &self.fixups {
            let target_offset = self.labels[label_id]
                .ok_or_else(|| Error::Unsupported("unresolved label".to_string()))?;

            // PVM jump offsets are relative to the instruction start.
            let instr_start = offsets[instr_idx];
            let relative_offset = (target_offset as i32) - (instr_start as i32);

            match &mut self.instructions[instr_idx] {
                Instruction::Jump { offset }
                | Instruction::BranchNeImm { offset, .. }
                | Instruction::BranchEqImm { offset, .. }
                | Instruction::BranchGeSImm { offset, .. }
                | Instruction::BranchLtUImm { offset, .. }
                | Instruction::BranchLeUImm { offset, .. }
                | Instruction::BranchGeUImm { offset, .. }
                | Instruction::BranchGtUImm { offset, .. }
                | Instruction::BranchLtSImm { offset, .. }
                | Instruction::BranchLeSImm { offset, .. }
                | Instruction::BranchGtSImm { offset, .. }
                | Instruction::BranchEq { offset, .. }
                | Instruction::BranchNe { offset, .. }
                | Instruction::BranchGeU { offset, .. }
                | Instruction::BranchLtU { offset, .. }
                | Instruction::BranchLtS { offset, .. }
                | Instruction::BranchGeS { offset, .. } => {
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

/// Get the i-th operand of an instruction as a `BasicValueEnum`.
pub fn get_operand(instr: InstructionValue<'_>, i: u32) -> Result<BasicValueEnum<'_>> {
    instr
        .get_operand(i)
        .and_then(inkwell::values::Operand::value)
        .ok_or_else(|| Error::Internal(format!("missing operand {i} in {:?}", instr.get_opcode())))
}

/// Get the i-th operand of an instruction as a `BasicBlock`.
pub fn get_bb_operand(instr: InstructionValue<'_>, i: u32) -> Result<BasicBlock<'_>> {
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
pub fn result_slot(e: &PvmEmitter<'_>, instr: InstructionValue<'_>) -> Result<i32> {
    let key = val_key_instr(instr);
    e.get_slot(key)
        .ok_or_else(|| Error::Internal(format!("no slot for {:?} result", instr.get_opcode())))
}

/// Detect the bit width of an instruction's result or first operand.
/// Checks the result type first (important for ZExt/SExt/Trunc where result
/// width differs from operand width), then falls back to operand inspection.
pub fn operand_bit_width(instr: InstructionValue<'_>) -> u32 {
    // Prefer the instruction's result type (correct for conversion instructions).
    if let inkwell::types::AnyTypeEnum::IntType(ty) = instr.get_type() {
        return ty.get_bit_width();
    }
    // Fallback: check the first operand's type.
    if let Some(op) = instr
        .get_operand(0)
        .and_then(inkwell::values::Operand::value)
        && let BasicValueEnum::IntValue(iv) = op
    {
        return iv.get_type().get_bit_width();
    }
    64 // default
}

/// Check whether `target_bb` has any phi nodes with incomings from `current_bb`.
pub fn has_phi_from<'ctx>(current_bb: BasicBlock<'ctx>, target_bb: BasicBlock<'ctx>) -> bool {
    for instr in target_bb.get_instructions() {
        use inkwell::values::InstructionOpcode;
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

/// Pre-scan function to allocate labels and slots.
pub fn pre_scan_function<'ctx>(emitter: &mut PvmEmitter<'ctx>, function: FunctionValue<'ctx>) {
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
    use inkwell::values::InstructionOpcode;
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

// Re-export scratch registers for other modules
pub use crate::abi::{ARGS_LEN_REG as SCRATCH1, ARGS_PTR_REG as SCRATCH2};
