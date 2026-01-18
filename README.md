# WASM-PVM: WebAssembly to PolkaVM Recompiler

A Rust compiler that translates WebAssembly (WASM) bytecode to PolkaVM (PVM) bytecode for execution on the JAM (Join-Accumulate Machine) protocol.

## Status: Active Development (58 tests passing)

**Project Goal**: Enable writing JAM programs in AssemblyScript (TypeScript-like) or hand-written WAT, compiled to PVM bytecode.

**V1 Milestone**: Compile [anan-as](https://github.com/polkavm/anan-as) (PVM interpreter in AssemblyScript) to WASM → PVM, and run a PVM interpreter inside a PVM interpreter.

### Working Features

**Arithmetic (i32 & i64)**:
- `add`, `sub`, `mul`, `div_u`, `div_s`, `rem_u`, `rem_s`
- All comparison operations (`eq`, `ne`, `lt_u/s`, `gt_u/s`, `le_u/s`, `ge_u/s`, `eqz`)

**Bitwise & Shift (i32 & i64)**:
- `and`, `or`, `xor`
- `shl`, `shr_u`, `shr_s`
- `rotl`, `rotr`
- `clz`, `ctz`, `popcnt`

**Control Flow**:
- `block`, `loop`, `if/else/end`
- `br`, `br_if`, `br_table`
- `return`, `unreachable`
- Block result values

**Memory Operations**:
- `i32.load/store`, `i64.load/store`
- Sub-word variants: `load8_u/s`, `load16_u/s`, `load32_u/s`, `store8`, `store16`, `store32`
- `memory.size`, `memory.grow` (returns -1)
- `global.get`, `global.set`

**Functions**:
- `call` with proper return value handling
- `call_indirect` (indirect function calls via table)
- Recursion support with proper call stack
- Local variables with spilling for functions with many locals
- `local.get`, `local.set`, `local.tee`
- `drop`, `select`

**Type Conversions**:
- `i32.wrap_i64`
- `i64.extend_i32_s`, `i64.extend_i32_u`

### Not Yet Implemented
- Data section initialization (WASM data segments)
- Floating point (rejected by design - PVM has no FP)
- Stack overflow detection for deep recursion
- Runtime signature validation for `call_indirect`

## Quick Start

### Build

```bash
cargo build --release
```

### Compile WASM to JAM

```bash
# From WAT (WebAssembly Text) file
cargo run -p wasm-pvm-cli -- compile examples-wat/add.jam.wat -o output.jam

# From WASM binary
cargo run -p wasm-pvm-cli -- compile input.wasm -o output.jam
```

### Run on PVM Interpreter

Requires Node.js and the anan-as PVM implementation (included as submodule):

```bash
# Setup (first time only)
cd vendor/anan-as && npm ci && npm run build && cd ../..

# Run with arguments (little-endian u32s)
npx tsx scripts/run-jam.ts output.jam --args=0500000007000000

# Example: add.jam.wat with args 5 and 7 → returns 12
npx tsx scripts/run-jam.ts output.jam --args=0500000007000000
# Output shows: As U32: 12
```

## WASM Program Convention

WASM programs must follow the SPI entrypoint convention:

```wat
(module
  (memory 1)
  
  ;; Required globals for return value
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  ;; Entry point - receives args pointer and length
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    ;; Read input from args_ptr (PVM address 0xFEFF0000)
    ;; Write output to heap (0x30100+)
    ;; Set result_ptr and result_len globals
    
    (global.set $result_ptr (i32.const 0x30100))
    (global.set $result_len (i32.const 4))
  )
)
```

### Memory Layout

| Address | Region |
|---------|--------|
| `0x00010000` | Read-only data (dispatch table for call_indirect) |
| `0x00030000` | Globals storage (compiler-managed) |
| `0x00030100` | User results area (256 bytes) |
| `0x00030200` | Spilled locals (512 bytes per function) |
| `0xFEFE0000` | Stack segment end |
| `0xFEFF0000` | Arguments (input data) |
| `0xFFFF0000` | EXIT address (HALT) |

## Examples

Working examples in `examples-wat/`:

| File | Description | Verified |
|------|-------------|----------|
| `add.jam.wat` | Add two u32 arguments | 5+7=12 |
| `factorial.jam.wat` | Compute n! using loop | 5!=120 |
| `fibonacci.jam.wat` | Fibonacci sequence | fib(10)=55 |
| `gcd.jam.wat` | GCD (Euclidean algorithm) | gcd(48,18)=6 |
| `is-prime.jam.wat` | Primality test | is_prime(97)=1 |
| `div.jam.wat` | Integer division | 20/5=4 |
| `call.jam.wat` | Function calls | double(5)=10 |
| `br-table.jam.wat` | Switch/jump table | br_table tests |
| `bit-ops.jam.wat` | clz, ctz, popcnt | bit operation tests |
| `rotate.jam.wat` | rotl, rotr | rotation tests |
| `entry-points.jam.wat` | Multiple entry points (main/main2) | PC=0 and PC=5 |
| `recursive.jam.wat` | Recursive factorial | tests call stack |
| `nested-calls.jam.wat` | Nested function calls | multi-level calls |
| `call-indirect.jam.wat` | Indirect function calls via table | dispatch tests |
| `i64-ops.jam.wat` | 64-bit integer operations | div, rem, shifts |
| `many-locals.jam.wat` | Functions with >4 local variables | spilling tests |
| `block-result.jam.wat` | Block result values | control flow |
| `block-br-test.jam.wat` | Block branch tests | br/br_if |
| `stack-test.jam.wat` | Operand stack tests | stack depth |

AssemblyScript examples in `examples-as/`:

| File | Description | Verified |
|------|-------------|----------|
| `add.ts` | Add two u32 arguments | 5+7=12 |
| `factorial.ts` | Compute n! using loop | 5!=120 |
| `fibonacci.ts` | Fibonacci sequence | fib(10)=55 |
| `gcd.ts` | GCD (Euclidean algorithm) | gcd(48,18)=6 |

## Project Structure

```
crates/
  wasm-pvm/           # Core library
    src/
      pvm/            # PVM instruction definitions
      translate/      # WASM → PVM translation
      spi.rs          # JAM format encoder
  wasm-pvm-cli/       # Command-line tool
examples-wat/         # Example WASM programs (*.jam.wat)
examples-as/          # AssemblyScript examples
scripts/
  run-jam.ts          # PVM test runner
  test-all.ts         # Automated test suite
vendor/
  anan-as/            # PVM reference interpreter (submodule)
```

## Documentation

- [PLAN.md](./PLAN.md) - Project roadmap and current progress
- [LEARNINGS.md](./LEARNINGS.md) - Technical discoveries and PVM instruction reference
- [KNOWN_ISSUES.md](./KNOWN_ISSUES.md) - Known bugs and improvements to address
- [AGENTS.md](./AGENTS.md) - Guidelines for AI agents working on this project
- [gp-0.7.2.md](./gp-0.7.2.md) - Gray Paper (JAM/PVM specification)

## Testing

```bash
# Run unit tests
cargo test

# Run clippy
cargo clippy -- -D warnings

# Run full integration test suite (58 tests)
npx tsx scripts/test-all.ts

# Test a single example
cargo run -p wasm-pvm-cli --quiet -- compile examples-wat/factorial.jam.wat -o /tmp/test.jam
npx tsx scripts/run-jam.ts /tmp/test.jam --args=05000000
```

## License

[MIT](./LICENSE)

## Contributing

See [AGENTS.md](./AGENTS.md) for coding guidelines and project conventions.
