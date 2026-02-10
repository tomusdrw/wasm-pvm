# WASM to PVM Recompiler - Project Plan

**Status**: LLVM backend functional, all 360 integration tests passing

---

## Remaining Work

### Correctness gaps

- [ ] Add division-by-zero and signed overflow (`INT_MIN / -1`) trap sequences
- [ ] Handle multi-value returns (`entry_returns_ptr_len` convention)

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

- [ ] Register allocator (currently uses stack-slot approach)
- [ ] Enable additional LLVM optimization passes beyond mem2reg
- [ ] Passive data segments (`memory.init`)

---

## Key Files

| File | Role |
|------|------|
| `src/llvm_frontend/function_builder.rs` | WASM -> LLVM IR translation (~1350 lines) |
| `src/llvm_backend/lowering.rs` | LLVM IR -> PVM bytecode lowering (~1900 lines) |
| `src/translate/wasm_module.rs` | WASM section parsing |
| `src/translate/mod.rs` | Compilation orchestration + SPI assembly |
| `src/translate/memory_layout.rs` | PVM memory address constants |
| `src/pvm/instruction.rs` | PVM instruction encoding |
| `tests/differential.rs` | 43 differential tests |

---

## Resources

- [LEARNINGS.md](./LEARNINGS.md) - Technical reference (PVM architecture, conventions)
- [AGENTS.md](./AGENTS.md) - Developer guidelines
- [gp-0.7.2.md](./gp-0.7.2.md) - Gray Paper (PVM specification)
- [review/](./review/) - Architecture review (2026-02-09)
