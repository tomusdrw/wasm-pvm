// WASM-level module merge: adapter WAT functions replace resolved imports.
//
// Given a main WASM module and an adapter WAT module, this produces a merged
// WASM binary where adapter exports that match main imports are inlined as
// local functions, and the matched imports are removed.
//
// Merged function index space:
//   [0..R)           = retained imports (main imports NOT resolved by adapter)
//   [R..R+A)         = adapter local functions
//   [R+A..R+A+M)     = main local functions
//
// Both adapter and main function bodies get index remapping via wasm-encoder's
// reencode module.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use std::collections::HashMap;

use crate::{Error, Result};

/// Merge an adapter WAT module into a main WASM module.
///
/// Adapter exports whose names match main imports replace those imports.
/// The adapter's own imports are carried through as retained imports.
/// Returns the merged WASM binary.
pub fn merge_adapter(main_wasm: &[u8], adapter_wat: &str) -> Result<Vec<u8>> {
    let adapter_wasm = wat::parse_str(adapter_wat)
        .map_err(|e| Error::Internal(format!("adapter WAT parse error: {e}")))?;

    let main = ParsedModule::parse(main_wasm, "main")?;
    let adapter = ParsedModule::parse(&adapter_wasm, "adapter")?;

    // Build resolution map: adapter export name -> adapter local func index.
    let mut adapter_export_map: HashMap<&str, u32> = HashMap::new();
    for (name, func_idx) in &adapter.func_exports {
        // Export index is global (imports + locals). Convert to local index.
        if *func_idx >= adapter.num_imported_funcs {
            adapter_export_map.insert(name, *func_idx - adapter.num_imported_funcs);
        }
    }

    // Determine which main imports are resolved by adapter exports.
    // resolved_main_imports[i] = Some(adapter_local_func_idx) if resolved.
    let mut resolved_main_imports: Vec<Option<u32>> = vec![None; main.num_imported_funcs as usize];
    for (i, imp) in main.func_imports.iter().enumerate() {
        if let Some(&adapter_local_idx) = adapter_export_map.get(imp.name.as_str()) {
            // Validate type signatures match.
            let main_type = main.types.get(imp.type_idx as usize).ok_or_else(|| {
                Error::Internal(format!(
                    "main import type index {} out of range",
                    imp.type_idx
                ))
            })?;
            let adapter_global_idx = adapter.num_imported_funcs + adapter_local_idx;
            let adapter_type_idx = adapter
                .func_type_indices
                .get(adapter_local_idx as usize)
                .ok_or_else(|| {
                    Error::Internal(format!(
                        "adapter local func {adapter_local_idx} type index out of range",
                    ))
                })?;
            let adapter_type = adapter.types.get(*adapter_type_idx as usize).ok_or_else(|| {
                Error::Internal(format!(
                    "adapter type index {adapter_type_idx} out of range for func {adapter_global_idx}",
                ))
            })?;

            if main_type.params != adapter_type.params || main_type.results != adapter_type.results
            {
                return Err(Error::Internal(format!(
                    "type mismatch for import '{}': main expects {:?} -> {:?}, adapter provides {:?} -> {:?}",
                    imp.name,
                    main_type.params,
                    main_type.results,
                    adapter_type.params,
                    adapter_type.results
                )));
            }

            resolved_main_imports[i] = Some(adapter_local_idx);
        }
    }

    // Build the merged module using wasm-encoder.
    build_merged_module(&main, &adapter, &resolved_main_imports)
}

/// Lightweight parsed representation of a WASM module for merging.
struct ParsedModule<'a> {
    types: Vec<FuncSig>,
    func_imports: Vec<FuncImport>,
    num_imported_funcs: u32,
    func_type_indices: Vec<u32>,
    func_exports: Vec<(String, u32)>,
    /// Raw function bodies (code section entries).
    code_bodies: Vec<RawFuncBody<'a>>,
    /// Global section raw bytes (for re-encoding).
    globals: Vec<RawGlobal<'a>>,
    /// Memory section.
    memories: Vec<wasmparser::MemoryType>,
    /// Table section.
    tables: Vec<wasmparser::TableType>,
    /// Element section raw entries.
    elements: Vec<RawElement<'a>>,
    /// Data section raw entries.
    data_segments: Vec<RawData<'a>>,
    /// Start function index (if any).
    start_func: Option<u32>,
    /// Non-function exports.
    other_exports: Vec<RawExport>,
    /// Non-function imports.
    other_imports: Vec<RawOtherImport>,
}

#[derive(Clone, Debug)]
struct FuncSig {
    params: Vec<wasmparser::ValType>,
    results: Vec<wasmparser::ValType>,
}

#[derive(Clone, Debug)]
struct FuncImport {
    module: String,
    name: String,
    type_idx: u32,
}

struct RawFuncBody<'a> {
    /// The raw bytes of the function body (locals + code).
    locals: Vec<(u32, wasmparser::ValType)>,
    /// Raw operators bytes for re-encoding.
    body_reader: wasmparser::OperatorsReader<'a>,
}

#[derive(Clone)]
struct RawGlobal<'a> {
    ty: wasmparser::GlobalType,
    init_expr: wasmparser::ConstExpr<'a>,
}

struct RawElement<'a> {
    raw: wasmparser::Element<'a>,
}

struct RawData<'a> {
    raw: wasmparser::Data<'a>,
}

struct RawExport {
    name: String,
    kind: wasmparser::ExternalKind,
    index: u32,
}

struct RawOtherImport {
    module: String,
    name: String,
    ty: wasmparser::TypeRef,
}

impl<'a> ParsedModule<'a> {
    fn parse(wasm: &'a [u8], label: &str) -> Result<Self> {
        let mut types = Vec::new();
        let mut func_imports = Vec::new();
        let mut other_imports = Vec::new();
        let mut num_imported_funcs: u32 = 0;
        let mut func_type_indices = Vec::new();
        let mut func_exports = Vec::new();
        let mut other_exports = Vec::new();
        let mut code_bodies = Vec::new();
        let mut globals = Vec::new();
        let mut memories = Vec::new();
        let mut tables = Vec::new();
        let mut elements = Vec::new();
        let mut data_segments = Vec::new();
        let mut start_func = None;

        for payload in wasmparser::Parser::new(0).parse_all(wasm) {
            match payload.map_err(|e| Error::Internal(format!("{label} parse error: {e}")))? {
                wasmparser::Payload::TypeSection(reader) => {
                    for rec_group in reader {
                        let rec_group =
                            rec_group.map_err(|e| Error::Internal(format!("{label} type: {e}")))?;
                        for sub_type in rec_group.into_types() {
                            if let wasmparser::CompositeInnerType::Func(f) =
                                &sub_type.composite_type.inner
                            {
                                types.push(FuncSig {
                                    params: f.params().to_vec(),
                                    results: f.results().to_vec(),
                                });
                            }
                        }
                    }
                }
                wasmparser::Payload::ImportSection(reader) => {
                    for import in reader {
                        let import =
                            import.map_err(|e| Error::Internal(format!("{label} import: {e}")))?;
                        match import.ty {
                            wasmparser::TypeRef::Func(type_idx) => {
                                func_imports.push(FuncImport {
                                    module: import.module.to_string(),
                                    name: import.name.to_string(),
                                    type_idx,
                                });
                                num_imported_funcs += 1;
                            }
                            _ => {
                                other_imports.push(RawOtherImport {
                                    module: import.module.to_string(),
                                    name: import.name.to_string(),
                                    ty: import.ty,
                                });
                            }
                        }
                    }
                }
                wasmparser::Payload::FunctionSection(reader) => {
                    for type_idx in reader {
                        let idx = type_idx
                            .map_err(|e| Error::Internal(format!("{label} func sec: {e}")))?;
                        func_type_indices.push(idx);
                    }
                }
                wasmparser::Payload::GlobalSection(reader) => {
                    for global in reader {
                        let g =
                            global.map_err(|e| Error::Internal(format!("{label} global: {e}")))?;
                        globals.push(RawGlobal {
                            ty: g.ty,
                            init_expr: g.init_expr,
                        });
                    }
                }
                wasmparser::Payload::MemorySection(reader) => {
                    for mem in reader {
                        let m = mem.map_err(|e| Error::Internal(format!("{label} memory: {e}")))?;
                        memories.push(m);
                    }
                }
                wasmparser::Payload::TableSection(reader) => {
                    for table in reader {
                        let t =
                            table.map_err(|e| Error::Internal(format!("{label} table: {e}")))?;
                        tables.push(t.ty);
                    }
                }
                wasmparser::Payload::ExportSection(reader) => {
                    for export in reader {
                        let e =
                            export.map_err(|e| Error::Internal(format!("{label} export: {e}")))?;
                        match e.kind {
                            wasmparser::ExternalKind::Func => {
                                func_exports.push((e.name.to_string(), e.index));
                            }
                            _ => {
                                other_exports.push(RawExport {
                                    name: e.name.to_string(),
                                    kind: e.kind,
                                    index: e.index,
                                });
                            }
                        }
                    }
                }
                wasmparser::Payload::ElementSection(reader) => {
                    for element in reader {
                        let el = element
                            .map_err(|e| Error::Internal(format!("{label} element: {e}")))?;
                        elements.push(RawElement { raw: el });
                    }
                }
                wasmparser::Payload::StartSection { func, .. } => {
                    start_func = Some(func);
                }
                wasmparser::Payload::CodeSectionEntry(body) => {
                    let locals_reader = body
                        .get_locals_reader()
                        .map_err(|e| Error::Internal(format!("{label} code locals: {e}")))?;
                    let mut locals = Vec::new();
                    for local in locals_reader {
                        let (count, ty) = local
                            .map_err(|e| Error::Internal(format!("{label} code local: {e}")))?;
                        locals.push((count, ty));
                    }
                    let operators = body
                        .get_operators_reader()
                        .map_err(|e| Error::Internal(format!("{label} code body: {e}")))?;
                    code_bodies.push(RawFuncBody {
                        locals,
                        body_reader: operators,
                    });
                }
                wasmparser::Payload::DataSection(reader) => {
                    for data in reader {
                        let d = data.map_err(|e| Error::Internal(format!("{label} data: {e}")))?;
                        data_segments.push(RawData { raw: d });
                    }
                }
                _ => {}
            }
        }

        Ok(ParsedModule {
            types,
            func_imports,
            num_imported_funcs,
            func_type_indices,
            func_exports,
            code_bodies,
            globals,
            memories,
            tables,
            elements,
            data_segments,
            start_func,
            other_exports,
            other_imports,
        })
    }
}

/// Build the merged WASM module.
fn build_merged_module(
    main: &ParsedModule,
    adapter: &ParsedModule,
    resolved_main_imports: &[Option<u32>],
) -> Result<Vec<u8>> {
    use wasm_encoder::{
        CodeSection, DataSection, ElementSection, ExportSection, FunctionSection, GlobalSection,
        ImportSection, MemorySection, Module, TableSection, TypeSection,
    };

    // 1. Build type dedup map: adapter type idx -> merged type idx.
    let mut merged_types: Vec<FuncSig> = main.types.clone();
    let mut adapter_type_remap: Vec<u32> = Vec::new();

    for adapter_type in &adapter.types {
        // Look for an existing matching type in merged_types.
        let existing = merged_types
            .iter()
            .position(|t| t.params == adapter_type.params && t.results == adapter_type.results);
        if let Some(idx) = existing {
            adapter_type_remap.push(idx as u32);
        } else {
            adapter_type_remap.push(merged_types.len() as u32);
            merged_types.push(adapter_type.clone());
        }
    }

    // 2. Build retained imports (main imports not resolved by adapter).
    let mut retained_main_imports: Vec<(usize, &FuncImport)> = Vec::new();
    let mut main_import_remap: Vec<u32> = vec![0; main.num_imported_funcs as usize];

    for (i, imp) in main.func_imports.iter().enumerate() {
        if resolved_main_imports[i].is_none() {
            main_import_remap[i] = retained_main_imports.len() as u32;
            retained_main_imports.push((i, imp));
        }
    }

    // Also carry through adapter func imports as retained.
    let adapter_import_start = retained_main_imports.len() as u32;
    let r = retained_main_imports.len() as u32 + adapter.num_imported_funcs;

    // Build adapter import remap: adapter import idx -> merged func idx.
    let mut adapter_import_remap: Vec<u32> = Vec::new();
    for i in 0..adapter.num_imported_funcs {
        adapter_import_remap.push(adapter_import_start + i);
    }

    // R = total retained imports count
    let retained_count = r;
    let adapter_local_count = adapter.func_type_indices.len() as u32;
    let _main_local_count = main.func_type_indices.len() as u32;

    // 3. Set remap for resolved main imports -> adapter local func in merged space.
    for (i, resolved) in resolved_main_imports.iter().enumerate() {
        if let Some(adapter_local_idx) = resolved {
            // Merged index: retained_count + adapter_local_idx
            main_import_remap[i] = retained_count + adapter_local_idx;
        }
    }

    // 4. Build function index remap tables.
    // Main func remap: main global func idx -> merged global func idx.
    let main_local_base = retained_count + adapter_local_count;
    let main_func_remap = |old_idx: u32| -> u32 {
        if (old_idx as usize) < main.num_imported_funcs as usize {
            main_import_remap[old_idx as usize]
        } else {
            let local_idx = old_idx - main.num_imported_funcs;
            main_local_base + local_idx
        }
    };

    // Adapter func remap: adapter global func idx -> merged global func idx.
    let adapter_func_remap = |old_idx: u32| -> u32 {
        if (old_idx as usize) < adapter.num_imported_funcs as usize {
            adapter_import_remap[old_idx as usize]
        } else {
            let local_idx = old_idx - adapter.num_imported_funcs;
            retained_count + local_idx
        }
    };

    // 5. Encode merged module.
    let mut module = Module::new();

    // Type section
    let mut type_section = TypeSection::new();
    for sig in &merged_types {
        type_section.ty().function(
            sig.params.iter().map(|v| encode_val_type(*v)),
            sig.results.iter().map(|v| encode_val_type(*v)),
        );
    }
    module.section(&type_section);

    // Import section: retained main imports + adapter imports
    let mut import_section = ImportSection::new();
    for (_, imp) in &retained_main_imports {
        import_section.import(
            &imp.module,
            &imp.name,
            wasm_encoder::EntityType::Function(imp.type_idx),
        );
    }
    for imp in &adapter.func_imports {
        let remapped_type = adapter_type_remap[imp.type_idx as usize];
        import_section.import(
            &imp.module,
            &imp.name,
            wasm_encoder::EntityType::Function(remapped_type),
        );
    }
    // Non-function imports from main.
    for imp in &main.other_imports {
        import_section.import(&imp.module, &imp.name, encode_type_ref(&imp.ty));
    }
    module.section(&import_section);

    // Function section: adapter locals then main locals (type indices)
    let mut func_section = FunctionSection::new();
    for &type_idx in &adapter.func_type_indices {
        func_section.function(adapter_type_remap[type_idx as usize]);
    }
    for &type_idx in &main.func_type_indices {
        func_section.function(type_idx);
    }
    module.section(&func_section);

    // Table section (from main only)
    if !main.tables.is_empty() {
        let mut table_section = TableSection::new();
        for t in &main.tables {
            table_section.table(encode_table_type(t));
        }
        module.section(&table_section);
    }

    // Memory section (from main only)
    if !main.memories.is_empty() {
        let mut mem_section = MemorySection::new();
        for m in &main.memories {
            mem_section.memory(encode_memory_type(m));
        }
        module.section(&mem_section);
    }

    // Global section (from main only)
    if !main.globals.is_empty() {
        let mut global_section = GlobalSection::new();
        for g in &main.globals {
            global_section.global(encode_global_type(g.ty), &encode_const_expr(&g.init_expr)?);
        }
        module.section(&global_section);
    }

    // Export section (from main, with remapped function indices)
    let mut export_section = ExportSection::new();
    for (name, func_idx) in &main.func_exports {
        export_section.export(
            name,
            wasm_encoder::ExportKind::Func,
            main_func_remap(*func_idx),
        );
    }
    for exp in &main.other_exports {
        export_section.export(&exp.name, encode_export_kind(exp.kind), exp.index);
    }
    module.section(&export_section);

    // Start section (from main, remapped)
    if let Some(start_idx) = main.start_func {
        module.section(&wasm_encoder::StartSection {
            function_index: main_func_remap(start_idx),
        });
    }

    // Element section (from main, with remapped function indices)
    if !main.elements.is_empty() {
        let mut elem_section = ElementSection::new();
        for el in &main.elements {
            encode_element(&mut elem_section, &el.raw, &main_func_remap)?;
        }
        module.section(&elem_section);
    }

    // Code section: adapter bodies first, then main bodies
    let mut code_section = CodeSection::new();
    for (body_idx, body) in adapter.code_bodies.iter().enumerate() {
        encode_function_body(
            &mut code_section,
            body,
            &adapter_func_remap,
            &adapter_type_remap,
            &format!("adapter func {body_idx}"),
        )?;
    }
    for (body_idx, body) in main.code_bodies.iter().enumerate() {
        encode_function_body_main(
            &mut code_section,
            body,
            &main_func_remap,
            main.num_imported_funcs,
            main.func_type_indices.len() as u32,
            &format!("main func {body_idx}"),
        )?;
    }
    module.section(&code_section);

    // Data section (from main only)
    if !main.data_segments.is_empty() {
        let mut data_section = DataSection::new();
        for seg in &main.data_segments {
            encode_data_segment(&mut data_section, &seg.raw)?;
        }
        module.section(&data_section);
    }

    Ok(module.finish())
}

fn encode_val_type(vt: wasmparser::ValType) -> wasm_encoder::ValType {
    match vt {
        wasmparser::ValType::I32 => wasm_encoder::ValType::I32,
        wasmparser::ValType::I64 => wasm_encoder::ValType::I64,
        wasmparser::ValType::F32 => wasm_encoder::ValType::F32,
        wasmparser::ValType::F64 => wasm_encoder::ValType::F64,
        wasmparser::ValType::V128 => wasm_encoder::ValType::V128,
        wasmparser::ValType::Ref(r) => wasm_encoder::ValType::Ref(encode_ref_type(r)),
    }
}

fn encode_ref_type(rt: wasmparser::RefType) -> wasm_encoder::RefType {
    if rt == wasmparser::RefType::FUNCREF {
        wasm_encoder::RefType::FUNCREF
    } else if rt == wasmparser::RefType::EXTERNREF {
        wasm_encoder::RefType::EXTERNREF
    } else {
        // PVM only supports funcref and externref. Other ref types (GC
        // proposals, etc.) would fail during PVM lowering, so this
        // fallback is only reached for WASM modules we cannot compile.
        tracing::warn!("unsupported ref type in adapter merge, falling back to funcref");
        wasm_encoder::RefType::FUNCREF
    }
}

fn encode_type_ref(tr: &wasmparser::TypeRef) -> wasm_encoder::EntityType {
    match tr {
        wasmparser::TypeRef::Func(idx) => wasm_encoder::EntityType::Function(*idx),
        wasmparser::TypeRef::Table(t) => wasm_encoder::EntityType::Table(encode_table_type(t)),
        wasmparser::TypeRef::Memory(m) => wasm_encoder::EntityType::Memory(encode_memory_type(m)),
        wasmparser::TypeRef::Global(g) => wasm_encoder::EntityType::Global(encode_global_type(*g)),
        wasmparser::TypeRef::Tag(_) => {
            tracing::warn!("unsupported Tag import in adapter merge, falling back to Function(0)");
            wasm_encoder::EntityType::Function(0)
        }
    }
}

fn encode_table_type(t: &wasmparser::TableType) -> wasm_encoder::TableType {
    wasm_encoder::TableType {
        element_type: encode_ref_type(t.element_type),
        minimum: t.initial,
        maximum: t.maximum,
        table64: t.table64,
        shared: false,
    }
}

fn encode_memory_type(m: &wasmparser::MemoryType) -> wasm_encoder::MemoryType {
    wasm_encoder::MemoryType {
        minimum: m.initial,
        maximum: m.maximum,
        memory64: m.memory64,
        shared: m.shared,
        page_size_log2: m.page_size_log2,
    }
}

fn encode_global_type(g: wasmparser::GlobalType) -> wasm_encoder::GlobalType {
    wasm_encoder::GlobalType {
        val_type: encode_val_type(g.content_type),
        mutable: g.mutable,
        shared: g.shared,
    }
}

fn encode_const_expr(expr: &wasmparser::ConstExpr) -> Result<wasm_encoder::ConstExpr> {
    let mut reader = expr.get_binary_reader();
    let mut bytes = Vec::new();
    while !reader.eof() {
        let start = reader.current_position();
        let op = reader
            .read_operator()
            .map_err(|e| Error::Internal(format!("const expr read error: {e}")))?;
        if matches!(op, wasmparser::Operator::End) {
            break;
        }
        let end = reader.current_position();
        // Get the raw bytes of this operator from the binary reader.
        let raw = &expr.get_binary_reader().read_bytes(end).unwrap_or_default();
        // We need the bytes from start..end
        bytes.extend_from_slice(&raw[start..end]);
    }
    Ok(wasm_encoder::ConstExpr::raw(bytes))
}

fn encode_export_kind(kind: wasmparser::ExternalKind) -> wasm_encoder::ExportKind {
    match kind {
        wasmparser::ExternalKind::Func => wasm_encoder::ExportKind::Func,
        wasmparser::ExternalKind::Table => wasm_encoder::ExportKind::Table,
        wasmparser::ExternalKind::Memory => wasm_encoder::ExportKind::Memory,
        wasmparser::ExternalKind::Global => wasm_encoder::ExportKind::Global,
        wasmparser::ExternalKind::Tag => wasm_encoder::ExportKind::Tag,
    }
}

fn encode_element(
    section: &mut wasm_encoder::ElementSection,
    el: &wasmparser::Element,
    func_remap: &dyn Fn(u32) -> u32,
) -> Result<()> {
    match &el.kind {
        wasmparser::ElementKind::Active {
            table_index,
            offset_expr,
        } => {
            let offset = encode_const_expr(offset_expr)?;
            let func_indices: Vec<u32> = match &el.items {
                wasmparser::ElementItems::Functions(reader) => {
                    let mut indices = Vec::new();
                    for idx in reader.clone() {
                        let idx =
                            idx.map_err(|e| Error::Internal(format!("element func index: {e}")))?;
                        indices.push(func_remap(idx));
                    }
                    indices
                }
                wasmparser::ElementItems::Expressions(_, reader) => {
                    let mut indices = Vec::new();
                    for expr in reader.clone() {
                        let expr =
                            expr.map_err(|e| Error::Internal(format!("element expression: {e}")))?;
                        if let Some(idx) = eval_const_ref(&expr) {
                            indices.push(func_remap(idx));
                        } else {
                            indices.push(u32::MAX);
                        }
                    }
                    indices
                }
            };

            let elements = wasm_encoder::Elements::Functions(std::borrow::Cow::Owned(func_indices));
            section.active(*table_index, &offset, elements);
        }
        wasmparser::ElementKind::Passive => {
            let func_indices: Vec<u32> = match &el.items {
                wasmparser::ElementItems::Functions(reader) => {
                    let mut indices = Vec::new();
                    for idx in reader.clone() {
                        let idx =
                            idx.map_err(|e| Error::Internal(format!("element func index: {e}")))?;
                        indices.push(func_remap(idx));
                    }
                    indices
                }
                wasmparser::ElementItems::Expressions(_, _) => {
                    return Err(Error::Internal(
                        "passive element with expressions not supported in merge".into(),
                    ));
                }
            };
            section.passive(wasm_encoder::Elements::Functions(std::borrow::Cow::Owned(
                func_indices,
            )));
        }
        wasmparser::ElementKind::Declared => {
            // Declared elements are for reference types; skip for now.
        }
    }
    Ok(())
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

fn encode_data_segment(
    section: &mut wasm_encoder::DataSection,
    data: &wasmparser::Data,
) -> Result<()> {
    match &data.kind {
        wasmparser::DataKind::Active {
            memory_index,
            offset_expr,
        } => {
            let offset = encode_const_expr(offset_expr)?;
            section.active(*memory_index, &offset, data.data.to_vec());
        }
        wasmparser::DataKind::Passive => {
            section.passive(data.data.to_vec());
        }
    }
    Ok(())
}

/// Encode a function body with index remapping (for adapter functions).
fn encode_function_body(
    section: &mut wasm_encoder::CodeSection,
    body: &RawFuncBody,
    func_remap: &dyn Fn(u32) -> u32,
    type_remap: &[u32],
    label: &str,
) -> Result<()> {
    let mut func = wasm_encoder::Function::new(
        body.locals
            .iter()
            .map(|(count, ty)| (*count, encode_val_type(*ty))),
    );

    let mut reader = body.body_reader.clone();
    while !reader.eof() {
        let op = reader
            .read()
            .map_err(|e| Error::Internal(format!("{label} operator read: {e}")))?;
        encode_operator_with_remap(&mut func, &op, func_remap, type_remap)?;
    }

    section.function(&func);
    Ok(())
}

/// Encode a function body with main module index remapping.
fn encode_function_body_main(
    section: &mut wasm_encoder::CodeSection,
    body: &RawFuncBody,
    func_remap: &dyn Fn(u32) -> u32,
    _num_imported_funcs: u32,
    _num_local_funcs: u32,
    label: &str,
) -> Result<()> {
    // Main types are not remapped (they keep their indices at the start of merged types).
    let empty_type_remap: Vec<u32> = Vec::new();
    let mut func = wasm_encoder::Function::new(
        body.locals
            .iter()
            .map(|(count, ty)| (*count, encode_val_type(*ty))),
    );

    let mut reader = body.body_reader.clone();
    while !reader.eof() {
        let op = reader
            .read()
            .map_err(|e| Error::Internal(format!("{label} operator read: {e}")))?;
        // For main functions, type indices stay the same (main types are at the start).
        // Pass empty remap so the fallback path in encode_operator_with_remap keeps indices as-is.
        encode_operator_with_remap(&mut func, &op, func_remap, &empty_type_remap)?;
    }

    section.function(&func);
    Ok(())
}

/// Encode a single WASM operator, remapping function and type indices.
fn encode_operator_with_remap(
    func: &mut wasm_encoder::Function,
    op: &wasmparser::Operator,
    func_remap: &dyn Fn(u32) -> u32,
    type_remap: &[u32],
) -> Result<()> {
    use wasm_encoder::Instruction as I;
    use wasmparser::Operator as O;

    match op {
        O::Call { function_index } => {
            func.instruction(&I::Call(func_remap(*function_index)));
        }
        O::ReturnCall { function_index } => {
            func.instruction(&I::ReturnCall(func_remap(*function_index)));
        }
        O::RefFunc { function_index } => {
            func.instruction(&I::RefFunc(func_remap(*function_index)));
        }
        O::CallIndirect {
            type_index,
            table_index,
            ..
        } => {
            let remapped_type = if (*type_index as usize) < type_remap.len() {
                type_remap[*type_index as usize]
            } else {
                *type_index
            };
            func.instruction(&I::CallIndirect {
                type_index: remapped_type,
                table_index: *table_index,
            });
        }
        O::ReturnCallIndirect {
            type_index,
            table_index,
        } => {
            let remapped_type = if (*type_index as usize) < type_remap.len() {
                type_remap[*type_index as usize]
            } else {
                *type_index
            };
            func.instruction(&I::ReturnCallIndirect {
                type_index: remapped_type,
                table_index: *table_index,
            });
        }
        // All other operators: pass through without remapping.
        // Use the raw encoding approach for efficiency.
        _ => {
            encode_passthrough_operator(func, op)?;
        }
    }

    Ok(())
}

/// Encode a WASM operator that doesn't need index remapping.
/// This is a large match that covers the core WASM instruction set.
fn encode_passthrough_operator(
    func: &mut wasm_encoder::Function,
    op: &wasmparser::Operator,
) -> Result<()> {
    use wasm_encoder::Instruction as I;
    use wasmparser::Operator as O;

    let instr = match op {
        O::Unreachable => I::Unreachable,
        O::Nop => I::Nop,
        O::Block { blockty } => I::Block(encode_block_type(*blockty)),
        O::Loop { blockty } => I::Loop(encode_block_type(*blockty)),
        O::If { blockty } => I::If(encode_block_type(*blockty)),
        O::Else => I::Else,
        O::End => I::End,
        O::Br { relative_depth } => I::Br(*relative_depth),
        O::BrIf { relative_depth } => I::BrIf(*relative_depth),
        O::BrTable { targets } => {
            let mut labels: Vec<u32> = Vec::new();
            for target in targets.targets() {
                labels.push(target.map_err(|e| Error::Internal(format!("br_table: {e}")))?);
            }
            let default = targets.default();
            func.instruction(&I::BrTable(std::borrow::Cow::Owned(labels), default));
            return Ok(());
        }
        O::Return => I::Return,
        O::Drop => I::Drop,
        O::Select => I::Select,
        O::TypedSelect { ty } => {
            func.instruction(&I::TypedSelect(encode_val_type(*ty)));
            return Ok(());
        }
        O::LocalGet { local_index } => I::LocalGet(*local_index),
        O::LocalSet { local_index } => I::LocalSet(*local_index),
        O::LocalTee { local_index } => I::LocalTee(*local_index),
        O::GlobalGet { global_index } => I::GlobalGet(*global_index),
        O::GlobalSet { global_index } => I::GlobalSet(*global_index),

        // Memory instructions
        O::I32Load { memarg } => I::I32Load(encode_memarg(memarg)),
        O::I64Load { memarg } => I::I64Load(encode_memarg(memarg)),
        O::F32Load { memarg } => I::F32Load(encode_memarg(memarg)),
        O::F64Load { memarg } => I::F64Load(encode_memarg(memarg)),
        O::I32Load8S { memarg } => I::I32Load8S(encode_memarg(memarg)),
        O::I32Load8U { memarg } => I::I32Load8U(encode_memarg(memarg)),
        O::I32Load16S { memarg } => I::I32Load16S(encode_memarg(memarg)),
        O::I32Load16U { memarg } => I::I32Load16U(encode_memarg(memarg)),
        O::I64Load8S { memarg } => I::I64Load8S(encode_memarg(memarg)),
        O::I64Load8U { memarg } => I::I64Load8U(encode_memarg(memarg)),
        O::I64Load16S { memarg } => I::I64Load16S(encode_memarg(memarg)),
        O::I64Load16U { memarg } => I::I64Load16U(encode_memarg(memarg)),
        O::I64Load32S { memarg } => I::I64Load32S(encode_memarg(memarg)),
        O::I64Load32U { memarg } => I::I64Load32U(encode_memarg(memarg)),
        O::I32Store { memarg } => I::I32Store(encode_memarg(memarg)),
        O::I64Store { memarg } => I::I64Store(encode_memarg(memarg)),
        O::F32Store { memarg } => I::F32Store(encode_memarg(memarg)),
        O::F64Store { memarg } => I::F64Store(encode_memarg(memarg)),
        O::I32Store8 { memarg } => I::I32Store8(encode_memarg(memarg)),
        O::I32Store16 { memarg } => I::I32Store16(encode_memarg(memarg)),
        O::I64Store8 { memarg } => I::I64Store8(encode_memarg(memarg)),
        O::I64Store16 { memarg } => I::I64Store16(encode_memarg(memarg)),
        O::I64Store32 { memarg } => I::I64Store32(encode_memarg(memarg)),
        O::MemorySize { mem, .. } => I::MemorySize(*mem),
        O::MemoryGrow { mem, .. } => I::MemoryGrow(*mem),
        O::MemoryInit { data_index, mem } => I::MemoryInit {
            data_index: *data_index,
            mem: *mem,
        },
        O::DataDrop { data_index } => I::DataDrop(*data_index),
        O::MemoryCopy { dst_mem, src_mem } => I::MemoryCopy {
            dst_mem: *dst_mem,
            src_mem: *src_mem,
        },
        O::MemoryFill { mem } => I::MemoryFill(*mem),

        // Constants
        O::I32Const { value } => I::I32Const(*value),
        O::I64Const { value } => I::I64Const(*value),
        O::F32Const { value } => I::F32Const(f32::from_bits(value.bits())),
        O::F64Const { value } => I::F64Const(f64::from_bits(value.bits())),

        // Comparison operators
        O::I32Eqz => I::I32Eqz,
        O::I32Eq => I::I32Eq,
        O::I32Ne => I::I32Ne,
        O::I32LtS => I::I32LtS,
        O::I32LtU => I::I32LtU,
        O::I32GtS => I::I32GtS,
        O::I32GtU => I::I32GtU,
        O::I32LeS => I::I32LeS,
        O::I32LeU => I::I32LeU,
        O::I32GeS => I::I32GeS,
        O::I32GeU => I::I32GeU,
        O::I64Eqz => I::I64Eqz,
        O::I64Eq => I::I64Eq,
        O::I64Ne => I::I64Ne,
        O::I64LtS => I::I64LtS,
        O::I64LtU => I::I64LtU,
        O::I64GtS => I::I64GtS,
        O::I64GtU => I::I64GtU,
        O::I64LeS => I::I64LeS,
        O::I64LeU => I::I64LeU,
        O::I64GeS => I::I64GeS,
        O::I64GeU => I::I64GeU,

        // Numeric operators
        O::I32Clz => I::I32Clz,
        O::I32Ctz => I::I32Ctz,
        O::I32Popcnt => I::I32Popcnt,
        O::I32Add => I::I32Add,
        O::I32Sub => I::I32Sub,
        O::I32Mul => I::I32Mul,
        O::I32DivS => I::I32DivS,
        O::I32DivU => I::I32DivU,
        O::I32RemS => I::I32RemS,
        O::I32RemU => I::I32RemU,
        O::I32And => I::I32And,
        O::I32Or => I::I32Or,
        O::I32Xor => I::I32Xor,
        O::I32Shl => I::I32Shl,
        O::I32ShrS => I::I32ShrS,
        O::I32ShrU => I::I32ShrU,
        O::I32Rotl => I::I32Rotl,
        O::I32Rotr => I::I32Rotr,
        O::I64Clz => I::I64Clz,
        O::I64Ctz => I::I64Ctz,
        O::I64Popcnt => I::I64Popcnt,
        O::I64Add => I::I64Add,
        O::I64Sub => I::I64Sub,
        O::I64Mul => I::I64Mul,
        O::I64DivS => I::I64DivS,
        O::I64DivU => I::I64DivU,
        O::I64RemS => I::I64RemS,
        O::I64RemU => I::I64RemU,
        O::I64And => I::I64And,
        O::I64Or => I::I64Or,
        O::I64Xor => I::I64Xor,
        O::I64Shl => I::I64Shl,
        O::I64ShrS => I::I64ShrS,
        O::I64ShrU => I::I64ShrU,
        O::I64Rotl => I::I64Rotl,
        O::I64Rotr => I::I64Rotr,

        // Conversions
        O::I32WrapI64 => I::I32WrapI64,
        O::I64ExtendI32S => I::I64ExtendI32S,
        O::I64ExtendI32U => I::I64ExtendI32U,
        O::I32Extend8S => I::I32Extend8S,
        O::I32Extend16S => I::I32Extend16S,
        O::I64Extend8S => I::I64Extend8S,
        O::I64Extend16S => I::I64Extend16S,
        O::I64Extend32S => I::I64Extend32S,

        // Reference types
        O::RefNull { hty } => I::RefNull(encode_heap_type(*hty)),
        O::RefIsNull => I::RefIsNull,

        // Table operations
        O::TableGet { table } => I::TableGet(*table),
        O::TableSet { table } => I::TableSet(*table),
        O::TableGrow { table } => I::TableGrow(*table),
        O::TableSize { table } => I::TableSize(*table),

        _ => {
            return Err(Error::Unsupported(format!(
                "unsupported operator in adapter merge: {op:?}"
            )));
        }
    };

    func.instruction(&instr);
    Ok(())
}

fn encode_block_type(bt: wasmparser::BlockType) -> wasm_encoder::BlockType {
    match bt {
        wasmparser::BlockType::Empty => wasm_encoder::BlockType::Empty,
        wasmparser::BlockType::Type(vt) => wasm_encoder::BlockType::Result(encode_val_type(vt)),
        wasmparser::BlockType::FuncType(idx) => wasm_encoder::BlockType::FunctionType(idx),
    }
}

fn encode_memarg(memarg: &wasmparser::MemArg) -> wasm_encoder::MemArg {
    wasm_encoder::MemArg {
        offset: memarg.offset,
        align: u32::from(memarg.align),
        memory_index: memarg.memory,
    }
}

fn encode_heap_type(hty: wasmparser::HeapType) -> wasm_encoder::HeapType {
    match hty {
        wasmparser::HeapType::Abstract { shared: _, ty } => match ty {
            wasmparser::AbstractHeapType::Extern => wasm_encoder::HeapType::Abstract {
                shared: false,
                ty: wasm_encoder::AbstractHeapType::Extern,
            },
            _ => wasm_encoder::HeapType::Abstract {
                shared: false,
                ty: wasm_encoder::AbstractHeapType::Func,
            },
        },
        wasmparser::HeapType::Concrete(idx) => {
            let module_idx = idx.as_module_index().unwrap_or_else(|| {
                tracing::warn!(
                    "concrete heap type without module index in adapter merge, falling back to 0"
                );
                0
            });
            wasm_encoder::HeapType::Concrete(module_idx)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_merge_single_import() {
        let main_wat = r#"
            (module
                (import "env" "abort" (func $abort (param i32 i32 i32 i32)))
                (memory (export "memory") 1)
                (func (export "main") (param i32 i32) (result i32)
                    (call $abort (i32.const 1) (i32.const 2) (i32.const 3) (i32.const 4))
                    (i32.const 0)
                )
            )
        "#;
        let adapter_wat = r#"
            (module
                (func (export "abort") (param i32 i32 i32 i32)
                    unreachable
                )
            )
        "#;

        let main_wasm = wat::parse_str(main_wat).expect("main WAT parse");
        let merged = merge_adapter(&main_wasm, adapter_wat).expect("merge");

        // The merged module should be valid WASM.
        wasmparser::validate(&merged).expect("merged module should be valid");

        // Parse merged module and verify: no imports, 2 functions.
        let merged_mod = ParsedModule::parse(&merged, "merged").expect("parse merged");
        assert_eq!(
            merged_mod.num_imported_funcs, 0,
            "abort import should be resolved"
        );
        assert_eq!(
            merged_mod.func_type_indices.len(),
            2,
            "adapter + main local funcs"
        );
    }

    #[test]
    fn test_merge_with_adapter_imports() {
        let main_wat = r#"
            (module
                (import "env" "host_call" (func $host_call (param i64 i64 i64 i64 i64 i64)))
                (import "env" "console.log" (func $log (param i32)))
                (memory (export "memory") 1)
                (func (export "main") (param i32 i32) (result i32)
                    (call $log (i32.const 42))
                    (i32.const 0)
                )
            )
        "#;
        let adapter_wat = r#"
            (module
                (import "env" "host_call" (func $host_call (param i64 i64 i64 i64 i64 i64)))
                (func (export "console.log") (param i32)
                    (call $host_call
                        (i64.const 3)
                        (i64.extend_i32_u (local.get 0))
                        (i64.const 0) (i64.const 0) (i64.const 0) (i64.const 0))
                )
            )
        "#;

        let main_wasm = wat::parse_str(main_wat).expect("main WAT parse");
        let merged = merge_adapter(&main_wasm, adapter_wat).expect("merge");

        wasmparser::validate(&merged).expect("merged module should be valid");

        let merged_mod = ParsedModule::parse(&merged, "merged").expect("parse merged");
        // host_call is retained from main + host_call from adapter = 2 imports
        assert_eq!(
            merged_mod.num_imported_funcs, 2,
            "host_call retained from main + adapter"
        );
        // console.log is resolved, so 2 local funcs: adapter's console.log body + main
        assert_eq!(merged_mod.func_type_indices.len(), 2);
    }

    #[test]
    fn test_merge_type_mismatch_error() {
        let main_wat = r#"
            (module
                (import "env" "foo" (func $foo (param i32) (result i32)))
                (func (export "main") (param i32 i32) (result i32)
                    (call $foo (i32.const 1))
                )
            )
        "#;
        let adapter_wat = r#"
            (module
                (func (export "foo") (param i32 i32) (result i32)
                    (i32.add (local.get 0) (local.get 1))
                )
            )
        "#;

        let main_wasm = wat::parse_str(main_wat).expect("main WAT parse");
        let result = merge_adapter(&main_wasm, adapter_wat);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("type mismatch"),
            "Expected type mismatch error, got: {err}"
        );
    }

    #[test]
    fn test_merge_multiple_imports_resolved() {
        let main_wat = r#"
            (module
                (import "env" "foo" (func $foo (param i32) (result i32)))
                (import "env" "bar" (func $bar (result i32)))
                (memory (export "memory") 1)
                (func (export "main") (param i32 i32) (result i32)
                    (i32.add
                        (call $foo (i32.const 1))
                        (call $bar))
                )
            )
        "#;
        let adapter_wat = r#"
            (module
                (func (export "foo") (param i32) (result i32)
                    (i32.mul (local.get 0) (i32.const 2))
                )
                (func (export "bar") (result i32)
                    (i32.const 42)
                )
            )
        "#;

        let main_wasm = wat::parse_str(main_wat).expect("main WAT parse");
        let merged = merge_adapter(&main_wasm, adapter_wat).expect("merge");

        wasmparser::validate(&merged).expect("merged module should be valid");

        let merged_mod = ParsedModule::parse(&merged, "merged").expect("parse merged");
        assert_eq!(merged_mod.num_imported_funcs, 0, "all imports resolved");
        assert_eq!(merged_mod.func_type_indices.len(), 3, "2 adapter + 1 main");
    }

    #[test]
    fn test_merge_partial_resolution() {
        // Only resolve one of two imports.
        let main_wat = r#"
            (module
                (import "env" "host_call" (func $host_call (param i64 i64 i64 i64 i64 i64)))
                (import "env" "abort" (func $abort (param i32 i32 i32 i32)))
                (memory (export "memory") 1)
                (func (export "main") (param i32 i32) (result i32)
                    (i32.const 0)
                )
            )
        "#;
        let adapter_wat = r#"
            (module
                (func (export "abort") (param i32 i32 i32 i32)
                    unreachable
                )
            )
        "#;

        let main_wasm = wat::parse_str(main_wat).expect("main WAT parse");
        let merged = merge_adapter(&main_wasm, adapter_wat).expect("merge");

        wasmparser::validate(&merged).expect("merged module should be valid");

        let merged_mod = ParsedModule::parse(&merged, "merged").expect("parse merged");
        // host_call is retained.
        assert_eq!(merged_mod.num_imported_funcs, 1);
        assert_eq!(merged_mod.func_imports[0].name, "host_call");
    }

    #[test]
    fn test_merged_module_compiles() {
        // End-to-end: merge then compile to PVM.
        let main_wat = r#"
            (module
                (import "env" "abort" (func $abort (param i32 i32 i32 i32)))
                (memory (export "memory") 1)
                (func (export "main") (param i32 i32) (result i32)
                    (call $abort (i32.const 1) (i32.const 2) (i32.const 3) (i32.const 4))
                    (i32.const 0)
                )
            )
        "#;
        let adapter_wat = r#"
            (module
                (func (export "abort") (param i32 i32 i32 i32)
                    unreachable
                )
            )
        "#;

        let main_wasm = wat::parse_str(main_wat).expect("main WAT parse");
        let merged = merge_adapter(&main_wasm, adapter_wat).expect("merge");

        // Compile the merged module.
        let result = crate::compile(&merged);
        assert!(
            result.is_ok(),
            "merged module should compile: {:?}",
            result.err()
        );
    }
}
