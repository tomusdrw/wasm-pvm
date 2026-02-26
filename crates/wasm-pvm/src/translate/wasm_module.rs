// Parsing code uses casts to convert WASM u64 fields to PVM u32/usize types.
#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use crate::{Error, Result};
use wasmparser::{FunctionBody, GlobalType, Parser, Payload};

use super::memory_layout;

/// Parsed WASM memory limits.
#[derive(Debug, Clone, Copy)]
pub struct MemoryLimits {
    /// Initial memory size in 64KB pages.
    pub initial_pages: u32,
    /// Maximum memory size in pages (None = no explicit limit).
    pub max_pages: Option<u32>,
}

impl Default for MemoryLimits {
    fn default() -> Self {
        Self {
            initial_pages: 1,
            max_pages: None,
        }
    }
}

/// Minimum WASM pages (64KB each) to pre-allocate for programs declaring (memory 0).
/// 16 pages = 1MB, sufficient for `AssemblyScript` programs compiled with --runtime stub.
const MIN_INITIAL_WASM_PAGES: u32 = 16;

/// Represents a data segment parsed from WASM.
pub struct DataSegment {
    /// Offset in WASM linear memory (active only). None for passive segments.
    pub offset: Option<u32>,
    /// The actual data bytes.
    pub data: Vec<u8>,
}

/// Parsed and pre-processed WASM module, usable by both legacy and LLVM pipelines.
pub struct WasmModule<'a> {
    // --- Raw parsed section data ---
    /// Function bodies from the code section.
    pub functions: Vec<FunctionBody<'a>>,
    /// All function types declared in the type section.
    pub func_types: Vec<wasmparser::FuncType>,
    /// Type index for each local function (parallels `functions`).
    pub function_type_indices: Vec<u32>,
    /// Global variable types.
    pub globals: Vec<GlobalType>,
    /// Initial values of global variables.
    pub global_init_values: Vec<i32>,
    /// Active data segments from the data section.
    pub data_segments: Vec<DataSegment>,
    /// Memory limits parsed from the memory section.
    pub memory_limits: MemoryLimits,
    /// Number of imported functions (precede local functions in global index space).
    pub num_imported_funcs: u32,
    /// Type indices for imported functions.
    pub imported_func_type_indices: Vec<u32>,
    /// Names of imported functions.
    pub imported_func_names: Vec<String>,

    // --- Derived data ---
    /// Local function index of the main entry point.
    pub main_func_local_idx: usize,
    /// Whether the WASM module exports a "main2" secondary entry point.
    pub has_secondary_entry: bool,
    /// Local function index of the secondary entry point (None if import or absent).
    pub secondary_entry_local_idx: Option<usize>,
    /// Local function index of the start function (None if import or absent).
    pub start_func_local_idx: Option<usize>,
    /// Whether the main entry function returns (i32, i32) for (ptr, len) convention.
    pub main_returns_ptr_len: bool,
    /// Whether the secondary entry function returns (i32, i32) for (ptr, len) convention.
    pub secondary_returns_ptr_len: bool,
    /// Global index for `result_ptr` (legacy entry convention, None if using ptr/len returns).
    pub result_ptr_global: Option<u32>,
    /// Global index for `result_len` (legacy entry convention, None if using ptr/len returns).
    pub result_len_global: Option<u32>,
    /// (`num_params`, `has_return`) for each function (imports first, then locals).
    pub function_signatures: Vec<(usize, bool)>,
    /// (`num_params`, `num_results`) for each type.
    pub type_signatures: Vec<(usize, usize)>,
    /// Function table for indirect calls (`u32::MAX` = invalid entry).
    pub function_table: Vec<u32>,
    /// Base address of WASM linear memory in PVM address space.
    pub wasm_memory_base: i32,
    /// Number of 4KB PVM heap pages needed.
    pub heap_pages: u16,
    /// Maximum WASM memory pages available for memory.grow.
    pub max_memory_pages: u32,
}

impl<'a> WasmModule<'a> {
    /// Parse and validate a WASM binary, producing a `WasmModule` with all derived data.
    pub fn parse(wasm: &'a [u8]) -> Result<Self> {
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
                        match data.kind {
                            wasmparser::DataKind::Active {
                                memory_index: _,
                                offset_expr,
                            } => {
                                let offset = eval_const_i32(&offset_expr)? as u32;
                                data_segments.push(DataSegment {
                                    offset: Some(offset),
                                    data: data.data.to_vec(),
                                });
                            }
                            wasmparser::DataKind::Passive => {
                                data_segments.push(DataSegment {
                                    offset: None,
                                    data: data.data.to_vec(),
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if functions.is_empty() {
            return Err(Error::NoExportedFunction);
        }

        // Convert main_func_idx from global to local function index
        let main_func_local_idx = if let Some(idx) = main_func_idx {
            idx as usize - num_imported_funcs as usize
        } else {
            tracing::warn!("No 'main' export found, defaulting to first local function");
            0
        };

        // Detect result_ptr / result_len globals (legacy entry convention)
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

        // Detect if entry functions return (i32, i32) for the new entry point convention
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
        let main_returns_ptr_len = func_returns_ptr_len(main_func_local_idx);

        // Resolve secondary entry from global to local function index
        let has_secondary_entry = secondary_entry_func_idx.is_some();
        let secondary_entry_local_idx = secondary_entry_func_idx.and_then(|idx| {
            idx.checked_sub(num_imported_funcs)
                .map(|v| v as usize)
                .or_else(|| {
                    tracing::warn!(
                        "secondary entry function {idx} is an imported function, ignoring"
                    );
                    None
                })
        });
        let secondary_returns_ptr_len =
            secondary_entry_local_idx.is_some_and(&func_returns_ptr_len);

        // Resolve start function from global to local function index
        let start_func_local_idx = start_func_idx.and_then(|idx| {
            idx.checked_sub(num_imported_funcs)
                .map(|v| v as usize)
                .or_else(|| {
                    tracing::warn!("start function {idx} is an imported function, ignoring");
                    None
                })
        });

        // Build function signatures: (num_params, has_return) indexed by global function index
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

        // Build type signatures: (num_params, num_results) for each type
        let type_signatures: Vec<(usize, usize)> = func_types
            .iter()
            .map(|f| (f.params().len(), f.results().len()))
            .collect();

        // Build function table from element sections
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

        let num_passive_segments = data_segments
            .iter()
            .filter(|seg| seg.offset.is_none())
            .count();
        // Validate that globals fit within the reserved window
        memory_layout::validate_globals_layout(globals.len(), num_passive_segments)
            .map_err(Error::Internal)?;

        // Compute WASM memory base
        let wasm_memory_base = memory_layout::compute_wasm_memory_base(
            functions.len(),
            globals.len(),
            num_passive_segments,
        );

        // Calculate heap and memory pages
        let (heap_pages, max_memory_pages) = calculate_heap_pages(
            functions.len(),
            &data_segments,
            &memory_limits,
            wasm_memory_base,
        )?;

        Ok(WasmModule {
            functions,
            func_types,
            function_type_indices,
            globals,
            global_init_values,
            data_segments,
            memory_limits,
            num_imported_funcs,
            imported_func_type_indices,
            imported_func_names,
            main_func_local_idx,
            has_secondary_entry,
            secondary_entry_local_idx,
            start_func_local_idx,
            main_returns_ptr_len,
            secondary_returns_ptr_len,
            result_ptr_global,
            result_len_global,
            function_signatures,
            type_signatures,
            function_table,
            wasm_memory_base,
            heap_pages,
            max_memory_pages,
        })
    }
}

/// Calculate heap pages needed and the maximum memory pages available.
/// Returns (`heap_pages`, `max_memory_pages`).
///
/// `heap_pages` determines how many zero-initialized writable pages are pre-allocated
/// in the SPI output. This covers the initial WASM linear memory, globals, and spilled
/// locals. Programs that need more memory at runtime can grow via `memory.grow` / `sbrk`,
/// which allocates pages on demand up to `max_memory_pages`.
fn calculate_heap_pages(
    num_functions: usize,
    data_segments: &[DataSegment],
    memory_limits: &MemoryLimits,
    wasm_memory_base: i32,
) -> Result<(u16, u32)> {
    let spilled_locals_end = memory_layout::SPILLED_LOCALS_BASE as usize
        + num_functions * memory_layout::SPILLED_LOCALS_PER_FUNC as usize;

    // max_memory_pages is the runtime limit for memory.grow (hardcoded in PVM code).
    let default_max_pages: u32 = if data_segments.is_empty() { 256 } else { 1024 };
    let max_memory_pages = match memory_limits.max_pages {
        Some(declared_max) => declared_max,
        None => default_max_pages,
    };

    // heap_pages uses initial_pages (not max) — only pre-allocate what the program
    // needs at startup. Additional memory is allocated on demand via sbrk/memory.grow.
    // We enforce a minimum of MIN_INITIAL_WASM_PAGES (16 pages = 1MB) because many
    // programs (especially AssemblyScript with --runtime stub) access memory without
    // calling memory.grow first. wasm_memory_base is 4KB-aligned (PVM page size);
    // the 64KB WASM page size only governs memory.grow granularity, not the base address.
    let initial_pages = memory_limits.initial_pages.max(MIN_INITIAL_WASM_PAGES);
    let wasm_memory_initial_end = wasm_memory_base as usize + (initial_pages as usize) * 64 * 1024;

    let end = spilled_locals_end.max(wasm_memory_initial_end);
    let total_bytes = end - 0x30000;
    // SPI heap_pages represents zero-init pages allocated AFTER rw_data. Since
    // build_rw_data() trims trailing zeros, the rw_data blob may not fully cover
    // the gap from globals_end to wasm_memory_base. The zero pages must fill both
    // the potentially-trimmed gap AND the full WASM heap. We add 1 WASM page
    // (16 PVM pages = 64KB) of headroom to account for rw_data trimming. This
    // doesn't increase JAM file size (heap_pages is just a 2-byte header field),
    // it only tells the runtime to pre-allocate more zero memory.
    const HEAP_PAGE_HEADROOM: usize = 16; // 1 WASM page in PVM pages
    let heap_pages = total_bytes.div_ceil(4096) + HEAP_PAGE_HEADROOM;
    let heap_pages = u16::try_from(heap_pages).map_err(|_| {
        Error::Internal(format!(
            "heap size {heap_pages} pages exceeds u16::MAX ({}) — module too large",
            u16::MAX
        ))
    })?;
    Ok((heap_pages, max_memory_pages))
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
