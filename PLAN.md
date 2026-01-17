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
0x00030000: Spilled locals (512 bytes per function)
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
- [x] memory.size - returns constant 256 pages
- [x] memory.grow - returns -1 (not supported)

#### Control Flow (Phase 3)
- [x] Translate `block` (forward branch target)
- [x] Translate `loop` (backward branch target)
- [x] Translate `br` (unconditional branch)
- [x] Translate `br_if` (conditional branch)
- [x] Translate `return`
- [x] Translate `if/else/end`
- [ ] Handle block result values (not yet implemented)

#### Integer Operations
- [x] i32.add, i64.add
- [x] i32.sub, i64.sub
- [x] i32.mul, i64.mul
- [x] i32.div_u, i32.div_s, i64.div_u, i64.div_s
- [x] i32.rem_u, i32.rem_s, i64.rem_u, i64.rem_s
- [x] i32.gt_u, i32.gt_s, i64.gt_u, i64.gt_s
- [x] i32.lt_u, i32.lt_s, i64.lt_u, i64.lt_s
- [x] i32.ge_u, i32.ge_s, i64.ge_u, i64.ge_s
- [x] i32.le_u, i32.le_s, i64.le_u, i64.le_s
- [x] i32.eq, i32.ne, i32.eqz, i64.eq, i64.ne, i64.eqz
- [x] i32.and, i32.or, i32.xor, i64.and, i64.or, i64.xor
- [x] i32.shl, i32.shr_u, i32.shr_s, i64.shl, i64.shr_u, i64.shr_s
- [x] local.tee
- [x] drop
- [x] select
- [x] unreachable (maps to TRAP)

#### Memory Operations (Phase 5)
- [x] i64.load
- [x] i64.store

#### Phase 4: AssemblyScript Examples
- [x] Set up AssemblyScript project in `examples-as/`
- [x] Create `add.ts`, `factorial.ts`, `fibonacci.ts`, `gcd.ts`
- [x] Verify AS output compiles through wasm-pvm
- [x] Document AssemblyScript → JAM workflow

#### Phase 4b: Test Suite & CI
- [x] Created `scripts/test-all.ts` - 44 tests across WAT and AS examples
- [x] GitHub Actions CI workflow (`.github/workflows/ci.yml`)

#### Phase 6: Functions & Calls (Partial)
- [x] Translate `call` instruction
- [x] Handle function prologues/epilogues
- [x] Multi-function compilation with proper offsets
- [x] Jump table for return addresses (PVM JUMP_IND requirement)
- [x] Local variable spilling (registers r9-r12 + memory at 0x30000)
- [x] Entry jump when main is not first function

### ✅ Examples Working (JAM Convention)
WAT examples (`examples-wat/*.jam.wat`):
- [x] `add.jam.wat` - reads two i32 args, returns sum
- [x] `factorial.jam.wat` - computes n! using loop
- [x] `fibonacci.jam.wat` - fibonacci sequence
- [x] `gcd.jam.wat` - GCD (Euclidean algorithm)
- [x] `is-prime.jam.wat` - primality test
- [x] `div.jam.wat` - unsigned division
- [x] `call.jam.wat` - function calls

AssemblyScript examples (`examples-as/assembly/*.ts`):
- [x] `add.ts` - reads two i32 args, returns sum
- [x] `factorial.ts` - computes n! using loop
- [x] `fibonacci.ts` - fibonacci sequence
- [x] `gcd.ts` - GCD (Euclidean algorithm)

---

## Next Steps

### Phase 5: Remaining Operations (Mostly Complete)
- [x] i64.div_u, i64.div_s, i64.rem_u, i64.rem_s
- [x] i64.ge_u, i64.ge_s, i64.le_u, i64.le_s
- [x] i64.and, i64.or, i64.xor
- [x] i64.shl, i64.shr_u, i64.shr_s
- [x] i64.load, i64.store
- [ ] Handle block result values

### Phase 7: Advanced Control Flow
- [ ] Translate `br_table` (switch/jump table)
- [ ] Multiple entry points (main at PC=0, main2 at PC=5)

### Phase 8: Recursion Support
- [ ] Implement proper call stack with frame pointer
- [ ] Push/pop spilled locals on call/return
- [ ] Handle deep recursion (stack overflow detection)

### Phase 9: Indirect Calls
- [ ] Parse WASM table section
- [ ] Build function table from WASM tables
- [ ] Translate `call_indirect`
- [ ] Validate function signatures at runtime

### Phase 10: Complete WASM MVP
- [ ] All remaining i32/i64 operations (rotl, rotr, clz, ctz, popcnt, etc.)
- [ ] i64.load, i64.store, i8/i16 load/store variants
- [ ] Proper WASM memory with base offset translation
- [ ] Data section initialization

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
- `scripts/test-all.ts` - Automated test suite (44 tests)
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
| Register pressure too high | Medium | ✅ Implemented spilling |
| Control flow edge cases | Medium | Comprehensive test suite |
| Memory model mismatch | Medium | Define clear address translation |
| Recursion stack overflow | Medium | Need proper call stack (Phase 8) |
| Performance issues | Low | Not a priority for v1 |

---

## Open Questions to Resolve

1. ~~**PVM Calling Convention**~~: ✅ Resolved - See SPI convention above
2. **Host Calls**: How to handle WASM imports? Map to PVM ecalli?
3. ~~**Memory Growth**~~: ✅ Returns -1 (not supported)
4. ~~**Floating Point**~~: ✅ Resolved - PVM has no FP, reject WASM with floats
5. **Stack Size**: Configurable in SPI format (stackSize field, up to 16MB)

---

## Success Criteria

### Minimum Viable Product ✅
- All example WAT files compile and execute correctly
- AssemblyScript examples compile and execute correctly
- CLI tool works: `wasm-pvm compile input.wasm -o output.spi`
- Basic error handling and messages

### Full Release
- WASM MVP compliance (except floats)
- Comprehensive test suite
- Documentation
- Recursion support
- Indirect calls
- Reasonable compilation speed

---

## Resources

- [Gray Paper](./gp-0.7.2.md) - PVM specification (Appendix A is key)
- [LEARNINGS.md](./LEARNINGS.md) - Technical discoveries & instruction reference
- [KNOWN_ISSUES.md](./KNOWN_ISSUES.md) - Known bugs and limitations
- [AGENTS.md](./AGENTS.md) - AI agent guidelines
- [Ananas PVM](./vendor/anan-as) - PVM reference implementation (submodule)
- [Zink Compiler](./vendor/zink) - WASM→EVM compiler for architecture inspiration (submodule)
- [WebAssembly Spec](https://webassembly.github.io/spec/)
- [AssemblyScript](https://www.assemblyscript.org/) - TypeScript-like language to WASM
