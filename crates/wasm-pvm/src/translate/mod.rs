mod codegen;
mod stack;

use crate::pvm::Instruction;
use crate::{Error, Result, SpiProgram};
use wasmparser::{FunctionBody, GlobalType, Parser, Payload};

pub use codegen::CompileContext;

pub fn compile(wasm: &[u8]) -> Result<SpiProgram> {
    let mut functions = Vec::new();
    let mut func_types: Vec<wasmparser::FuncType> = Vec::new();
    let mut function_type_indices = Vec::new();
    let mut globals: Vec<GlobalType> = Vec::new();
    let mut global_names: Vec<Option<String>> = Vec::new();
    let mut main_func_idx: Option<u32> = None;

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
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export?;
                    if export.name == "main"
                        && let wasmparser::ExternalKind::Func = export.kind
                    {
                        main_func_idx = Some(export.index);
                    }
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

    let main_func_idx = main_func_idx.unwrap_or(0) as usize;

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

    let function_signatures: Vec<(usize, bool)> = function_type_indices
        .iter()
        .map(|&type_idx| {
            let func_type = func_types.get(type_idx as usize);
            let num_params = func_type.map_or(0, |f| f.params().len());
            let has_return = func_type.is_some_and(|f| !f.results().is_empty());
            (num_params, has_return)
        })
        .collect();

    let mut all_instructions: Vec<Instruction> = Vec::new();
    let mut all_call_fixups: Vec<(usize, codegen::CallFixup)> = Vec::new();
    let mut function_offsets: Vec<usize> = Vec::new();

    let needs_entry_jump = main_func_idx != 0;
    if needs_entry_jump {
        all_instructions.push(Instruction::Jump { offset: 0 });
    }

    for (func_idx, func) in functions.iter().enumerate() {
        let (num_params, has_return) = function_signatures
            .get(func_idx)
            .copied()
            .unwrap_or((0, false));

        let is_main = func_idx == main_func_idx;

        let ctx = CompileContext {
            num_params,
            num_locals: 0,
            num_globals: globals.len(),
            result_ptr_global: if is_main { result_ptr_global } else { None },
            result_len_global: if is_main { result_len_global } else { None },
            is_main,
            has_return,
            function_offsets: vec![],
            function_signatures: function_signatures.clone(),
            func_idx,
        };

        let func_start_offset: usize = all_instructions.iter().map(|i| i.encode().len()).sum();
        function_offsets.push(func_start_offset);

        let translation = codegen::translate_function(func, &ctx)?;

        let instr_base = all_instructions.len();
        for fixup in translation.call_fixups {
            all_call_fixups.push((instr_base, fixup));
        }

        all_instructions.extend(translation.instructions);
    }

    let jump_table =
        resolve_call_fixups(&mut all_instructions, &all_call_fixups, &function_offsets)?;

    if needs_entry_jump {
        let main_offset = function_offsets[main_func_idx] as i32;
        if let Instruction::Jump { offset } = &mut all_instructions[0] {
            *offset = main_offset;
        }
    }

    let blob = crate::pvm::ProgramBlob::new(all_instructions).with_jump_table(jump_table);

    let heap_pages = calculate_heap_pages(functions.len());

    Ok(SpiProgram::new(blob).with_heap_pages(heap_pages))
}

fn calculate_heap_pages(num_functions: usize) -> u16 {
    let globals_size = 256;
    let user_results_size = 256;
    let spilled_locals_size = num_functions * codegen::SPILLED_LOCALS_PER_FUNC as usize;
    let min_user_heap = 4096;

    let total_bytes = globals_size + user_results_size + spilled_locals_size + min_user_heap;
    let pages = total_bytes.div_ceil(4096);

    pages.max(16) as u16
}

fn resolve_call_fixups(
    instructions: &mut [Instruction],
    call_fixups: &[(usize, codegen::CallFixup)],
    function_offsets: &[usize],
) -> Result<Vec<u32>> {
    let mut jump_table: Vec<u32> = Vec::new();

    for (instr_base, fixup) in call_fixups {
        let target_offset = function_offsets
            .get(fixup.target_func as usize)
            .ok_or_else(|| {
                Error::Unsupported(format!("call to unknown function {}", fixup.target_func))
            })?;

        let return_addr_idx = instr_base + fixup.return_addr_instr;
        let jump_idx = instr_base + fixup.jump_instr;

        let return_addr_offset: usize = instructions[..=jump_idx]
            .iter()
            .map(|i| i.encode().len())
            .sum();

        let jump_table_index = jump_table.len();
        jump_table.push(return_addr_offset as u32);

        let jump_table_address = (jump_table_index as u64 + 1) * 2;

        if let Instruction::LoadImm64 { value, .. } = &mut instructions[return_addr_idx] {
            *value = jump_table_address;
        }

        let jump_start_offset: usize = instructions[..jump_idx]
            .iter()
            .map(|i| i.encode().len())
            .sum();
        let relative_offset = (*target_offset as i32) - (jump_start_offset as i32);

        if let Instruction::Jump { offset } = &mut instructions[jump_idx] {
            *offset = relative_offset;
        }
    }
    Ok(jump_table)
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
