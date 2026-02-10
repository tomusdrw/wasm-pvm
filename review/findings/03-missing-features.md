# 03 - Missing Features and Incomplete Implementations (Updated 2026-02-10)

**Category**: Feature Gaps  
**Impact**: WASM spec compliance, compatibility with existing code  
**Status**: Significant feature completeness in LLVM backend

---

## Summary

Since the original review, the compiler has achieved **360 integration tests passing** with the LLVM backend. This indicates substantial feature completeness. However, some features remain missing or incomplete.

**Feature Completeness**:

| Feature Category | Status | Notes |
|-----------------|--------|-------|
| i32/i64 arithmetic | ✅ Complete | Full support |
| Control flow (block/if/loop/br) | ✅ Complete | All structures |
| Function calls (direct/indirect) | ✅ Complete | Full support |
| Memory operations (load/store) | ✅ Complete | All variants |
| Bulk memory (fill/copy/grow) | ✅ Complete | Full support |
| Local variables | ✅ Complete | Full support |
| Globals | ✅ Complete | Full support |
| Type conversions | ✅ Complete | All conversions |
| Floating point | ❌ Rejected | PVM limitation |
| Passive data segments | ⚠️ Partial | Not implemented |
| Host calls (ecalli) | ❌ Missing | Not implemented |
| Multi-value returns | ⚠️ Partial | Globals-based workaround |

---

## Confirmed Missing Features

### Feature 1: Passive Data Segments (`memory.init`) 🔵

**Status**: Known Limitation   
**Severity**: Low  
**WASM Spec**: MVP feature for data segment initialization

#### Description

Only active data segments (initialized at module load time) are supported. Passive segments and the `memory.init` instruction are not implemented.

#### Evidence

```rust
// translate/mod.rs
// Passive data segments are ignored for now (used with memory.init)
```

#### Impact

- Cannot use `memory.init` to initialize memory at runtime
- Some WASM toolchains may generate passive segments
- Workaround: Use active segments only

#### Relevance

The AssemblyScript compiler used for testing appears to only use active segments, so this limitation doesn't block the primary use case.

---

### Feature 2: Floating Point Support ❌

**Status**: By Design (PVM Limitation)  
**Severity**: N/A (Expected Behavior)  
**WASM Spec**: MVP feature, but PVM lacks FP support

#### Description

PVM has no floating-point instructions. The compiler cannot support f32/f64 operations.

#### Current Handling

- WASM modules with float operations are rejected
- Some float truncation operations are stubbed to return 0 for dead code elimination paths

```rust
// ir/instruction.rs
I32TruncSatF64U,  // Stub - returns 0
I32TruncSatF64S,  // Stub - returns 0
// ... more truncation stubs
```

#### Impact

- Cannot compile programs with floating point math
- Math libraries using f64 fail
- AssemblyScript's `Math` module is unavailable

#### Workarounds

1. Use fixed-point arithmetic (integer math with scaling)
2. Implement software floating point (very slow)
3. Avoid float operations in source code

**Note**: For the anan-as use case, floats were eliminated by modifying the AssemblyScript source to use integer-only operations.

---

### Feature 3: Host Calls via `ecalli` 🔵

**Status**: Not Implemented   
**Severity**: Medium (for WASI support)  
**WASM Spec**: Host function call mechanism

#### Description

The PVM has an `ecalli` instruction for calling host-provided functions. The compiler doesn't support generating it.

#### Current Import Handling

Imported functions are stubbed:
- `abort` → TRAP
- Others → no-op with dummy return value (0)

```rust
// translate/codegen.rs (legacy)
if import_name == "abort" {
    emitter.emit(Instruction::Trap);
}
// For has_return, push dummy value
```

#### Impact

- Cannot use WASI (WebAssembly System Interface)
- No file I/O, clock, random, etc.
- Cannot call host-provided functions

#### When Needed

- WASI support for standard library functions
- Host-defined functionality
- System services access

**Recommendation**: Document as known limitation for V1. Implement in V2 if WASI support is required.

---

### Feature 4: True Multi-Value Returns 🟡

**Status**: Partial   
**Severity**: Low  
**WASM Spec**: Post-MVP proposal (now standard)

#### Description

WASM supports functions returning multiple values (e.g., `(result i32 i32)`). The compiler has partial support via a workaround.

#### Current Implementation

**Workaround**: Entry functions return `(ptr, len)` via globals or registers:

```rust
// Globals-based (legacy convention)
(global $result_ptr (mut i32) (i32.const 0))
(global $result_len (mut i32) (i32.const 0))

// Register-based (SPI convention)
// Entry returns ptr in r7, len in r8
```

#### Evidence

```rust
// translate/mod.rs
let entry_returns_ptr_len = if is_main {
    module.main_returns_ptr_len
} else { ... };

// Entry functions write to result globals or use r7/r8
```

#### Limitations

- Only entry functions (main/secondary) can effectively return multiple values
- Regular functions with multi-value returns may not work correctly
- The `entry_returns_ptr_len` flag controls this behavior

---

## Implementation Gaps

### Gap 1: Division Overflow Checking 🔴

**Status**: Needs Verification  
**Location**: `llvm_frontend/function_builder.rs`

**Concern**: LLVM's division instructions produce `poison` on overflow, not traps. WASM requires traps.

**WASM Required Behavior**:
- `i32.div_s` with divisor 0 → trap
- `i32.div_s` with `INT_MIN / -1` → trap

**LLVM Behavior**:
- `sdiv` with divisor 0 → undefined behavior (not trap)
- `sdiv` with `INT_MIN / -1` → poison value

**Action Required**: Verify the LLVM frontend emits explicit checks before division operations.

---

### Gap 2: Import Stub Completeness 🔴

**Status**: Needs Verification  
**Location**: `llvm_backend/lowering.rs`

**Concern**: The import handling may not be complete for all scenarios.

**Required Behavior**:
- Pop arguments from stack
- Push dummy return value (0) if `has_return`
- `abort` → TRAP

**Status**: Needs verification:
1. Does it recognize indirect calls to imports?
2. Does it push dummy return values?
3. How does it handle `call_indirect` to imported functions?

---

### Gap 3: Memory Bounds Checking 🟡

**Status**: Relies on PVM Hardware  
**Location**: `llvm_backend/lowering.rs`

**Current Approach**:
- WASM addresses are translated by adding `wasm_memory_base`
- PVM will trap on out-of-bounds access (address < 0x10000)
- No explicit WASM bounds checking (e.g., `addr + offset < memory_size`)

**Security Implication**:
- This is the standard approach for WASM-to-native compilation
- PVM's memory protection provides sandboxing
- Could potentially read/write beyond WASM memory into PVM runtime memory

**Comparison**: Similar to how wasmtime/wasm3 work - rely on hardware memory protection.

---

## Post-MVP Proposal Support

| Proposal | Status | Notes |
|----------|--------|-------|
| **Sign-extension ops** | ✅ Supported | `i32.extend8_s`, `i32.extend16_s`, etc. |
| **Non-trapping FP-to-int** | ⚠️ Stubbed | Returns 0 (for dead code) |
| **Multi-value** | 🟡 Partial | Entry functions only |
| **Bulk memory** | ✅ Supported | `memory.fill`, `memory.copy`, `memory.grow` |
| **Reference types** | ❌ Not implemented | `externref`, `funcref` |
| **SIMD** | ❌ Not implemented | Would require PVM support |
| **Threads** | ❌ Not implemented | Atomic operations, shared memory |
| **Fixed-width SIMD** | ❌ Not implemented | `v128` type |

---

## Feature Completeness Matrix (Detailed)

### Core Operations

| Operation | Status | Notes |
|-----------|--------|-------|
| `i32.add/sub/mul` | ✅ | Full support |
| `i32.div_u/div_s` | ⚠️ | Need overflow check verification |
| `i32.rem_u/rem_s` | ✅ | Full support |
| `i64.add/sub/mul` | ✅ | Full support |
| `i64.div_u/div_s` | ⚠️ | Need overflow check verification |
| `i64.rem_u/rem_s` | ✅ | Full support |
| `i32.and/or/xor/shl/shr` | ✅ | Full support |
| `i64.and/or/xor/shl/shr` | ✅ | Full support |
| `i32.clz/ctz/popcnt` | ✅ | Full support |
| `i64.clz/ctz/popcnt` | ✅ | Full support |
| `i32.rotl/rotr` | ✅ | Full support |
| `i64.rotl/rotr` | ✅ | Full support |
| `f32`/`f64` arithmetic | ❌ | PVM limitation (rejected) |

### Comparisons

| Operation | Status | Notes |
|-----------|--------|-------|
| `i32.eq/ne/lt/gt/le/ge` | ✅ | Full support |
| `i64.eq/ne/lt/gt/le/ge` | ✅ | Full support |
| `i32.eqz` | ✅ | Full support |
| `i64.eqz` | ✅ | Full support |

### Memory Operations

| Operation | Status | Notes |
|-----------|--------|-------|
| `i32.load/store` | ✅ | Full support |
| `i64.load/store` | ✅ | Full support |
| `i32.load8_u/s` | ✅ | Full support |
| `i32.load16_u/s` | ✅ | Full support |
| `i64.load8_u/s` | ✅ | Full support |
| `i64.load16_u/s` | ✅ | Full support |
| `i64.load32_u/s` | ✅ | Full support |
| `i32.store8/16` | ✅ | Full support |
| `i64.store8/16/32` | ✅ | Full support |
| `memory.size` | ✅ | Full support |
| `memory.grow` | ✅ | SBRK-based |
| `memory.fill` | ✅ | Loop-based |
| `memory.copy` | ✅ | Forward+backward paths |
| `memory.init` | ❌ | Passive segments not supported |

### Control Flow

| Operation | Status | Notes |
|-----------|--------|-------|
| `block`/`end` | ✅ | Full support |
| `if`/`else` | ✅ | Full support |
| `loop` | ✅ | Full support |
| `br` | ✅ | Full support |
| `br_if` | ✅ | Full support |
| `br_table` | ✅ | Linear search implementation |
| `return` | ✅ | Full support |
| `unreachable` | ✅ | Maps to TRAP |

### Function Operations

| Operation | Status | Notes |
|-----------|--------|-------|
| `call` (direct) | ✅ | Full support |
| `call_indirect` | ✅ | Signature check + dispatch table |
| `return_call` (tail call) | ❌ | Not in MVP |
| Multi-value return | ⚠️ | Entry functions only |

### Variable Access

| Operation | Status | Notes |
|-----------|--------|-------|
| `local.get/set/tee` | ✅ | Full support |
| `global.get/set` | ✅ | Full support |
| `drop` | ✅ | Full support |
| `select` | ✅ | Full support |

### Conversions

| Operation | Status | Notes |
|-----------|--------|-------|
| `i32.wrap_i64` | ✅ | Full support |
| `i64.extend_i32_u/s` | ✅ | Full support |
| `i32.extend8_s` | ✅ | Sign-extension ops |
| `i32.extend16_s` | ✅ | Sign-extension ops |
| `i64.extend8/16/32_s` | ✅ | Sign-extension ops |
| `i32.trunc_sat_f32_u` | ⚠️ | Stubbed (returns 0) |
| `i32.trunc_sat_f64_u` | ⚠️ | Stubbed (returns 0) |

---

## Recommendations

### Priority 1: Verify Critical Gaps

1. **Division overflow checks in LLVM** - Add explicit trap generation
2. **Import return values in LLVM** - Ensure dummy values are pushed
3. **Memory.copy overlap in LLVM** - Verify both paths work

### Priority 2: Nice to Have

4. **Passive data segments** - For completeness
5. **True multi-value returns** - Remove globals-based workaround
6. **ecalli support** - For WASI compatibility

### Priority 3: Future (V2+)

7. Reference types
8. Exception handling
9. Tail calls

---

## Conclusion

The compiler has achieved **impressive feature completeness** with 360 integration tests passing. The vast majority of WASM MVP features are supported.

**Key Gaps Remaining**:
- Division overflow traps (needs verification)
- Import handling completeness (needs verification)
- Passive data segments (low priority)
- Floating point (PVM limitation, by design)

**Verdict**: Feature-complete for the intended use case (AssemblyScript → PVM). Minor gaps remain for full WASM spec compliance.

---

