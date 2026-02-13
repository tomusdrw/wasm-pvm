# WASM-PVM: WebAssembly to PolkaVM Recompiler

A Rust compiler that translates WebAssembly (WASM) bytecode to PolkaVM (PVM) bytecode for execution on the JAM (Join-Accumulate Machine) protocol.

## Status: Active Development (360 integration tests passing)

**Project Goal**: Enable writing JAM programs in AssemblyScript (TypeScript-like) or hand-written WAT, compiled to PVM bytecode.

**Architecture**: `WASM → [inkwell] → LLVM IR → [mem2reg] → [Rust PVM backend] → PVM bytecode`

The compiler uses LLVM 18 (via inkwell) as its intermediate representation, with a custom Rust-based PVM backend that reads LLVM IR and emits PVM bytecode. This gives us LLVM's SSA/CFG representation and optimization passes without requiring a native LLVM C++ target backend. PVM-specific intrinsic functions (`@__pvm_load_i32`, `@__pvm_store_i32`, etc.) are used for memory operations to avoid `unsafe` code.

**Current State**:
- All 360 TypeScript integration tests and all Rust unit tests pass
- anan-as (PVM interpreter in AssemblyScript) compiles to a 423KB JAM file
- `unsafe_code = "deny"` enforced at workspace level

### Working Features

**Arithmetic (i32 & i64)**: add, sub, mul, div_u, div_s, rem_u, rem_s, all comparisons (eq, ne, lt_u/s, gt_u/s, le_u/s, ge_u/s, eqz), clz, ctz, popcnt, rotl, rotr, bitwise (and, or, xor, shl, shr_u, shr_s)

**Control Flow**: block, loop, if/else/end, br, br_if, br_table, return, unreachable, block result values

**Memory**: i32/i64 load/store, sub-word variants (load8_u/s, load16_u/s, load32_u/s, store8, store16, store32), memory.size, memory.grow, memory.fill, memory.copy, global.get/set, data section initialization

**Functions**: call, call_indirect (with signature validation), recursion, stack overflow detection (64KB default), local variables with spilling, local.get/set/tee, drop, select

**Type Conversions**: i32.wrap_i64, i64.extend_i32_s/u, sign extensions (i32.extend8_s, i32.extend16_s, i64.extend8_s/16_s/32_s)

**Import Handling**: Imported functions are stubbed (abort → TRAP, others → no-op with return value)

### Not Yet Implemented
- Division-by-zero and signed overflow trap sequences
- Multi-value returns (`entry_returns_ptr_len` convention)
- Host calls via `ecalli` instruction
- Floating point (rejected by design — PVM has no FP)
- Register allocator (currently uses stack-slot approach)

## Quick Start

### Build

Requires LLVM 18. On macOS: `brew install llvm@18` and set `LLVM_SYS_181_PREFIX=/opt/homebrew/opt/llvm@18`

```bash
cargo build --release
```

### Compile WASM to JAM

```bash
# From WAT (WebAssembly Text) file
cargo run -p wasm-pvm-cli -- compile tests/fixtures/wat/add.jam.wat -o output.jam

# From WASM binary
cargo run -p wasm-pvm-cli -- compile input.wasm -o output.jam
```

### Run on PVM Interpreter

Requires Bun and the anan-as PVM implementation (included as submodule):

```bash
# Setup (first time only)
cd vendor/anan-as && npm ci && npm run build && cd ../..

# Run with arguments (little-endian u32s)
cd tests && bun utils/run-jam.ts output.jam --args=0500000007000000

# Example: add.jam.wat with args 5 and 7 -> returns 12
```

## WASM Program Convention

WASM programs follow the SPI entrypoint convention. The entry function returns `(i32, i32)` for `(ptr, len)`:

```wat
(module
  (memory 1)
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (i32.const 0x30100)  ;; result pointer
    (i32.const 4)        ;; result length
  )
)
```

### Memory Layout

| Address | Region |
|---------|--------|
| `0x00010000` | Read-only data (dispatch table for `call_indirect`) |
| `0x00030000` | Globals storage (compiler-managed) |
| `0x00030100` | User heap |
| `0x00040000` | Spilled locals (512 bytes per function) |
| `0x00050000+` | WASM linear memory base (data sections placed here) |
| `0xFEFE0000` | Stack segment end |
| `0xFEFF0000` | Arguments (input data) |
| `0xFFFF0000` | EXIT address (HALT) |

See `crates/wasm-pvm/src/translate/memory_layout.rs` for the full layout with ASCII art diagram.

## Project Structure

```
crates/
  wasm-pvm/              # Core library
    src/
      llvm_frontend/     # WASM -> LLVM IR translation
        function_builder.rs  # Core translator (~1350 lines)
      llvm_backend/      # LLVM IR -> PVM bytecode lowering
        lowering.rs      # Core lowering (~1900 lines)
      translate/         # Compilation orchestration
        mod.rs           # Pipeline dispatch + SPI assembly
        wasm_module.rs   # WASM section parsing
        memory_layout.rs # PVM memory address constants
      pvm/               # PVM instruction definitions
      spi.rs             # JAM format encoder
  wasm-pvm-cli/          # Command-line tool
tests/                   # Integration tests & tooling
  fixtures/
    wat/                 # WAT test programs (43 fixtures)
    assembly/            # AssemblyScript examples
  utils/                 # Utility scripts (run-jam, verify-jam)
  helpers/               # Test helpers
  data/                  # Test definitions
vendor/
  anan-as/               # PVM reference interpreter (submodule)
```

## Documentation

- [LEARNINGS.md](./LEARNINGS.md) - Technical reference (PVM architecture, conventions)
- [AGENTS.md](./AGENTS.md) - Guidelines for AI agents working on this project
- [gp-0.7.2.md](./gp-0.7.2.md) - Gray Paper (JAM/PVM specification)
- [review/](./review/) - Architecture review (2026-02-09)

## Testing

```bash
# Run Rust unit tests
cargo test

# Run clippy
cargo clippy -- -D warnings

# Run full integration test suite (360 tests)
cd tests && bun test

# Test a single example
cargo run -p wasm-pvm-cli --quiet -- compile tests/fixtures/wat/factorial.jam.wat -o /tmp/test.jam
cd tests && bun utils/run-jam.ts /tmp/test.jam --args=05000000
```

## License

[MIT](./LICENSE)

## Contributing

See [AGENTS.md](./AGENTS.md) for coding guidelines and project conventions.
