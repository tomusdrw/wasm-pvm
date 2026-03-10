# Contributing

Contributions are welcome! This page covers coding conventions, project structure, and where to look for different tasks.

## Code Style

- `rustfmt` defaults, `clippy` warnings treated as errors
- `unsafe_code = "deny"` at workspace level
- `thiserror` for error types, `tracing` for logging
- Unit tests inline under `#[cfg(test)]`

## Naming Conventions

- Types: `PascalCase`
- Functions: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Indicate WASM vs PVM context in names where relevant

## Where to Look

| Task | Location |
|------|----------|
| Add WASM operator | `crates/wasm-pvm/src/llvm_frontend/function_builder.rs` |
| Add PVM lowering (arithmetic) | `crates/wasm-pvm/src/llvm_backend/alu.rs` |
| Add PVM lowering (memory) | `crates/wasm-pvm/src/llvm_backend/memory.rs` |
| Add PVM lowering (control flow) | `crates/wasm-pvm/src/llvm_backend/control_flow.rs` |
| Add PVM lowering (calls) | `crates/wasm-pvm/src/llvm_backend/calls.rs` |
| Add PVM lowering (intrinsics) | `crates/wasm-pvm/src/llvm_backend/intrinsics.rs` |
| Modify emitter core | `crates/wasm-pvm/src/llvm_backend/emitter.rs` |
| Add PVM instruction | `crates/wasm-pvm/src/pvm/opcode.rs` + `crates/wasm-pvm/src/pvm/instruction.rs` |
| Modify register allocator | `crates/wasm-pvm/src/llvm_backend/regalloc.rs` |
| Modify peephole optimizer | `crates/wasm-pvm/src/pvm/peephole.rs` |
| Fix WASM parsing | `crates/wasm-pvm/src/translate/wasm_module.rs` |
| Fix compilation pipeline | `crates/wasm-pvm/src/translate/mod.rs` |
| Fix adapter merge | `crates/wasm-pvm/src/translate/adapter_merge.rs` |
| Add integration test | `tests/layer{1,2,3}/*.test.ts` |

## Anti-Patterns (Forbidden)

1. **No `unsafe` code** — strictly forbidden by workspace lint
2. **No panics in library code** — use `Result<>` with `Error::Internal`
3. **No floating point** — PVM lacks FP support; reject WASM floats
4. **Don't break register conventions** — hardcoded in multiple files
5. **Don't change opcode numbers** — would break existing JAM files

## Building & Testing

See the [Getting Started](./getting-started.md) and [Testing](./testing.md) chapters.

## Documentation Policy

After every task or commit, update relevant documentation:
- `AGENTS.md` — new modules, build process changes, conventions
- [`learnings.md`](./learnings.md) — technical discoveries and debugging insights
- [`architecture.md`](./architecture.md) — ABI or calling convention changes
- [`internals/`](./internals/translation.md) — module-specific implementation details
- [`SUMMARY.md`](./SUMMARY.md) — when adding new documentation pages
