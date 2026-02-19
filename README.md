# WASM-PVM: WebAssembly to PolkaVM Recompiler

A Rust compiler that translates WebAssembly (WASM) bytecode to PolkaVM (PVM) bytecode for execution on the JAM (Join-Accumulate Machine) protocol.

## Status: Active Development (370+ integration tests passing)

**Project Goal**: Enable writing JAM programs in AssemblyScript (TypeScript-like) or hand-written WAT, compiled to PVM bytecode.

**Architecture**: `WASM → [adapter merge] → [inkwell] → LLVM IR → [mem2reg] → [Rust PVM backend] → PVM bytecode`

The compiler uses LLVM 18 (via inkwell) as its intermediate representation, with a custom Rust-based PVM backend that reads LLVM IR and emits PVM bytecode. This gives us LLVM's SSA/CFG representation and optimization passes without requiring a native LLVM C++ target backend. PVM-specific intrinsic functions (`@__pvm_load_i32`, `@__pvm_store_i32`, etc.) are used for memory operations to avoid `unsafe` code.

**Current State**:
- All 370+ TypeScript integration tests and all Rust unit tests pass
- anan-as (PVM interpreter in AssemblyScript) compiles to a ~441KB JAM file
- `unsafe_code = "deny"` enforced at workspace level

### Working Features

**Arithmetic (i32 & i64)**: add, sub, mul, div_u, div_s, rem_u, rem_s, all comparisons (eq, ne, lt_u/s, gt_u/s, le_u/s, ge_u/s, eqz), clz, ctz, popcnt, rotl, rotr, bitwise (and, or, xor, shl, shr_u, shr_s)

**Control Flow**: block, loop, if/else/end, br, br_if, br_table, return, unreachable, block result values

**Memory**: i32/i64 load/store, sub-word variants (load8_u/s, load16_u/s, load32_u/s, store8, store16, store32), memory.size, memory.grow, memory.fill, memory.copy, global.get/set, data section initialization

**Functions**: call, call_indirect (with signature validation), recursion, stack overflow detection (64KB default), local variables with spilling, local.get/set/tee, drop, select

**Type Conversions**: i32.wrap_i64, i64.extend_i32_s/u, sign extensions (i32.extend8_s, i32.extend16_s, i64.extend8_s/16_s/32_s)

**Import Handling**: Text-based import maps (`--imports`) for simple mappings (trap, nop), and WAT adapter files (`--adapter`) for complex import resolution with arbitrary WASM logic (pointer conversion, memory reads, multi-arg host calls)

### Not Yet Implemented
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

# With import map and adapter (see "Import Handling" below)
cargo run -p wasm-pvm-cli -- compile input.wasm -o output.jam \
  --imports imports.txt --adapter adapter.wat
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

WASM programs follow the SPI entrypoint convention. The entry function receives `(args_ptr, args_len)` and communicates results via exported globals `result_ptr` and `result_len`:

```wat
(module
  (memory 1)
  (global $result_ptr (export "result_ptr") (mut i32) (i32.const 0))
  (global $result_len (export "result_len") (mut i32) (i32.const 0))
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    ;; Use memory.grow or heap.alloc (in AS) for result buffers.
    ;; Do NOT hardcode addresses — use dynamic allocation.
  )
)
```

In AssemblyScript, use `heap.alloc(size)` to allocate result buffers:

```typescript
export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  const buf = heap.alloc(256);
  const a = load<i32>(args_ptr);
  const b = load<i32>(args_ptr + 4);
  store<i32>(buf, a + b);
  result_ptr = buf as i32;
  result_len = 4;
}
```

### Memory Layout

| Address | Region |
|---------|--------|
| `0x00010000` | Read-only data (dispatch table, passive segments) |
| `0x00030000` | Globals storage (compiler-managed, 4 bytes per global) |
| `0x0003FF00` | Parameter overflow area (5th+ args for `call_indirect`) |
| `0x00040000` | Spilled locals (512 bytes per function) |
| `0x00050000+` | WASM linear memory base (computed dynamically) |
| `0xFEFE0000` | Stack segment end (stack grows downward) |
| `0xFEFF0000` | Arguments (input data) |
| `0xFFFF0000` | EXIT address (HALT) |

User programs should use `heap.alloc()` (AssemblyScript) or `memory.grow` (WASM) for dynamic memory allocation. See `crates/wasm-pvm/src/translate/memory_layout.rs` for the full layout.

## Import Handling

WASM modules that import external functions (e.g., `abort`, `console.log`) need those imports resolved before compilation. The compiler provides two mechanisms that can be used independently or together.

### Import Map (`--imports`)

A text file mapping import names to simple actions. Use for straightforward cases like trapping or firing a host call.

**Format**: one mapping per line, `name = action`. Lines starting with `#` are comments.

```text
# my-imports.txt
abort = trap
console.log = nop
```

**Available actions**:

| Action | Effect |
|--------|--------|
| `trap` | Emit an `unreachable` (panic) when called |
| `nop` | Do nothing and return zero |

**Usage**:

```bash
wasm-pvm compile input.wasm -o output.jam --imports my-imports.txt
```

### Adapter WAT (`--adapter`)

A WAT (WebAssembly Text) module whose exported functions replace matching imports in the main module. The adapter is merged into the main WASM at the binary level before compilation, enabling arbitrary WASM logic for import resolution.

Use adapters when you need to:
- Convert WASM pointers to PVM addresses
- Read memory (e.g., string lengths from object headers)
- Compute derived values or restructure arguments
- Map one import to a multi-register host call

**Adapter format**:

```wat
(module
  ;; Import compiler intrinsics (recognized automatically):
  (import "env" "host_call" (func $host_call (param i64 i64 i64 i64 i64 i64)))
  (import "env" "pvm_ptr" (func $pvm_ptr (param i64) (result i64)))

  ;; Each export replaces the matching import in the main module.
  ;; The export name must match the import name, and the type signature
  ;; must match the original import's signature.

  (func (export "abort") (param i32 i32 i32 i32)
    unreachable
  )

  (func (export "console.log") (param i32)
    ;; Map to JIP-1 logging host call (ecalli 100)
    (call $host_call
      (i64.const 100)                                    ;; ecalli index
      (i64.const 3)                                      ;; r7: log level
      (i64.const 0)                                      ;; r8: target pointer
      (i64.const 0)                                      ;; r9: target length
      (call $pvm_ptr (i64.extend_i32_u (local.get 0)))   ;; r10: PVM message pointer
      (i64.extend_i32_u (i32.load offset=0               ;; r11: message byte length
        (i32.sub (local.get 0) (i32.const 4))))           ;;   read from AS header at ptr-4
    )
  )
)
```

**Available intrinsics** (imported by adapters from `"env"`):

| Intrinsic | Signature | Effect |
|-----------|-----------|--------|
| `host_call` | `(i64, i64, i64, i64, i64, i64) → void` | First arg = ecalli index, remaining 5 args map to registers r7–r11 |
| `pvm_ptr` | `(i64) → i64` | Converts a WASM address to a PVM address (adds `wasm_memory_base`) |

**Usage**:

```bash
wasm-pvm compile input.wasm -o output.jam --adapter my-adapter.wat
```

### Composing Both Mechanisms

When both `--imports` and `--adapter` are provided, the adapter merge runs first (resolving some imports), then the import map handles any remaining unresolved imports. This lets you use adapters for complex cases and simple text mappings for everything else.

```bash
wasm-pvm compile input.wasm -o output.jam \
  --adapter adapter.wat \
  --imports remaining.txt
```

**Import validation**: The compiler requires all imports to be resolved. Any import not handled by an adapter, an import map, or a known intrinsic (`host_call`, `pvm_ptr`) will produce a compilation error. The sole exception is `abort`, which is mapped to `trap` by default when no explicit import map is provided.

### Auto-discovery in Tests

The test build system (`tests/build.ts`) automatically discovers import files for AssemblyScript sources:
- `tests/fixtures/imports/<name>.adapter.wat` → passed as `--adapter`
- `tests/fixtures/imports/<name>.imports` → passed as `--imports`

where `<name>` matches the AS source filename (without `.ts`).

## Project Structure

```
crates/
  wasm-pvm/              # Core library
    src/
      llvm_frontend/     # WASM -> LLVM IR translation
        function_builder.rs  # Core translator (~1350 lines)
      llvm_backend/      # LLVM IR -> PVM bytecode lowering (see ARCHITECTURE.md)
        mod.rs           # Public API + instruction dispatch
        emitter.rs       # PvmEmitter struct + value management
        alu.rs           # Arithmetic, logic, comparisons, conversions
        memory.rs        # Load/store, memory intrinsics
        control_flow.rs  # Branches, phi nodes, switch, return
        calls.rs         # Direct/indirect calls, import stubs
        intrinsics.rs    # PVM + LLVM intrinsic lowering
      translate/         # Compilation orchestration
        mod.rs           # Pipeline dispatch + SPI assembly
        adapter_merge.rs # WAT adapter merge into WASM before compilation
        wasm_module.rs   # WASM section parsing
        memory_layout.rs # PVM memory address constants
      pvm/               # PVM instruction definitions
      spi.rs             # JAM format encoder
  wasm-pvm-cli/          # Command-line tool
tests/                   # Integration tests & tooling
  fixtures/
    wat/                 # WAT test programs (43 fixtures)
    assembly/            # AssemblyScript examples (~64 files)
    imports/             # Import maps (.imports) and adapter WAT files (.adapter.wat)
  utils/                 # Utility scripts (run-jam, verify-jam)
  helpers/               # Test helpers (compile.ts, run.ts)
  data/                  # Test definitions
vendor/
  anan-as/               # PVM reference interpreter (submodule)
```

## Documentation

- [ARCHITECTURE.md](./ARCHITECTURE.md) - Register conventions, calling convention, stack frame layout, memory map
- [LEARNINGS.md](./LEARNINGS.md) - Technical reference (PVM architecture, debugging journal)
- [AGENTS.md](./AGENTS.md) - Guidelines for AI agents working on this project
- [gp-0.7.2.md](./gp-0.7.2.md) - Gray Paper (JAM/PVM specification)
- [review/](./review/) - Architecture review (2026-02-09)

## Testing

```bash
# Run Rust unit tests
cargo test

# Run clippy
cargo clippy -- -D warnings

# IMPORTANT: Build test artifacts first!
cd tests && bun build.ts

# Run full integration test suite
# Use `bun run test` (builds then tests), NOT `bun test` (tests only)
cd tests && bun run test

# Quick development check - Layer 1 tests only (fast)
cd tests && bun test layer1/

# Test a single example
cargo run -p wasm-pvm-cli --quiet -- compile tests/fixtures/wat/factorial.jam.wat -o /tmp/test.jam
cd tests && bun utils/run-jam.ts /tmp/test.jam --args=05000000
```

### Test Organization & Workflow

The test suite is organized into five layers:

- **Layer 1** (`layer1/`): Core/smoke tests (~50 tests) - Fast, run these during development
- **Layer 2** (`layer2/`): Feature tests (~100 tests) - Extended functionality
- **Layer 3** (`layer3/`): Regression/edge cases (~220 tests) - Bug fixes and edge cases
- **Layer 4** (`layer4/`): PVM-in-PVM smoke tests (3 tests) - Quick pvm-in-pvm sanity check
- **Layer 5** (`layer5/`): Comprehensive PVM-in-PVM tests - Full test suite running inside the PVM interpreter

**Development workflow**:
1. Make your changes to the Rust code
2. Run Rust unit tests: `cargo test`
3. Build test artifacts: `cd tests && bun build.ts`
4. Quick validation: `cd tests && bun test layer1/` (fast)
5. Full validation before committing: `cd tests && bun run test` (~20 seconds)

**Note**: `bun run test` from the `tests/` directory runs `bun build.ts && bun test`, ensuring JAM files are always up-to-date. `bun test` alone will fail if the test artifacts haven't been built. If you've changed AS fixture source files (`.ts`), delete cached WASM first: `rm -f tests/build/wasm/*.wasm`.

**PVM-in-PVM Testing**:
- Layer 4 runs a quick sanity check (3 tests) to verify the PVM interpreter works when compiled to PVM
- Layer 5 runs all compatible tests inside the PVM interpreter (some suites are skipped due to timeouts or unhandled host calls)
- Run with: `cd tests && bun test layer4/ layer5/ --test-name-pattern "pvm-in-pvm"`
- In CI, PVM-in-PVM tests run in a separate job after regular integration tests pass

## License

[MIT](./LICENSE)

## Contributing

See [AGENTS.md](./AGENTS.md) for coding guidelines and project conventions.
