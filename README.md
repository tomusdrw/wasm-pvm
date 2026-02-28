# WASM-PVM: WebAssembly to PolkaVM Recompiler

> **WARNING: This project is largely vibe-coded.**
> It was built iteratively with heavy AI assistance (Claude). While it has 412 passing integration tests and
> produces working PVM bytecode, the internals may contain unconventional patterns, over-engineering in some
> places, and under-engineering in others. Use at your own risk. Contributions and proper engineering reviews
> are very welcome!

A Rust compiler that translates WebAssembly (WASM) bytecode into [PolkaVM](https://github.com/paritytech/polkavm) (PVM) bytecode for execution on the [JAM](https://graypaper.com/) (Join-Accumulate Machine) protocol. Write your JAM programs in [AssemblyScript](https://www.assemblyscript.org/) (TypeScript-like), hand-written WAT, or any language that compiles to WASM — and run them on PVM.

```text
WASM  ──►  LLVM IR  ──►  PVM bytecode  ──►  JAM program (.jam)
      inkwell    mem2reg       Rust backend
```

## Getting Started

### Prerequisites

- **Rust** (stable, edition 2024)
- **LLVM 18** — the compiler uses [inkwell](https://github.com/TheDan64/inkwell) (LLVM 18 bindings)
  - macOS: `brew install llvm@18` then `export LLVM_SYS_181_PREFIX=/opt/homebrew/opt/llvm@18`
  - Ubuntu: `apt install llvm-18-dev`
- **Bun** (for running integration tests and the JAM runner) — [bun.sh](https://bun.sh)

### Build

```bash
git clone https://github.com/tomusdrw/wasm-pvm.git
cd wasm-pvm
cargo build --release
```

### Hello World: Compile & Run

Create a simple WAT program that adds two numbers:

```wat
;; add.wat
(module
  (memory 1)
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    ;; Read two i32 args, add them, write result to memory
    (i32.store (i32.const 0)
      (i32.add
        (i32.load (local.get $args_ptr))
        (i32.load (i32.add (local.get $args_ptr) (i32.const 4)))))
    (i32.const 0)   ;; result pointer
    (i32.const 4))) ;; result length
```

Compile it to a JAM blob and run it:

```bash
# Compile WAT → JAM
cargo run -p wasm-pvm-cli -- compile add.wat -o add.jam

# Run with two u32 arguments: 5 and 7 (little-endian hex)
npx @fluffylabs/anan-as run add.jam 0500000007000000
# Output: 0c000000  (12 in little-endian)
```

### Inspect the Output

Upload the resulting `.jam` file to the [**PVM Debugger**](https://github.com/fluffylabs/pvm-debugger) for step-by-step execution, disassembly, register inspection, and gas metering visualization.

### AssemblyScript Example

You can also write programs in AssemblyScript:

```typescript
// fibonacci.ts
export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  const buf = heap.alloc(256);
  let n = load<i32>(args_ptr);
  let a: i32 = 0;
  let b: i32 = 1;

  while (n > 0) {
    b = a + b;
    a = b - a;
    n = n - 1;
  }

  store<i32>(buf, a);
  result_ptr = buf as i32;
  result_len = 4;
}
```

Compile via the AssemblyScript compiler to WASM, then use `wasm-pvm-cli` to produce a JAM blob. See the `tests/fixtures/assembly/` directory for more examples.

## How It Works

The compiler pipeline:

1. **Adapter merge** (optional) — merges a WAT adapter module into the WASM binary, replacing matching imports with adapter function bodies
2. **WASM → LLVM IR** — translates WASM opcodes to LLVM IR using [inkwell](https://github.com/TheDan64/inkwell) (LLVM 18 bindings), with PVM-specific intrinsics for memory operations
3. **LLVM optimization passes** — `mem2reg` (SSA promotion), `instcombine`, `simplifycfg`, `gvn`, `dce`, and optional function inlining
4. **LLVM IR → PVM bytecode** — a custom Rust backend reads LLVM IR and emits PVM instructions with per-block register caching (store-load forwarding)
5. **SPI assembly** — packages the bytecode into a JAM/SPI program blob with entry headers, jump tables, and data sections

### Key Design Decisions

- **Stack-slot approach**: every SSA value gets a dedicated 8-byte memory offset from SP, with a **linear-scan register allocator** that assigns long-lived values to r5/r6 to eliminate redundant memory traffic across block boundaries and loops
- **Per-block register cache**: eliminates redundant loads when a value is reused shortly after being computed (~50% gas reduction)
- **No `unsafe` code**: `deny(unsafe_code)` enforced at workspace level
- **No floating point**: PVM lacks FP support; WASM floats are rejected at compile time
- **All optimizations are toggleable**: `--no-llvm-passes`, `--no-peephole`, `--no-register-cache`, `--no-icmp-fusion`, `--no-shrink-wrap`, `--no-dead-store-elim`, `--no-const-prop`, `--no-inline`, `--no-cross-block-cache`, `--no-register-alloc`, `--no-fallthrough-jumps`

### Benchmark: Optimizations Impact

All PVM-level optimizations enabled (default):

| Benchmark | WASM size | JAM size | Code size | Gas Used |
|-----------|----------|----------|-----------|----------|
| add(5,7) | 66 B | 201 B | 130 B | 39 |
| fib(20) | 108 B | 270 B | 186 B | 612 |
| factorial(10) | 100 B | 242 B | 161 B | 269 |
| is_prime(25) | 160 B | 327 B | 238 B | 78 |
| AS fib(10) | 266 B | 712 B | 576 B | 325 |
| AS factorial(7) | 265 B | 701 B | 566 B | 282 |
| AS gcd(2017,200) | 260 B | 691 B | 562 B | 191 |
| AS decoder | 1.5 KB | 21.0 KB | 7.0 KB | 751 |
| AS array | 1.4 KB | 20.1 KB | 6.2 KB | 648 |
| anan-as PVM interpreter | 58.3 KB | 179.5 KB | 127.2 KB | - |

PVM-in-PVM: programs executed inside the anan-as PVM interpreter (outer gas cost):

| Benchmark | JAM Size | Code Size | Outer Gas | Direct Gas | Overhead |
|-----------|----------|-----------|-----------|------------|----------|
| TRAP (interpreter overhead) | 21 B | 1 B | 22,470 | - | - |
| add(5,7) | 201 B | 130 B | 1,176,696 | 39 | 30,172x |
| AS fib(10) | 712 B | 576 B | 1,723,810 | 325 | 5,304x |
| JAM-SDK fib(10)\* | 25.4 KB | 16.2 KB | 6,679,366 | 42 | 159,509x |
| Jambrains fib(10)\* | 61.1 KB | - | 6,477,292 | 1 | 6,477,292x |
| JADE fib(10)\* | 67.3 KB | 45.7 KB | 18,193,275 | 504 | 36,098x |

\*JAM-SDK fib(10), Jambrains fib(10), and JADE fib(10) exit on unhandled host calls before the fibonacci computation runs. The gas cost reflects program parsing/loading only (26 KB, 61 KB, and 67 KB binaries respectively), not execution.

## Memory layout summary

The JAM blob reserves separate ranges for RO data, a guard gap, globals/overflow metadata, and the WASM heap; see [`ARCHITECTURE.md`](ARCHITECTURE.md#memory-layout) for the full breakdown, including `GLOBAL_MEMORY_BASE`, `PARAM_OVERFLOW_BASE`, `SPILLED_LOCALS_BASE`, and how `wasm_memory_base` is computed.

The SPI `rw_data` section is simply a contiguous copy of every byte from `GLOBAL_MEMORY_BASE` up to the highest initialized heap address, which is why stub AssemblyScript fixtures such as `decoder-test`/`array-test` emit ~13 KB of RW data even though only a handful of bytes are non-zero: the encoder must preserve the absolute addresses of the data segments, so the zero stretch between globals and the first heap byte is encoded verbatim. Keeping globals/data near the heap base or introducing sparse RW descriptors (future work) are the only ways to shrink those blobs without redesigning SPI.

## Supported WASM Features

| Category | Operations |
|----------|-----------|
| **Arithmetic** (i32 & i64) | add, sub, mul, div_u/s, rem_u/s, all comparisons, clz, ctz, popcnt, rotl, rotr, bitwise ops |
| **Control flow** | block, loop, if/else, br, br_if, br_table, return, unreachable, block results |
| **Memory** | load/store (all widths), memory.size, memory.grow, memory.fill, memory.copy, globals, data sections |
| **Functions** | call, call_indirect (with signature validation), recursion, stack overflow detection |
| **Type conversions** | wrap, extend_s/u, sign extensions (i32/i64 extend8/16/32_s) |
| **Imports** | Text-based import maps (`--imports`) and WAT adapter files (`--adapter`) |

**Not supported**: floating point (by design — PVM has no FP instructions).

## CLI Usage

```bash
# Compile WAT or WASM to JAM
wasm-pvm compile input.wat -o output.jam
wasm-pvm compile input.wasm -o output.jam

# With import resolution
wasm-pvm compile input.wasm -o output.jam \
  --imports imports.txt \
  --adapter adapter.wat

# Disable specific optimizations
wasm-pvm compile input.wasm -o output.jam --no-inline --no-peephole

# Disable all optimizations
wasm-pvm compile input.wasm -o output.jam \
  --no-llvm-passes --no-peephole --no-register-cache \
  --no-icmp-fusion --no-shrink-wrap --no-dead-store-elim \
  --no-const-prop --no-inline --no-cross-block-cache \
  --no-register-alloc
```

See the [Import Handling](#import-handling) section for details on resolving WASM imports.

## Project Structure

```text
crates/
  wasm-pvm/              # Core compiler library
    src/
      llvm_frontend/     # WASM → LLVM IR translation
      llvm_backend/      # LLVM IR → PVM bytecode lowering
      translate/         # Compilation orchestration & SPI assembly
      pvm/               # PVM instruction definitions & peephole optimizer
  wasm-pvm-cli/          # Command-line interface
tests/                   # 412 integration tests (TypeScript/Bun)
  fixtures/
    wat/                 # WAT test programs
    assembly/            # AssemblyScript examples
    imports/             # Import maps & adapter files
vendor/
  anan-as/               # PVM interpreter (submodule)
```

## Testing

```bash
# Rust unit tests
cargo test

# Lint
cargo clippy -- -D warnings

# Integration tests (builds artifacts, then runs all layers)
cd tests && bun run test

# Quick validation (Layer 1 smoke tests only)
cd tests && bun test layer1/
```

The test suite is organized into layers:

- **Layer 1**: Core/smoke tests (~50 tests) — fast, run during development
- **Layer 2**: Feature tests (~140 tests)
- **Layer 3**: Regression/edge cases (~220 tests)
- **Layer 4-5**: PVM-in-PVM tests — the PVM interpreter itself compiled to PVM, running the test suite inside PVM

## Import Handling

WASM modules that import external functions need those imports resolved before compilation. Two mechanisms are available:

### Import Map (`--imports`)

A text file mapping import names to simple actions:

```text
# my-imports.txt
abort = trap        # emit unreachable (panic)
console.log = nop   # do nothing, return zero
```

### Adapter WAT (`--adapter`)

A WAT module whose exports replace matching imports, enabling arbitrary logic for import resolution (pointer conversion, memory reads, host calls):

```wat
(module
  (import "env" "host_call" (func $host_call (param i64 i64 i64 i64 i64 i64)))
  (import "env" "pvm_ptr" (func $pvm_ptr (param i64) (result i64)))

  (func (export "console.log") (param i32)
    (call $host_call
      (i64.const 100)                                    ;; ecalli index
      (i64.const 3)                                      ;; log level
      (i64.const 0) (i64.const 0)                        ;; target ptr/len
      (call $pvm_ptr (i64.extend_i32_u (local.get 0)))   ;; message ptr
      (i64.extend_i32_u (i32.load offset=0
        (i32.sub (local.get 0) (i32.const 4))))))        ;; message len
)
```

When both `--imports` and `--adapter` are provided, the adapter runs first, then the import map handles remaining unresolved imports. All imports must be resolved or compilation fails.

## Resources

- **[PVM Debugger](https://github.com/fluffylabs/pvm-debugger)** — upload `.jam` files for disassembly, step-by-step execution, and register/gas inspection
- **[JAM Gray Paper](https://graypaper.com/)** — the JAM protocol specification (PVM is defined in Appendix A)
- **[ananas (anan-as)](https://github.com/tomusdrw/anan-as)** — the reference PVM implementation
- **[AssemblyScript](https://www.assemblyscript.org/)** — TypeScript-like language that compiles to WASM
- **[ARCHITECTURE.md](./ARCHITECTURE.md)** — register conventions, calling convention, stack frame layout, memory map
- **[LEARNINGS.md](./LEARNINGS.md)** — technical reference and debugging journal

## License

[MIT](./LICENSE)

## Contributing

Contributions are welcome! See [AGENTS.md](./AGENTS.md) for coding guidelines, project conventions, and a map of the codebase.
