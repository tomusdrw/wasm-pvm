# 01 - Architectural Design Review

**Category**: Architecture Assessment  
**Impact**: Long-term maintainability and extensibility  
**Status**: ✅ Clean LLVM-based architecture

---

## Executive Summary

The WASM→PVM compiler has a **clean, modern architecture** using LLVM 18 as its intermediate representation. The legacy direct-translator backend has been removed, leaving a single, well-structured compilation pipeline.

**Architecture Overview**:

```
WASM → [llvm_frontend] → LLVM IR → [mem2reg] → [llvm_backend] → PVM bytecode
       (~1350 lines)     (SSA form)    (~1900 lines)      (SPI format)
```

---

## Architecture Strengths

### ✅ LLVM-Based IR Layer

**Status**: **ACHIEVED**

The compiler uses LLVM 18 via the `inkwell` crate as its intermediate representation.

**Benefits**:
- Proper SSA form via LLVM's mem2reg pass
- Mature optimization infrastructure
- Battle-tested control flow handling
- Type system alignment with WASM

**Implementation**:
```rust
// llvm_frontend/mod.rs
pub fn translate_wasm_to_llvm<'ctx>(
    context: &'ctx Context,
    wasm_module: &WasmModule,
) -> Result<Module<'ctx>> {
    let translator = WasmToLlvm::new(context, "wasm_module");
    translator.translate_module(wasm_module)
}
```

---

### ✅ Clear Phase Separation

**Status**: **ACHIEVED**

The compiler has clear, well-defined phases:

| Phase | Module | Responsibility |
|-------|--------|---------------|
| 1. Parsing | `translate/wasm_module.rs` | WASM sections → structured data |
| 2. IR Generation | `llvm_frontend/` | WASM → LLVM IR |
| 3. Optimization | LLVM passes | mem2reg, instcombine, etc. |
| 4. Lowering | `llvm_backend/` | LLVM IR → PVM instructions |
| 5. Assembly | `translate/mod.rs` | Entry headers, jump tables, data |

**Benefit**: Each phase can be understood, tested, and modified independently.

---

### ✅ Memory Layout Abstraction

**Status**: **ACHIEVED**

All PVM memory addresses centralized with helper functions.

**Location**: `translate/memory_layout.rs`

```rust
pub const RO_DATA_BASE: i32 = 0x10000;
pub const GLOBAL_MEMORY_BASE: i32 = 0x30000;
pub const SPILLED_LOCALS_BASE: i32 = 0x40000;

/// Full ASCII art diagram included
pub fn compute_wasm_memory_base(num_local_funcs: usize) -> i32 { ... }
pub fn spilled_local_addr(func_idx: usize, local_offset: i32) -> i32 { ... }
```

**Benefits**:
- Single source of truth for memory layout
- Self-documenting with ASCII art
- Helper functions prevent calculation errors

---

### ✅ Clean Module Structure

**Status**: **ACHIEVED**

No legacy code remaining. Clean separation:

```
crates/wasm-pvm/src/
├── lib.rs                    # Public API
├── error.rs                  # Error types
├── llvm_frontend/            # WASM → LLVM IR
├── llvm_backend/             # LLVM IR → PVM
├── translate/               # Parsing + orchestration
├── pvm/                     # Instruction encoding
└── spi.rs                   # SPI/JAM format
```

---

## Architecture Tradeoffs

### ⚠️ Stack-Slot Allocation (V1 Tradeoff)

**Current**: Each SSA value gets a dedicated stack slot

**Code**:
```rust
// llvm_backend/lowering.rs
fn alloc_slot_for_key(&mut self, key: ValKey) -> i32 {
    let offset = self.next_slot_offset;
    self.value_slots.insert(key, offset);
    self.next_slot_offset += 8;
    offset
}
```

**Rationale**: "Correctness-first, no register allocation"

**Impact**:
- Larger stack frames
- More memory operations
- Simpler implementation

**Future**: V2 should implement proper register allocation (linear scan or graph coloring).

---

### ⚠️ LLVM API Coupling

**Current**: Heavy dependence on inkwell/LLVM

**Impact**:
- Requires LLVM knowledge to contribute
- Hard to test without LLVM present
- API changes require updates

**Rationale**: Leverages mature infrastructure vs building custom IR

**Mitigation**: Well-contained within frontend/backend modules.

---

## Component Analysis

### LLVM Frontend

**Location**: `llvm_frontend/function_builder.rs` (~1350 lines)

**Strengths**:
- Clean separation from backend
- Uses LLVM's mature control flow (BasicBlocks, Phi nodes)
- PVM intrinsics abstract memory operations
- Proper error handling

**Complexity**: WASM structured control flow (block/if/loop) mapped to LLVM BasicBlocks is inherently complex but well-contained.

---

### LLVM Backend

**Location**: `llvm_backend/lowering.rs` (~1900 lines)

**Strengths**:
- Works with LLVM's SSA form
- Clear value slot abstraction
- Structured handling of LLVM basic blocks

**Concerns**:
- Large file size
- Stack-slot approach is simple but inefficient

**Responsibilities**:
1. Read LLVM IR basic blocks
2. Allocate stack slots for SSA values
3. Emit PVM instructions
4. Resolve jump fixups

---

### Translation Orchestration

**Location**: `translate/mod.rs` (~400 lines)

**Clean implementation**:
- Single `compile()` entry point
- Clear phase coordination
- No feature flags or conditional compilation

---

## Comparison: Before and After

| Aspect | Original State | Current State |
|--------|---------------|---------------|
| **IR Layer** | None (direct translation) | ✅ LLVM 18 SSA IR |
| **Architecture** | Monolithic 2400-line module | ✅ Separated phases |
| **Memory Layout** | Scattered constants | ✅ Centralized |
| **Backend Count** | 1 (direct) → 2 (direct + LLVM) | ✅ 1 (LLVM only) |
| **Code Paths** | Feature-flagged dual paths | ✅ Single clean path |
| **Maintainability** | High technical debt | ✅ Clean codebase |

---

## Testing Architecture

### Differential Tests

43 tests verify that both backends (when legacy existed) produced equivalent results:

```rust
// tests/differential.rs
fn differential_compile(fixture_name: &str) {
    let wasm = load_wat_fixture(fixture_name);
    let module = WasmModule::parse(&wasm).unwrap();
    
    let legacy = compile_legacy(&module).unwrap();
    let llvm = compile_via_llvm(&module).unwrap();
    
    assert_eq!(legacy.heap_pages(), llvm.heap_pages());
    assert_eq!(legacy.rw_data(), llvm.rw_data());
}
```

**Note**: These tests were valuable during the transition. Can be removed or repurposed for comparing against wasmtime.

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| LLVM version upgrade | Medium | Medium | Pin inkwell version |
| LLVM API breaking changes | Low | High | inkwell provides stability |
| Stack-slot performance | Certain | Medium | V2 register allocation |
| Complex function compilation | Medium | Low | Frame size limits |

---

## Recommendations

### Short Term

1. **Verify edge case correctness**:
   - Division overflow handling
   - Memory.copy overlap

2. **Add documentation**:
   - LLVM lowering strategy
   - Control flow translation approach

### Medium Term

3. **Explore optimizations**:
   - Custom LLVM passes
   - Slot reuse within functions

4. **Add fuzzing**:
   - wasm-smith integration
   - Differential testing against wasmtime

### Long Term (V2)

5. **Register allocation**:
   - Replace stack-slot approach
   - Linear scan or graph coloring

6. **Consider custom IR**:
   - If LLVM coupling becomes problematic
   - Use existing `ir/` infrastructure

---

## Summary

The WASM→PVM compiler now has a **clean, modern, maintainable architecture**:

✅ **LLVM-based IR** - SSA form, mature optimizations  
✅ **Clear phase separation** - Parsing → IR → Lowering → Assembly  
✅ **Centralized abstractions** - Memory layout, instruction encoding  
✅ **Clean module structure** - No legacy code  
✅ **Single code path** - No dual maintenance burden  

**Remaining tradeoffs**:
- Stack-slot allocation (V1 simplicity)
- LLVM API coupling (infrastructure vs custom)

**Verdict**: Architecture is sound and ready for production use with minor edge case verification.

---

*Review conducted 2026-02-10*
