use crate::pvm::Instruction;
use crate::{Error, Result};
use wasmparser::{FunctionBody, Operator, ValType};

use super::stack::StackMachine;

pub fn translate_function(
    body: &FunctionBody,
    result_type: Option<ValType>,
) -> Result<Vec<Instruction>> {
    let mut instructions = Vec::new();
    let mut stack = StackMachine::new();

    let locals_reader = body.get_locals_reader()?;
    for local in locals_reader {
        let (_count, _ty) = local?;
    }

    let mut ops_reader = body.get_operators_reader()?;
    while !ops_reader.eof() {
        let op = ops_reader.read()?;
        translate_op(&op, &mut instructions, &mut stack)?;
    }

    if result_type.is_some() && stack.depth() > 0 {
        let result_reg = stack.peek(0);
        if result_reg != 2 {
            instructions.push(Instruction::Add32 {
                dst: 2,
                src1: result_reg,
                src2: result_reg,
            });
            instructions.push(Instruction::LoadImm { reg: 3, value: 0 });
            instructions.push(Instruction::Add32 {
                dst: 2,
                src1: result_reg,
                src2: 3,
            });
        }
    }

    // HALT: jump to r0 which contains 0xFFFF0000 (EXIT address)
    if instructions.is_empty() || !instructions.last().unwrap().is_terminating() {
        instructions.push(Instruction::JumpInd { reg: 0, offset: 0 });
    }

    Ok(instructions)
}

fn translate_op(
    op: &Operator,
    instructions: &mut Vec<Instruction>,
    stack: &mut StackMachine,
) -> Result<()> {
    match op {
        Operator::I32Const { value } => {
            let reg = stack.push();
            instructions.push(Instruction::LoadImm { reg, value: *value });
        }
        Operator::I64Const { value } => {
            let reg = stack.push();
            instructions.push(Instruction::LoadImm64 {
                reg,
                value: *value as u64,
            });
        }
        Operator::I32Add => {
            let src2 = stack.pop();
            let src1 = stack.pop();
            let dst = stack.push();
            instructions.push(Instruction::Add32 { dst, src1, src2 });
        }
        Operator::I64Add => {
            let src2 = stack.pop();
            let src1 = stack.pop();
            let dst = stack.push();
            instructions.push(Instruction::Add64 { dst, src1, src2 });
        }
        Operator::End => {}
        Operator::Return => {
            instructions.push(Instruction::JumpInd { reg: 0, offset: 0 });
        }
        _ => {
            return Err(Error::Unsupported(format!("{op:?}")));
        }
    }
    Ok(())
}
