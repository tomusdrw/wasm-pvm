use crate::pvm::Instruction;
use crate::{Error, Result};
use wasmparser::{FunctionBody, Operator};

use super::stack::StackMachine;

const ARGS_PTR_REG: u8 = 7;
const ARGS_LEN_REG: u8 = 8;
const FIRST_LOCAL_REG: u8 = 9;
const MAX_LOCAL_REGS: usize = 4;
const GLOBAL_MEMORY_BASE: i32 = 0x20000;
const EXIT_ADDRESS: i32 = -65536;

pub struct CompileContext {
    pub num_params: usize,
    pub num_locals: usize,
    pub num_globals: usize,
    pub result_ptr_global: Option<u32>,
    pub result_len_global: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
enum ControlFrame {
    Block { end_label: usize },
    Loop { start_label: usize },
    If { else_label: usize, end_label: usize },
}

struct CodeEmitter {
    instructions: Vec<Instruction>,
    labels: Vec<Option<usize>>,
    fixups: Vec<(usize, usize)>,
    control_stack: Vec<ControlFrame>,
    stack: StackMachine,
}

impl CodeEmitter {
    fn new() -> Self {
        Self {
            instructions: Vec::new(),
            labels: Vec::new(),
            fixups: Vec::new(),
            control_stack: Vec::new(),
            stack: StackMachine::new(),
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

    #[allow(dead_code)]
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

    fn push_block(&mut self) -> usize {
        let end_label = self.alloc_label();
        self.control_stack.push(ControlFrame::Block { end_label });
        end_label
    }

    fn push_loop(&mut self) -> usize {
        let start_label = self.alloc_label();
        self.define_label(start_label);
        self.control_stack.push(ControlFrame::Loop { start_label });
        start_label
    }

    fn pop_control(&mut self) -> Option<ControlFrame> {
        self.control_stack.pop()
    }

    fn get_branch_target(&self, depth: u32) -> Result<usize> {
        let idx = self.control_stack.len().checked_sub(1 + depth as usize);
        let frame = idx.and_then(|i| self.control_stack.get(i));
        match frame {
            Some(ControlFrame::Block { end_label } | ControlFrame::If { end_label, .. }) => {
                Ok(*end_label)
            }
            Some(ControlFrame::Loop { start_label }) => Ok(*start_label),
            None => Err(Error::Unsupported(format!(
                "branch depth {depth} out of range"
            ))),
        }
    }

    fn push_if(&mut self, cond_reg: u8) {
        let else_label = self.alloc_label();
        let end_label = self.alloc_label();
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
        });
    }
}

pub fn translate_function(body: &FunctionBody, ctx: &CompileContext) -> Result<Vec<Instruction>> {
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

    emit_epilogue(&mut emitter, ctx);

    emitter.resolve_fixups()?;

    Ok(emitter.instructions)
}

fn emit_prologue(emitter: &mut CodeEmitter, ctx: &CompileContext) {
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

fn emit_epilogue(emitter: &mut CodeEmitter, ctx: &CompileContext) {
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
}

fn local_reg(idx: usize) -> Option<u8> {
    if idx < MAX_LOCAL_REGS {
        Some(FIRST_LOCAL_REG + idx as u8)
    } else {
        None
    }
}

fn global_offset(idx: u32) -> i32 {
    GLOBAL_MEMORY_BASE + (idx as i32) * 4
}

fn translate_op(
    op: &Operator,
    emitter: &mut CodeEmitter,
    _ctx: &CompileContext,
    _total_locals: usize,
) -> Result<()> {
    match op {
        Operator::LocalGet { local_index } => {
            let idx = *local_index as usize;
            if let Some(reg) = local_reg(idx) {
                let dst = emitter.stack.push();
                emitter.emit(Instruction::AddImm32 {
                    dst,
                    src: reg,
                    value: 0,
                });
            } else {
                return Err(Error::Unsupported(format!(
                    "local.get {idx} exceeds register limit"
                )));
            }
        }
        Operator::LocalSet { local_index } => {
            let idx = *local_index as usize;
            if let Some(reg) = local_reg(idx) {
                let src = emitter.stack.pop();
                emitter.emit(Instruction::AddImm32 {
                    dst: reg,
                    src,
                    value: 0,
                });
            } else {
                return Err(Error::Unsupported(format!(
                    "local.set {idx} exceeds register limit"
                )));
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
                return Err(Error::Unsupported(format!(
                    "local.tee {idx} exceeds register limit"
                )));
            }
        }
        Operator::GlobalGet { global_index } => {
            let offset = global_offset(*global_index);
            let dst = emitter.stack.push();
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
            let src = emitter.stack.pop();
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
        Operator::I32Load { memarg } => {
            let addr = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::LoadIndU32 {
                dst,
                base: addr,
                offset: memarg.offset as i32,
            });
        }
        Operator::I32Store { memarg } => {
            let value = emitter.stack.pop();
            let addr = emitter.stack.pop();
            emitter.emit(Instruction::StoreIndU32 {
                base: addr,
                src: value,
                offset: memarg.offset as i32,
            });
        }
        Operator::I32Const { value } => {
            let reg = emitter.stack.push();
            emitter.emit(Instruction::LoadImm { reg, value: *value });
        }
        Operator::I64Const { value } => {
            let reg = emitter.stack.push();
            emitter.emit(Instruction::LoadImm64 {
                reg,
                value: *value as u64,
            });
        }
        Operator::I32Add => {
            let src2 = emitter.stack.pop();
            let src1 = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::Add32 { dst, src1, src2 });
        }
        Operator::I32Sub => {
            let src2 = emitter.stack.pop();
            let src1 = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::Sub32 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        Operator::I32Mul => {
            let src2 = emitter.stack.pop();
            let src1 = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::Mul32 { dst, src1, src2 });
        }
        Operator::I32RemU => {
            let src2 = emitter.stack.pop();
            let src1 = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::RemU32 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        Operator::I32RemS => {
            let src2 = emitter.stack.pop();
            let src1 = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::RemS32 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        Operator::I32Eq => {
            let b = emitter.stack.pop();
            let a = emitter.stack.pop();
            let dst = emitter.stack.push();
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
        Operator::I32Ne => {
            let b = emitter.stack.pop();
            let a = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::Xor {
                dst,
                src1: a,
                src2: b,
            });
            let one = emitter.stack.push();
            emitter.emit(Instruction::LoadImm { reg: one, value: 0 });
            let _ = emitter.stack.pop();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: one,
                src2: dst,
            });
        }
        Operator::I32And => {
            let b = emitter.stack.pop();
            let a = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::And {
                dst,
                src1: a,
                src2: b,
            });
        }
        Operator::I32Or => {
            let b = emitter.stack.pop();
            let a = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::Or {
                dst,
                src1: a,
                src2: b,
            });
        }
        Operator::I32Xor => {
            let b = emitter.stack.pop();
            let a = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::Xor {
                dst,
                src1: a,
                src2: b,
            });
        }
        Operator::I32Shl => {
            let shift = emitter.stack.pop();
            let value = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::ShloL32 {
                dst,
                src1: shift,
                src2: value,
            });
        }
        Operator::I32ShrU => {
            let shift = emitter.stack.pop();
            let value = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::ShloR32 {
                dst,
                src1: shift,
                src2: value,
            });
        }
        Operator::I32ShrS => {
            let shift = emitter.stack.pop();
            let value = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::SharR32 {
                dst,
                src1: shift,
                src2: value,
            });
        }
        Operator::Nop => {}
        Operator::I64Add => {
            let src2 = emitter.stack.pop();
            let src1 = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::Add64 { dst, src1, src2 });
        }
        Operator::I32GtU => {
            let b = emitter.stack.pop();
            let a = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: a,
                src2: b,
            });
        }
        Operator::I32GtS => {
            let b = emitter.stack.pop();
            let a = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::SetLtS {
                dst,
                src1: a,
                src2: b,
            });
        }
        Operator::I32LtU => {
            let b = emitter.stack.pop();
            let a = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: b,
                src2: a,
            });
        }
        Operator::I32LtS => {
            let b = emitter.stack.pop();
            let a = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::SetLtS {
                dst,
                src1: b,
                src2: a,
            });
        }
        Operator::I32GeU => {
            let b = emitter.stack.pop();
            let a = emitter.stack.pop();
            let dst = emitter.stack.push();
            // a >= b is !(a < b)
            // PVM SET_LT_U computes: dst = (src2 < src1)
            // For (a < b): need src2=a, src1=b
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: b,
                src2: a,
            });
            let one = emitter.stack.push();
            emitter.emit(Instruction::LoadImm { reg: one, value: 1 });
            let _ = emitter.stack.pop();
            emitter.emit(Instruction::Xor {
                dst,
                src1: dst,
                src2: one,
            });
        }
        Operator::I32LeU => {
            let b = emitter.stack.pop();
            let a = emitter.stack.pop();
            let dst = emitter.stack.push();
            // a <= b is !(b < a)
            // PVM SET_LT_U computes: dst = (src2 < src1)
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: a,
                src2: b,
            });
            let one = emitter.stack.push();
            emitter.emit(Instruction::LoadImm { reg: one, value: 1 });
            let _ = emitter.stack.pop();
            emitter.emit(Instruction::Xor {
                dst,
                src1: dst,
                src2: one,
            });
        }
        Operator::I32LeS => {
            let b = emitter.stack.pop();
            let a = emitter.stack.pop();
            let dst = emitter.stack.push();
            // a <= b is !(b < a)
            // PVM SET_LT_S computes: dst = (src2 < src1)
            // So we need src2=b, src1=a to get (b < a)
            emitter.emit(Instruction::SetLtS {
                dst,
                src1: a,
                src2: b,
            });
            let one = emitter.stack.push();
            emitter.emit(Instruction::LoadImm { reg: one, value: 1 });
            let _ = emitter.stack.pop();
            emitter.emit(Instruction::Xor {
                dst,
                src1: dst,
                src2: one,
            });
        }
        Operator::I32Eqz => {
            let src = emitter.stack.pop();
            let dst = emitter.stack.push();
            emitter.emit(Instruction::SetLtUImm { dst, src, value: 1 });
        }
        Operator::Block { blockty: _ } => {
            emitter.push_block();
        }
        Operator::Loop { blockty: _ } => {
            emitter.emit(Instruction::Fallthrough);
            emitter.push_loop();
        }
        Operator::If { blockty: _ } => {
            let cond = emitter.stack.pop();
            emitter.push_if(cond);
        }
        Operator::Else => {
            if let Some(ControlFrame::If {
                else_label,
                end_label,
            }) = emitter.pop_control()
            {
                emitter.emit_jump_to_label(end_label);
                emitter.define_label(else_label);
                emitter
                    .control_stack
                    .push(ControlFrame::Block { end_label });
            }
        }
        Operator::End => match emitter.pop_control() {
            Some(ControlFrame::Block { end_label }) => {
                emitter.emit(Instruction::Fallthrough);
                emitter.define_label(end_label);
            }
            Some(ControlFrame::If {
                else_label,
                end_label,
            }) => {
                emitter.emit(Instruction::Fallthrough);
                emitter.define_label(else_label);
                emitter.define_label(end_label);
            }
            _ => {}
        },
        Operator::Br { relative_depth } => {
            let target = emitter.get_branch_target(*relative_depth)?;
            emitter.emit_jump_to_label(target);
        }
        Operator::BrIf { relative_depth } => {
            let cond = emitter.stack.pop();
            let target = emitter.get_branch_target(*relative_depth)?;
            emitter.emit_branch_ne_imm_to_label(cond, 0, target);
        }
        Operator::Return => {
            emitter.emit(Instruction::LoadImm {
                reg: 2,
                value: EXIT_ADDRESS,
            });
            emitter.emit(Instruction::JumpInd { reg: 2, offset: 0 });
        }
        _ => {
            return Err(Error::Unsupported(format!("{op:?}")));
        }
    }
    Ok(())
}
