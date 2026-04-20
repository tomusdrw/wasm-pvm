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
pub(crate) const MIN_INITIAL_WASM_PAGES: u32 = 16;

/// Default `memory.grow` ceiling when the WASM module doesn't declare a maximum.
/// 16 WASM pages = 1 MB — conservative default aligned with PVM recommendations.
const DEFAULT_MAX_PAGES: u32 = 16;

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
    /// (`num_params`, `has_return`) for each function (imports first, then locals).
    pub function_signatures: Vec<(usize, bool)>,
    /// (`num_params`, `num_results`) for each type.
    pub type_signatures: Vec<(usize, usize)>,
    /// Function table for indirect calls (`u32::MAX` = invalid entry).
    pub function_table: Vec<u32>,
    /// WASM global indices of all exported functions.
    pub exported_wasm_func_indices: Vec<u32>,
    /// Base address of WASM linear memory in PVM address space.
    pub wasm_memory_base: i32,
    /// Maximum WASM memory pages available for memory.grow.
    pub max_memory_pages: u32,
    /// Whether the module uses `memory.size`, `memory.grow`, or `memory.init`.
    /// These are the only ops that read/write the compiler-managed memory-size
    /// global, so if none of them appear we skip emitting that 4-byte slot.
    pub needs_memory_size_global: bool,
    /// Whether the compiler must reserve a parameter-overflow area in the PVM
    /// data region. True iff any local function type has more than
    /// `MAX_LOCAL_REGS` (4) parameters; callers spill the 5th+ args there and
    /// callees read them back in the prologue. When false, the 256-byte
    /// overflow reservation (and the 4KB page alignment that follows) can be
    /// skipped, letting `wasm_memory_base` sit at `GLOBAL_MEMORY_BASE` itself.
    pub needs_param_overflow: bool,
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
        let mut global_init_values: Vec<i32> = Vec::new();
        let mut main_func_idx: Option<u32> = None;
        let mut secondary_entry_func_idx: Option<u32> = None;
        let mut start_func_idx: Option<u32> = None;
        let mut exported_wasm_func_indices: Vec<u32> = Vec::new();
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
                        if export.kind == wasmparser::ExternalKind::Func {
                            exported_wasm_func_indices.push(export.index);
                            let is_imported = export.index < num_imported_funcs;
                            let is_main_name = matches!(
                                export.name,
                                "main"
                                    | "refine"
                                    | "refine_ext"
                                    | "is_authorized"
                                    | "is_authorized_ext"
                            );
                            let is_secondary_name =
                                matches!(export.name, "main2" | "accumulate" | "accumulate_ext");
                            if is_imported && (is_main_name || is_secondary_name) {
                                return Err(Error::Internal(format!(
                                    "Entry export '{}' refers to imported function index {}",
                                    export.name, export.index
                                )));
                            }
                            match export.name {
                                "main" => {
                                    main_func_idx = Some(export.index);
                                }
                                "refine" | "refine_ext" | "is_authorized" | "is_authorized_ext"
                                    if main_func_idx.is_none() =>
                                {
                                    main_func_idx = Some(export.index);
                                }
                                "main2" => {
                                    secondary_entry_func_idx = Some(export.index);
                                }
                                "accumulate" | "accumulate_ext"
                                    if secondary_entry_func_idx.is_none() =>
                                {
                                    secondary_entry_func_idx = Some(export.index);
                                }
                                _ => {}
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

        let needs_memory_size_global = scan_needs_memory_size_global(&functions)?;
        let needs_param_overflow =
            function_type_indices
                .iter()
                .any(|&type_idx| match func_types.get(type_idx as usize) {
                    Some(ft) => ft.params().len() > crate::abi::MAX_LOCAL_REGS,
                    None => false,
                });

        // Compute WASM memory base
        let wasm_memory_base = memory_layout::compute_wasm_memory_base(
            globals.len(),
            num_passive_segments,
            needs_memory_size_global,
            needs_param_overflow,
        );

        // max_memory_pages is the runtime limit for memory.grow (hardcoded in PVM code).
        // When the WASM module doesn't declare a max, use DEFAULT_MAX_PAGES (1 MB).
        // When it does, respect its preference (but warn in CLI output if large).
        let max_memory_pages = memory_limits
            .max_pages
            .unwrap_or(DEFAULT_MAX_PAGES)
            .max(memory_limits.initial_pages);

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
            function_signatures,
            type_signatures,
            function_table,
            exported_wasm_func_indices,
            wasm_memory_base,
            max_memory_pages,
            needs_memory_size_global,
            needs_param_overflow,
        })
    }
}

/// Scan function bodies for any operator that reads/writes the compiler-managed
/// memory-size global (`memory.size`, `memory.grow`, `memory.init`).
fn scan_needs_memory_size_global(functions: &[FunctionBody<'_>]) -> Result<bool> {
    for body in functions {
        let mut reader = body
            .get_operators_reader()
            .map_err(|e| Error::Internal(format!("operator reader: {e}")))?;
        while !reader.eof() {
            match reader
                .read()
                .map_err(|e| Error::Internal(format!("operator read: {e}")))?
            {
                wasmparser::Operator::MemorySize { .. }
                | wasmparser::Operator::MemoryGrow { .. }
                | wasmparser::Operator::MemoryInit { .. } => return Ok(true),
                _ => {}
            }
        }
    }
    Ok(false)
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

#[cfg(test)]
mod tests {
    use super::WasmModule;

    #[test]
    fn main_export_name_overrides_alias() {
        let wasm = wat::parse_str(
            r#"(module
                (func $canonical_main (export "main"))
                (func $alias_main (export "refine"))
            )"#,
        )
        .expect("valid WAT");
        let module = WasmModule::parse(&wasm).expect("valid module");

        assert_eq!(module.main_func_local_idx, 0);
    }

    #[test]
    fn secondary_main2_export_name_overrides_alias() {
        let wasm = wat::parse_str(
            r#"(module
                (func $main (export "main"))
                (func $canonical_secondary (export "main2"))
                (func $alias_secondary (export "accumulate_ext"))
            )"#,
        )
        .expect("valid WAT");
        let module = WasmModule::parse(&wasm).expect("valid module");

        assert!(module.has_secondary_entry);
        assert_eq!(module.secondary_entry_local_idx, Some(1));
    }

    #[test]
    fn reverse_main_export_name_overrides_alias() {
        let wasm = wat::parse_str(
            r#"(module
                (func $canonical_main)
                (func $alias_main)
                (export "refine" (func $alias_main))
                (export "main" (func $canonical_main))
            )"#,
        )
        .expect("valid WAT");
        let module = WasmModule::parse(&wasm).expect("valid module");

        assert_eq!(module.main_func_local_idx, 0);
    }

    #[test]
    fn reverse_secondary_main2_export_name_overrides_alias() {
        let wasm = wat::parse_str(
            r#"(module
                (func $main (export "main"))
                (func $canonical_secondary)
                (func $alias_secondary)
                (export "accumulate_ext" (func $alias_secondary))
                (export "main2" (func $canonical_secondary))
            )"#,
        )
        .expect("valid WAT");
        let module = WasmModule::parse(&wasm).expect("valid module");

        assert!(module.has_secondary_entry);
        assert_eq!(module.secondary_entry_local_idx, Some(1));
    }

    #[test]
    fn imported_entry_export_returns_error() {
        let wasm = wat::parse_str(
            r#"(module
                (import "env" "main_import" (func $main_import))
                (func $local_main)
                (export "main" (func $main_import))
            )"#,
        )
        .expect("valid WAT");

        match WasmModule::parse(&wasm) {
            Ok(_) => panic!("must reject imported main export"),
            Err(crate::Error::Internal(msg)) => {
                assert!(
                    msg.contains("imported function index"),
                    "unexpected error message: {msg}"
                );
            }
            Err(err) => panic!("unexpected error: {err}"),
        }
    }
}
