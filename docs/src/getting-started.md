# Getting Started

## Prerequisites

- **Rust** (stable, edition 2024)
- **LLVM 18** — the compiler uses [inkwell](https://github.com/TheDan64/inkwell) (LLVM 18 bindings)
  - macOS: `brew install llvm@18` then `export LLVM_SYS_181_PREFIX=/opt/homebrew/opt/llvm@18`
  - Ubuntu: `apt install llvm-18-dev`
- **Bun** (for running integration tests and the JAM runner) — [bun.sh](https://bun.sh)

## Build

```bash
git clone https://github.com/fluffylabs/wasm-pvm.git
cd wasm-pvm
cargo build --release
```

## Hello World: Compile & Run

Create a simple WAT program that adds two numbers:

```wat
;; add.wat
(module
  (memory 1)
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    ;; Read two i32 args, add them, write result to memory
    (i32.store (i32.const 0)
      (i32.add
        (i32.load (local.get $args_ptr))
        (i32.load (i32.add (local.get $args_ptr) (i32.const 4)))))
    (i64.const 17179869184)))  ;; packed ptr=0, len=4
```

Compile it to a JAM blob and run it:

```bash
# Compile WAT → JAM
cargo run -p wasm-pvm-cli -- compile add.wat -o add.jam

# Run with two u32 arguments: 5 and 7 (little-endian hex)
npx @fluffylabs/anan-as run add.jam 0500000007000000
# Output: 0c000000  (12 in little-endian)
```

## Inspect the Output

Upload the resulting `.jam` file to the [PVM Debugger](https://github.com/fluffylabs/pvm-debugger) for step-by-step execution, disassembly, register inspection, and gas metering visualization.

## AssemblyScript Example

You can also write programs in AssemblyScript:

```typescript
// fibonacci.ts
export function main(args_ptr: i32, args_len: i32): i64 {
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
  return (buf as i64) | ((4 as i64) << 32);  // packed ptr + len
}
```

Compile via the AssemblyScript compiler to WASM, then use `wasm-pvm-cli` to produce a JAM blob. See the `tests/fixtures/assembly/` directory for more examples.

## Entry Function ABI

All entry functions must use the signature `main(args_ptr: i32, args_len: i32) -> i64`. The i64 return value packs a result pointer (lower 32 bits) and result length (upper 32 bits). The compiler unpacks this into PVM's SPI convention (`r7` = start address, `r8` = end address).

For WAT programs, the common "return 4 bytes at address 0" constant is `(i64.const 17179869184)` (= `4 << 32`).

For AssemblyScript, use: `return (ptr as i64) | ((len as i64) << 32)`.
