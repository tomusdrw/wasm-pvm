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
    /// Initial values of global variables, captured as i64 to cover both
    /// `(global i32 ...)` and `(global i64 ...)`. Float/ref/v128 globals are
    /// either rejected outright or recorded as zero (see `Self::parse`).
    pub global_init_values: Vec<i64>,
    /// Byte width of each global's storage slot (4 for i32/f32, 8 for i64/f64).
    /// Parallels `globals` / `global_init_values`.
    pub global_widths: Vec<u32>,
    /// Absolute PVM address of each global's storage slot. Pre-computed at
    /// parse time so backend lowering doesn't need to re-sum widths per access.
    /// Parallels `globals` / `global_init_values` / `global_widths`.
    pub global_offsets: Vec<i32>,
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
    /// Optional display names for local functions, indexed by local function index.
    /// Populated from the WASM "name" custom section, falling back to export names.
    /// `None` means no name is known; callers should use a synthetic identifier.
    pub local_function_names: Vec<Option<String>>,

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
    /// Base address of WASM linear memory in PVM address space.
    pub wasm_memory_base: i32,
    /// Maximum WASM memory pages available for memory.grow.
    pub max_memory_pages: u32,
    /// Whether the module uses `memory.size`, `memory.grow`, or `memory.init`.
    /// These are the only ops that read/write the compiler-managed memory-size
    /// global, so if none of them appear we skip emitting that 4-byte slot.
    pub needs_memory_size_global: bool,
    /// Whether the compiler must reserve a parameter-overflow area in the PVM
    /// data region. True iff any type signature in the module has more than
    /// `MAX_LOCAL_REGS` (4) parameters — covering both local function
    /// declarations and `call_indirect` type annotations, since the caller
    /// writes args[4..] to the overflow area even when the caller's own
    /// function is low-arity. When false, the 256-byte overflow reservation
    /// is skipped and `wasm_memory_base` sits tight against the end of the
    /// globals/passive-length region (no 4KB alignment is applied; see
    /// `compute_wasm_memory_base` for the full layout rules).
    pub needs_param_overflow: bool,

    /// Libcall recognition metadata. Populated during parsing by scanning
    /// for compiler-builtins functions whose bodies the LLVM frontend can
    /// replace with hand-crafted PVM-friendly implementations (`__multi3`,
    /// `__udivti3`). `None` here means the function wasn't found or didn't
    /// match the expected shape — recognition will silently no-op in that
    /// case. See `llvm_frontend::libcall_recognition` for the full design.
    pub libcall_targets: LibcallTargets,
}

/// Information about compiler-builtins libcalls discovered during parsing.
///
/// Each field is the global function index (imports first, then locals) of
/// a recognized libcall. `None` indicates the libcall wasn't found or its
/// shape didn't match — the frontend then falls through to normal
/// translation of the original body.
#[derive(Debug, Clone, Default)]
pub struct LibcallTargets {
    /// Global function index of `__multi3`, if present.
    pub multi3: Option<usize>,
    /// Global function index of `__udivti3`, if present AND its single
    /// body-internal `Call` target is identified (`udivti3_slow_path`).
    pub udivti3: Option<usize>,
    /// Global function index of `__udivti3`'s slow-path callee — the
    /// function `__udivti3` tail-calls (compiler-builtins'
    /// `specialized_div_rem` or equivalent). The synthesized `__udivti3`
    /// body forwards to this for the non-u64/u64 case.
    ///
    /// We capture this *during parsing* by scanning `__udivti3`'s WASM
    /// operators for the single `Call` instruction in its body. Without
    /// this reference the frontend can't construct a fallback path and
    /// silently skips `__udivti3` recognition (it can still optimize
    /// `__multi3`).
    pub udivti3_slow_path: Option<usize>,
    /// WASM global index of the `__stack_pointer` global, captured by
    /// scanning `__udivti3` for its first `GlobalGet` operator. The
    /// original `__udivti3` allocates a 32-byte stack frame (because
    /// `specialized_div_rem` writes 32 bytes of `(quotient, remainder)`),
    /// reads/writes the quotient back, then restores the pointer. Our
    /// synthesized slow path replicates that frame setup, so it needs
    /// the global's index. `None` if scanning fails — recognition then
    /// silently no-ops for `__udivti3`.
    pub udivti3_stack_pointer_global: Option<usize>,
}

impl<'a> WasmModule<'a> {
    /// Display name for a local function: name-section entry, exported name, or
    /// the synthetic `wasm_func_<global_idx>` placeholder.
    #[must_use]
    pub fn local_function_display_name(&self, local_idx: usize) -> String {
        if let Some(Some(name)) = self.local_function_names.get(local_idx) {
            return name.clone();
        }
        let global_idx = self.num_imported_funcs as usize + local_idx;
        format!("wasm_func_{global_idx}")
    }

    /// Parse and validate a WASM binary, producing a `WasmModule` with all derived data.
    pub fn parse(wasm: &'a [u8]) -> Result<Self> {
        wasmparser::validate(wasm)
            .map_err(|e| Error::Internal(format!("WASM validation error: {e}")))?;

        let mut functions = Vec::new();
        let mut func_types: Vec<wasmparser::FuncType> = Vec::new();
        let mut function_type_indices = Vec::new();
        let mut globals: Vec<GlobalType> = Vec::new();
        let mut global_init_values: Vec<i64> = Vec::new();
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
        // Raw (global_func_idx, name) pairs collected from the WASM "name" custom
        // section. Resolved into `local_function_names` after parsing finishes,
        // since the name section may precede the import section in pathological
        // modules and `num_imported_funcs` may not yet be known.
        let mut name_section_entries: Vec<(u32, String)> = Vec::new();
        // First export name observed for each global function index (fallback when
        // the name section is absent).
        let mut export_name_by_global_idx: std::collections::BTreeMap<u32, String> =
            std::collections::BTreeMap::new();

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
                        // Only integer globals are supported. Float globals (f32/f64)
                        // would have their initial value silently zeroed by
                        // `eval_const_global_init` and could subsequently be observed
                        // via `i32.reinterpret_f32` / `i64.reinterpret_f64` or by
                        // forwarding the global into another function, producing
                        // wrong results — `--trap-floats` only traps float *operators*,
                        // not the integer-typed plumbing around float globals.
                        // v128 and ref-type globals have no lowering path at all.
                        match g.ty.content_type {
                            wasmparser::ValType::I32 | wasmparser::ValType::I64 => {}
                            other => {
                                return Err(Error::Unsupported(format!(
                                    "WASM global type {other:?} is not supported (only i32 and i64 globals are supported)"
                                )));
                            }
                        }
                        globals.push(g.ty);
                        let init_value = eval_const_global_init(&g.init_expr)?;
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
                            export_name_by_global_idx
                                .entry(export.index)
                                .or_insert_with(|| export.name.to_string());
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
                Payload::CustomSection(custom) => {
                    if let wasmparser::KnownCustom::Name(reader) = custom.as_known() {
                        for subsection in reader {
                            let subsection = subsection?;
                            if let wasmparser::Name::Function(map) = subsection {
                                for naming in map {
                                    let naming = naming?;
                                    name_section_entries
                                        .push((naming.index, naming.name.to_string()));
                                }
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

        // Reject signatures that would overflow the fixed 256-byte param
        // overflow window. Call lowering writes arg[i] at
        // `param_overflow_base + (i - MAX_LOCAL_REGS) * 8` without bounds
        // checking, so arity > MAX_TOTAL_PARAMS would corrupt the start of
        // WASM linear memory. Enforce the cap here rather than at the emit
        // site so both local-function and `call_indirect` type annotations
        // are covered uniformly.
        if let Some((idx, ft)) = func_types
            .iter()
            .enumerate()
            .find(|(_, ft)| ft.params().len() > memory_layout::MAX_TOTAL_PARAMS)
        {
            return Err(Error::Internal(format!(
                "type {idx} has {arity} params; maximum supported is {max} \
                (4 in registers + 32 in the {bytes}-byte param-overflow area)",
                arity = ft.params().len(),
                max = memory_layout::MAX_TOTAL_PARAMS,
                bytes = memory_layout::PARAM_OVERFLOW_SIZE,
            )));
        }

        // The parameter-overflow area is needed whenever any signature in the
        // module carries more than `MAX_LOCAL_REGS` params — both for local
        // function declarations (caller writes args[4..] before the call) *and*
        // for `call_indirect` type annotations (caller writes args[4..] even if
        // the caller function itself has ≤4 params). Scanning all types is a
        // conservative superset: it may reserve 256 bytes of overflow for a
        // module that declares a high-arity type it never calls, but avoids the
        // real correctness bug of writing into unreserved memory.
        let needs_param_overflow = func_types
            .iter()
            .any(|ft| ft.params().len() > crate::abi::MAX_LOCAL_REGS);

        // Compute per-global storage widths (4 B for i32/f32, 8 B for i64/f64),
        // then precompute absolute PVM addresses so backend lowering doesn't
        // need to re-sum widths per access.
        let global_widths: Vec<u32> = globals
            .iter()
            .map(|g| memory_layout::global_storage_width(g.content_type))
            .collect();
        let global_offsets =
            memory_layout::compute_global_offsets(&global_widths, needs_memory_size_global);

        // Compute WASM memory base
        let wasm_memory_base = memory_layout::compute_wasm_memory_base(
            &global_widths,
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

        // Resolve display names for local functions: prefer the name section,
        // fall back to the export section.
        let mut local_function_names: Vec<Option<String>> = vec![None; functions.len()];
        for (global_idx, name) in &name_section_entries {
            if let Some(local_idx) = (*global_idx).checked_sub(num_imported_funcs)
                && let Some(slot) = local_function_names.get_mut(local_idx as usize)
            {
                *slot = Some(name.clone());
            }
        }
        for (global_idx, name) in &export_name_by_global_idx {
            if let Some(local_idx) = (*global_idx).checked_sub(num_imported_funcs)
                && let Some(slot) = local_function_names.get_mut(local_idx as usize)
                && slot.is_none()
            {
                *slot = Some(name.clone());
            }
        }

        // Libcall recognition pre-scan: locate `__multi3` and `__udivti3` by
        // name, and for `__udivti3` capture the body-internal `Call` target
        // (compiler-builtins' `specialized_div_rem`) so the LLVM frontend can
        // emit a slow-path forward to it. Each function is recognized only if
        // its signature matches the expected libcall ABI (5 i64 params, no
        // return — both libcalls share this shape due to the C-style `sret`
        // convention).
        let libcall_targets = scan_libcall_targets(
            &local_function_names,
            &functions,
            &function_type_indices,
            &func_types,
            num_imported_funcs as usize,
        );

        Ok(WasmModule {
            functions,
            func_types,
            function_type_indices,
            globals,
            global_init_values,
            global_widths,
            global_offsets,
            data_segments,
            memory_limits,
            num_imported_funcs,
            imported_func_type_indices,
            imported_func_names,
            local_function_names,
            main_func_local_idx,
            has_secondary_entry,
            secondary_entry_local_idx,
            start_func_local_idx,
            function_signatures,
            type_signatures,
            function_table,
            wasm_memory_base,
            max_memory_pages,
            needs_memory_size_global,
            needs_param_overflow,
            libcall_targets,
        })
    }
}

/// Locate compiler-builtins libcalls by name in the parsed WASM module.
///
/// Returns global function indices (imports first, then locals) for each
/// recognized libcall. For `__udivti3`, also scans the body to find the
/// single `Call` instruction — the target is captured as the slow-path
/// fallback that the synthesized body will forward to for the non-u64/u64
/// case.
///
/// Each libcall is recognized only if both:
///   1. Its name matches in the name section / exports.
///   2. Its signature is exactly `(i32 sret, i64 a_lo, i64 a_hi, i64 b_lo,
///      i64 b_hi) -> void` — represented in our i64-uniform IR as 5 i64
///      params, no return. The signature gate prevents user functions
///      that happen to share a name (extremely unlikely — these names are
///      reserved by the C/Rust ABI) from being silently mis-translated.
///
/// If `__udivti3` is found but no `Call` instruction is in its body (e.g.
/// it was already inlined, replaced, or has an unexpected shape), the
/// `udivti3` field is set to `None` — recognition silently no-ops.
fn scan_libcall_targets(
    local_function_names: &[Option<String>],
    functions: &[FunctionBody<'_>],
    function_type_indices: &[u32],
    func_types: &[wasmparser::FuncType],
    num_imported_funcs: usize,
) -> LibcallTargets {
    let mut targets = LibcallTargets::default();

    for (local_idx, name) in local_function_names.iter().enumerate() {
        let Some(name) = name else { continue };
        if !has_libcall_signature(local_idx, function_type_indices, func_types) {
            continue;
        }
        let global_idx = num_imported_funcs + local_idx;
        match name.as_str() {
            "__multi3" => targets.multi3 = Some(global_idx),
            "__udivti3" => {
                // Capture the slow-path target by scanning `__udivti3`'s
                // body for its single `Call` operator. Compiler-builtins'
                // `__udivti3` body is the canonical thin wrapper:
                //   (alloc stack slot, call specialized_div_rem,
                //    copy result, return). The first `Call` is the
                //    specialized_div_rem we need to forward to.
                //
                // We also need the `__stack_pointer` global index — the
                // first `GlobalGet` in the body — so the synthesized
                // slow path can replicate the original frame setup that
                // `specialized_div_rem` requires (it writes 32 bytes,
                // callers only allocate 16 for the quotient).
                if let Some(body) = functions.get(local_idx) {
                    let slow_path = find_first_call_target(body);
                    let stack_ptr = find_first_global_get(body);
                    // Verify the slow-path callee has the same canonical
                    // WASM type as `__udivti3` (`(i32, i64, i64, i64, i64) → []`).
                    // Without this guard a weird-shaped input WASM (whose
                    // `__udivti3` happens to call some unrelated function
                    // first) would make the synthesized body call that
                    // function with mismatched argument types and fail
                    // LLVM `verify` later. Silently skip recognition in
                    // that case so compilation still succeeds with the
                    // original body.
                    let slow_path_ok = slow_path.is_some_and(|sp| {
                        let sp_local = (sp as usize).checked_sub(num_imported_funcs);
                        sp_local.is_some_and(|li| {
                            has_libcall_signature(li, function_type_indices, func_types)
                        })
                    });
                    if let (true, Some(stack_ptr), Some(slow_path)) =
                        (slow_path_ok, stack_ptr, slow_path)
                    {
                        targets.udivti3 = Some(global_idx);
                        targets.udivti3_slow_path = Some(slow_path as usize);
                        targets.udivti3_stack_pointer_global = Some(stack_ptr as usize);
                    }
                }
                // If any scan check fails we silently skip recognition.
                // There's no slow path to forward to safely, and binary long
                // division is too large to be a viable replacement (see
                // `docs/src/learnings.md`).
            }
            _ => {}
        }
    }

    targets
}

/// Check that a local function has the canonical compiler-builtins libcall
/// signature: `(i32 sret, i64 a_lo, i64 a_hi, i64 b_lo, i64 b_hi) -> []`.
///
/// Validating the exact raw WASM value types (not just arity / return-ness)
/// rules out user functions that happen to share `__multi3` / `__udivti3` /
/// `specialized_div_rem` names but have wholly different shapes — e.g. an
/// all-i64 5-param function. Those names are reserved by the C/Rust ABI
/// for this exact signature, so any legitimate caller emits exactly this
/// shape; we treat the canonical types as part of the recognition gate.
fn has_libcall_signature(
    local_idx: usize,
    function_type_indices: &[u32],
    func_types: &[wasmparser::FuncType],
) -> bool {
    use wasmparser::ValType;
    const EXPECTED_PARAMS: [ValType; 5] = [
        ValType::I32,
        ValType::I64,
        ValType::I64,
        ValType::I64,
        ValType::I64,
    ];

    let Some(&type_idx) = function_type_indices.get(local_idx) else {
        return false;
    };
    let Some(func_type) = func_types.get(type_idx as usize) else {
        return false;
    };
    func_type.params() == EXPECTED_PARAMS && func_type.results().is_empty()
}

/// Find the global function index that this body's first `Call` operator
/// targets. WASM's `call` is index-based and uses the global function
/// space (imports first, then local funcs); the returned index can be
/// dropped straight into a downstream lookup like `module.functions[idx]`.
///
/// Returns `None` if the body has no `Call` operator. This is the failure
/// mode for `__udivti3` recognition: a body without an internal call
/// means we can't construct a slow-path fallback.
fn find_first_call_target(body: &FunctionBody<'_>) -> Option<u32> {
    let reader = body.get_operators_reader().ok()?;
    for op in reader {
        let op = op.ok()?;
        if let wasmparser::Operator::Call { function_index } = op {
            return Some(function_index);
        }
    }
    None
}

/// Find the first `global.get` operator in the body and return its index.
/// Used to identify the `__stack_pointer` global from `__udivti3`'s
/// frame-setup prologue (the very first thing it does is read the stack
/// pointer). Returns `None` if no `global.get` appears.
fn find_first_global_get(body: &FunctionBody<'_>) -> Option<u32> {
    let reader = body.get_operators_reader().ok()?;
    for op in reader {
        let op = op.ok()?;
        if let wasmparser::Operator::GlobalGet { global_index } = op {
            return Some(global_index);
        }
    }
    None
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

/// Evaluate a global's initializer constant-expression. Only single-literal
/// `i32.const` / `i64.const` initializers are supported; legal-but-unsupported
/// init-expressions (e.g. `global.get` referencing an imported const global,
/// `ref.func`, `ref.null`, or extended-const arithmetic like
/// `i32.const 5; i32.const 7; i32.add`) error out instead of silently producing
/// the first literal and dropping the rest. wasmparser's `EXTENDED_CONST`
/// feature is on by default, so multi-operator const-exprs *can* reach this
/// function after validation — we have to inspect the trailing operator and
/// require `End`, not just return on the first literal.
fn eval_const_global_init(expr: &wasmparser::ConstExpr) -> Result<i64> {
    let mut reader = expr.get_binary_reader();
    if reader.eof() {
        // Empty const-expr; treat as zero (no value produced).
        return Ok(0);
    }
    let value = match reader.read_operator()? {
        wasmparser::Operator::I32Const { value } => i64::from(value),
        wasmparser::Operator::I64Const { value } => value,
        // `End` alone produces no value — zero is the only sensible default.
        wasmparser::Operator::End => return Ok(0),
        other => {
            return Err(Error::Unsupported(format!(
                "unsupported global init expression: {other:?} (only i32.const and i64.const literals are supported)"
            )));
        }
    };
    // Reject extended-const-expression tails (e.g. an `i32.add` after a literal
    // pair) instead of silently using only the first literal. The const-expr
    // must consist of exactly one literal followed by `End`.
    match reader.read_operator()? {
        wasmparser::Operator::End => Ok(value),
        other => Err(Error::Unsupported(format!(
            "unsupported global init expression: trailing operator {other:?} after literal (only single-literal i32.const/i64.const initializers are supported)"
        ))),
    }
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
    fn param_overflow_reserved_for_high_arity_indirect_call() {
        // Regression: a module whose only local function has ≤4 params but
        // performs `call_indirect` through a type with >4 params must reserve
        // the param-overflow area. Before the fix, the scan only inspected
        // local function declarations (via `function_type_indices`) and
        // missed the indirect-call type, causing `lower_pvm_call_indirect` to
        // write args[4..] into unreserved memory in release builds (and
        // panic the `debug_assert!` at calls.rs:505 in debug builds).
        let wasm = wat::parse_str(
            r#"(module
                (type $narrow (func (param i32 i32) (result i64)))
                (type $wide (func (param i32 i32 i32 i32 i32) (result i32)))
                (table 1 funcref)
                (func $main (export "main") (type $narrow)
                    (drop (call_indirect (type $wide)
                        (i32.const 0) (i32.const 0) (i32.const 0)
                        (i32.const 0) (i32.const 0) (i32.const 0)))
                    (i64.const 0))
            )"#,
        )
        .expect("valid WAT");
        let module = WasmModule::parse(&wasm).expect("valid module");

        // Sanity: the only local function has 2 params (≤ MAX_LOCAL_REGS).
        let max_local_arity = module
            .function_type_indices
            .iter()
            .map(|&ti| module.func_types[ti as usize].params().len())
            .max()
            .unwrap_or(0);
        assert!(
            max_local_arity <= crate::abi::MAX_LOCAL_REGS,
            "fixture invariant: no local function should be high-arity"
        );

        // The flag must still be set because the indirect-call type is wide.
        assert!(
            module.needs_param_overflow,
            "high-arity call_indirect type must force overflow reservation"
        );
    }

    #[test]
    fn rejects_signature_exceeding_overflow_capacity() {
        // A signature with MAX_TOTAL_PARAMS + 1 params cannot be lowered: call
        // sites would write arg[MAX_TOTAL_PARAMS] at offset
        // `(MAX_TOTAL_PARAMS - MAX_LOCAL_REGS) * 8 = 256` in the overflow
        // window, stomping the first 8 bytes of WASM linear memory.
        let params: String = (0..super::memory_layout::MAX_TOTAL_PARAMS + 1)
            .map(|_| " i32")
            .collect();
        let wasm = wat::parse_str(format!(
            r#"(module
                (type $oversized (func (param{params}) (result i32)))
                (func $main (export "main") (param i32 i32) (result i64)
                    (i64.const 0))
            )"#
        ))
        .expect("valid WAT");

        match WasmModule::parse(&wasm) {
            Ok(_) => panic!("must reject oversized signature"),
            Err(crate::Error::Internal(msg)) => {
                assert!(
                    msg.contains("maximum supported"),
                    "unexpected error message: {msg}"
                );
            }
            Err(err) => panic!("unexpected error: {err}"),
        }
    }

    #[test]
    fn accepts_signature_at_overflow_capacity_boundary() {
        // Boundary: MAX_TOTAL_PARAMS (36) params is allowed — fills exactly
        // the 4 registers + 32 overflow slots.
        let params: String = (0..super::memory_layout::MAX_TOTAL_PARAMS)
            .map(|_| " i32")
            .collect();
        let wasm = wat::parse_str(format!(
            r#"(module
                (type $maximal (func (param{params}) (result i32)))
                (func $main (export "main") (param i32 i32) (result i64)
                    (i64.const 0))
            )"#
        ))
        .expect("valid WAT");

        let module = WasmModule::parse(&wasm).expect("boundary arity must parse");
        assert!(module.needs_param_overflow);
    }

    #[test]
    fn display_name_uses_name_section_entry() {
        // `$identifier` in WAT becomes a name-section entry; that wins over
        // the export alias.
        let wasm = wat::parse_str(
            r#"(module
                (func $canonical (export "main") (param i32 i32) (result i64) (i64.const 0))
            )"#,
        )
        .expect("valid WAT");
        let module = WasmModule::parse(&wasm).expect("valid module");
        assert_eq!(module.local_function_display_name(0), "canonical");
    }

    #[test]
    fn display_name_falls_back_to_export_name() {
        // No `$identifier` → no name-section entry → display falls back to the
        // export name.
        let wasm = wat::parse_str(
            r#"(module
                (func (export "main") (param i32 i32) (result i64) (i64.const 0))
                (func (export "helper"))
            )"#,
        )
        .expect("valid WAT");
        let module = WasmModule::parse(&wasm).expect("valid module");
        assert_eq!(module.local_function_display_name(0), "main");
        assert_eq!(module.local_function_display_name(1), "helper");
    }

    #[test]
    fn display_name_synthetic_when_unexported_and_no_name_section() {
        // Second function has no export and no `$identifier` → both fallback
        // sources are empty → synthetic `wasm_func_<global_idx>` name.
        let wasm = wat::parse_str(
            r#"(module
                (func (export "main") (param i32 i32) (result i64) (i64.const 0))
                (func)
            )"#,
        )
        .expect("valid WAT");
        let module = WasmModule::parse(&wasm).expect("valid module");
        // No imports, so local_idx 1 → global_idx 1 → "wasm_func_1".
        assert_eq!(module.local_function_display_name(1), "wasm_func_1");
    }

    #[test]
    fn display_name_out_of_bounds_returns_synthetic_no_panic() {
        let wasm = wat::parse_str(
            r#"(module (func (export "main") (param i32 i32) (result i64) (i64.const 0)))"#,
        )
        .expect("valid WAT");
        let module = WasmModule::parse(&wasm).expect("valid module");
        // Past the end of `local_function_names` must fall through to the
        // synthetic formatter rather than panicking.
        assert_eq!(module.local_function_display_name(99), "wasm_func_99");
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
