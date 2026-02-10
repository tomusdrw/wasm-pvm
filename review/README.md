# WASM→PVM Compiler Architecture Review

**Compiler Expert Assessment**  
**Date**: 2026-02-09  
**Scope**: Comprehensive architectural review of the WASM-to-PVM recompiler  
**Status**: Critical issues identified, redesign recommended

---

## Executive Summary

This review presents a critical analysis of the WASM→PVM compiler architecture. While the compiler successfully passes 62 integration tests and produces working PVM bytecode, the **current design is fundamentally flawed** for long-term maintainability, correctness, and extensibility.

**Verdict**: The compiler works, but its architecture is a "house of cards" that will collapse under the weight of future requirements (optimizations, debugging, complex features).

### Key Findings at a Glance

| Category         | Issues Found                      | Severity    |
| ---------------- | --------------------------------- | ----------- |
| **Architecture** | 8 critical design flaws           | 🔴 Critical |
| **Correctness**  | 3 known bugs + 5 potential issues | 🔴 High     |
| **Completeness** | 4 missing features                | 🟡 Medium   |
| **Code Quality** | 12 code smells                    | 🟡 Medium   |
| **Performance**  | 5 inefficiencies                  | 🟢 Low      |

### Recommendation: Incremental Redesign Required

The current architecture is a **direct translator** that lacks:
1. An intermediate representation (IR) layer
2. Separation of parsing, analysis, and code generation
3. A proper register allocator
4. Control flow graph analysis
5. Defensive correctness checks

**Recommended Approach**: Rebuild incrementally with a proper compiler pipeline while maintaining backward compatibility with existing tests.

---

## Table of Contents

1. [Architectural Design Flaws](./01-architectural-flaws.md)
2. [Correctness Issues and Bugs](./02-correctness-issues.md)
3. [Missing Features](./03-missing-features.md)
4. [Code Quality Issues](./04-code-quality.md)
5. [Performance Inefficiencies](./05-performance.md)
6. [Proposed New Architecture](./06-proposed-architecture.md)
7. [Testing Strategy](./07-testing-strategy.md)
8. [Incremental Rebuilding Plan](./08-rebuilding-plan.md)

---

## Quick Reference: Critical Files

| File                   | Lines | Responsibility            | Risk Level  |
| ---------------------- | ----- | ------------------------- | ----------- |
| `translate/codegen.rs` | 2,400 | Core translation logic    | 🔴 Critical |
| `translate/mod.rs`     | 800   | Compilation orchestration | 🟡 High     |
| `pvm/instruction.rs`   | 335   | PVM instruction encoding  | 🟢 Low      |
| `translate/stack.rs`   | 150   | Operand stack management  | 🟡 Medium   |

---

## Risk Assessment Matrix

| Risk | Probability | Impact | Risk Score |
|------|-------------|--------|------------|
| Silent data corruption from memory.copy | High | Critical | 🔴 **Critical** |
| Division by zero/overflow | Medium | High | 🔴 **High** |
| Stack overflow in complex programs | Medium | High | 🔴 **High** |
| Inability to add optimizations | High | Medium | 🟡 **Medium** |
| Register allocation bugs | Medium | High | 🔴 **High** |
| Maintenance burden | Certain | Medium | 🟡 **Medium** |

---

## Summary of Critical Issues

### 1. No Intermediate Representation (IR)

The compiler translates WASM directly to PVM machine code in a single pass. This is the root cause of many architectural problems:

- Cannot perform optimizations (constant folding, dead code elimination)
- Cannot do proper dataflow analysis
- Cannot verify correctness before code generation
- Debugging is nearly impossible (no IR to inspect)

**Example**: To add constant folding, you'd need to modify every operator handler in the 600-line `translate_op()` match statement.

### 2. Monolithic 2400-Line CodeGen Module

The `codegen.rs` file violates every principle of software engineering:

- Single function `translate_op()` handles 100+ WASM operators
- No separation between instruction selection and register allocation
- Control flow handling is scattered and ad-hoc
- Label management is manual and error-prone

### 3. Ad-Hoc Register Allocation

The register allocator is a hardcoded mess:

```rust
// From codegen.rs - hardcoded register assignments
const ARGS_PTR_REG: u8 = 7;      // r7
const ARGS_LEN_REG: u8 = 8;      // r8
const STACK_PTR_REG: u8 = 1;     // r1
const FIRST_LOCAL_REG: u8 = 9;   // r9
const MAX_LOCAL_REGS: usize = 4; // Only 4 locals in registers!
```

- No register liveness analysis
- Manual spill decisions scattered across code
- No way to optimize register usage

### 4. Manual Stack Management with Spilling

The operand stack spilling logic is fragile:

```rust
// From stack.rs
pub const fn needs_spill(depth: usize) -> bool {
    depth >= STACK_REG_COUNT  // Hardcoded at depth 5
}
```

- Spill decisions are based on depth, not liveness
- Manual spill/restore in function calls
- Complex interplay between `spill_push()`, `spill_pop()`, `pending_spill`

### 5. Control Flow is Hand-Crafted

Control flow translation uses manual label allocation and fixup:

```rust
// From codegen.rs
let else_label = self.alloc_label();
let end_label = self.alloc_label();
// ... emit code ...
self.fixups.push((fixup_idx, else_label));
// ... later ...
self.resolve_fixups()?;
```

- No control flow graph (CFG)
- No basic block analysis
- No dominance analysis for optimizations
- Manual fixup resolution is error-prone

### 6. Memory Model is Hardcoded and Fragile

Memory addresses are scattered as magic constants:

```rust
// From codegen.rs
const GLOBAL_MEMORY_BASE: i32 = 0x30000;
const SPILLED_LOCALS_BASE: i32 = 0x40000;
const EXIT_ADDRESS: i32 = -65536;
const RO_DATA_BASE: i32 = 0x10000;
```

- No memory layout abstraction
- Address calculations scattered throughout code
- Easy to break consistency

### 7. Testing is Insufficient

- Only 30 Rust unit tests vs 62 integration tests
- No unit tests for translation logic
- No property-based testing
- No fuzzing
- Integration tests run in TypeScript, not Rust

### 8. Error Handling is Incomplete

- Many `unwrap()` and `assert!()` calls instead of proper error propagation
- Division overflow not checked
- Memory bounds not validated at compile time

---

## Next Steps

See the detailed findings in each category document:

1. [01-architectural-flaws.md](./01-architectural-flaws.md) - Detailed architectural critique
2. [02-correctness-issues.md](./02-correctness-issues.md) - Bugs and correctness problems
3. [03-missing-features.md](./03-missing-features.md) - What's not implemented
4. [04-code-quality.md](./04-code-quality.md) - Code smells and maintainability issues
5. [05-performance.md](./05-performance.md) - Performance bottlenecks
6. [06-proposed-architecture.md](./06-proposed-architecture.md) - Better design proposal
7. [07-testing-strategy.md](./07-testing-strategy.md) - Comprehensive testing plan
8. [08-rebuilding-plan.md](./08-rebuilding-plan.md) - How to rebuild incrementally

---

*This review was conducted with the explicit instruction to take an extremely critical view. While the compiler works for the current test suite, the architecture requires significant improvement for production use.*
