# Translation Module

**Purpose**: Compilation orchestration and WASM → PVM translation

## Files

| File | Role |
|------|------|
| `mod.rs` | Pipeline dispatch (`compile_via_llvm` / `compile_legacy`), SPI assembly, entry header |
| `wasm_module.rs` | Shared WASM section parsing into `WasmModule` struct |
| `memory_layout.rs` | PVM memory address constants and helpers |
| `codegen.rs` | Legacy backend — direct WASM → PVM translation |
| `stack.rs` | Legacy operand stack to register mapping |

## Compilation Pipeline

### LLVM Backend (`--features llvm-backend`)
1. `wasm_module.rs` parses WASM sections → `WasmModule`
2. `llvm_frontend/function_builder.rs` translates each function → LLVM IR
3. LLVM `mem2reg` pass promotes alloca'd locals to SSA registers
4. `llvm_backend/lowering.rs` reads LLVM IR → emits PVM bytecode
5. `mod.rs` builds entry header, dispatch tables, ro_data/rw_data → `SpiProgram`

### Legacy Backend (default)
1. `wasm_module.rs` parses WASM sections → `WasmModule`
2. `codegen.rs` directly translates WASM operators → PVM instructions
3. `mod.rs` builds entry header, dispatch tables, ro_data/rw_data → `SpiProgram`

## Key Functions in `mod.rs`

- `compile()` — Dispatches to LLVM or legacy backend based on feature flag
- `compile_via_llvm()` — Full LLVM pipeline
- `compile_legacy()` — Direct translation pipeline
- Entry header emission, data segment copying, dispatch table construction

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
3. **Don't break memory layout constants** — Used by both backends
