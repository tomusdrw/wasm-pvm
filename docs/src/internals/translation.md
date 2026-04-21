# Translation Module

The translation module orchestrates the end-to-end WASM → LLVM IR → PVM lowering and assembles the final SPI/JAM output.

Source: `crates/wasm-pvm/src/translate/`

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
- `compute_wasm_memory_base()` lays out (in order) the (optional) mem-size slot at `GLOBAL_MEMORY_BASE`, user globals, passive segment lengths, and (optionally) the 256-byte parameter overflow area, then places `wasm_memory_base` immediately after. **No 4KB alignment** is applied — anan-as page-aligns the rw_data tail (`heapZerosStart`) separately, so the base may sit at any byte offset. Mem-size is emitted only when the module uses `memory.size`/`memory.grow`/`memory.init`; overflow (tracked by `needs_param_overflow`) is emitted only when any module type signature has more than `MAX_LOCAL_REGS` (4) parameters — this covers both local function declarations and `call_indirect` target types.
- `build_rw_data()` copies globals and active segments into a contiguous image, then trims trailing zero bytes before SPI encoding.
- Call return addresses are pre-assigned as jump-table refs `((idx + 1) * 2)` at emission time; fixup resolution accepts direct (`LoadImmJump`) and indirect (`LoadImm` / `LoadImmJumpInd`) return-address carriers.
- Export parsing tracks `exported_wasm_func_indices` in WASM global index space for dead-function-elimination roots; entry resolution prefers canonical names (`main`, `main2`) over aliases (`refine*`, `accumulate*`) regardless of export order.
- Entry exports (`main`/`main2` and aliases) must target local (non-imported) functions; imported targets are rejected during parse with `Error::Internal` to avoid index-underflow panics.

## Current Memory Layout

| Address | Purpose |
|---------|---------|
| `0x10000` | Read-only data |
| `0x30000` | Mem-size slot (4 bytes, only when `memory.size`/`grow`/`init` used), then user globals, passive segment length slots, and (when any type signature has >4 params) a 256-byte parameter overflow area. Total size = `align_up_8(globals_region_size(...)) + 256` when overflow is reserved (the overflow base is 8-byte aligned — see `compute_param_overflow_base`), else just `globals_region_size(...)`. |
| `region_end` | WASM linear memory — placed **without 4KB alignment** immediately after the last region. For a module that only declares memory and never uses `memory.size`/`grow`/`init`, `wasm_memory_base` collapses to `0x30000`. A memory-op-using program with zero user globals, no passive segments, and no overflow lands at `0x30004`. A program that also needs overflow (e.g. a 5+ param `call_indirect` target) lands at `0x30108`. |

## Anti-Patterns

1. Don't change layout constants without validating pvm-in-pvm tests.
2. Don't bypass `Result` error handling with panics in library code.
3. Don't assume `rw_data` must include trailing zero bytes.
