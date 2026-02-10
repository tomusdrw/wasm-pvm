# LLVM IR Backend — Design & Status

## Architecture

```
WASM → [inkwell] → LLVM IR → [mem2reg] → [Rust PVM backend] → PVM bytecode
```

The PVM backend is written in Rust (reading LLVM IR via inkwell's iteration API), NOT as a native LLVM C++ target. This gives us LLVM's SSA/CFG representation and optimization passes without the enormous effort of writing a TableGen-based LLVM backend.

**Constraint**: `unsafe_code = "deny"` at workspace level. PVM-specific intrinsic functions are used for memory operations to avoid inkwell's `unsafe` GEP/inttoptr APIs.

## Status

All 8 migration phases are complete:

1. **Infrastructure** — Feature flag setup, `WasmModule` extraction
2. **Arithmetic** — Constants, locals, arithmetic, bitwise, comparisons, type conversions
3. **Control flow** — Block/loop/if → LLVM basic blocks, branches, phi nodes
4. **Memory & calls** — PVM intrinsics for loads/stores, direct and indirect calls
5. **Core lowering** — LLVM IR → PVM bytecode for arithmetic, memory, control flow
6. **Phi nodes & calls** — Phi elimination, full PVM calling convention
7. **Integration** — Full pipeline wired together, all 360 tests passing
8. **Differential testing** — 43 tests comparing both backends

### Remaining gaps

- Division-by-zero and signed overflow (`INT_MIN / -1`) trap sequences not yet emitted
- Multi-value returns (`entry_returns_ptr_len`) not yet handled via LLVM multi-value return
- Import function calls emit Trap (same as legacy for unsupported imports)

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Skip old IR, go `wasmparser::Operator` → LLVM IR directly | The old IR is a 1:1 WASM copy; no abstraction benefit |
| Use intrinsic functions for PVM memory ops | Avoids `unsafe` code, prevents LLVM from wrong assumptions about PVM memory model |
| Every SSA value gets a stack slot in PVM backend | Correctness-first; register allocation is a future optimization |
| Use `alloca` + `mem2reg` for WASM locals | Standard LLVM frontend pattern; avoids manual SSA/phi construction |
| All values as i64 internally | PVM registers are 64-bit; simplifies translation |
| Feature-flag both pipelines | Safe migration; can diff-test and fall back |

## Key Files

| File | Role |
|------|------|
| `src/llvm_frontend/function_builder.rs` | WASM → LLVM IR translation (~1350 lines) |
| `src/llvm_backend/lowering.rs` | LLVM IR → PVM bytecode lowering (~1900 lines) |
| `src/translate/wasm_module.rs` | Shared WASM parsing (both pipelines) |
| `src/translate/mod.rs` | Compilation orchestration + SPI assembly |
| `src/translate/memory_layout.rs` | PVM memory address constants |
| `tests/differential.rs` | 43 differential tests (both backends) |

## Dependencies

- `inkwell` 0.8 with `llvm18-1` feature
- LLVM 18 system dependency
- macOS: `brew install llvm@18`, set `LLVM_SYS_181_PREFIX=/opt/homebrew/opt/llvm@18`
