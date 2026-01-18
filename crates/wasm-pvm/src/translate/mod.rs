mod codegen;
mod stack;

use crate::pvm::Instruction;
use crate::{Error, Result, SpiProgram};
use wasmparser::{FunctionBody, GlobalType, Parser, Payload};

pub use codegen::CompileContext;

const ENTRY_HEADER_SIZE: usize = 10;

/// Represents an active data segment parsed from WASM
struct DataSegment {
    /// Offset in WASM linear memory (where the data goes)
    offset: u32,
    /// The actual data bytes
    data: Vec<u8>,
}

pub fn compile(wasm: &[u8]) -> Result<SpiProgram> {
    let mut functions = Vec::new();
    let mut func_types: Vec<wasmparser::FuncType> = Vec::new();
    let mut function_type_indices = Vec::new();
    let mut globals: Vec<GlobalType> = Vec::new();
    let mut global_names: Vec<Option<String>> = Vec::new();
    let mut main_func_idx: Option<u32> = None;
    let mut secondary_entry_func_idx: Option<u32> = None;
    let mut tables: Vec<wasmparser::TableType> = Vec::new();
    let mut table_elements: Vec<(u32, u32, Vec<u32>)> = Vec::new();
    let mut data_segments: Vec<DataSegment> = Vec::new();
    let mut num_imported_funcs: u32 = 0;
    let mut imported_func_type_indices: Vec<u32> = Vec::new();

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
            Payload::ImportSection(reader) => {
                for import in reader {
                    let import = import?;
                    if let wasmparser::TypeRef::Func(type_idx) = import.ty {
                        num_imported_funcs += 1;
                        imported_func_type_indices.push(type_idx);
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
            Payload::TableSection(reader) => {
                for table in reader {
                    tables.push(table?.ty);
                }
            }
            Payload::ElementSection(reader) => {
                for element in reader {
                    let element = element?;
                    if let wasmparser::ElementKind::Active {
                        table_index,
                        offset_expr,
                    } = element.kind
                    {
                        let table_idx = table_index.unwrap_or(0);
                        let offset = eval_const_i32(&offset_expr)?;
                        let func_indices: Vec<u32> = match element.items {
                            wasmparser::ElementItems::Functions(reader) => {
                                reader.into_iter().collect::<std::result::Result<_, _>>()?
                            }
                            wasmparser::ElementItems::Expressions(_, reader) => {
                                let mut indices = Vec::new();
                                for expr in reader {
                                    let expr = expr?;
                                    if let Some(idx) = eval_const_ref(&expr) {
                                        indices.push(idx);
                                    }
                                }
                                indices
                            }
                        };
                        table_elements.push((table_idx, offset as u32, func_indices));
                    }
                }
            }
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export?;
                    if let wasmparser::ExternalKind::Func = export.kind {
                        if export.name == "main" {
                            main_func_idx = Some(export.index);
                        } else if export.name == "main2" {
                            secondary_entry_func_idx = Some(export.index);
                        }
                    }
                }
            }
            Payload::CodeSectionEntry(body) => {
                functions.push(body);
            }
            Payload::DataSection(reader) => {
                for data in reader {
                    let data = data?;
                    if let wasmparser::DataKind::Active {
                        memory_index: _,
                        offset_expr,
                    } = data.kind
                    {
                        let offset = eval_const_i32(&offset_expr)? as u32;
                        data_segments.push(DataSegment {
                            offset,
                            data: data.data.to_vec(),
                        });
                    }
                    // Passive data segments are ignored for now (used with memory.init)
                }
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

    // Function signatures indexed by global function index (imports first, then locals)
    let function_signatures: Vec<(usize, bool)> = imported_func_type_indices
        .iter()
        .chain(function_type_indices.iter())
        .map(|&type_idx| {
            let func_type = func_types.get(type_idx as usize);
            let num_params = func_type.map_or(0, |f| f.params().len());
            let has_return = func_type.is_some_and(|f| !f.results().is_empty());
            (num_params, has_return)
        })
        .collect();

    let type_signatures: Vec<(usize, usize)> = func_types
        .iter()
        .map(|f| (f.params().len(), f.results().len()))
        .collect();

    let table_size = tables.first().map_or(0, |t| t.initial as usize);
    let mut function_table: Vec<u32> = vec![u32::MAX; table_size];
    for (table_idx, offset, func_indices) in &table_elements {
        if *table_idx == 0 {
            for (i, &func_idx) in func_indices.iter().enumerate() {
                let idx = *offset as usize + i;
                if idx < function_table.len() {
                    function_table[idx] = func_idx;
                }
            }
        }
    }

    let mut all_instructions: Vec<Instruction> = Vec::new();
    let mut all_call_fixups: Vec<(usize, codegen::CallFixup)> = Vec::new();
    let mut all_indirect_call_fixups: Vec<(usize, codegen::IndirectCallFixup)> = Vec::new();
    let mut function_offsets: Vec<usize> = Vec::new();

    all_instructions.push(Instruction::Jump { offset: 0 });

    if secondary_entry_func_idx.is_some() {
        all_instructions.push(Instruction::Jump { offset: 0 });
    } else {
        all_instructions.push(Instruction::Trap);
        all_instructions.push(Instruction::Fallthrough);
        all_instructions.push(Instruction::Fallthrough);
        all_instructions.push(Instruction::Fallthrough);
        all_instructions.push(Instruction::Fallthrough);
    }

    let entry_header_bytes: usize = all_instructions.iter().map(|i| i.encode().len()).sum();
    debug_assert_eq!(
        entry_header_bytes, ENTRY_HEADER_SIZE,
        "Entry header must be exactly 10 bytes"
    );

    let secondary_entry_idx_resolved = secondary_entry_func_idx.map(|idx| idx as usize);

    for (local_func_idx, func) in functions.iter().enumerate() {
        // Global function index = num_imported_funcs + local_func_idx
        let global_func_idx = num_imported_funcs as usize + local_func_idx;

        let (num_params, has_return) = function_signatures
            .get(global_func_idx)
            .copied()
            .unwrap_or((0, false));

        let is_main = local_func_idx == main_func_idx;
        let is_secondary_entry = secondary_entry_idx_resolved == Some(local_func_idx);

        let is_entry_func = is_main || is_secondary_entry;
        let ctx = CompileContext {
            num_params,
            num_locals: 0,
            num_globals: globals.len(),
            result_ptr_global: if is_entry_func {
                result_ptr_global
            } else {
                None
            },
            result_len_global: if is_entry_func {
                result_len_global
            } else {
                None
            },
            is_main: is_entry_func,
            has_return,
            function_offsets: vec![],
            function_signatures: function_signatures.clone(),
            func_idx: global_func_idx,
            function_table: function_table.clone(),
            type_signatures: type_signatures.clone(),
            num_imported_funcs: num_imported_funcs as usize,
        };

        let func_start_offset: usize = all_instructions.iter().map(|i| i.encode().len()).sum();
        function_offsets.push(func_start_offset);

        let translation = codegen::translate_function(func, &ctx)?;

        let instr_base = all_instructions.len();
        for fixup in translation.call_fixups {
            all_call_fixups.push((instr_base, fixup));
        }
        for fixup in translation.indirect_call_fixups {
            all_indirect_call_fixups.push((instr_base, fixup));
        }

        all_instructions.extend(translation.instructions);
    }

    let (jump_table, func_entry_jump_table_base) = resolve_call_fixups(
        &mut all_instructions,
        &all_call_fixups,
        &all_indirect_call_fixups,
        &function_offsets,
    )?;

    let main_offset = function_offsets[main_func_idx] as i32;
    if let Instruction::Jump { offset } = &mut all_instructions[0] {
        *offset = main_offset;
    }

    if let Some(secondary_idx) = secondary_entry_idx_resolved {
        let secondary_offset = function_offsets[secondary_idx] as i32 - 5;
        if let Instruction::Jump { offset } = &mut all_instructions[1] {
            *offset = secondary_offset;
        }
    }

    let mut ro_data = vec![0u8];
    if !function_table.is_empty() {
        ro_data.clear();
        for &func_idx in &function_table {
            if func_idx == u32::MAX {
                ro_data.extend_from_slice(&u32::MAX.to_le_bytes());
            } else {
                let jump_ref = 2 * (func_entry_jump_table_base + func_idx as usize + 1) as u32;
                ro_data.extend_from_slice(&jump_ref.to_le_bytes());
            }
        }
    }

    let blob = crate::pvm::ProgramBlob::new(all_instructions).with_jump_table(jump_table);

    // Build rw_data from WASM data segments
    // Data segments specify offsets in WASM linear memory (starting at 0)
    // We need to place them at WASM_MEMORY_BASE (0x50000) in PVM
    // But rw_data is placed at 0x30000 by the SPI loader, so we need to account for that
    //
    // Memory layout after SPI loading:
    // - 0x30000: Start of RW data segment (globals at 0x30000, spilled locals at 0x40000)
    // - 0x50000: WASM linear memory base (WASM_MEMORY_BASE)
    //
    // rw_data byte 0 goes to PVM address 0x30000
    // rw_data byte N goes to PVM address 0x30000 + N
    // WASM memory offset 0 should be at PVM address 0x50000 = 0x30000 + 0x20000
    // So WASM data at offset X goes to rw_data byte (0x20000 + X)
    let rw_data = build_rw_data(&data_segments);

    let heap_pages = calculate_heap_pages(functions.len(), &data_segments);

    Ok(SpiProgram::new(blob)
        .with_heap_pages(heap_pages)
        .with_ro_data(ro_data)
        .with_rw_data(rw_data))
}

/// Build the rw_data section from WASM data segments.
///
/// The RW data segment in SPI is loaded at 0x30000. WASM linear memory starts at
/// WASM_MEMORY_BASE (0x50000). So WASM offset X maps to rw_data offset (0x20000 + X).
fn build_rw_data(data_segments: &[DataSegment]) -> Vec<u8> {
    if data_segments.is_empty() {
        return Vec::new();
    }

    // Calculate the size of rw_data needed
    // We need to cover from 0x30000 to at least WASM_MEMORY_BASE + max(wasm_offset + data_len)
    let wasm_to_rw_offset = codegen::WASM_MEMORY_BASE as u32 - 0x30000;

    let max_end = data_segments
        .iter()
        .map(|seg| wasm_to_rw_offset + seg.offset + seg.data.len() as u32)
        .max()
        .unwrap_or(0);

    let mut rw_data = vec![0u8; max_end as usize];

    for seg in data_segments {
        let rw_offset = (wasm_to_rw_offset + seg.offset) as usize;
        if rw_offset + seg.data.len() <= rw_data.len() {
            rw_data[rw_offset..rw_offset + seg.data.len()].copy_from_slice(&seg.data);
        }
    }

    rw_data
}

fn calculate_heap_pages(num_functions: usize, data_segments: &[DataSegment]) -> u16 {
    // Memory layout:
    // 0x30000-0x300FF: Globals (256 bytes)
    // 0x30100-0x3FFFF: User heap (~64KB)
    // 0x40000+: Spilled locals (512 bytes per function)
    // 0x50000+: WASM linear memory (data segments + dynamic allocation)
    //
    // The heap segment starts at 0x30000 (GLOBAL_MEMORY_BASE).
    // We need to allocate enough pages to cover:
    // 1. Spilled locals
    // 2. WASM linear memory (including data segments)
    let spilled_locals_end = codegen::SPILLED_LOCALS_BASE as usize
        + num_functions * codegen::SPILLED_LOCALS_PER_FUNC as usize;

    // WASM memory end (include data segments + some extra for heap)
    let wasm_memory_end = if data_segments.is_empty() {
        codegen::WASM_MEMORY_BASE as usize + 64 * 1024 // 64KB default
    } else {
        let max_data_end = data_segments
            .iter()
            .map(|seg| codegen::WASM_MEMORY_BASE as usize + seg.offset as usize + seg.data.len())
            .max()
            .unwrap_or(codegen::WASM_MEMORY_BASE as usize);
        // Add 64KB for dynamic allocation
        max_data_end + 64 * 1024
    };

    let end = spilled_locals_end.max(wasm_memory_end);

    // Total bytes from heap base (0x30000) to end
    let total_bytes = end - 0x30000;
    let pages = total_bytes.div_ceil(4096);

    pages.max(16) as u16
}

fn resolve_call_fixups(
    instructions: &mut [Instruction],
    call_fixups: &[(usize, codegen::CallFixup)],
    indirect_call_fixups: &[(usize, codegen::IndirectCallFixup)],
    function_offsets: &[usize],
) -> Result<(Vec<u32>, usize)> {
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

    for (instr_base, fixup) in indirect_call_fixups {
        let return_addr_idx = instr_base + fixup.return_addr_instr;
        let jump_ind_idx = instr_base + fixup.jump_ind_instr;

        let return_addr_offset: usize = instructions[..=jump_ind_idx]
            .iter()
            .map(|i| i.encode().len())
            .sum();

        let jump_table_index = jump_table.len();
        jump_table.push(return_addr_offset as u32);

        let jump_table_address = (jump_table_index as u64 + 1) * 2;

        if let Instruction::LoadImm64 { value, .. } = &mut instructions[return_addr_idx] {
            *value = jump_table_address;
        }
    }

    let func_entry_base = jump_table.len();
    for &offset in function_offsets {
        jump_table.push(offset as u32);
    }

    Ok((jump_table, func_entry_base))
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

fn eval_const_i32(expr: &wasmparser::ConstExpr) -> Result<i32> {
    let mut reader = expr.get_binary_reader();
    while !reader.eof() {
        match reader.read_operator()? {
            wasmparser::Operator::I32Const { value } => return Ok(value),
            wasmparser::Operator::End => break,
            _ => {}
        }
    }
    Ok(0)
}

fn eval_const_ref(expr: &wasmparser::ConstExpr) -> Option<u32> {
    let mut reader = expr.get_binary_reader();
    while !reader.eof() {
        if let Ok(op) = reader.read_operator() {
            match op {
                wasmparser::Operator::RefFunc { function_index } => return Some(function_index),
                wasmparser::Operator::End => break,
                _ => {}
            }
        } else {
            break;
        }
    }
    None
}
