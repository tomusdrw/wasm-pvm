mod codegen;
mod stack;

use crate::{Error, Result, SpiProgram};
use wasmparser::{FunctionBody, GlobalType, Parser, Payload};

pub use codegen::CompileContext;

pub fn compile(wasm: &[u8]) -> Result<SpiProgram> {
    let mut functions = Vec::new();
    let mut func_types: Vec<wasmparser::FuncType> = Vec::new();
    let mut function_type_indices = Vec::new();
    let mut globals: Vec<GlobalType> = Vec::new();
    let mut global_names: Vec<Option<String>> = Vec::new();

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
            Payload::GlobalSection(reader) => {
                for global in reader {
                    let g = global?;
                    globals.push(g.ty);
                    global_names.push(None);
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

    let num_params = func_type.map_or(0, |f| f.params().len());

    let mut result_ptr_global = None;
    let mut result_len_global = None;

    for (i, name) in global_names.iter().enumerate() {
        if let Some(n) = name {
            if n == "result_ptr" || n == "$result_ptr" {
                result_ptr_global = Some(i as u32);
            } else if n == "result_len" || n == "$result_len" {
                result_len_global = Some(i as u32);
            }
        }
    }

    if result_ptr_global.is_none()
        && result_len_global.is_none()
        && globals.len() >= 2
        && globals[0].mutable
        && globals[1].mutable
    {
        result_ptr_global = Some(0);
        result_len_global = Some(1);
    }

    let ctx = CompileContext {
        num_params,
        num_locals: 0,
        num_globals: globals.len(),
        result_ptr_global,
        result_len_global,
    };

    let instructions = codegen::translate_function(func, &ctx)?;
    let blob = crate::pvm::ProgramBlob::new(instructions);

    Ok(SpiProgram::new(blob))
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn is_float_op(op: &wasmparser::Operator) -> bool {
    use wasmparser::Operator::{
        F32Abs, F32Add, F32Ceil, F32Const, F32Copysign, F32Div, F32Eq, F32Floor, F32Ge, F32Gt,
        F32Le, F32Load, F32Lt, F32Max, F32Min, F32Mul, F32Ne, F32Nearest, F32Neg, F32Sqrt,
        F32Store, F32Sub, F32Trunc, F64Abs, F64Add, F64Ceil, F64Const, F64Copysign, F64Div, F64Eq,
        F64Floor, F64Ge, F64Gt, F64Le, F64Load, F64Lt, F64Max, F64Min, F64Mul, F64Ne, F64Nearest,
        F64Neg, F64Sqrt, F64Store, F64Sub, F64Trunc,
    };
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
