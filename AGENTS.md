# WASM-PVM Project - AI Agent Knowledge Base

**Project**: WebAssembly to PolkaVM (PVM) bytecode recompiler  
**Stack**: Rust (core) + TypeScript (tests) + AssemblyScript (examples)  
**Docs**: `PLAN.md` (roadmap), `LEARNINGS.md` (tech discoveries), `gp-0.7.2.md` (PVM spec)

---

## Quick Start

```bash
# Build
cargo build --release

# Test
cargo test                    # Unit tests (Rust)
bun scripts/test-all.ts       # Integration tests (62 tests)

# Compile WASM → JAM
cargo run -p wasm-pvm-cli -- compile examples-wat/add.jam.wat -o dist/add.jam

# Run JAM
bun scripts/run-jam.ts dist/add.jam --args=0500000007000000
```

---

## Structure

```
crates/
├── wasm-pvm/              # Core library
│   └── src/
│       ├── translate/     # WASM→PVM translation [COMPLEX - see AGENTS.md]
│       │   ├── codegen.rs (2402 lines - main logic)
│       │   ├── mod.rs     (683 lines - orchestration)
│       │   └── stack.rs   (152 lines - operand stack)
│       ├── pvm/           # PVM instruction definitions
│       │   ├── instruction.rs  # Instruction enum + encoding
│       │   ├── opcode.rs       # Opcode constants
│       │   └── blob.rs         # Program blob format
│       ├── spi.rs         # JAM/SPI format encoder
│       └── error.rs       # Error types (thiserror)
└── wasm-pvm-cli/          # CLI binary
    └── src/main.rs        # Single-file CLI (62 lines)

scripts/                   # TypeScript tooling [see AGENTS.md]
├── test-all.ts            # Test runner
├── run-jam.ts             # JAM execution
└── test-cases.ts          # Test definitions

examples-wat/              # WAT test programs
examples-as/               # AssemblyScript examples
vendor/                    # Git submodules (anan-as)
```

---

## Domain Knowledge

### WASM (Source)
- Stack-based bytecode
- Structured control flow (blocks, loops, if/else)
- Linear memory (0-indexed, 64KB pages)

### PVM (Target)
- Register-based (13 x 64-bit registers)
- Flat control flow with jumps/branches
- Gas metering on all instructions
- Memory: addresses < 2^16 panic

### Translation Challenges
1. Stack→Registers: Map WASM operand stack to r2-r6
2. Structured→Flat: Convert WASM blocks to PVM jumps
3. Address translation: WASM 0-based → PVM 0x50000-based

---

## Conventions

### Code Style
- `rustfmt` defaults, `clippy` warnings = errors
- `unsafe_code = "deny"` (workspace lint)
- `thiserror` for errors, `tracing` for logging
- Unit tests inline under `#[cfg(test)]`

### Naming
- Types: `PascalCase`, Functions: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Indicate WASM vs PVM context in names

### Project-Specific
- No `lib/` folder in crates - flat src structure
- Integration tests in TypeScript (not `tests/*.rs`)
- Memory addresses hardcoded as magic constants
- 4 local variables max without spilling

---

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| Add WASM operator | `translate/codegen.rs` | `translate_op()` match arm |
| Add PVM instruction | `pvm/opcode.rs` + `pvm/instruction.rs` | Add enum + encoding |
| Fix translation bug | `translate/codegen.rs` | Check `emit_*` functions |
| Add test case | `scripts/test-cases.ts` | Hex args, little-endian |
| Fix test execution | `scripts/test-all.ts` | `runJamFile()` |
| Fix result parsing | `scripts/run-jam.ts` | Memory chunk reconstruction |
| Debug execution step-by-step | `scripts/trace-steps.ts` | Shows PC, gas, registers per step |
| Quick trace (simple programs) | `scripts/trace-jam.ts` | 50 steps default, minimal output |
| Verify JAM file structure | `scripts/verify-jam.ts` | Parse headers, jump table, code |
| Add global handling | `translate/mod.rs` | `compile()` globals section |
| Update PVM spec | `gp-0.7.2.md` | Appendix A is key |

---

## Anti-Patterns (Forbidden)

1. **No `unsafe` code** - Strictly forbidden by workspace lint
2. **No panics in library code** - Use `Result<>` with `Error::Internal`
3. **No floating point** - PVM lacks FP support; reject WASM floats
4. **Don't break register conventions** - Hardcoded in multiple files
5. **No standard Rust test dir** - Use TypeScript for integration tests
6. **NEVER use --no-verify on git push** - Always ensure tests and linters pass before pushing

---

## Memory Layout (Hardcoded)

| Address | Purpose |
|---------|---------|
| `0x30000` | Globals storage |
| `0x30100` | User heap (results) |
| `0x40000` | Spilled locals |
| `0x50000` | WASM linear memory base |
| `0xFEFF0000` | Arguments (args_ptr) |
| `0xFFFF0000` | EXIT address (HALT) |

---

## Register Allocation

| Register | Usage |
|----------|-------|
| r0 | Return address (jump table index) |
| r1 | Stack pointer / Return value |
| r2-r6 | Operand stack (5 slots) |
| r7 | SPI args pointer / scratch |
| r8 | SPI args length / saved table idx |
| r9-r12 | Local variables (first 4) |

Spilled locals: `0x30200 + (func_idx * 512) + ((local_idx - 4) * 8)`

---

## Subdirectory Docs

- **`crates/wasm-pvm/src/translate/AGENTS.md`** - Translation module details, codegen patterns
- **`crates/wasm-pvm/src/pvm/AGENTS.md`** - PVM instruction encoding
- **`scripts/AGENTS.md`** - TypeScript test tooling

---

## Common Tasks

### Add WASM Instruction
1. Find in WASM spec → determine PVM sequence
2. Add to `translate/codegen.rs:translate_op()`
3. Add test case to `scripts/test-cases.ts`
4. Update `LEARNINGS.md` if non-obvious

### Debug Translation Issue
1. Execute and inspect result: `bun scripts/run-jam.ts <file> --args=...`
2. Step-by-step trace: `bun scripts/trace-steps.ts <file> <args> <steps>`
3. Verify JAM structure: `bun scripts/verify-jam.ts <file>`
4. Compare expected vs actual instruction sequence
5. Check register allocation in `codegen.rs`
6. Verify control flow graph

### Add Test Case
```typescript
// In scripts/test-cases.ts
{ 
  name: 'mytest',
  tests: [
    { args: '05000000', expected: 5, description: 'Test 5' }
  ]
}
```
Args are hex little-endian u32s.

---

## Contact

Maintainer: @tomusdrw  
PVM questions: Gray Paper or PolkaVM repo
