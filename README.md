# WASM-PVM: WebAssembly to PolkaVM Recompiler

A Rust compiler that translates WebAssembly (WASM) bytecode to PolkaVM (PVM) bytecode for execution on the JAM (Join-Accumulate Machine) protocol.

## Status: Early Development

**Project Goal**: Enable writing JAM programs in AssemblyScript (TypeScript-like) or hand-written WAT, compiled to PVM bytecode.

**Working features:**
- Basic WASM parsing and translation
- Integer arithmetic (`add`, `sub`, `mul`, `rem_u`)
- Control flow (`block`, `loop`, `if/else`, `br`, `br_if`)
- Comparison operations (`gt_u`, `gt_s`, `lt_u`, `lt_s`, `ge_u`, `le_u`, `le_s`, `eq`, `eqz`)
- Memory operations (`i32.load`, `i32.store`)
- Global variables (`global.get`, `global.set`)
- JAM output format (`.jam` files)

**Current priority**: AssemblyScript examples and test suite.

**Not yet implemented:**
- Division, bitwise AND/OR operations
- Function calls (`call`, `call_indirect`)
- Floating point (will be rejected - PVM has no FP support)

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
    ;; Write output to heap (0x20100+)
    ;; Set result_ptr and result_len globals
    
    (global.set $result_ptr (i32.const 0x20100))
    (global.set $result_len (i32.const 4))
  )
)
```

### Memory Layout

| Address | Region |
|---------|--------|
| `0x00010000` | Read-only data |
| `0x00020000` | Globals storage (compiler-managed) |
| `0x00020100` | User heap (for results) |
| `0xFEFE0000` | Stack end |
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

# Test compilation of an example
cargo run -p wasm-pvm-cli -- compile examples-wat/factorial-spi.wat -o /tmp/test.spi
npx tsx scripts/run-spi.ts /tmp/test.spi --args=05000000
```

## License

[MIT](./LICENSE) (TODO: Add license file)

## Contributing

See [AGENTS.md](./AGENTS.md) for coding guidelines and project conventions.
