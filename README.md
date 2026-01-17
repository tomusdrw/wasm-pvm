# WASM-PVM: WebAssembly to PolkaVM Recompiler

A Rust compiler that translates WebAssembly (WASM) bytecode to PolkaVM (PVM) bytecode for execution on the JAM (Join-Accumulate Machine) protocol.

## Status: Early Development

**Project Goal**: Enable writing JAM programs in AssemblyScript (TypeScript-like) or hand-written WAT, compiled to PVM bytecode.

**Working features:**
- Basic WASM parsing and translation
- Integer arithmetic (`add`, `sub`, `mul`)
- Control flow (`block`, `loop`, `br`, `br_if`)
- Comparison operations (`gt_u`, `gt_s`, `lt_u`, `lt_s`, `ge_u`, `le_u`, `eqz`)
- Memory operations (`i32.load`, `i32.store`)
- Global variables (`global.get`, `global.set`)
- SPI (Standard Program Interface) output format for JAM

**Current priority**: AssemblyScript examples and test suite.

**Not yet implemented:**
- `if/else/end` control flow
- Division, remainder, bitwise operations
- Function calls (`call`, `call_indirect`)
- Floating point (will be rejected - PVM has no FP support)

## Quick Start

### Build

```bash
cargo build --release
```

### Compile WASM to PVM

```bash
# From WAT (WebAssembly Text) file
cargo run -p wasm-pvm-cli -- compile examples-wat/add-spi.wat -o output.spi

# From WASM binary
cargo run -p wasm-pvm-cli -- compile input.wasm -o output.spi
```

### Run on PVM Interpreter

Requires Node.js and the anan-as PVM implementation (included as submodule):

```bash
# Setup (first time only)
cd vendor/anan-as && npm ci && npm run build && cd ../..

# Run with arguments (little-endian u32s)
npx tsx scripts/run-spi.ts output.spi --args=05000000070000000

# Example: add-spi.wat with args 5 and 7 → returns 12
npx tsx scripts/run-spi.ts output.spi --args=0500000007000000
# Output: r7=0x20100 (result address), r8=4 (result length)
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
| `add-spi.wat` | Add two u32 arguments | 5+7=12 |
| `factorial-spi.wat` | Compute n! using loop | 5!=120 |

## Project Structure

```
crates/
  wasm-pvm/           # Core library
    src/
      pvm/            # PVM instruction definitions
      translate/      # WASM → PVM translation
      spi.rs          # SPI format encoder
  wasm-pvm-cli/       # Command-line tool
examples-wat/         # Example WASM programs (WAT format)
scripts/
  run-spi.ts          # PVM test runner
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
