mod codegen;
pub mod memory_layout;
mod stack;

use crate::pvm::Instruction;
use crate::{Error, Result, SpiProgram};
use wasmparser::{GlobalType, Parser, Payload};

pub use codegen::CompileContext;

const ENTRY_HEADER_SIZE: usize = 10;

/// Parsed WASM memory limits
#[derive(Debug, Clone, Copy)]
struct MemoryLimits {
    /// Initial memory size in 64KB pages
    initial_pages: u32,
    /// Maximum memory size in pages (None = no explicit limit)
    max_pages: Option<u32>,
}

impl Default for MemoryLimits {
    fn default() -> Self {
        // Default: 1 page initial, no max limit
        Self {
            initial_pages: 1,
            max_pages: None,
        }
    }
}

/// Represents an active data segment parsed from WASM
struct DataSegment {
    /// Offset in WASM linear memory (where the data goes)
    offset: u32,
    /// The actual data bytes
    data: Vec<u8>,
}

pub fn compile(wasm: &[u8]) -> Result<SpiProgram> {
    // Validate WASM module before attempting translation.
    // This catches malformed/invalid modules early with clear error messages.
    wasmparser::validate(wasm)
        .map_err(|e| Error::Internal(format!("WASM validation error: {e}")))?;

    let mut functions = Vec::new();
    let mut func_types: Vec<wasmparser::FuncType> = Vec::new();
    let mut function_type_indices = Vec::new();
    let mut globals: Vec<GlobalType> = Vec::new();
    let mut global_names: Vec<Option<String>> = Vec::new();
    let mut global_init_values: Vec<i32> = Vec::new();
    let mut main_func_idx: Option<u32> = None;
    let mut secondary_entry_func_idx: Option<u32> = None;
    let mut start_func_idx: Option<u32> = None;
    let mut tables: Vec<wasmparser::TableType> = Vec::new();
    let mut table_elements: Vec<(u32, u32, Vec<u32>)> = Vec::new();
    let mut data_segments: Vec<DataSegment> = Vec::new();
    let mut memory_limits = MemoryLimits::default();
    let mut num_imported_funcs: u32 = 0;
    let mut imported_func_type_indices: Vec<u32> = Vec::new();
    let mut imported_func_names: Vec<String> = Vec::new();

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
                        imported_func_names.push(import.name.to_string());
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
                    // Extract initial value from the const expression
                    let init_value = eval_const_i32(&g.init_expr)?;
                    global_init_values.push(init_value);
                }
            }
            Payload::StartSection { func, .. } => {
                start_func_idx = Some(func);
            }
            Payload::TableSection(reader) => {
                for table in reader {
                    tables.push(table?.ty);
                }
            }
            Payload::MemorySection(reader) => {
                // Parse the first memory (WASM MVP only supports one memory)
                if let Some(memory) = reader.into_iter().next() {
                    let mem = memory?;
                    memory_limits = MemoryLimits {
                        initial_pages: mem.initial as u32,
                        max_pages: mem.maximum.map(|m| m as u32),
                    };
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
                    match export.kind {
                        wasmparser::ExternalKind::Func => {
                            if export.name == "main" {
                                main_func_idx = Some(export.index);
                            } else if export.name == "main2" {
                                secondary_entry_func_idx = Some(export.index);
                            }
                        }
                        wasmparser::ExternalKind::Global => {
                            // Record global names from exports for result_ptr/result_len detection
                            let idx = export.index as usize;
                            if idx < global_names.len() {
                                global_names[idx] = Some(export.name.to_string());
                            }
                        }
                        _ => {}
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

    // main_func_idx is the global function index from exports
    // Convert to local function index by subtracting imported functions
    let main_func_idx = main_func_idx.map_or(0, |idx| idx as usize - num_imported_funcs as usize);

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

    // Detect if entry functions return (i32, i32) for the new entry point convention.
    // When an entry function returns two i32 values, they are treated as (result_ptr, result_len).
    let func_returns_ptr_len = |local_idx: usize| -> bool {
        function_type_indices
            .get(local_idx)
            .and_then(|&type_idx| func_types.get(type_idx as usize))
            .is_some_and(|ft| {
                ft.results().len() == 2
                    && ft.results()[0] == wasmparser::ValType::I32
                    && ft.results()[1] == wasmparser::ValType::I32
            })
    };
    let main_returns_ptr_len = func_returns_ptr_len(main_func_idx);

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

    // Compute WASM memory base dynamically to avoid overlap with spilled locals.
    // With many functions, the spilled locals region (0x40000 + N*512) can overlap
    // with the WASM memory region if it's at a fixed 0x50000.
    let wasm_memory_base = memory_layout::compute_wasm_memory_base(functions.len());

    // Calculate heap/memory info early since we need max_memory_pages for codegen.
    // The min floor of 1024 WASM pages is applied inside when no explicit max is declared.
    let (heap_pages, max_memory_pages) = calculate_heap_pages(
        functions.len(),
        &data_segments,
        &memory_limits,
        wasm_memory_base,
    )?;
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

    // Convert secondary entry from global to local function index
    let secondary_entry_idx_resolved = secondary_entry_func_idx.and_then(|idx| {
        idx.checked_sub(num_imported_funcs)
            .map(|v| v as usize)
            .or_else(|| {
                eprintln!(
                    "Warning: secondary entry function {idx} is an imported function, ignoring"
                );
                None
            })
    });

    // Convert start function from global to local function index
    let start_func_idx_resolved = start_func_idx.and_then(|idx| {
        idx.checked_sub(num_imported_funcs)
            .map(|v| v as usize)
            .or_else(|| {
                eprintln!("Warning: start function {idx} is an imported function, ignoring");
                None
            })
    });

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

        let entry_returns_ptr_len = if is_main {
            main_returns_ptr_len
        } else if is_secondary_entry {
            secondary_entry_idx_resolved.is_some_and(&func_returns_ptr_len)
        } else {
            false
        };

        let ctx = CompileContext {
            num_params,
            num_locals: 0,
            num_globals: globals.len(),
            result_ptr_global: if is_entry_func && !entry_returns_ptr_len {
                result_ptr_global
            } else {
                None
            },
            result_len_global: if is_entry_func && !entry_returns_ptr_len {
                result_len_global
            } else {
                None
            },
            is_main: is_entry_func,
            has_return,
            entry_returns_ptr_len,
            function_offsets: vec![],
            function_signatures: function_signatures.clone(),
            func_idx: global_func_idx,
            function_table: function_table.clone(),
            type_signatures: type_signatures.clone(),
            num_imported_funcs: num_imported_funcs as usize,
            imported_func_names: imported_func_names.clone(),
            stack_size: memory_layout::DEFAULT_STACK_SIZE,
            initial_memory_pages: memory_limits.initial_pages,
            max_memory_pages,
            wasm_memory_base,
        };

        // Record function start offset BEFORE any start function prologue injection.
        // This ensures the JUMP from entry header targets the prologue (which is a valid basic block start),
        // not the function body (which isn't a basic block start because the prologue doesn't end with a terminating instruction).
        let func_start_offset: usize = all_instructions.iter().map(|i| i.encode().len()).sum();
        function_offsets.push(func_start_offset);

        // If this is an entry function and we have a start function, execute the start function first.
        // We need to preserve r7 (args_ptr) and r8 (args_len) as they are used by main.
        // We use stack to save/restore registers.
        if let Some(start_local_idx) = start_func_idx_resolved.filter(|_| is_entry_func) {
            // Save r7 and r8 to stack
            // 1. Reserve 16 bytes on stack: sub r1, r1, 16
            all_instructions.push(Instruction::AddImm64 {
                dst: codegen::STACK_PTR_REG,
                src: codegen::STACK_PTR_REG,
                value: -16,
            });

            // 2. Store r7 (offset 0): store64 r7, [r1+0] -> StoreIndU64 { base: r1, src: r7, offset: 0 }
            all_instructions.push(Instruction::StoreIndU64 {
                base: codegen::STACK_PTR_REG,
                src: codegen::ARGS_PTR_REG,
                offset: 0,
            });

            // 3. Store r8 (offset 8): store64 r8, [r1+8] -> StoreIndU64 { base: r1, src: r8, offset: 8 }
            all_instructions.push(Instruction::StoreIndU64 {
                base: codegen::STACK_PTR_REG,
                src: codegen::ARGS_LEN_REG,
                offset: 8,
            });

            // Call start function
            let current_instr_idx = all_instructions.len();

            // loadimm64 r0, <placeholder>
            all_instructions.push(Instruction::LoadImm64 {
                reg: codegen::RETURN_ADDR_REG,
                value: 0,
            });

            // Jump to target
            all_instructions.push(Instruction::Jump { offset: 0 });

            // Fixup for this call
            let call_fixup = codegen::CallFixup {
                target_func: start_local_idx as u32,
                return_addr_instr: 0, // Relative to start of sequence
                jump_instr: 1,        // relative to start of sequence
            };

            // We register this fixup with base = current_instr_idx
            all_call_fixups.push((current_instr_idx, call_fixup));

            // Restore r7 and r8 from stack
            // 1. Load r7: load64 r7, [r1+0]
            all_instructions.push(Instruction::LoadIndU64 {
                dst: codegen::ARGS_PTR_REG,
                base: codegen::STACK_PTR_REG,
                offset: 0,
            });

            // 2. Load r8: load64 r8, [r1+8]
            all_instructions.push(Instruction::LoadIndU64 {
                dst: codegen::ARGS_LEN_REG,
                base: codegen::STACK_PTR_REG,
                offset: 8,
            });

            // 3. Restore stack: add r1, r1, 16
            all_instructions.push(Instruction::AddImm64 {
                dst: codegen::STACK_PTR_REG,
                src: codegen::STACK_PTR_REG,
                value: 16,
            });
        }

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

    // Build dispatch table for call_indirect.
    // Each entry is 8 bytes: 4 bytes jump address + 4 bytes type index (for signature validation).
    let mut ro_data = vec![0u8];
    if !function_table.is_empty() {
        ro_data.clear();
        for &func_idx in &function_table {
            if func_idx == u32::MAX {
                // Invalid entry - store u32::MAX for both jump address and type index
                ro_data.extend_from_slice(&u32::MAX.to_le_bytes()); // jump address
                ro_data.extend_from_slice(&u32::MAX.to_le_bytes()); // type index
            } else if (func_idx as usize) < num_imported_funcs as usize {
                // Imported function - can't be called via call_indirect in PVM
                ro_data.extend_from_slice(&u32::MAX.to_le_bytes()); // jump address
                ro_data.extend_from_slice(&u32::MAX.to_le_bytes()); // type index
            } else {
                // Valid local entry - convert WASM func_idx to local function index
                let local_func_idx = func_idx as usize - num_imported_funcs as usize;
                // Jump ref uses local_func_idx to index into jump table function entries
                let jump_ref = 2 * (func_entry_jump_table_base + local_func_idx + 1) as u32;
                ro_data.extend_from_slice(&jump_ref.to_le_bytes());
                // Get the type index for this local function
                let type_idx = *function_type_indices
                    .get(local_func_idx)
                    .unwrap_or(&u32::MAX);
                ro_data.extend_from_slice(&type_idx.to_le_bytes());
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
    // rw_data_section byte 0 goes to PVM address 0x30000
    // rw_data_section byte N goes to PVM address 0x30000 + N
    // WASM memory offset 0 should be at PVM address wasm_memory_base
    // So WASM data at offset X goes to rw_data_section byte (wasm_memory_base - 0x30000 + X)
    // (heap_pages and max_memory_pages were already calculated earlier)
    let _ = max_memory_pages; // silence unused variable warning

    let rw_data_section = build_rw_data(
        &data_segments,
        &global_init_values,
        memory_limits.initial_pages,
        wasm_memory_base,
    );

    Ok(SpiProgram::new(blob)
        .with_heap_pages(heap_pages)
        .with_ro_data(ro_data)
        .with_rw_data(rw_data_section))
}

/// Build the `rw_data` section from WASM data segments and global initializers.
///
/// The RW data segment in SPI is loaded at 0x30000. Layout:
/// - 0x30000+ : Global variables (4 bytes each, including compiler-managed memory size)
/// - `wasm_memory_base`+ : WASM linear memory (data segments)
///
/// WASM linear memory starts at `wasm_memory_base`.
/// So WASM offset X maps to `rw_data` offset (`wasm_memory_base` - 0x30000 + X).
///
/// The compiler-managed memory size global is stored at index `num_user_globals`,
/// i.e., right after all user-defined globals.
fn build_rw_data(
    data_segments: &[DataSegment],
    global_init_values: &[i32],
    initial_memory_pages: u32,
    wasm_memory_base: i32,
) -> Vec<u8> {
    // Calculate the minimum size needed for globals
    // +1 for the compiler-managed memory size global
    let globals_end = (global_init_values.len() + 1) * 4;

    // Calculate the size needed for data segments
    let wasm_to_rw_offset = wasm_memory_base as u32 - 0x30000;

    let data_end = data_segments
        .iter()
        .map(|seg| wasm_to_rw_offset + seg.offset + seg.data.len() as u32)
        .max()
        .unwrap_or(0) as usize;

    // Total size is the max of globals area and data segments
    let total_size = globals_end.max(data_end);

    if total_size == 0 {
        return Vec::new();
    }

    let mut rw_data = vec![0u8; total_size];

    // Initialize user globals at the start of rw_data (0x30000+)
    for (i, &value) in global_init_values.iter().enumerate() {
        let offset = i * 4;
        if offset + 4 <= rw_data.len() {
            rw_data[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
    }

    // Initialize compiler-managed memory size global (right after user globals)
    let mem_size_offset = global_init_values.len() * 4;
    if mem_size_offset + 4 <= rw_data.len() {
        rw_data[mem_size_offset..mem_size_offset + 4]
            .copy_from_slice(&initial_memory_pages.to_le_bytes());
    }

    // Copy data segments to their WASM memory locations
    for seg in data_segments {
        let rw_offset = (wasm_to_rw_offset + seg.offset) as usize;
        if rw_offset + seg.data.len() <= rw_data.len() {
            rw_data[rw_offset..rw_offset + seg.data.len()].copy_from_slice(&seg.data);
        }
    }

    rw_data
}

/// Calculate heap pages needed and the maximum memory pages available.
/// Returns (`heap_pages`, `max_memory_pages`).
fn calculate_heap_pages(
    num_functions: usize,
    data_segments: &[DataSegment],
    memory_limits: &MemoryLimits,
    wasm_memory_base: i32,
) -> Result<(u16, u32)> {
    // Memory layout:
    // 0x30000-0x300FF: Globals (256 bytes)
    // 0x30100-0x3FFFF: User heap (~64KB)
    // 0x40000+: Spilled locals (512 bytes per function)
    // wasm_memory_base+: WASM linear memory (data segments + dynamic allocation)
    //
    // The heap segment starts at 0x30000 (GLOBAL_MEMORY_BASE).
    // We need to allocate enough pages to cover:
    // 1. Spilled locals
    // 2. WASM linear memory (including data segments)
    let spilled_locals_end = memory_layout::SPILLED_LOCALS_BASE as usize
        + num_functions * memory_layout::SPILLED_LOCALS_PER_FUNC as usize;

    // Determine the maximum WASM memory pages we'll allow.
    // Respect the module's explicit max; only apply a floor when no max is declared.
    let default_max_pages: u32 = if data_segments.is_empty() { 256 } else { 1024 };
    let max_memory_pages = match memory_limits.max_pages {
        Some(declared_max) => declared_max,
        None => default_max_pages.max(1024),
    };

    // WASM memory end based on max pages allocation
    // Each WASM page is 64KB (65536 bytes)
    let wasm_memory_end = wasm_memory_base as usize + (max_memory_pages as usize) * 64 * 1024;

    let end = spilled_locals_end.max(wasm_memory_end);

    // Total bytes from heap base (0x30000) to end
    let total_bytes = end - 0x30000;
    let heap_pages = total_bytes.div_ceil(4096);

    // Ensure a minimum of 1024 PVM pages (4MB) for heap
    let heap_pages = heap_pages.max(1024);
    let heap_pages = u16::try_from(heap_pages).map_err(|_| {
        Error::Internal(format!(
            "heap size {heap_pages} pages exceeds u16::MAX ({}) — module too large",
            u16::MAX
        ))
    })?;
    Ok((heap_pages, max_memory_pages))
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
