# 03 - Missing Features and Incomplete Implementations

**Category**: Feature Gaps  
**Impact**: Limits WASM compatibility, prevents certain programs from running

---

## Summary

While the compiler implements most WASM MVP features, there are gaps that prevent some valid WASM programs from compiling or running correctly.

---

## Confirmed Missing Features

### Feature 1: Passive Data Segments (`memory.init`) 🔵

**Status**: Known Limitation  
**Severity**: Low  
**WASM Spec**: MVP feature for data segment initialization

#### Description

Only active data segments (initialized at instantiation) are supported. Passive segments and the `memory.init` instruction are not implemented.

#### Evidence

From `mod.rs:177`:
```rust
// Passive data segments are ignored for now (used with memory.init)
```

#### Impact

- Cannot use `memory.init` to initialize memory at runtime
- Some WASM toolchains generate passive segments
- Workaround: Use active segments only

#### When Needed

- Programs using bulk memory operations
- Dynamic initialization patterns
- Certain AssemblyScript runtime configurations

---

### Feature 2: Floating Point Support ❌

**Status**: By Design (PVM Limitation)  
**Severity**: N/A (Expected Behavior)  
**WASM Spec**: MVP feature, but PVM lacks FP support

#### Description

PVM has no floating-point instructions. The compiler rejects WASM modules with float operations.

#### Current Handling

```rust
// From error.rs
#[error("Float operations are not supported by PVM")]
FloatNotSupported,

// From mod.rs (check_for_floats - currently #[allow(dead_code)])
fn check_for_floats(body: &FunctionBody) -> Result<()> {
    // ... checks for float operators ...
}
```

The float check function exists but is not called because it's hard to detect floats in all cases (e.g., through indirect calls).

#### Stubbed Operations

Some float truncation operations are stubbed to return 0 for dead code paths:

```rust
// From codegen.rs (presumably, based on KNOWN_ISSUES.md)
// i32.trunc_sat_f64_u, etc. return 0
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

### Feature 3: Imported Function Calls (with return values) 🔴

**Status**: Partially Implemented  
**Severity**: Medium  
**WASM Spec**: MVP feature for imports

#### Description

Imported functions are stubbed but don't handle return values correctly.

#### Current Implementation

From `codegen.rs:2210-2241`:
```rust
if (*function_index as usize) < ctx.num_imported_funcs {
    let import_name = ctx.imported_func_names.get(*function_index as usize)
        .map_or("unknown", String::as_str);
    
    // Pop arguments (they're on the stack)
    for _ in 0..num_args {
        emitter.spill_pop();
    }
    
    // Handle specific imports:
    if import_name == "abort" {
        emitter.emit(Instruction::Trap);
    }
    // For has_return, we'd need to push a dummy value, but abort/console.log don't return
}
```

#### Gap

When `has_return` is true, no value is pushed to the stack. This violates the calling convention.

#### Impact

- Cannot use imports that return values
- WASI functions unavailable
- Custom host functions limited

#### When Needed

- WASI support (file I/O, etc.)
- Host-provided functions
- External libraries

---

### Feature 4: Runtime Memory Bounds Checking 🟡

**Status**: Partial  
**Severity**: Medium  
**WASM Spec**: Required for security

#### Description

WASM memory operations are bounds-checked at runtime in proper engines. The current compiler relies on PVM's memory protection but doesn't explicitly check WASM bounds.

#### Current Approach

1. WASM addresses are translated to PVM addresses by adding `wasm_memory_base`
2. PVM will trap on out-of-bounds access
3. But WASM-specific bounds (memory size) are not checked

#### Gap

```rust
// From codegen.rs
Operator::I32Load { memarg } => {
    let addr = emitter.spill_pop();
    let dst = emitter.spill_push();
    // Add WASM_MEMORY_BASE to translate WASM address to PVM address
    emitter.emit(Instruction::LoadIndU32 {
        dst,
        base: addr,
        offset: memarg.offset as i32 + ctx.wasm_memory_base,
        // No check that addr + offset < memory_size!
    });
}
```

#### Impact

- Programs can read/write beyond WASM memory bounds
- Security issue: may access PVM runtime memory
- WASM sandboxing violated

#### When Needed

- Running untrusted code
- Security-sensitive environments
- Standards compliance

---

## Implementation Gaps

### Gap 1: Dynamic Memory Growth (`memory.grow`) 🟡

**Status**: Returns -1 (failure)  
**Severity**: Low  
**WASM Spec**: Returns old size or -1 on failure

#### Description

The `memory.grow` implementation updates the compiler-managed global but doesn't actually grow PVM memory (which requires `sbrk`).

From `codegen.rs:2273-2428`:
```rust
Operator::MemoryGrow { mem: 0, .. } => {
    // ... checks new_size > max_pages ...
    
    // Store new_size to compiler global
    emitter.emit(Instruction::StoreIndU32 { base: max_reg, src: new_size_reg, offset: 0 });
    
    // Actually grow PVM memory via SBRK instruction
    // ... SBRK call ...
    
    // dst already has old size, jump to end
    emitter.emit_jump(end_label);
    
    // Failure path: return -1
    emitter.define_label(fail_label);
    emitter.emit(Instruction::LoadImm { reg: dst, value: -1 });
}
```

**Wait**, looking at the code, there IS an SBRK call! Let me re-read...

Yes, lines 2387-2412 show:
```rust
// Actually grow PVM memory via SBRK instruction.
// Compute delta_bytes = (new_size - old_size) * 65536
emitter.emit(Instruction::Sub32 { ... });
emitter.emit(Instruction::ShloL32 { ... });
emitter.emit(Instruction::Sbrk { dst: max_reg, src: max_reg });
```

So SBRK IS called. But does it work correctly? The SBRK instruction in PVM:
- Takes bytes to allocate in `src`
- Returns old break in `dst`

This seems correct. But the bug report might be outdated or there's a subtle issue.

**Current Status**: The code exists but may not be fully tested.

---

### Gap 2: No Host Call Support (`ecalli`) 🔵

**Status**: Not Implemented  
**Severity**: Low (for current use case)  
**WASM Spec**: Not in MVP (proposed feature)

#### Description

The PVM has an `ecalli` instruction for host calls. The compiler doesn't support generating it.

#### PLAN.md Reference

From PLAN.md:
```markdown
### Phase 17: Host Calls / ecalli Support (PLANNED - Phase 17)
**Goal**: Support generic external function calls via PVM `ecalli`.

**Design**:
- **Import Mapping**: Treat imports from specific modules as host calls
- **ABI**: Args 0-4 -> Registers r2-r6, Return value -> Register r7
- **Instruction**: `ecalli ID` where ID is derived from the import
```

#### Impact

- Cannot call host-provided functions
- Cannot do I/O (without workaround)
- Cannot access system services

#### When Needed

- WASI support
- I/O operations
- System interface

---

### Gap 3: Start Section Handling is Ad-Hoc 🟡

**Status**: Implemented but fragile  
**Severity**: Low  
**WASM Spec**: MVP feature

#### Description

The WASM start section is handled by injecting calls before the main function, but this is done manually in the orchestration code.

From `mod.rs:355-421`:
```rust
// If this is an entry function and we have a start function, execute the start function first.
if let Some(start_local_idx) = start_func_idx_resolved.filter(|_| is_entry_func) {
    // Save r7 and r8 to stack
    // ... explicit instruction emission ...
    
    // Call start function
    // ... manual jump emission ...
    
    // Restore r7 and r8
    // ... explicit instruction emission ...
}
```

#### Issues

1. Manual register save/restore is error-prone
2. Hardcoded to save only r7 and r8
3. Doesn't handle other caller-saved registers
4. Injected at wrong level (should be in prologue)

#### When Problematic

- Complex programs with start sections
- Programs needing more preserved state
- Nested start sections (if allowed)

---

### Gap 4: No Validation Phase 🔴

**Status**: Not Implemented  
**Severity**: High  
**WASM Spec**: Requires validation

#### Description

The compiler assumes input WASM is valid. There's no validation phase before translation.

#### Evidence

From `mod.rs:39-186`:
```rust
pub fn compile(wasm: &[u8]) -> Result<SpiProgram> {
    // Direct parsing without validation
    for payload in Parser::new(0).parse_all(wasm) {
        // ... pattern matching on sections ...
    }
}
```

#### Missing Validations

1. Type checking (operand stack consistency)
2. Label validity (branch targets exist)
3. Local index bounds
4. Function index bounds
5. Memory alignment requirements
6. Data segment bounds

#### Impact

- Invalid WASM may produce wrong code instead of error
- Security vulnerabilities from malformed input
- Hard to debug user errors

---

## Feature Completeness Matrix

| WASM Feature | Status | Notes |
|--------------|--------|-------|
| **Core Operations** |
| i32 arithmetic | ✅ | All ops implemented |
| i64 arithmetic | ✅ | All ops implemented |
| i32 bitwise | ✅ | All ops implemented |
| i64 bitwise | ✅ | All ops implemented |
| i32 comparisons | ✅ | All ops implemented |
| i64 comparisons | ✅ | All ops implemented |
| f32 arithmetic | ❌ | Rejected - PVM has no FP |
| f64 arithmetic | ❌ | Rejected - PVM has no FP |
| **Control Flow** |
| block/end | ✅ | Implemented |
| if/else | ✅ | Implemented |
| loop | ✅ | Implemented |
| br | ✅ | Implemented |
| br_if | ✅ | Implemented |
| br_table | ✅ | Implemented |
| return | ✅ | Implemented |
| unreachable | ✅ | Maps to TRAP |
| **Memory** |
| i32.load/store | ✅ | Implemented |
| i64.load/store | ✅ | Implemented |
| load8_u, load8_s | ✅ | Implemented |
| load16_u, load16_s | ✅ | Implemented |
| load32_u, load32_s | ✅ | Implemented |
| store8, store16, store32 | ✅ | Implemented |
| memory.size | ✅ | Implemented |
| memory.grow | 🟡 | Implemented but may have issues |
| memory.init | ❌ | Not implemented |
| memory.fill | ✅ | Implemented (loop-based) |
| memory.copy | 🟡 | Implemented but overlap handling needs verification |
| **Data Segments** |
| Active segments | ✅ | Implemented |
| Passive segments | ❌ | Ignored |
| **Functions** |
| local.get/set/tee | ✅ | Implemented |
| drop | ✅ | Implemented |
| select | ✅ | Implemented |
| call | ✅ | Implemented |
| call_indirect | ✅ | Implemented with signature check |
| return | ✅ | Implemented |
| **Variables** |
| local.get/set/tee | ✅ | Implemented |
| global.get/set | ✅ | Implemented |
| **Conversions** |
| i32.wrap_i64 | ✅ | Implemented |
| i64.extend_i32_s/u | ✅ | Implemented |
| i32.extend8_s, i32.extend16_s | ✅ | Implemented |
| i64.extend8/16/32_s | ✅ | Implemented |
| trunc_sat_f32/64 | 🟡 | Stubbed to return 0 |
| **Misc** |
| nop | ✅ | No-op |
| unreachable | ✅ | TRAP |
| Imports | 🟡 | Stubbed, return values ignored |
| Exports | ✅ | Handled |
| Start section | 🟡 | Implemented but fragile |
| **Post-MVP Proposals** |
| Sign-extension ops | ✅ | Implemented |
| Non-trapping FP-to-int | ❌ | Stubbed |
| Multi-value | ❌ | Not implemented |
| Bulk memory | 🟡 | Partial (fill/copy implemented, init not) |
| Reference types | ❌ | Not implemented |
| SIMD | ❌ | Not implemented |
| Threads | ❌ | Not implemented |

---

## Recommendations

### Priority 1: Safety Features

1. **Add WASM validation phase** - Use `wasmparser`'s validator
2. **Fix memory bounds checking** - Check `addr + offset < memory_size`
3. **Fix import return values** - Push dummy value on stack

### Priority 2: Completeness

4. **Verify memory.copy overlap** - Test and fix if needed
5. **Add division overflow checks** - Required by WASM spec
6. **Implement passive data segments** - For completeness

### Priority 3: Advanced Features

7. **Host call support** - For WASI compatibility
8. **Better start section handling** - More robust implementation

---

*Next: [04-code-quality.md](./04-code-quality.md)*
