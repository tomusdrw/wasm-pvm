# WASM-PVM: WebAssembly to PolkaVM Recompiler

A Rust compiler that translates WebAssembly (WASM) bytecode into [PolkaVM](https://github.com/paritytech/polkavm) (PVM) bytecode for execution on the [JAM](https://graypaper.com/) (Join-Accumulate Machine) protocol. Write your JAM programs in [AssemblyScript](https://www.assemblyscript.org/) (TypeScript-like), hand-written WAT, or any language that compiles to WASM — and run them on PVM.

```text
WASM  ──►  LLVM IR  ──►  PVM bytecode  ──►  JAM program (.jam)
      inkwell    mem2reg       Rust backend
```

## Key Features

- **Multi-language input**: AssemblyScript, hand-written WAT, or any WASM-targeting language
- **LLVM-powered**: Uses [inkwell](https://github.com/TheDan64/inkwell) (LLVM 18 bindings) for IR generation and optimization
- **No `unsafe` code**: `deny(unsafe_code)` enforced at workspace level
- **Toggleable optimizations**: Every non-trivial optimization can be individually disabled via CLI flags
- **Comprehensive test suite**: 800+ tests across unit, integration, differential, and PVM-in-PVM layers

## Supported WASM Features

| Category | Operations |
|----------|-----------|
| **Arithmetic** (i32 & i64) | add, sub, mul, div_u/s, rem_u/s, all comparisons, clz, ctz, popcnt, rotl, rotr, bitwise ops |
| **Control flow** | block, loop, if/else, br, br_if, br_table, return, unreachable, block results |
| **Memory** | load/store (all widths), memory.size, memory.grow, memory.fill, memory.copy, globals, data sections |
| **Functions** | call, call_indirect (with signature validation), recursion, stack overflow detection |
| **Type conversions** | wrap, extend_s/u, sign extensions (i32/i64 extend8/16/32_s) |
| **Imports** | Text-based import maps and WAT adapter files |

**Not supported**: floating point (by design — PVM has no FP instructions).

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
tests/                   # Integration tests (TypeScript/Bun)
  fixtures/
    wat/                 # WAT test programs
    assembly/            # AssemblyScript examples
    imports/             # Import maps & adapter files
vendor/
  anan-as/               # PVM interpreter (submodule)
```

## Resources

- [PVM Debugger](https://github.com/fluffylabs/pvm-debugger) — upload `.jam` files for disassembly, step-by-step execution, and register/gas inspection
- [PVM Decompiler](https://github.com/tomusdrw/pvm-decompiler) — decompile PVM bytecode back to human-readable form
- [ananas (anan-as)](https://github.com/tomusdrw/anan-as) — PVM interpreter written in AssemblyScript, compiled to PVM itself for PVM-in-PVM execution
- [as-lan](https://github.com/tomusdrw/as-lan) — example AssemblyScript project compiled from WASM to PVM
- [JAM Gray Paper](https://graypaper.com/) — the JAM protocol specification (PVM is defined in Appendix A)
- [AssemblyScript](https://www.assemblyscript.org/) — TypeScript-like language that compiles to WASM
