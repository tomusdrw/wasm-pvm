use crate::pvm::Instruction;
use crate::{Error, Result};
use wasmparser::{FunctionBody, Operator};

use super::stack::StackMachine;

const ARGS_PTR_REG: u8 = 7;
const ARGS_LEN_REG: u8 = 8;
const FIRST_LOCAL_REG: u8 = 9;
const MAX_LOCAL_REGS: usize = 4;
const GLOBAL_MEMORY_BASE: i32 = 0x30000;
const EXIT_ADDRESS: i32 = -65536;
const RETURN_ADDR_REG: u8 = 0;
const STACK_PTR_REG: u8 = 1;
const RETURN_VALUE_REG: u8 = 7;
const SAVED_TABLE_IDX_REG: u8 = 8;
const RO_DATA_BASE: i32 = 0x10000;
/// Base address for spilled locals in memory
/// Layout: 0x30000-0x300FF globals, 0x30100+ user heap, 0x40000+ spilled locals
/// User heap can use up to ~64KB (0x30100 to 0x3FFFF) before colliding with spilled locals
pub const SPILLED_LOCALS_BASE: i32 = 0x40000;
/// Bytes allocated per function for spilled locals (64 locals * 8 bytes)
pub const SPILLED_LOCALS_PER_FUNC: i32 = 512;

pub struct CompileContext {
    pub num_params: usize,
    pub num_locals: usize,
    pub num_globals: usize,
    pub result_ptr_global: Option<u32>,
    pub result_len_global: Option<u32>,
    pub is_main: bool,
    pub has_return: bool,
    pub function_offsets: Vec<usize>,
    pub function_signatures: Vec<(usize, bool)>,
    pub func_idx: usize,
    pub function_table: Vec<u32>,
    pub type_signatures: Vec<(usize, usize)>,
}

#[derive(Debug, Clone)]
pub struct CallFixup {
    pub return_addr_instr: usize,
    pub jump_instr: usize,
    pub target_func: u32,
}

#[derive(Debug, Clone)]
pub struct IndirectCallFixup {
    pub return_addr_instr: usize,
    pub jump_ind_instr: usize,
}

#[derive(Debug, Clone, Copy)]
enum ControlFrame {
    Block {
        end_label: usize,
        stack_depth: usize,
        has_result: bool,
    },
    Loop {
        start_label: usize,
        stack_depth: usize,
    },
    If {
        else_label: usize,
        end_label: usize,
        stack_depth: usize,
        has_result: bool,
    },
}

const OPERAND_SPILL_BASE: i32 = -0x100;
/// Alternate register for second spilled operand in binary operations
/// Using r8 because r7 is the primary spill temp, and r8 is free during computation
/// (only used at function call boundaries for args length)
const SPILL_ALT_REG: u8 = 8;

struct CodeEmitter {
    instructions: Vec<Instruction>,
    labels: Vec<Option<usize>>,
    fixups: Vec<(usize, usize)>,
    control_stack: Vec<ControlFrame>,
    stack: StackMachine,
    call_fixups: Vec<CallFixup>,
    indirect_call_fixups: Vec<IndirectCallFixup>,
    pending_spill: Option<usize>,
    /// Tracks the register used by the last `spill_pop` if it was a spilled value
    last_spill_pop_reg: Option<u8>,
}

impl CodeEmitter {
    fn new() -> Self {
        Self {
            instructions: Vec::new(),
            labels: Vec::new(),
            fixups: Vec::new(),
            control_stack: Vec::new(),
            stack: StackMachine::new(),
            call_fixups: Vec::new(),
            indirect_call_fixups: Vec::new(),
            pending_spill: None,
            last_spill_pop_reg: None,
        }
    }

    fn alloc_label(&mut self) -> usize {
        let id = self.labels.len();
        self.labels.push(None);
        id
    }

    fn define_label(&mut self, label: usize) {
        self.labels[label] = Some(self.current_offset());
    }

    fn current_offset(&self) -> usize {
        self.instructions.iter().map(|i| i.encode().len()).sum()
    }

    fn emit(&mut self, instr: Instruction) {
        self.instructions.push(instr);
    }

    fn flush_pending_spill(&mut self) {
        if let Some(spill_depth) = self.pending_spill.take() {
            let offset = OPERAND_SPILL_BASE + StackMachine::spill_offset(spill_depth);
            self.emit(Instruction::StoreIndU64 {
                base: STACK_PTR_REG,
                src: StackMachine::reg_at_depth(spill_depth),
                offset,
            });
        }
    }

    fn spill_push(&mut self) -> u8 {
        self.flush_pending_spill();
        self.last_spill_pop_reg = None; // Clear spill tracking on push
        let depth = self.stack.depth();
        let reg = self.stack.push();
        if StackMachine::needs_spill(depth) {
            // Mark this depth for spilling - the actual spill happens
            // after the caller writes the value to the register
            self.pending_spill = Some(depth);
        }
        reg
    }

    #[allow(dead_code)]
    fn spill_finalize(&mut self) {
        self.flush_pending_spill();
    }

    fn spill_pop(&mut self) -> u8 {
        self.flush_pending_spill();
        let depth = self.stack.depth();
        if depth > 0 && StackMachine::needs_spill(depth - 1) {
            let offset = OPERAND_SPILL_BASE + StackMachine::spill_offset(depth - 1);
            // Use alternate register if we just popped another spilled value into the default register
            let default_reg = StackMachine::reg_at_depth(depth - 1);
            let dst = if self.last_spill_pop_reg == Some(default_reg) {
                SPILL_ALT_REG
            } else {
                default_reg
            };
            self.emit(Instruction::LoadIndU64 {
                dst,
                base: STACK_PTR_REG,
                offset,
            });
            self.last_spill_pop_reg = Some(dst);
            self.stack.pop();
            return dst;
        }
        self.last_spill_pop_reg = None;
        self.stack.pop()
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

    fn resolve_fixups(&mut self) -> Result<()> {
        for (instr_idx, label_id) in &self.fixups {
            let target_offset = self.labels[*label_id]
                .ok_or_else(|| Error::Unsupported("unresolved label".to_string()))?;

            let instr_start: usize = self.instructions[..*instr_idx]
                .iter()
                .map(|i| i.encode().len())
                .sum();

            let relative_offset = (target_offset as i32) - (instr_start as i32);

            match &mut self.instructions[*instr_idx] {
                Instruction::Jump { offset }
                | Instruction::BranchNeImm { offset, .. }
                | Instruction::BranchEqImm { offset, .. } => {
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

    fn push_block(&mut self, has_result: bool) -> usize {
        let end_label = self.alloc_label();
        let stack_depth = self.stack.depth();
        self.control_stack.push(ControlFrame::Block {
            end_label,
            stack_depth,
            has_result,
        });
        end_label
    }

    fn push_loop(&mut self) -> usize {
        let start_label = self.alloc_label();
        let stack_depth = self.stack.depth();
        self.define_label(start_label);
        self.control_stack.push(ControlFrame::Loop {
            start_label,
            stack_depth,
        });
        start_label
    }

    fn pop_control(&mut self) -> Option<ControlFrame> {
        self.control_stack.pop()
    }

    fn get_branch_info(&self, depth: u32) -> Option<(usize, usize, bool)> {
        let idx = self.control_stack.len().checked_sub(1 + depth as usize)?;
        let frame = self.control_stack.get(idx)?;
        match frame {
            ControlFrame::Block {
                end_label,
                stack_depth,
                has_result,
            }
            | ControlFrame::If {
                end_label,
                stack_depth,
                has_result,
                ..
            } => Some((*end_label, *stack_depth, *has_result)),
            ControlFrame::Loop {
                start_label,
                stack_depth,
            } => Some((*start_label, *stack_depth, false)),
        }
    }

    fn push_if(&mut self, cond_reg: u8, has_result: bool) {
        let else_label = self.alloc_label();
        let end_label = self.alloc_label();
        let stack_depth = self.stack.depth();
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, else_label));
        self.emit(Instruction::BranchEqImm {
            reg: cond_reg,
            value: 0,
            offset: 0,
        });
        self.control_stack.push(ControlFrame::If {
            else_label,
            end_label,
            stack_depth,
            has_result,
        });
    }

    fn emit_call(&mut self, target_func_idx: u32, num_args: usize, has_return: bool) {
        // Calculate how many operand stack values will remain after popping args
        // These are values that belong to the caller and must be preserved
        let stack_depth_before_args = self.stack.depth().saturating_sub(num_args);

        // Frame layout on stack (growing down):
        // [sp+0]: return address (r0)
        // [sp+8..40]: locals r9-r12 (4 * 8 = 32 bytes)
        // [sp+40..]: caller's operand stack values (stack_depth_before_args * 8 bytes)
        let frame_size = 40 + (stack_depth_before_args * 8) as i32;

        self.emit(Instruction::AddImm64 {
            dst: STACK_PTR_REG,
            src: STACK_PTR_REG,
            value: -frame_size,
        });

        // Save return address
        self.emit(Instruction::StoreIndU64 {
            base: STACK_PTR_REG,
            src: RETURN_ADDR_REG,
            offset: 0,
        });

        // Save locals r9-r12
        for i in 0..MAX_LOCAL_REGS {
            let reg = FIRST_LOCAL_REG + i as u8;
            self.emit(Instruction::StoreIndU64 {
                base: STACK_PTR_REG,
                src: reg,
                offset: (8 + i * 8) as i32,
            });
        }

        // Save caller's operand stack values (those below the arguments)
        for i in 0..stack_depth_before_args {
            let reg = StackMachine::reg_at_depth(i);
            self.emit(Instruction::StoreIndU64 {
                base: STACK_PTR_REG,
                src: reg,
                offset: (40 + i * 8) as i32,
            });
        }

        // Pop arguments and copy to local registers for the callee
        for i in 0..num_args {
            let src = self.spill_pop();
            let dst = FIRST_LOCAL_REG + (num_args - 1 - i) as u8;
            self.emit(Instruction::AddImm32 { dst, src, value: 0 });
        }

        let return_addr_instr_idx = self.instructions.len();
        self.emit(Instruction::LoadImm64 {
            reg: RETURN_ADDR_REG,
            value: 0,
        });

        let jump_instr_idx = self.instructions.len();
        self.emit(Instruction::Jump { offset: 0 });

        // Return point
        self.emit(Instruction::Fallthrough);

        // Copy return value to operand stack (before restoring caller's stack)
        // We use a temporary approach: put it in r7, then we'll copy to the right place
        // after restoring the caller's operand stack
        let return_in_r7 = has_return;

        // Restore return address
        self.emit(Instruction::LoadIndU64 {
            dst: RETURN_ADDR_REG,
            base: STACK_PTR_REG,
            offset: 0,
        });

        // Restore locals r9-r12
        for i in 0..MAX_LOCAL_REGS {
            let reg = FIRST_LOCAL_REG + i as u8;
            self.emit(Instruction::LoadIndU64 {
                dst: reg,
                base: STACK_PTR_REG,
                offset: (8 + i * 8) as i32,
            });
        }

        // Restore caller's operand stack values
        for i in 0..stack_depth_before_args {
            let reg = StackMachine::reg_at_depth(i);
            self.emit(Instruction::LoadIndU64 {
                dst: reg,
                base: STACK_PTR_REG,
                offset: (40 + i * 8) as i32,
            });
        }

        self.emit(Instruction::AddImm64 {
            dst: STACK_PTR_REG,
            src: STACK_PTR_REG,
            value: frame_size,
        });

        // Now that caller's operand stack is restored, push the return value if any
        if return_in_r7 {
            let dst = self.spill_push();
            self.emit(Instruction::AddImm32 {
                dst,
                src: RETURN_VALUE_REG,
                value: 0,
            });
        }

        self.call_fixups.push(CallFixup {
            return_addr_instr: return_addr_instr_idx,
            jump_instr: jump_instr_idx,
            target_func: target_func_idx,
        });
    }

    fn emit_call_indirect(&mut self, num_args: usize, has_return: bool) {
        let table_idx_reg = self.spill_pop();

        self.emit(Instruction::AddImm32 {
            dst: SAVED_TABLE_IDX_REG,
            src: table_idx_reg,
            value: 0,
        });

        let stack_depth_before_args = self.stack.depth().saturating_sub(num_args);
        let frame_size = 40 + (stack_depth_before_args * 8) as i32;

        self.emit(Instruction::AddImm64 {
            dst: STACK_PTR_REG,
            src: STACK_PTR_REG,
            value: -frame_size,
        });

        self.emit(Instruction::StoreIndU64 {
            base: STACK_PTR_REG,
            src: RETURN_ADDR_REG,
            offset: 0,
        });

        for i in 0..MAX_LOCAL_REGS {
            let reg = FIRST_LOCAL_REG + i as u8;
            self.emit(Instruction::StoreIndU64 {
                base: STACK_PTR_REG,
                src: reg,
                offset: (8 + i * 8) as i32,
            });
        }

        for i in 0..stack_depth_before_args {
            let reg = StackMachine::reg_at_depth(i);
            self.emit(Instruction::StoreIndU64 {
                base: STACK_PTR_REG,
                src: reg,
                offset: (40 + i * 8) as i32,
            });
        }

        for i in 0..num_args {
            let src = self.spill_pop();
            let dst = FIRST_LOCAL_REG + (num_args - 1 - i) as u8;
            self.emit(Instruction::AddImm32 { dst, src, value: 0 });
        }

        self.emit(Instruction::Add32 {
            dst: SAVED_TABLE_IDX_REG,
            src1: SAVED_TABLE_IDX_REG,
            src2: SAVED_TABLE_IDX_REG,
        });
        self.emit(Instruction::Add32 {
            dst: SAVED_TABLE_IDX_REG,
            src1: SAVED_TABLE_IDX_REG,
            src2: SAVED_TABLE_IDX_REG,
        });

        self.emit(Instruction::AddImm32 {
            dst: SAVED_TABLE_IDX_REG,
            src: SAVED_TABLE_IDX_REG,
            value: RO_DATA_BASE,
        });
        self.emit(Instruction::LoadIndU32 {
            dst: SAVED_TABLE_IDX_REG,
            base: SAVED_TABLE_IDX_REG,
            offset: 0,
        });

        let return_addr_instr_idx = self.instructions.len();
        self.emit(Instruction::LoadImm64 {
            reg: RETURN_ADDR_REG,
            value: 0,
        });

        let jump_ind_instr_idx = self.instructions.len();
        self.emit(Instruction::JumpInd {
            reg: SAVED_TABLE_IDX_REG,
            offset: 0,
        });

        self.emit(Instruction::Fallthrough);

        self.indirect_call_fixups.push(IndirectCallFixup {
            return_addr_instr: return_addr_instr_idx,
            jump_ind_instr: jump_ind_instr_idx,
        });

        let return_in_r7 = has_return;

        self.emit(Instruction::LoadIndU64 {
            dst: RETURN_ADDR_REG,
            base: STACK_PTR_REG,
            offset: 0,
        });

        for i in 0..MAX_LOCAL_REGS {
            let reg = FIRST_LOCAL_REG + i as u8;
            self.emit(Instruction::LoadIndU64 {
                dst: reg,
                base: STACK_PTR_REG,
                offset: (8 + i * 8) as i32,
            });
        }

        for i in 0..stack_depth_before_args {
            let reg = StackMachine::reg_at_depth(i);
            self.emit(Instruction::LoadIndU64 {
                dst: reg,
                base: STACK_PTR_REG,
                offset: (40 + i * 8) as i32,
            });
        }

        self.emit(Instruction::AddImm64 {
            dst: STACK_PTR_REG,
            src: STACK_PTR_REG,
            value: frame_size,
        });

        if return_in_r7 {
            let dst = self.spill_push();
            self.emit(Instruction::AddImm32 {
                dst,
                src: RETURN_VALUE_REG,
                value: 0,
            });
        }
    }
}

pub struct FunctionTranslation {
    pub instructions: Vec<Instruction>,
    pub call_fixups: Vec<CallFixup>,
    pub indirect_call_fixups: Vec<IndirectCallFixup>,
}

pub fn translate_function(
    body: &FunctionBody,
    ctx: &CompileContext,
) -> Result<FunctionTranslation> {
    let mut emitter = CodeEmitter::new();

    let mut total_locals = ctx.num_params;
    let locals_reader = body.get_locals_reader()?;
    for local in locals_reader {
        let (count, _ty) = local?;
        total_locals += count as usize;
    }

    emit_prologue(&mut emitter, ctx);

    let ops: Vec<Operator> = body
        .get_operators_reader()?
        .into_iter()
        .collect::<std::result::Result<_, _>>()?;

    for op in &ops {
        translate_op(op, &mut emitter, ctx, total_locals)?;
    }

    emit_epilogue(&mut emitter, ctx, ctx.has_return);

    emitter.resolve_fixups()?;

    Ok(FunctionTranslation {
        instructions: emitter.instructions,
        call_fixups: emitter.call_fixups,
        indirect_call_fixups: emitter.indirect_call_fixups,
    })
}

fn emit_prologue(emitter: &mut CodeEmitter, ctx: &CompileContext) {
    if ctx.is_main {
        if ctx.num_params >= 1 {
            emitter.emit(Instruction::AddImm32 {
                dst: FIRST_LOCAL_REG,
                src: ARGS_PTR_REG,
                value: 0,
            });
        }
        if ctx.num_params >= 2 {
            emitter.emit(Instruction::AddImm32 {
                dst: FIRST_LOCAL_REG + 1,
                src: ARGS_LEN_REG,
                value: 0,
            });
        }
    }
}

fn emit_epilogue(emitter: &mut CodeEmitter, ctx: &CompileContext, has_return: bool) {
    if ctx.is_main {
        if let Some(ptr_idx) = ctx.result_ptr_global {
            let offset = (ptr_idx as i32) * 4 + GLOBAL_MEMORY_BASE;
            emitter.emit(Instruction::LoadImm {
                reg: 2,
                value: offset,
            });
            emitter.emit(Instruction::LoadIndU32 {
                dst: ARGS_PTR_REG,
                base: 2,
                offset: 0,
            });
        }
        if let Some(len_idx) = ctx.result_len_global {
            let offset = (len_idx as i32) * 4 + GLOBAL_MEMORY_BASE;
            emitter.emit(Instruction::LoadImm {
                reg: 2,
                value: offset,
            });
            emitter.emit(Instruction::LoadIndU32 {
                dst: ARGS_LEN_REG,
                base: 2,
                offset: 0,
            });
        }

        emitter.emit(Instruction::LoadImm {
            reg: 2,
            value: EXIT_ADDRESS,
        });
        emitter.emit(Instruction::JumpInd { reg: 2, offset: 0 });
    } else {
        if has_return {
            let ret_val = emitter.spill_pop();
            emitter.emit(Instruction::AddImm32 {
                dst: RETURN_VALUE_REG,
                src: ret_val,
                value: 0,
            });
        }
        emitter.emit(Instruction::JumpInd {
            reg: RETURN_ADDR_REG,
            offset: 0,
        });
    }
}

fn local_reg(idx: usize) -> Option<u8> {
    if idx < MAX_LOCAL_REGS {
        Some(FIRST_LOCAL_REG + idx as u8)
    } else {
        None
    }
}

fn spilled_local_offset(func_idx: usize, local_idx: usize) -> i32 {
    SPILLED_LOCALS_BASE
        + (func_idx as i32) * SPILLED_LOCALS_PER_FUNC
        + ((local_idx - MAX_LOCAL_REGS) as i32) * 8
}

fn global_offset(idx: u32) -> i32 {
    GLOBAL_MEMORY_BASE + (idx as i32) * 4
}

fn translate_op(
    op: &Operator,
    emitter: &mut CodeEmitter,
    ctx: &CompileContext,
    _total_locals: usize,
) -> Result<()> {
    match op {
        Operator::LocalGet { local_index } => {
            let idx = *local_index as usize;
            if let Some(reg) = local_reg(idx) {
                let dst = emitter.spill_push();
                emitter.emit(Instruction::AddImm32 {
                    dst,
                    src: reg,
                    value: 0,
                });
            } else {
                let offset = spilled_local_offset(ctx.func_idx, idx);
                let dst = emitter.spill_push();
                emitter.emit(Instruction::LoadImm {
                    reg: dst,
                    value: offset,
                });
                emitter.emit(Instruction::LoadIndU32 {
                    dst,
                    base: dst,
                    offset: 0,
                });
            }
        }
        Operator::LocalSet { local_index } => {
            let idx = *local_index as usize;
            if let Some(reg) = local_reg(idx) {
                let src = emitter.spill_pop();
                emitter.emit(Instruction::AddImm32 {
                    dst: reg,
                    src,
                    value: 0,
                });
            } else {
                let offset = spilled_local_offset(ctx.func_idx, idx);
                let src = emitter.spill_pop();
                let temp = if src == 2 { 3 } else { 2 };
                emitter.emit(Instruction::LoadImm {
                    reg: temp,
                    value: offset,
                });
                emitter.emit(Instruction::StoreIndU32 {
                    base: temp,
                    src,
                    offset: 0,
                });
            }
        }
        Operator::LocalTee { local_index } => {
            let idx = *local_index as usize;
            if let Some(reg) = local_reg(idx) {
                let src = emitter.stack.peek(0);
                emitter.emit(Instruction::AddImm32 {
                    dst: reg,
                    src,
                    value: 0,
                });
            } else {
                let offset = spilled_local_offset(ctx.func_idx, idx);
                let src = emitter.stack.peek(0);
                let temp = if src == 2 { 3 } else { 2 };
                emitter.emit(Instruction::LoadImm {
                    reg: temp,
                    value: offset,
                });
                emitter.emit(Instruction::StoreIndU32 {
                    base: temp,
                    src,
                    offset: 0,
                });
            }
        }
        Operator::GlobalGet { global_index } => {
            let offset = global_offset(*global_index);
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LoadImm {
                reg: dst,
                value: offset,
            });
            emitter.emit(Instruction::LoadIndU32 {
                dst,
                base: dst,
                offset: 0,
            });
        }
        Operator::GlobalSet { global_index } => {
            let offset = global_offset(*global_index);
            let src = emitter.spill_pop();
            let temp = if src == 2 { 3 } else { 2 };
            emitter.emit(Instruction::LoadImm {
                reg: temp,
                value: offset,
            });
            emitter.emit(Instruction::StoreIndU32 {
                base: temp,
                src,
                offset: 0,
            });
        }
        Operator::I32Load { memarg } | Operator::I64Load32U { memarg } => {
            let addr = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LoadIndU32 {
                dst,
                base: addr,
                offset: memarg.offset as i32,
            });
        }
        Operator::I32Store { memarg } | Operator::I64Store32 { memarg } => {
            let value = emitter.spill_pop();
            let addr = emitter.spill_pop();
            emitter.emit(Instruction::StoreIndU32 {
                base: addr,
                src: value,
                offset: memarg.offset as i32,
            });
        }
        Operator::I64Load { memarg } => {
            let base = self.stack.pop(ctx)?;
            let offset = memarg.offset as i64;
            let dest = self.stack.push_temp(ctx)?;
            log::debug!(
                "I64Load: base={:?}, offset={}, dest={:?}",
                base,
                offset,
                dest
            );
            ctx.emit(Instruction::LoadI64 {
                dest,
                base: base.into(),
                offset,
            });
        }
        Operator::I64Store { memarg } => {
            let value = emitter.spill_pop();
            let addr = emitter.spill_pop();
            emitter.emit(Instruction::StoreIndU64 {
                base: addr,
                src: value,
                offset: memarg.offset as i32,
            });
        }
        Operator::I32Const { value } => {
            let reg = emitter.spill_push();
            emitter.emit(Instruction::LoadImm { reg, value: *value });
        }
        Operator::I64Const { value } => {
            let reg = emitter.spill_push();
            emitter.emit(Instruction::LoadImm64 {
                reg,
                value: *value as u64,
            });
        }
        Operator::I32Add => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Add32 { dst, src1, src2 });
        }
        Operator::I32Sub => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Sub32 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        Operator::I32Mul => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Mul32 { dst, src1, src2 });
        }
        Operator::I32DivU => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::DivU32 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        Operator::I32DivS => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::DivS32 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        Operator::I32RemU => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::RemU32 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        Operator::I32RemS => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::RemS32 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        Operator::I32Eq | Operator::I64Eq => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Xor {
                dst,
                src1: a,
                src2: b,
            });
            emitter.emit(Instruction::SetLtUImm {
                dst,
                src: dst,
                value: 1,
            });
        }
        Operator::I32Ne | Operator::I64Ne => {
            // NE: a != b → (a XOR b) != 0 → 0 < (a XOR b)
            // PVM SetLtU semantics: dst = src2 < src1
            // So for 0 < xor_result, we need SetLtU { src1: xor_result, src2: 0 }
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Xor {
                dst,
                src1: a,
                src2: b,
            });
            let zero = emitter.spill_push();
            emitter.emit(Instruction::LoadImm {
                reg: zero,
                value: 0,
            });
            let _ = emitter.spill_pop();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: dst,  // xor_result
                src2: zero, // 0 < xor_result when xor_result > 0
            });
        }
        Operator::I32And | Operator::I64And => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::And {
                dst,
                src1: a,
                src2: b,
            });
        }
        Operator::I32Or | Operator::I64Or => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Or {
                dst,
                src1: a,
                src2: b,
            });
        }
        Operator::I32Xor | Operator::I64Xor => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Xor {
                dst,
                src1: a,
                src2: b,
            });
        }
        Operator::I32Shl => {
            let shift = emitter.spill_pop();
            let value = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::ShloL32 {
                dst,
                src1: shift,
                src2: value,
            });
        }
        Operator::I32ShrU => {
            let shift = emitter.spill_pop();
            let value = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::ShloR32 {
                dst,
                src1: shift,
                src2: value,
            });
        }
        Operator::I32ShrS => {
            let shift = emitter.spill_pop();
            let value = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SharR32 {
                dst,
                src1: shift,
                src2: value,
            });
        }
        Operator::I64Shl => {
            let shift = emitter.spill_pop();
            let value = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::ShloL64 {
                dst,
                src1: shift,
                src2: value,
            });
        }
        Operator::I64ShrU => {
            let shift = emitter.spill_pop();
            let value = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::ShloR64 {
                dst,
                src1: shift,
                src2: value,
            });
        }
        Operator::I64ShrS => {
            let shift = emitter.spill_pop();
            let value = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SharR64 {
                dst,
                src1: shift,
                src2: value,
            });
        }
        Operator::Nop => {}
        Operator::Unreachable => {
            emitter.emit(Instruction::Trap);
        }
        Operator::Drop => {
            let _ = emitter.spill_pop();
        }
        Operator::I64Add => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Add64 { dst, src1, src2 });
        }
        Operator::I64Sub => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Sub64 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        Operator::I64Mul => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Mul64 { dst, src1, src2 });
        }
        Operator::I64DivU => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::DivU64 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        Operator::I64DivS => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::DivS64 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        Operator::I64RemU => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::RemU64 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        Operator::I64RemS => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::RemS64 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        Operator::I32GtU | Operator::I64GtU => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: a,
                src2: b,
            });
        }
        Operator::I32GtS | Operator::I64GtS => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtS {
                dst,
                src1: a,
                src2: b,
            });
        }
        // For WASM i32.lt_X: push a, push b, lt → result = a < b
        // Pop b (top), pop a (second)
        // PVM SetLt semantics: dst = src2 < src1 (verified in anan-as: reg[b] < reg[a])
        // For a < b, we need SetLt { src1: b, src2: a } so it computes a < b
        Operator::I32LtU | Operator::I64LtU => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: b,
                src2: a,
            });
        }
        Operator::I32LtS | Operator::I64LtS => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtS {
                dst,
                src1: b,
                src2: a,
            });
        }
        Operator::I32GeU | Operator::I64GeU => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: b,
                src2: a,
            });
            let one = emitter.spill_push();
            emitter.emit(Instruction::LoadImm { reg: one, value: 1 });
            let _ = emitter.spill_pop();
            emitter.emit(Instruction::Xor {
                dst,
                src1: dst,
                src2: one,
            });
        }
        Operator::I32GeS | Operator::I64GeS => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtS {
                dst,
                src1: b,
                src2: a,
            });
            let one = emitter.spill_push();
            emitter.emit(Instruction::LoadImm { reg: one, value: 1 });
            let _ = emitter.spill_pop();
            emitter.emit(Instruction::Xor {
                dst,
                src1: dst,
                src2: one,
            });
        }
        Operator::I32LeU | Operator::I64LeU => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: a,
                src2: b,
            });
            let one = emitter.spill_push();
            emitter.emit(Instruction::LoadImm { reg: one, value: 1 });
            let _ = emitter.spill_pop();
            emitter.emit(Instruction::Xor {
                dst,
                src1: dst,
                src2: one,
            });
        }
        Operator::I32LeS | Operator::I64LeS => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtS {
                dst,
                src1: a,
                src2: b,
            });
            let one = emitter.spill_push();
            emitter.emit(Instruction::LoadImm { reg: one, value: 1 });
            let _ = emitter.spill_pop();
            emitter.emit(Instruction::Xor {
                dst,
                src1: dst,
                src2: one,
            });
        }
        Operator::I32Eqz | Operator::I64Eqz => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtUImm { dst, src, value: 1 });
        }
        Operator::Block { blockty } => {
            let has_result = !matches!(blockty, wasmparser::BlockType::Empty);
            emitter.push_block(has_result);
        }
        Operator::Loop { blockty: _ } => {
            emitter.emit(Instruction::Fallthrough);
            emitter.push_loop();
        }
        Operator::If { blockty } => {
            let has_result = !matches!(blockty, wasmparser::BlockType::Empty);
            let cond = emitter.spill_pop();
            emitter.push_if(cond, has_result);
        }
        Operator::Else => {
            if let Some(ControlFrame::If {
                else_label,
                end_label,
                stack_depth,
                has_result,
            }) = emitter.pop_control()
            {
                emitter.emit_jump_to_label(end_label);
                emitter.emit(Instruction::Fallthrough);
                emitter.define_label(else_label);
                if has_result {
                    emitter.stack.set_depth(stack_depth);
                }
                emitter.control_stack.push(ControlFrame::Block {
                    end_label,
                    stack_depth,
                    has_result,
                });
            }
        }
        Operator::End => match emitter.pop_control() {
            Some(ControlFrame::Block {
                end_label,
                stack_depth,
                has_result,
            }) => {
                emitter.emit(Instruction::Fallthrough);
                emitter.define_label(end_label);
                if has_result {
                    emitter.stack.set_depth(stack_depth + 1);
                }
            }
            Some(ControlFrame::If {
                else_label,
                end_label,
                stack_depth,
                has_result,
            }) => {
                emitter.emit(Instruction::Fallthrough);
                emitter.define_label(else_label);
                emitter.define_label(end_label);
                if has_result {
                    emitter.stack.set_depth(stack_depth + 1);
                }
            }
            Some(ControlFrame::Loop { .. }) => {
                emitter.emit(Instruction::Fallthrough);
            }
            None => {}
        },
        Operator::Br { relative_depth } => {
            if let Some((target, target_depth, has_result)) =
                emitter.get_branch_info(*relative_depth)
            {
                if has_result && emitter.stack.depth() > target_depth {
                    let src = StackMachine::reg_at_depth(emitter.stack.depth() - 1);
                    let dst = StackMachine::reg_at_depth(target_depth);
                    if src != dst {
                        emitter.emit(Instruction::AddImm32 { dst, src, value: 0 });
                    }
                }
                emitter.emit_jump_to_label(target);
            }
        }
        Operator::BrIf { relative_depth } => {
            let cond = emitter.spill_pop();
            if let Some((target, target_depth, has_result)) =
                emitter.get_branch_info(*relative_depth)
            {
                if has_result && emitter.stack.depth() > target_depth {
                    let end_label = emitter.alloc_label();
                    emitter.emit_branch_eq_imm_to_label(cond, 0, end_label);
                    let src = StackMachine::reg_at_depth(emitter.stack.depth() - 1);
                    let dst = StackMachine::reg_at_depth(target_depth);
                    if src != dst {
                        emitter.emit(Instruction::AddImm32 { dst, src, value: 0 });
                    }
                    emitter.emit_jump_to_label(target);
                    emitter.emit(Instruction::Fallthrough);
                    emitter.define_label(end_label);
                } else {
                    emitter.emit_branch_ne_imm_to_label(cond, 0, target);
                }
            }
        }
        Operator::BrTable { targets } => {
            let index_reg = emitter.spill_pop();
            let target_depths: Vec<u32> = targets.targets().map(|t| t.unwrap()).collect();
            let default_depth = targets.default();

            for (i, &depth) in target_depths.iter().enumerate() {
                if let Some((target, target_depth, has_result)) = emitter.get_branch_info(depth) {
                    let next_label = emitter.alloc_label();
                    emitter.emit_branch_ne_imm_to_label(index_reg, i as i32, next_label);
                    if has_result && emitter.stack.depth() > target_depth {
                        let src = StackMachine::reg_at_depth(emitter.stack.depth() - 1);
                        let dst = StackMachine::reg_at_depth(target_depth);
                        if src != dst {
                            emitter.emit(Instruction::AddImm32 { dst, src, value: 0 });
                        }
                    }
                    emitter.emit_jump_to_label(target);
                    emitter.emit(Instruction::Fallthrough);
                    emitter.define_label(next_label);
                }
            }

            if let Some((target, target_depth, has_result)) = emitter.get_branch_info(default_depth)
            {
                if has_result && emitter.stack.depth() > target_depth {
                    let src = StackMachine::reg_at_depth(emitter.stack.depth() - 1);
                    let dst = StackMachine::reg_at_depth(target_depth);
                    if src != dst {
                        emitter.emit(Instruction::AddImm32 { dst, src, value: 0 });
                    }
                }
                emitter.emit_jump_to_label(target);
            }
        }
        Operator::Return => {
            emitter.emit(Instruction::LoadImm {
                reg: 2,
                value: EXIT_ADDRESS,
            });
            emitter.emit(Instruction::JumpInd { reg: 2, offset: 0 });
        }
        Operator::Call { function_index } => {
            let (num_args, has_return) = ctx
                .function_signatures
                .get(*function_index as usize)
                .copied()
                .unwrap_or((0, false));
            emitter.emit_call(*function_index, num_args, has_return);
        }
        Operator::CallIndirect {
            type_index,
            table_index,
        } => {
            if *table_index != 0 {
                return Err(Error::Unsupported(format!(
                    "call_indirect with table index {table_index}"
                )));
            }
            let (num_args, num_results) = ctx
                .type_signatures
                .get(*type_index as usize)
                .copied()
                .unwrap_or((0, 0));
            let has_return = num_results > 0;
            emitter.emit_call_indirect(num_args, has_return);
        }
        Operator::MemorySize { mem: 0, .. } => {
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LoadImm {
                reg: dst,
                value: 256,
            });
        }
        Operator::MemoryGrow { mem: 0, .. } => {
            let _ = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LoadImm {
                reg: dst,
                value: -1,
            });
        }
        Operator::Select => {
            let cond = emitter.spill_pop();
            let val2 = emitter.spill_pop();
            let val1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            let else_label = emitter.alloc_label();
            let end_label = emitter.alloc_label();
            emitter.emit_branch_eq_imm_to_label(cond, 0, else_label);
            emitter.emit(Instruction::AddImm32 {
                dst,
                src: val1,
                value: 0,
            });
            emitter.emit_jump_to_label(end_label);
            emitter.emit(Instruction::Fallthrough);
            emitter.define_label(else_label);
            emitter.emit(Instruction::AddImm32 {
                dst,
                src: val2,
                value: 0,
            });
            emitter.emit(Instruction::Fallthrough);
            emitter.define_label(end_label);
        }
        Operator::I32Clz => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LeadingZeroBits32 { dst, src });
        }
        Operator::I64Clz => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LeadingZeroBits64 { dst, src });
        }
        Operator::I32Ctz => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::TrailingZeroBits32 { dst, src });
        }
        Operator::I64Ctz => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::TrailingZeroBits64 { dst, src });
        }
        Operator::I32Popcnt => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::CountSetBits32 { dst, src });
        }
        Operator::I64Popcnt => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::CountSetBits64 { dst, src });
        }
        Operator::I32WrapI64 => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::AddImm32 { dst, src, value: 0 });
        }
        Operator::I64ExtendI32S => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SignExtend16 { dst, src });
            emitter.emit(Instruction::SignExtend16 { dst, src: dst });
        }
        Operator::I64ExtendI32U => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            if src != dst {
                emitter.emit(Instruction::AddImm32 { dst, src, value: 0 });
            }
        }
        Operator::I32Load8U { memarg } | Operator::I64Load8U { memarg } => {
            let addr = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LoadIndU8 {
                dst,
                base: addr,
                offset: memarg.offset as i32,
            });
        }
        Operator::I32Load8S { memarg } | Operator::I64Load8S { memarg } => {
            let addr = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LoadIndI8 {
                dst,
                base: addr,
                offset: memarg.offset as i32,
            });
        }
        Operator::I32Load16U { memarg } | Operator::I64Load16U { memarg } => {
            let addr = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LoadIndU16 {
                dst,
                base: addr,
                offset: memarg.offset as i32,
            });
        }
        Operator::I32Load16S { memarg } | Operator::I64Load16S { memarg } => {
            let addr = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LoadIndI16 {
                dst,
                base: addr,
                offset: memarg.offset as i32,
            });
        }
        Operator::I64Load32S { memarg } => {
            let addr = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LoadIndU32 {
                dst,
                base: addr,
                offset: memarg.offset as i32,
            });
            emitter.emit(Instruction::SignExtend16 { dst, src: dst });
            emitter.emit(Instruction::SignExtend16 { dst, src: dst });
        }
        Operator::I32Store8 { memarg } | Operator::I64Store8 { memarg } => {
            let val = emitter.spill_pop();
            let addr = emitter.spill_pop();
            emitter.emit(Instruction::StoreIndU8 {
                base: addr,
                src: val,
                offset: memarg.offset as i32,
            });
        }
        Operator::I32Store16 { memarg } | Operator::I64Store16 { memarg } => {
            let val = emitter.spill_pop();
            let addr = emitter.spill_pop();
            emitter.emit(Instruction::StoreIndU16 {
                base: addr,
                src: val,
                offset: memarg.offset as i32,
            });
        }
        Operator::I32Rotl => {
            let n = emitter.spill_pop();
            let value = emitter.spill_pop();
            let result = emitter.spill_push();
            emitter.emit(Instruction::AddImm32 {
                dst: 7,
                src: value,
                value: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: 8,
                src: n,
                value: 0,
            });
            emitter.emit(Instruction::ShloL32 {
                dst: result,
                src1: 8,
                src2: 7,
            });
            emitter.emit(Instruction::LoadImm { reg: n, value: 32 });
            emitter.emit(Instruction::Sub32 {
                dst: 8,
                src1: 8,
                src2: n,
            });
            emitter.emit(Instruction::ShloR32 {
                dst: 7,
                src1: 8,
                src2: 7,
            });
            emitter.emit(Instruction::Or {
                dst: result,
                src1: result,
                src2: 7,
            });
        }
        Operator::I32Rotr => {
            let n = emitter.spill_pop();
            let value = emitter.spill_pop();
            let result = emitter.spill_push();
            emitter.emit(Instruction::AddImm32 {
                dst: 7,
                src: value,
                value: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: 8,
                src: n,
                value: 0,
            });
            emitter.emit(Instruction::ShloR32 {
                dst: result,
                src1: 8,
                src2: 7,
            });
            emitter.emit(Instruction::LoadImm { reg: n, value: 32 });
            emitter.emit(Instruction::Sub32 {
                dst: 8,
                src1: 8,
                src2: n,
            });
            emitter.emit(Instruction::ShloL32 {
                dst: 7,
                src1: 8,
                src2: 7,
            });
            emitter.emit(Instruction::Or {
                dst: result,
                src1: result,
                src2: 7,
            });
        }
        Operator::I64Rotl => {
            let n = emitter.spill_pop();
            let value = emitter.spill_pop();
            let result = emitter.spill_push();
            emitter.emit(Instruction::AddImm32 {
                dst: 7,
                src: value,
                value: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: 8,
                src: n,
                value: 0,
            });
            emitter.emit(Instruction::ShloL64 {
                dst: result,
                src1: 8,
                src2: 7,
            });
            emitter.emit(Instruction::LoadImm { reg: n, value: 64 });
            emitter.emit(Instruction::Sub64 {
                dst: 8,
                src1: 8,
                src2: n,
            });
            emitter.emit(Instruction::ShloR64 {
                dst: 7,
                src1: 8,
                src2: 7,
            });
            emitter.emit(Instruction::Or {
                dst: result,
                src1: result,
                src2: 7,
            });
        }
        Operator::I64Rotr => {
            let n = emitter.spill_pop();
            let value = emitter.spill_pop();
            let result = emitter.spill_push();
            emitter.emit(Instruction::AddImm32 {
                dst: 7,
                src: value,
                value: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: 8,
                src: n,
                value: 0,
            });
            emitter.emit(Instruction::ShloR64 {
                dst: result,
                src1: 8,
                src2: 7,
            });
            emitter.emit(Instruction::LoadImm { reg: n, value: 64 });
            emitter.emit(Instruction::Sub64 {
                dst: 8,
                src1: 8,
                src2: n,
            });
            emitter.emit(Instruction::ShloL64 {
                dst: 7,
                src1: 8,
                src2: 7,
            });
            emitter.emit(Instruction::Or {
                dst: result,
                src1: result,
                src2: 7,
            });
        }
        _ => {
            return Err(Error::Unsupported(format!("{op:?}")));
        }
    }
    Ok(())
}
