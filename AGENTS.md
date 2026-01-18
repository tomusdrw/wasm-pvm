# WASM-PVM Project - AI Agent Guidelines

This document provides context and guidelines for AI agents working on the WASM to PVM recompiler project.

---

## Project Context

**Goal**: Build a recompiler that translates WebAssembly (WASM) bytecode to PolkaVM (PVM) bytecode.

**Language**: Rust (latest stable)

**Key Documents**:
- `PLAN.md` - Project roadmap and architecture
- `LEARNINGS.md` - Technical discoveries and design decisions
- `gp-0.7.2.md` - Gray Paper (PVM specification)
- `examples-wat/` - Example WASM programs in text format

---

## Code Standards

### Rust Style
- Follow `rustfmt` defaults
- Use `clippy` with default lints (treat warnings as errors)
- Prefer `thiserror` for error types
- Use `log` or `tracing` for diagnostics
- Document public APIs with doc comments

### Project Structure
```
crates/
├── wasm-pvm/           # Main recompiler library
│   ├── src/
│   │   ├── lib.rs
│   │   ├── wasm/       # WASM parsing and analysis
│   │   ├── ir/         # Intermediate representation
│   │   ├── pvm/        # PVM instruction definitions
│   │   ├── codegen/    # Code generation
│   │   └── error.rs
│   └── Cargo.toml
├── wasm-pvm-cli/       # Command-line interface
└── pvm-asm/            # PVM assembler/disassembler
```

### Naming Conventions
- Types: `PascalCase`
- Functions/methods: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`
- Use descriptive names that indicate WASM vs PVM context

### Error Handling
- Use `Result<T, E>` for fallible operations
- Define domain-specific error types
- Include source location in error messages when available
- Never panic in library code (except for internal bugs)

---

## Domain Knowledge

### WASM Concepts (Source)
- **Stack-based**: Operations push/pop from implicit operand stack
- **Structured control flow**: Blocks, loops, if/else with branch targets
- **Locals**: Function-local variables accessed by index
- **Linear memory**: Byte-addressable, grows in 64KB pages
- **Tables**: For indirect function calls

### PVM Concepts (Target)
- **Register-based**: 13 general-purpose 64-bit registers
- **Basic blocks**: Flat control flow with jumps/branches
- **Gas metering**: All instructions cost gas
- **Paged memory**: Addresses < 2^16 cause panic
- **Host calls**: `ecalli` instruction for external functions

### Translation Challenges
1. **Stack → Registers**: Map WASM operand stack to PVM registers
2. **Structured → Flat**: Convert WASM blocks to PVM jumps
3. **Address translation**: WASM 0-indexed memory to PVM paged memory

---

## Working on This Project

### Before Making Changes
1. Read relevant sections of `PLAN.md` and `LEARNINGS.md`
2. Check `gp-0.7.2.md` (Gray Paper) for PVM specification details
3. Review existing code patterns in the codebase
4. Run tests to ensure baseline is working

### When Implementing Features
1. Start with a failing test (if appropriate)
2. Implement the minimal solution
3. Add documentation for public APIs
4. Update `LEARNINGS.md` with any new discoveries
5. Run `cargo fmt` and `cargo clippy`

### When Adding Tests
- Unit tests go in the same file as the code (under `#[cfg(test)]`)
- Integration tests go in `tests/` directory
- WAT test files go in `tests/wat/` or `examples-wat/`
- Name test functions descriptively: `test_<what>_<scenario>`

### Key Files to Know

| File | Purpose |
|------|---------|
| `PLAN.md` | Project roadmap, current phase |
| `LEARNINGS.md` | Technical discoveries, design decisions |
| `gp-0.7.2.md` | PVM specification (Appendix A is key) |
| `examples-wat/*.wat` | Example WASM programs to compile |

---

## Common Tasks

### Adding a New WASM Instruction
1. Find the instruction in WASM spec
2. Determine equivalent PVM sequence
3. Add translation in `codegen/` module
4. Add test case
5. Document any non-obvious mappings in `LEARNINGS.md`

### Adding a New PVM Instruction
1. Find the instruction in Gray Paper Appendix A
2. Add to `PvmOpcode` enum in `pvm/` module
3. Implement encoding
4. Add to disassembler
5. Add tests

### Debugging Translation Issues
1. Disassemble the generated PVM code
2. Compare with expected instruction sequence
3. Check register allocation decisions
4. Verify control flow graph

---

## Testing Commands

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_add_function

# Run with output
cargo test -- --nocapture

# Check formatting
cargo fmt --check

# Run lints
cargo clippy -- -D warnings

# Run a specific example
cargo run -p wasm-pvm-cli -- compile examples-wat/add.wat -o output.pvm

# Run full integration test suite (56 tests)
npx tsx scripts/test-all.ts

# Run a compiled JAM file
npx tsx scripts/run-jam.ts /tmp/out.jam --args=<hex>
```

---

## Asking for Clarification

If you're unsure about:
- **PVM semantics**: Check `gp-0.7.2.md` Appendix A, or ask
- **WASM semantics**: Check official WASM spec, or ask
- **Design decisions**: Check `LEARNINGS.md`, or ask
- **Project direction**: Check `PLAN.md`, or ask

When in doubt, ask rather than guess. Document any answers in the appropriate file.

---

## Updating Documentation

### When to Update LEARNINGS.md
- Discovered non-obvious PVM behavior
- Made a design decision with alternatives
- Found a gotcha or edge case
- Resolved an open question

### When to Update PLAN.md
- Completed a phase/milestone
- Scope changed
- New risks identified
- Timeline adjusted

### When to Update AGENTS.md
- Project structure changed
- New conventions established
- Common tasks changed

---

## Quick Reference: PVM Registers (Implemented)

| Register | Usage in wasm-pvm |
|----------|-------------------|
| r0 | Return address (jump table index for function calls) |
| r1 | Stack pointer / Return value from function calls |
| r2-r6 | Operand stack (5 slots) |
| r7 | SPI args pointer (0xFEFF0000) / scratch for rotates |
| r8 | SPI args length / scratch for rotates |
| r9-r12 | Local variables (first 4 locals) |

**Memory for spilled locals**: `0x20200 + (func_idx * 512) + ((local_idx - 4) * 8)`

---

## Quick Reference: Gas Costs

From Gray Paper - each instruction has a gas cost (ϱ∆). Some examples:
- trap: 0
- fallthrough: 0
- ecalli: 0
- load_imm_64: 0
- (Check Appendix A for full list)

---

## Contact

Project maintainer: @tomusdrw

For questions about PVM: Check Gray Paper or PolkaVM repository issues.
