# WASM to PVM Recompiler - Project Plan

## Project Overview

Build a **WASM (WebAssembly) to PVM (Polka Virtual Machine)** recompiler in Rust. The recompiler takes WASM bytecode and produces equivalent PVM bytecode that can execute on the PolkaVM.

### Goals
1. **Correctness** - Produce semantically equivalent PVM code
2. **Completeness** - Support core WASM MVP features
3. **Testability** - Comprehensive test suite with reference interpreter
4. **Maintainability** - Clean architecture following Rust best practices

### Non-Goals (Initial Version)
- Performance optimization (focus on correctness first)
- WASM proposals beyond MVP (SIMD, threads, etc.)
- Floating point support (**PVM has no FP - reject WASM with floats**)

### Output Format
**Primary target: JAM SPI (Standard Program Interface) format**
- Includes memory layout (RO data, RW data, heap, stack)
- Proper register initialization for JAM execution
- See LEARNINGS.md for detailed SPI format specification

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         WASM Binary                              │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    1. WASM Parser (wasmparser)                   │
│                    Parses WASM binary to module                  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    2. Module Analyzer                            │
│  - Validate module                                               │
│  - Collect function signatures                                   │
│  - Analyze imports/exports                                       │
│  - Build type information                                        │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    3. Function Translator                        │
│  For each function:                                              │
│  - Parse WASM instructions                                       │
│  - Build control flow graph                                      │
│  - Convert stack ops to IR                                       │
│  - Register allocation                                           │
│  - Generate PVM instructions                                     │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    4. Module Linker                              │
│  - Resolve function addresses                                    │
│  - Build jump tables                                             │
│  - Layout memory sections                                        │
│  - Generate initialization code                                  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    5. PVM Encoder                                │
│  - Encode instructions to bytes                                  │
│  - Build opcode bitmask                                          │
│  - Encode jump table                                             │
│  - Produce final program blob                                    │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    6. SPI Packager                               │
│  - Package RO data section                                       │
│  - Package RW data section                                       │
│  - Set heap/stack sizes                                          │
│  - Produce SPI binary                                            │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SPI Binary (JAM-compatible)                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Foundation (Week 1-2)

### 1.1 Project Setup
- [ ] Initialize Rust project with Cargo workspace
- [ ] Set up directory structure:
  ```
  wasm-pvm/
  ├── Cargo.toml (workspace)
  ├── crates/
  │   ├── wasm-pvm/          # Main library
  │   ├── wasm-pvm-cli/      # CLI tool
  │   └── pvm-asm/           # PVM assembler/disassembler
  ├── tests/
  │   ├── wat/               # WAT test files
  │   └── integration/       # Integration tests
  └── docs/
  ```
- [ ] Add dependencies: wasmparser, thiserror, etc.
- [ ] Set up CI (GitHub Actions) with clippy, rustfmt, tests

### 1.2 PVM Instruction Definitions
- [ ] Define `PvmOpcode` enum with all opcodes from Gray Paper
- [ ] Define `PvmInstruction` struct with operands
- [ ] Implement instruction encoding to bytes
- [ ] Implement opcode bitmask generation
- [ ] Write PVM assembler (text → binary)
- [ ] Write PVM disassembler (binary → text)

### 1.3 Basic Infrastructure
- [ ] Define error types
- [ ] Set up logging/tracing
- [ ] Create basic CLI structure

**Deliverable:** Can assemble/disassemble PVM programs

---

## Phase 2: Simple Functions (Week 3-4)

### 2.1 WASM Parsing
- [ ] Parse WASM module using wasmparser
- [ ] Extract function types and bodies
- [ ] Handle imports/exports metadata

### 2.2 Basic Translation
- [ ] Translate simple arithmetic (i32.add, i32.sub, i32.mul, etc.)
- [ ] Handle local variables (local.get, local.set, local.tee)
- [ ] Implement operand stack → register mapping
- [ ] Handle function return values

### 2.3 Register Allocation (Simple)
- [ ] Define register usage convention (SPI-compatible):
  - r0: Reserved (SPI convention)
  - r1: Stack pointer (SPI convention)
  - r2-r6: Operand stack / temporaries (5 regs)
  - r7-r8: Reserved (args ptr/len in SPI)
  - r9-r12: Local variables / callee-saved (4 regs)
- [ ] Implement stack spilling when registers exhausted
- [ ] Track register liveness

**Deliverable:** Can compile `add.wat` example

---

## Phase 3: Control Flow (Week 5-6)

### 3.1 Basic Blocks
- [ ] Identify basic blocks in WASM function
- [ ] Build control flow graph
- [ ] Assign addresses to basic blocks

### 3.2 Structured Control Flow
- [ ] Translate `block` (forward branch target)
- [ ] Translate `loop` (backward branch target)
- [ ] Translate `br` (unconditional branch)
- [ ] Translate `br_if` (conditional branch)
- [ ] Translate `if/else/end`
- [ ] Handle block result values

### 3.3 Advanced Branches
- [ ] Translate `br_table` (switch statement)
- [ ] Handle nested blocks
- [ ] Multi-value returns from blocks

**Deliverable:** Can compile `factorial.wat`, `fibonacci.wat`, `gcd.wat`

---

## Phase 4: Functions & Calls (Week 7-8)

### 4.1 Direct Calls
- [ ] Define calling convention (SPI-compatible):
  - Arguments in registers r2-r6 (first 5 args)
  - Additional arguments on stack (via r1)
  - Return value in r2
  - Callee-saved: r9-r12
- [ ] Translate `call` instruction
- [ ] Handle function prologues/epilogues
- [ ] Manage call stack

### 4.2 Indirect Calls
- [ ] Build function table
- [ ] Translate `call_indirect`
- [ ] Table bounds checking

### 4.3 Return Handling
- [ ] Translate `return` instruction
- [ ] Handle early returns from nested blocks

**Deliverable:** Can compile `nth-prime.wat` (uses function calls)

---

## Phase 5: Memory Operations (Week 9-10)

### 5.1 Memory Layout
- [ ] Define memory base address (>= 0x10000 for PVM safety)
- [ ] Handle data segments initialization
- [ ] Translate memory addresses

### 5.2 Load/Store
- [ ] Translate i32.load, i32.store
- [ ] Translate i64.load, i64.store
- [ ] Handle alignment
- [ ] Handle load/store with offset

### 5.3 Memory Management
- [ ] Translate memory.size
- [ ] Translate memory.grow (via host call?)
- [ ] Handle out-of-bounds access

**Deliverable:** Can compile programs using linear memory

---

## Phase 6: Complete WASM MVP (Week 11-12)

### 6.1 Remaining Integer Operations
- [ ] All i32 operations (clz, ctz, popcnt, rotl, rotr, etc.)
- [ ] All i64 operations
- [ ] Sign extension operations
- [ ] Comparison operations

### 6.2 Globals
- [ ] Translate global.get, global.set
- [ ] Handle mutable vs immutable globals
- [ ] Initialize globals from data

### 6.3 Remaining Features
- [ ] unreachable instruction
- [ ] nop instruction
- [ ] select instruction
- [ ] drop instruction

**Deliverable:** Full WASM MVP support (except floats)

---

## Phase 7: Testing & Polish (Week 13-14)

### 7.1 Test Suite
- [ ] Unit tests for each instruction
- [ ] Integration tests with WAT files
- [ ] Fuzzing with arbitrary WASM
- [ ] Comparison testing with reference interpreter

### 7.2 Tooling
- [ ] CLI improvements (verbose output, debug info)
- [ ] Error messages with source locations
- [ ] Documentation

### 7.3 Optimization (if time permits)
- [ ] Peephole optimizations
- [ ] Dead code elimination
- [ ] Better register allocation

**Deliverable:** Production-ready recompiler

---

## Testing Strategy

### Unit Tests
- Each WASM instruction maps correctly to PVM sequence
- Register allocation works for various scenarios
- Control flow translation is correct

### Integration Tests
- Compile WAT files in `examples-wat/`
- Execute on PVM interpreter
- Compare output with WASM interpreter

### Conformance Tests
- WebAssembly spec test suite (subset)
- Custom edge case tests

### Test Infrastructure Needed
- PVM interpreter (for running generated code)
- WASM interpreter (for reference outputs)
- Test harness comparing both

---

## Dependencies

### Required Crates
- `wasmparser` - Parse WASM binary format
- `thiserror` - Error handling
- `log` / `tracing` - Logging
- `clap` - CLI argument parsing

### Development Crates
- `wat` - Parse WAT (text format) for tests
- `pretty_assertions` - Better test diffs
- `proptest` / `arbitrary` - Property-based testing

### Optional Crates
- `walrus` - WASM transformation library (alternative to wasmparser)
- `cranelift` - Could use for IR (might be overkill)

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| PVM instruction set insufficient | High | Check all WASM ops map to PVM early |
| Register pressure too high | Medium | Implement robust spilling |
| Control flow edge cases | Medium | Comprehensive test suite |
| Memory model mismatch | Medium | Define clear address translation |
| Performance issues | Low | Not a priority for v1 |

---

## Open Questions to Resolve

1. ~~**PVM Calling Convention**~~: ✅ Resolved - See SPI format in LEARNINGS.md
2. **Host Calls**: How to handle WASM imports? Map to PVM ecalli?
3. **Memory Growth**: SBRK instruction available (opcode 101)
4. ~~**Floating Point**~~: ✅ Resolved - PVM has no FP, reject WASM with floats
5. **Stack Size**: Configurable in SPI format (stackSize field, up to 16MB)

---

## Success Criteria

### Minimum Viable Product
- All example WAT files compile and execute correctly
- CLI tool works: `wasm-pvm compile input.wasm -o output.pvm`
- Basic error handling and messages

### Full Release
- WASM MVP compliance (except floats)
- Comprehensive test suite
- Documentation
- Reasonable compilation speed

---

## Resources

- [Gray Paper](./gp-0.7.2.md) - PVM specification (Appendix A is key)
- [LEARNINGS.md](./LEARNINGS.md) - Technical discoveries & instruction reference
- [AGENTS.md](./AGENTS.md) - AI agent guidelines
- [Ananas PVM](./vendor/anan-as) - PVM reference implementation (submodule)
- [Zink Compiler](./vendor/zink) - WASM→EVM compiler for architecture inspiration (submodule)
- [WebAssembly Spec](https://webassembly.github.io/spec/)
