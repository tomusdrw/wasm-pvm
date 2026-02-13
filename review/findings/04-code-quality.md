# 04 - Code Quality Issues

**Category**: Maintainability
**Date**: 2026-02-13 (Updated for V2 Architecture)
**Status**: Good

---

## Summary

The V2 architecture (LLVM-based) significantly improved code quality. The codebase is now modular, typed, and follows standard compiler patterns.

---

## Improvements

1.  **Modular Structure**:
    *   `llvm_frontend`: Handles WASM parsing and IR generation.
    *   `llvm_backend`: Handles PVM lowering.
    *   `translate`: Orchestration.
    *   Clean separation of concerns.

2.  **Safety**:
    *   No `unsafe` blocks found in the core logic.
    *   Proper error propagation using `Result`.

3.  **Readability**:
    *   Code is well-commented.
    *   Variable names are descriptive.
    *   Constants (like registers) are defined at the top of files.

---

## Remaining Code Smells

### 1. Large Files
*   `crates/wasm-pvm/src/llvm_backend/lowering.rs`: ~2300 lines.
    *   Contains lowering logic for ALL instructions.
    *   Could be split into submodules: `alu.rs`, `memory.rs`, `control_flow.rs`.

### 2. Duplicate Lowering Logic
*   Binary operations (`Add`, `Sub`, `Mul`...) share very similar boilerplate.
    *   Currently handled via `lower_binary_arith` helper, which is good.
    *   PVM intrinsic lowering (`emit_pvm_load`, `emit_pvm_store`) has repetitive match arms.

### 3. Magic Constants
*   Some constants in `lowering.rs` (e.g., `STACK_PTR_REG = 1`) are duplicated from `translate/mod.rs`.
*   Ideally, these should be in a shared `constants.rs` or `pvm_abi.rs`.

---

## Recommendations

1.  **Refactor `lowering.rs`**: Split into smaller modules by instruction category.
2.  **Centralize ABI Constants**: Move register definitions and memory layout constants to a common crate or module shared by frontend, backend, and tests.
