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

## SPI Entrypoint Convention

All WASM programs targeting PVM/JAM must follow this convention:

### Function Signature
```wat
(func (export "main") (param $args_ptr i32) (param $args_len i32)
  ;; args_ptr = r7 (PVM address of SPI args, e.g., 0xFEFF0000)
  ;; args_len = r8 (length of args in bytes)
  ...
)

;; Optional second entry point for JAM (PC=5)
(func (export "main2") (param $args_ptr i32) (param $args_len i32)
  ...
)
```

### Return Value Convention
```wat
(global $result_ptr (mut i32) (i32.const 0))
(global $result_len (mut i32) (i32.const 0))

;; In function body:
(global.set $result_ptr (i32.const 0x20100))  ;; PVM address of result
(global.set $result_len (i32.const 4))         ;; Length in bytes
```

### Memory Layout
```
0x00010000: RO data segment
0x00020000: Globals storage (0x20000 + idx*4)
0x00020100: User heap starts here
0xFEFE0000: Stack segment end
0xFEFF0000: Args segment (read via args_ptr)
0xFFFF0000: EXIT address (HALT)
```

---

## Current Progress

### ✅ Completed

#### Phase 1: Foundation
- [x] Initialize Rust project with Cargo workspace
- [x] Set up directory structure (crates/wasm-pvm, crates/wasm-pvm-cli)
- [x] Add dependencies: wasmparser, thiserror, anyhow, clap
- [x] Define `Opcode` enum with essential opcodes
- [x] Define `Instruction` enum with operands
- [x] Implement instruction encoding to bytes
- [x] Implement opcode bitmask generation
- [x] Create basic CLI structure
- [x] Set up test infrastructure (scripts/run-spi.ts with anan-as)

#### Phase 2: Simple Functions
- [x] Parse WASM module using wasmparser
- [x] Extract function types and bodies
- [x] Translate simple arithmetic (i32.add, i64.add)
- [x] Translate i32.const, i64.const
- [x] Handle local variables (local.get, local.set)
- [x] Implement operand stack → register mapping (r2-r6)
- [x] SPI entrypoint convention (args_ptr/args_len params)
- [x] Return value via globals ($result_ptr, $result_len)
- [x] Hardcoded EXIT address (0xFFFF0000)

#### Memory Operations
- [x] i32.load - direct PVM memory access
- [x] i32.store - direct PVM memory access
- [x] global.get / global.set - stored at 0x20000 + idx*4

#### Control Flow (Phase 3 - DONE)
- [x] Translate `block` (forward branch target)
- [x] Translate `loop` (backward branch target)
- [x] Translate `br` (unconditional branch)
- [x] Translate `br_if` (conditional branch)
- [x] Translate `return`
- [ ] Translate `if/else/end` (not yet implemented)
- [ ] Handle block result values (not yet implemented)

#### Integer Operations (Partial)
- [x] i32.add, i64.add
- [x] i32.sub
- [x] i32.mul
- [x] i32.gt_u, i32.gt_s
- [x] i32.lt_u, i32.lt_s
- [x] i32.ge_u
- [x] i32.le_u
- [x] i32.eqz

### ✅ Examples Working (SPI Convention)
- [x] `add-spi.wat` - reads two i32 args, returns sum (verified: 5+7=12)
- [x] `factorial-spi.wat` - computes n! using loop (verified: 5!=120)

### 📁 Legacy Examples (To Be Migrated)
These will be migrated to SPI convention and deduplicated:
- `add.wat` - migrate to SPI (may duplicate add-spi.wat)
- `add-args.wat` - migrate to SPI (may duplicate add-spi.wat)
- `factorial.wat` - migrate to SPI (may duplicate factorial-spi.wat)
- `fibonacci.wat` - migrate to SPI
- `gcd.wat` - migrate to SPI
- `is-prime.wat` - migrate to SPI
- `nth-prime.wat` - migrate to SPI

---

## Next Steps

### Phase 3: AssemblyScript Examples (HIGH PRIORITY)
**Goal**: Enable writing JAM programs in TypeScript-like AssemblyScript.

- [ ] Set up AssemblyScript project in `examples-as/`
  - [ ] Initialize npm/package.json
  - [ ] Configure AS compiler (no runtime, raw WASM export)
  - [ ] Create build script
- [ ] Create `add.ts` - simple addition (read args, return sum)
- [ ] Create `factorial.ts` - iterative factorial
- [ ] Verify AS output compiles through wasm-pvm
- [ ] Document AssemblyScript → JAM workflow
- [ ] Add any missing WASM ops needed by AS output

### Phase 4: Example Migration & Test Suite
**Goal**: All examples use SPI convention, automated testing on every change.

#### 4.1: Migrate Legacy Examples
- [ ] Migrate `add.wat` → SPI (delete if same as add-spi.wat)
- [ ] Migrate `add-args.wat` → SPI (delete if same as add-spi.wat)
- [ ] Migrate `factorial.wat` → SPI (delete if same as factorial-spi.wat)
- [ ] Migrate `fibonacci.wat` → `fibonacci-spi.wat`
- [ ] Migrate `gcd.wat` → `gcd-spi.wat`
- [ ] Migrate `is-prime.wat` → `is-prime-spi.wat`
- [ ] Migrate `nth-prime.wat` → `nth-prime-spi.wat`

#### 4.2: Automated Test Suite
- [ ] Create `scripts/test-all.ts` - compile and run all examples
- [ ] Define expected outputs for each example
- [ ] Exit with error if any test fails
- [ ] Add `npm test` or `cargo test` integration

#### 4.3: CI/CD
- [ ] Add GitHub Actions workflow (`.github/workflows/ci.yml`)
  - [ ] Run `cargo test`
  - [ ] Run `cargo clippy`
  - [ ] Run example test suite
  - [ ] Run on PRs and pushes to main

### Phase 5: Complete Control Flow
- [ ] Translate `if/else/end`
- [ ] Handle block result values (blocks that produce values)
- [ ] Nested control flow testing

### Phase 6: Remaining Integer Operations
- [ ] i32.div_u, i32.div_s
- [ ] i32.rem_u, i32.rem_s
- [ ] i32.and, i32.or, i32.xor
- [ ] i32.shl, i32.shr_u, i32.shr_s
- [ ] i32.eq, i32.ne
- [ ] i32.le_s, i32.ge_s (signed variants)
- [ ] Corresponding i64 operations

### Phase 8: Functions & Calls
- [ ] Translate `call` instruction
- [ ] Handle function prologues/epilogues
- [ ] Translate `call_indirect`
- [ ] Build function table

### Phase 9: Complete WASM MVP
- [ ] unreachable, nop, select, drop
- [ ] memory.size, memory.grow (SBRK)
- [ ] All remaining i32/i64 operations
- [ ] Multiple entry points (main at PC=0, main2 at PC=5)

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

## Testing Strategy

### Unit Tests
- Each WASM instruction maps correctly to PVM sequence
- Register allocation works for various scenarios
- Control flow translation is correct

### Integration Tests
- Compile WAT files in `examples-wat/`
- Compile AssemblyScript files in `examples-as/`
- Execute on PVM interpreter (anan-as)
- Compare output with expected values

### Test Infrastructure
- `scripts/run-spi.ts` - Run SPI binaries on anan-as interpreter
- `vendor/anan-as` - PVM reference implementation (submodule)

---

## Dependencies

### Required Crates
- `wasmparser` - Parse WASM binary format
- `thiserror` - Error handling
- `anyhow` - Error context for CLI
- `clap` - CLI argument parsing

### Development Crates
- `wat` - Parse WAT (text format) for tests

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

1. ~~**PVM Calling Convention**~~: ✅ Resolved - See SPI convention above
2. **Host Calls**: How to handle WASM imports? Map to PVM ecalli?
3. **Memory Growth**: SBRK instruction available (opcode 101)
4. ~~**Floating Point**~~: ✅ Resolved - PVM has no FP, reject WASM with floats
5. **Stack Size**: Configurable in SPI format (stackSize field, up to 16MB)

---

## Success Criteria

### Minimum Viable Product
- All example WAT files compile and execute correctly
- AssemblyScript examples compile and execute correctly
- CLI tool works: `wasm-pvm compile input.wasm -o output.spi`
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
- [AssemblyScript](https://www.assemblyscript.org/) - TypeScript-like language to WASM
