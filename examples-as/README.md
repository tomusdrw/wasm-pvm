# AssemblyScript Examples for JAM/PVM

Write JAM programs in TypeScript-like AssemblyScript, compile to WASM, then to PVM bytecode.

## Quick Start

```bash
# Install dependencies
npm install

# Build all examples
npm run build

# Compile to PVM and run (from project root)
cd ..
cargo run -p wasm-pvm-cli -- compile examples-as/build/add.wasm -o /tmp/add.spi
npx tsx scripts/run-spi.ts /tmp/add.spi --args=0500000007000000
# Result: 5 + 7 = 12
```

## SPI Convention

All programs must follow the SPI (Standard Program Interface) entrypoint convention:

```typescript
const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  // Read input from args_ptr (PVM address 0xFEFF0000)
  const a = load<i32>(args_ptr);
  const b = load<i32>(args_ptr + 4);
  
  // Compute result
  const result = a + b;
  
  // Write result to heap
  store<i32>(RESULT_HEAP, result);
  
  // Set result globals (read by PVM epilogue)
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
```

### Memory Layout

| Address | Region |
|---------|--------|
| `0x00020000` | Globals storage (compiler-managed) |
| `0x00020100` | User heap (write results here) |
| `0xFEFE0000` | Stack end |
| `0xFEFF0000` | Arguments (input data from `args_ptr`) |
| `0xFFFF0000` | EXIT address (HALT) |

### Key Points

1. **Export mutable globals** `result_ptr` and `result_len` - the compiler reads these to return values
2. **Use `load<T>()` and `store<T>()`** for direct memory access (no runtime needed)
3. **Write results to `0x30100`** or higher (user heap region)
4. **Integer types only** - PVM has no floating point support

## Build Configuration

The project uses `--runtime stub` for minimal WASM output:

```bash
asc assembly/add.ts \
  -o build/add.wasm \
  -t build/add.wat \
  --runtime stub \
  --noAssert \
  --optimizeLevel 3 \
  --shrinkLevel 2 \
  --converge \
  --use abort=
```

Key flags:
- `--runtime stub` - Minimal runtime, no GC
- `--noAssert` - Remove assertion overhead
- `--use abort=` - Remove abort import (PVM can't call external functions)

## Examples

| File | Description | Test |
|------|-------------|------|
| `add.ts` | Add two u32 values | `--args=0500000007000000` → 12 |
| `factorial.ts` | Compute n! | `--args=05000000` → 120 |

## Full Workflow

```bash
# 1. Write AssemblyScript (assembly/myprogram.ts)

# 2. Add build script to package.json:
#    "build:myprogram": "asc assembly/myprogram.ts -o build/myprogram.wasm ..."

# 3. Build WASM
npm run build:myprogram

# 4. Inspect generated WAT (optional)
cat build/myprogram.wat

# 5. Compile WASM to PVM SPI format
cargo run -p wasm-pvm-cli -- compile build/myprogram.wasm -o myprogram.spi

# 6. Run on PVM interpreter
npx tsx ../scripts/run-spi.ts myprogram.spi --args=<hex-encoded-input>
```

## Limitations

- **No floating point** - PVM doesn't support f32/f64
- **No imports** - PVM programs are self-contained
- **Limited locals** - Currently 4 local variables supported
- **No call/call_indirect** - Function calls not yet implemented
