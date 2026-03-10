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
| Add WASM operator | `llvm_frontend/function_builder.rs` |
| Add PVM lowering (arithmetic) | `llvm_backend/alu.rs` |
| Add PVM lowering (memory) | `llvm_backend/memory.rs` |
| Add PVM lowering (control flow) | `llvm_backend/control_flow.rs` |
| Add PVM lowering (calls) | `llvm_backend/calls.rs` |
| Add PVM lowering (intrinsics) | `llvm_backend/intrinsics.rs` |
| Modify emitter core | `llvm_backend/emitter.rs` |
| Add PVM instruction | `pvm/opcode.rs` + `pvm/instruction.rs` |
| Modify register allocator | `llvm_backend/regalloc.rs` |
| Modify peephole optimizer | `pvm/peephole.rs` |
| Fix WASM parsing | `translate/wasm_module.rs` |
| Fix compilation pipeline | `translate/mod.rs` |
| Fix adapter merge | `translate/adapter_merge.rs` |
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
- `docs/src/learnings.md` — technical discoveries and debugging insights
- `docs/src/architecture.md` — ABI or calling convention changes
- `docs/src/internals/` — module-specific implementation details
- `docs/src/SUMMARY.md` — when adding new documentation pages
