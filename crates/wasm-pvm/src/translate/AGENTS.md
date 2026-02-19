# Translation Module

**Purpose**: Compilation orchestration and WASM → PVM translation

## Files

| File | Role |
|------|------|
| `mod.rs` | Pipeline dispatch, SPI assembly, entry header |
| `wasm_module.rs` | WASM section parsing into `WasmModule` struct |
| `memory_layout.rs` | PVM memory address constants and helpers |

## Compilation Pipeline

1. `wasm_module.rs` parses WASM sections → `WasmModule`
2. `llvm_frontend/function_builder.rs` translates each function → LLVM IR
3. LLVM `mem2reg` pass promotes alloca'd locals to SSA registers
4. `llvm_backend/lowering.rs` reads LLVM IR → emits PVM bytecode
5. `mod.rs` builds entry header, dispatch tables, ro_data/rw_data → `SpiProgram`

## Key Functions

### `mod.rs`
- `compile()` — Full compilation pipeline
- Entry header emission, data segment copying, dispatch table construction

### `wasm_module.rs`
- `WasmModule::parse()` — Parse all WASM sections
- `calculate_heap_pages()` — Compute SPI `heap_pages` from `initial_pages` (not `max_pages`). Uses minimum 16 WASM pages for `(memory 0)` programs. Returns `(heap_pages, max_memory_pages)` tuple.

## Memory Layout (from `memory_layout.rs`)

| Address | Purpose |
|---------|---------|
| `0x10000` | Read-only data (dispatch table) |
| `0x30000` | Globals storage |
| `0x30100` | User heap |
| `0x40000` | Spilled locals (512 bytes per function) |
| `0x50000+` | WASM linear memory base |

## Anti-Patterns

1. **Never add `unsafe`** — Workspace forbids it
2. **No panics** — Use `Result<>` with `Error::Internal`
3. **Don't break memory layout constants** — Used across the pipeline
