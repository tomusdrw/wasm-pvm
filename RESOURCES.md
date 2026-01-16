# Resources

## Specifications
- [Gray Paper v0.7.2](./gp-0.7.2.md) - JAM Protocol & PVM specification (Appendix A)
- [WebAssembly Spec](https://webassembly.github.io/spec/) - WASM specification

## Reference Implementations (Submodules)
- [Ananas PVM](./vendor/anan-as) - AssemblyScript PVM implementation
  - Key files: `assembly/spi.ts`, `assembly/instructions.ts`, `assembly/program.ts`
  - Contains SPI format details, instruction encoding, memory layout
- [Zink Compiler](./vendor/zink) - WASM→EVM compiler (architecture inspiration)
  - Key files: `codegen/src/visitor/mod.rs`, `codegen/src/codegen/function.rs`
  - Uses wasmparser's VisitOperator pattern

## Test Files
- [Minimal WASM examples](./examples-wat) - WAT test programs
  - `add.wat` - Simple arithmetic
  - `factorial.wat` - Loops
  - `fibonacci.wat` - Loops with multiple locals
  - `gcd.wat` - Euclidean algorithm
  - `is-prime.wat` - Conditionals and loops
  - `nth-prime.wat` - Function calls

## Project Documentation
- [PLAN.md](./PLAN.md) - Project roadmap and architecture
- [LEARNINGS.md](./LEARNINGS.md) - Technical discoveries and instruction reference
- [AGENTS.md](./AGENTS.md) - AI agent guidelines
