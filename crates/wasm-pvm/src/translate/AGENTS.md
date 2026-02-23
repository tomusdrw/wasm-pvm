# Translation Module

**Purpose**: Orchestrate end-to-end WASM → LLVM IR → PVM lowering and assemble final SPI/JAM output.

## Files

| File | Role |
|------|------|
| `mod.rs` | Pipeline dispatch, SPI assembly, entry header + data sections |
| `wasm_module.rs` | WASM section parsing into `WasmModule` |
| `memory_layout.rs` | Memory address constants and helper functions |

## Pipeline

1. Parse module sections in `wasm_module.rs` (`WasmModule::parse()`).
2. Translate WASM operators to LLVM IR in `llvm_frontend/function_builder.rs`.
3. Run LLVM optimization pipeline (`mem2reg`, `instcombine`, `simplifycfg`, optional inlining, cleanup passes).
4. Lower LLVM IR to PVM instructions in `llvm_backend/mod.rs`.
5. Build SPI sections in `mod.rs`:
   - Entry header and dispatch tables
   - `ro_data` (jump table refs + passive data)
   - `rw_data` (globals + active data segments), with trailing zero trim
   - Encoded PVM blob + metadata

## Key Behaviors

- `calculate_heap_pages()` uses WASM `initial_pages` (not max), with a minimum of 16 WASM pages for `(memory 0)`.
- `compute_wasm_memory_base()` enforces 64KB alignment (currently `0x40000`).
- `build_rw_data()` copies globals and active segments into a contiguous image, then trims trailing zero bytes before SPI encoding.

## Current Memory Layout

| Address | Purpose |
|---------|---------|
| `0x10000` | Read-only data |
| `0x30000` | Globals storage |
| `0x32000` | Parameter overflow area |
| `0x32100+` | Spilled-locals base (spills are stack-based; base kept for layout/alignment) |
| `0x40000+` | WASM linear memory base (64KB-aligned) |

## Anti-Patterns

1. Don't change layout constants without validating pvm-in-pvm tests.
2. Don't bypass `Result` error handling with panics in library code.
3. Don't assume `rw_data` must include trailing zero bytes.
