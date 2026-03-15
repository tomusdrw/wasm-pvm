/// Compilation statistics collected during the WASM-to-PVM pipeline.
#[derive(Debug, Clone)]
pub struct CompileStats {
    // ── Input ──
    pub local_functions: usize,
    pub imported_functions: usize,
    pub globals: usize,
    pub active_data_segments: usize,
    pub passive_data_segments: usize,
    pub function_table_entries: usize,
    pub initial_memory_pages: u32,
    pub max_memory_pages: u32,
    pub import_resolutions: Vec<ImportResolution>,

    // ── Memory Layout ──
    pub wasm_memory_base: i32,
    pub globals_region_bytes: usize,
    pub ro_data_bytes: usize,
    pub rw_data_bytes: usize,
    pub heap_pages: u16,
    pub stack_size: u32,

    // ── Output ──
    pub pvm_instructions: usize,
    pub code_bytes: usize,
    pub jump_table_entries: usize,
    pub dead_functions_eliminated: usize,
    pub spi_blob_bytes: usize,

    // ── Per-function ──
    pub functions: Vec<FunctionStats>,
}

/// How an imported function was resolved.
#[derive(Debug, Clone)]
pub struct ImportResolution {
    pub name: String,
    pub action: String,
}

/// Per-function compilation statistics.
#[derive(Debug, Clone)]
pub struct FunctionStats {
    pub name: String,
    pub index: usize,
    pub instruction_count: usize,
    pub frame_size: i32,
    pub is_leaf: bool,
    pub is_entry: bool,
    pub is_dead: bool,
    pub regalloc: FunctionRegAllocStats,
    /// Instructions before dead store elimination (0 if DSE disabled).
    pub pre_dse_instructions: usize,
    /// Instructions before peephole (0 if peephole disabled).
    pub pre_peephole_instructions: usize,
}

/// Register allocation statistics for a single function.
#[derive(Debug, Clone, Default)]
pub struct FunctionRegAllocStats {
    pub total_values: usize,
    pub allocated_values: usize,
    pub registers_used: Vec<String>,
    pub skipped_reason: Option<String>,
    pub load_hits: usize,
    pub load_reloads: usize,
    pub load_moves: usize,
    pub store_hits: usize,
    pub store_moves: usize,
}
