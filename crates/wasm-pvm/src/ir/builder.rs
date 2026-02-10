use crate::{Error, Result};
use wasmparser::{FunctionBody, Operator};

use super::IrInstruction;

/// Build IR from a WASM function body.
///
/// Walks `wasmparser::Operator`s and maps each to the corresponding `IrInstruction`.
/// This is a straightforward 1:1 mapping with no stack simulation.
pub fn build_ir(body: &FunctionBody) -> Result<(usize, Vec<IrInstruction>)> {
    let mut total_locals = 0usize;
    let locals_reader = body.get_locals_reader()?;
    for local in locals_reader {
        let (count, _ty) = local?;
        total_locals += count as usize;
    }

    let ops: Vec<Operator> = body
        .get_operators_reader()?
        .into_iter()
        .collect::<std::result::Result<_, _>>()?;

    let mut ir = Vec::with_capacity(ops.len());
    for op in &ops {
        ir.push(translate_operator(op)?);
    }

    Ok((total_locals, ir))
}

fn translate_operator(op: &Operator) -> Result<IrInstruction> {
    match op {
        // Constants
        Operator::I32Const { value } => Ok(IrInstruction::I32Const(*value)),
        Operator::I64Const { value } => Ok(IrInstruction::I64Const(*value)),

        // Locals / Globals
        Operator::LocalGet { local_index } => Ok(IrInstruction::LocalGet(*local_index)),
        Operator::LocalSet { local_index } => Ok(IrInstruction::LocalSet(*local_index)),
        Operator::LocalTee { local_index } => Ok(IrInstruction::LocalTee(*local_index)),
        Operator::GlobalGet { global_index } => Ok(IrInstruction::GlobalGet(*global_index)),
        Operator::GlobalSet { global_index } => Ok(IrInstruction::GlobalSet(*global_index)),

        // Arithmetic i32
        Operator::I32Add => Ok(IrInstruction::I32Add),
        Operator::I32Sub => Ok(IrInstruction::I32Sub),
        Operator::I32Mul => Ok(IrInstruction::I32Mul),
        Operator::I32DivU => Ok(IrInstruction::I32DivU),
        Operator::I32DivS => Ok(IrInstruction::I32DivS),
        Operator::I32RemU => Ok(IrInstruction::I32RemU),
        Operator::I32RemS => Ok(IrInstruction::I32RemS),

        // Arithmetic i64
        Operator::I64Add => Ok(IrInstruction::I64Add),
        Operator::I64Sub => Ok(IrInstruction::I64Sub),
        Operator::I64Mul => Ok(IrInstruction::I64Mul),
        Operator::I64DivU => Ok(IrInstruction::I64DivU),
        Operator::I64DivS => Ok(IrInstruction::I64DivS),
        Operator::I64RemU => Ok(IrInstruction::I64RemU),
        Operator::I64RemS => Ok(IrInstruction::I64RemS),

        // Bitwise
        Operator::I32And => Ok(IrInstruction::I32And),
        Operator::I32Or => Ok(IrInstruction::I32Or),
        Operator::I32Xor => Ok(IrInstruction::I32Xor),
        Operator::I64And => Ok(IrInstruction::I64And),
        Operator::I64Or => Ok(IrInstruction::I64Or),
        Operator::I64Xor => Ok(IrInstruction::I64Xor),

        // Shifts
        Operator::I32Shl => Ok(IrInstruction::I32Shl),
        Operator::I32ShrU => Ok(IrInstruction::I32ShrU),
        Operator::I32ShrS => Ok(IrInstruction::I32ShrS),
        Operator::I64Shl => Ok(IrInstruction::I64Shl),
        Operator::I64ShrU => Ok(IrInstruction::I64ShrU),
        Operator::I64ShrS => Ok(IrInstruction::I64ShrS),

        // Rotations
        Operator::I32Rotl => Ok(IrInstruction::I32Rotl),
        Operator::I32Rotr => Ok(IrInstruction::I32Rotr),
        Operator::I64Rotl => Ok(IrInstruction::I64Rotl),
        Operator::I64Rotr => Ok(IrInstruction::I64Rotr),

        // Bit counting
        Operator::I32Clz => Ok(IrInstruction::I32Clz),
        Operator::I32Ctz => Ok(IrInstruction::I32Ctz),
        Operator::I32Popcnt => Ok(IrInstruction::I32Popcnt),
        Operator::I64Clz => Ok(IrInstruction::I64Clz),
        Operator::I64Ctz => Ok(IrInstruction::I64Ctz),
        Operator::I64Popcnt => Ok(IrInstruction::I64Popcnt),

        // Comparisons
        Operator::I32Eqz => Ok(IrInstruction::I32Eqz),
        Operator::I32Eq => Ok(IrInstruction::I32Eq),
        Operator::I32Ne => Ok(IrInstruction::I32Ne),
        Operator::I32LtU => Ok(IrInstruction::I32LtU),
        Operator::I32LtS => Ok(IrInstruction::I32LtS),
        Operator::I32GtU => Ok(IrInstruction::I32GtU),
        Operator::I32GtS => Ok(IrInstruction::I32GtS),
        Operator::I32LeU => Ok(IrInstruction::I32LeU),
        Operator::I32LeS => Ok(IrInstruction::I32LeS),
        Operator::I32GeU => Ok(IrInstruction::I32GeU),
        Operator::I32GeS => Ok(IrInstruction::I32GeS),
        Operator::I64Eqz => Ok(IrInstruction::I64Eqz),
        Operator::I64Eq => Ok(IrInstruction::I64Eq),
        Operator::I64Ne => Ok(IrInstruction::I64Ne),
        Operator::I64LtU => Ok(IrInstruction::I64LtU),
        Operator::I64LtS => Ok(IrInstruction::I64LtS),
        Operator::I64GtU => Ok(IrInstruction::I64GtU),
        Operator::I64GtS => Ok(IrInstruction::I64GtS),
        Operator::I64LeU => Ok(IrInstruction::I64LeU),
        Operator::I64LeS => Ok(IrInstruction::I64LeS),
        Operator::I64GeU => Ok(IrInstruction::I64GeU),
        Operator::I64GeS => Ok(IrInstruction::I64GeS),

        // Memory loads
        Operator::I32Load { memarg } => Ok(IrInstruction::I32Load {
            offset: memarg.offset,
        }),
        Operator::I64Load { memarg } => Ok(IrInstruction::I64Load {
            offset: memarg.offset,
        }),
        Operator::I32Load8U { memarg } => Ok(IrInstruction::I32Load8U {
            offset: memarg.offset,
        }),
        Operator::I32Load8S { memarg } => Ok(IrInstruction::I32Load8S {
            offset: memarg.offset,
        }),
        Operator::I32Load16U { memarg } => Ok(IrInstruction::I32Load16U {
            offset: memarg.offset,
        }),
        Operator::I32Load16S { memarg } => Ok(IrInstruction::I32Load16S {
            offset: memarg.offset,
        }),
        Operator::I64Load8U { memarg } => Ok(IrInstruction::I64Load8U {
            offset: memarg.offset,
        }),
        Operator::I64Load8S { memarg } => Ok(IrInstruction::I64Load8S {
            offset: memarg.offset,
        }),
        Operator::I64Load16U { memarg } => Ok(IrInstruction::I64Load16U {
            offset: memarg.offset,
        }),
        Operator::I64Load16S { memarg } => Ok(IrInstruction::I64Load16S {
            offset: memarg.offset,
        }),
        Operator::I64Load32U { memarg } => Ok(IrInstruction::I64Load32U {
            offset: memarg.offset,
        }),
        Operator::I64Load32S { memarg } => Ok(IrInstruction::I64Load32S {
            offset: memarg.offset,
        }),

        // Memory stores
        Operator::I32Store { memarg } => Ok(IrInstruction::I32Store {
            offset: memarg.offset,
        }),
        Operator::I64Store { memarg } => Ok(IrInstruction::I64Store {
            offset: memarg.offset,
        }),
        Operator::I32Store8 { memarg } => Ok(IrInstruction::I32Store8 {
            offset: memarg.offset,
        }),
        Operator::I32Store16 { memarg } => Ok(IrInstruction::I32Store16 {
            offset: memarg.offset,
        }),
        Operator::I64Store8 { memarg } => Ok(IrInstruction::I64Store8 {
            offset: memarg.offset,
        }),
        Operator::I64Store16 { memarg } => Ok(IrInstruction::I64Store16 {
            offset: memarg.offset,
        }),
        Operator::I64Store32 { memarg } => Ok(IrInstruction::I64Store32 {
            offset: memarg.offset,
        }),

        // Memory management
        Operator::MemorySize { mem: 0, .. } => Ok(IrInstruction::MemorySize),
        Operator::MemoryGrow { mem: 0, .. } => Ok(IrInstruction::MemoryGrow),
        Operator::MemoryFill { mem: 0 } => Ok(IrInstruction::MemoryFill),
        Operator::MemoryCopy {
            dst_mem: 0,
            src_mem: 0,
        } => Ok(IrInstruction::MemoryCopy),

        // Control flow
        Operator::Block { blockty } => {
            let has_result = !matches!(blockty, wasmparser::BlockType::Empty);
            Ok(IrInstruction::Block { has_result })
        }
        Operator::Loop { blockty: _ } => Ok(IrInstruction::Loop),
        Operator::If { blockty } => {
            let has_result = !matches!(blockty, wasmparser::BlockType::Empty);
            Ok(IrInstruction::If { has_result })
        }
        Operator::Else => Ok(IrInstruction::Else),
        Operator::End => Ok(IrInstruction::End),
        Operator::Br { relative_depth } => Ok(IrInstruction::Br(*relative_depth)),
        Operator::BrIf { relative_depth } => Ok(IrInstruction::BrIf(*relative_depth)),
        Operator::BrTable { targets } => {
            let target_depths: Vec<u32> = targets
                .targets()
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let default = targets.default();
            Ok(IrInstruction::BrTable {
                targets: target_depths,
                default,
            })
        }
        Operator::Return => Ok(IrInstruction::Return),

        // Calls
        Operator::Call { function_index } => Ok(IrInstruction::Call(*function_index)),
        Operator::CallIndirect {
            type_index,
            table_index,
        } => Ok(IrInstruction::CallIndirect {
            type_idx: *type_index,
            table_idx: *table_index,
        }),

        // Stack manipulation
        Operator::Drop => Ok(IrInstruction::Drop),
        Operator::Select => Ok(IrInstruction::Select),

        // Misc
        Operator::Unreachable => Ok(IrInstruction::Unreachable),
        Operator::Nop => Ok(IrInstruction::Nop),

        // Conversions
        Operator::I32WrapI64 => Ok(IrInstruction::I32WrapI64),
        Operator::I64ExtendI32S => Ok(IrInstruction::I64ExtendI32S),
        Operator::I64ExtendI32U => Ok(IrInstruction::I64ExtendI32U),
        Operator::I32Extend8S => Ok(IrInstruction::I32Extend8S),
        Operator::I32Extend16S => Ok(IrInstruction::I32Extend16S),
        Operator::I64Extend8S => Ok(IrInstruction::I64Extend8S),
        Operator::I64Extend16S => Ok(IrInstruction::I64Extend16S),
        Operator::I64Extend32S => Ok(IrInstruction::I64Extend32S),

        // Float truncation stubs
        Operator::I32TruncSatF64U => Ok(IrInstruction::I32TruncSatF64U),
        Operator::I32TruncSatF64S => Ok(IrInstruction::I32TruncSatF64S),
        Operator::I32TruncSatF32U => Ok(IrInstruction::I32TruncSatF32U),
        Operator::I32TruncSatF32S => Ok(IrInstruction::I32TruncSatF32S),
        Operator::I64TruncSatF64U => Ok(IrInstruction::I64TruncSatF64U),
        Operator::I64TruncSatF64S => Ok(IrInstruction::I64TruncSatF64S),
        Operator::I64TruncSatF32U => Ok(IrInstruction::I64TruncSatF32U),
        Operator::I64TruncSatF32S => Ok(IrInstruction::I64TruncSatF32S),

        _ => Err(Error::Unsupported(format!("{op:?}"))),
    }
}
