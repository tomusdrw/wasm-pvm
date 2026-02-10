# WASM to PVM Recompiler - Project Plan

**Status**: LLVM backend functional, legacy backend still available

---

## Current State

The compiler has two backends gated by feature flags:

- **LLVM backend** (`--features llvm-backend`): `WASM -> LLVM IR -> mem2reg -> PVM bytecode`. Uses inkwell/LLVM 18.
- **Legacy backend** (default): Direct `WASM -> IR -> PVM bytecode` translation.

Both backends pass all 360 TypeScript integration tests and all Rust tests. 43 differential tests verify both produce structurally identical output (same rw_data, ro_data structure, heap_pages).

---

## Remaining Work

### Make LLVM backend the default

- [ ] Add division-by-zero and signed overflow (`INT_MIN / -1`) trap sequences to LLVM backend
- [ ] Handle multi-value returns (`entry_returns_ptr_len` convention) in LLVM backend
- [ ] Switch default feature to `llvm-backend`
- [ ] Gate legacy backend behind `legacy-backend` feature
- [ ] Remove legacy code: `src/ir/`, `src/translate/codegen.rs`, `src/translate/stack.rs`

### ecalli support

- [ ] Support generic external function calls via PVM `ecalli` instruction
- [ ] Helper to define external functions (register content and special exit code)

### PVM-in-PVM

- [ ] Debug inner interpreter PANIC (BUG-4: PANICs at PC 56 with exitCode 0)
- [ ] Execute `add.jam.wat` through compiled anan-as
- [ ] Run full test suite through PVM-in-PVM

### Testing improvements

- [ ] Expand differential tests to compare execution results (not just compilation)
- [ ] Add property-based tests / fuzzing
- [ ] Increase test coverage

### Future optimizations (post-correctness)

- [ ] Register allocator for LLVM backend (currently uses stack-slot approach)
- [ ] Enable additional LLVM optimization passes beyond mem2reg
- [ ] Passive data segments (`memory.init`)

---

## Key Files

| File | Role |
|------|------|
| `src/llvm_frontend/function_builder.rs` | WASM -> LLVM IR translation (~1350 lines) |
| `src/llvm_backend/lowering.rs` | LLVM IR -> PVM bytecode lowering (~1900 lines) |
| `src/translate/wasm_module.rs` | Shared WASM parsing (both pipelines) |
| `src/translate/mod.rs` | Compilation orchestration + legacy backend |
| `src/translate/memory_layout.rs` | PVM memory address constants |
| `src/pvm/instruction.rs` | PVM instruction encoding |
| `tests/differential.rs` | 43 differential tests (both backends) |

---

## Resources

- [LEARNINGS.md](./LEARNINGS.md) - Technical reference (PVM architecture, conventions)
- [AGENTS.md](./AGENTS.md) - Developer guidelines
- [gp-0.7.2.md](./gp-0.7.2.md) - Gray Paper (PVM specification)
- [review/](./review/) - Architecture review (2026-02-09)
