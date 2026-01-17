# AssemblyScript Examples

This directory contains AssemblyScript examples that compile to WASM and can be run through wasm-pvm.

## Setup

```bash
npm init -y
npm install --save-dev assemblyscript
npx asinit .
```

## Convention

All programs must follow the SPI entrypoint convention:

```typescript
// Globals for return value
let result_ptr: i32 = 0;
let result_len: i32 = 0;

// Entry point - receives args pointer and length
export function main(args_ptr: i32, args_len: i32): void {
  // Read args from memory at args_ptr
  // Write result to heap (0x20100+)
  // Set result_ptr and result_len
}
```

## Compilation

```bash
# Compile without runtime (raw WASM)
npx asc src/add.ts -o build/add.wasm --runtime stub --exportStart=_start --noAssert
```

## Examples

- `add.ts` - Add two u32 values from args
- `factorial.ts` - Compute factorial
- `fibonacci.ts` - Compute fibonacci number
- `is-prime.ts` - Check if number is prime
