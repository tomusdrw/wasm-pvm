mod codegen;
mod stack;

use crate::{Error, Result, SpiProgram};
use wasmparser::{FunctionBody, Parser, Payload};

pub fn compile(wasm: &[u8]) -> Result<SpiProgram> {
    let mut functions = Vec::new();
    let mut func_types: Vec<wasmparser::FuncType> = Vec::new();
    let mut function_type_indices = Vec::new();

    for payload in Parser::new(0).parse_all(wasm) {
        match payload? {
            Payload::TypeSection(reader) => {
                for rec_group in reader {
                    for sub_type in rec_group?.into_types() {
                        if let wasmparser::CompositeInnerType::Func(f) =
                            &sub_type.composite_type.inner
                        {
                            func_types.push(f.clone());
                        }
                    }
                }
            }
            Payload::FunctionSection(reader) => {
                for type_idx in reader {
                    function_type_indices.push(type_idx?);
                }
            }
            Payload::CodeSectionEntry(body) => {
                functions.push(body);
            }
            _ => {}
        }
    }

    if functions.is_empty() {
        return Err(Error::NoExportedFunction);
    }

    let func = &functions[0];
    let type_idx = function_type_indices.first().copied().unwrap_or(0);
    let func_type = func_types.get(type_idx as usize);

    let result_type = func_type.and_then(|f| f.results().first().copied());

    let instructions = codegen::translate_function(func, result_type)?;
    let blob = crate::pvm::ProgramBlob::new(instructions);

    Ok(SpiProgram::new(blob))
}

fn check_for_floats(body: &FunctionBody) -> Result<()> {
    let mut reader = body.get_operators_reader()?;
    while !reader.eof() {
        let op = reader.read()?;
        if is_float_op(&op) {
            return Err(Error::FloatNotSupported);
        }
    }
    Ok(())
}

fn is_float_op(op: &wasmparser::Operator) -> bool {
    use wasmparser::Operator::*;
    matches!(
        op,
        F32Load { .. }
            | F64Load { .. }
            | F32Store { .. }
            | F64Store { .. }
            | F32Const { .. }
            | F64Const { .. }
            | F32Eq
            | F32Ne
            | F32Lt
            | F32Gt
            | F32Le
            | F32Ge
            | F64Eq
            | F64Ne
            | F64Lt
            | F64Gt
            | F64Le
            | F64Ge
            | F32Abs
            | F32Neg
            | F32Ceil
            | F32Floor
            | F32Trunc
            | F32Nearest
            | F32Sqrt
            | F32Add
            | F32Sub
            | F32Mul
            | F32Div
            | F32Min
            | F32Max
            | F32Copysign
            | F64Abs
            | F64Neg
            | F64Ceil
            | F64Floor
            | F64Trunc
            | F64Nearest
            | F64Sqrt
            | F64Add
            | F64Sub
            | F64Mul
            | F64Div
            | F64Min
            | F64Max
            | F64Copysign
    )
}
