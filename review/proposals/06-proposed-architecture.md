# 06 - Architecture Implementation

**Category**: Architecture Status  
**Goal**: Document implemented architecture  
**Status**: ✅ LLVM-based architecture complete

---

## Summary

The WASM→PVM compiler has been implemented as a **clean, LLVM-based compiler** with a single backend. The architecture uses LLVM 18 as its intermediate representation, providing SSA form and mature optimization infrastructure.

## Implemented Architecture

### Compilation Pipeline

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           WASM Binary Input                                  │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  PHASE 1: WASM Parsing (translate/wasm_module.rs)                            │
│  - wasmparser-based section extraction                                       │
│  - Produces WasmModule struct                                                │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  PHASE 2: LLVM IR Generation (llvm_frontend/function_builder.rs)            │
│  - Translates WASM operators → LLVM IR via inkwell                         │
│  - Uses alloca for locals initially                                          │
│  - PVM intrinsics for memory ops (@__pvm_load_i32, etc.)                     │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  PHASE 3: LLVM Optimization                                                │
│  - mem2reg: Promotes alloca locals to SSA registers                          │
│  - instcombine: Combines redundant instructions                            │
│  - simplifycfg: Simplifies control flow                                      │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  PHASE 4: PVM Lowering (llvm_backend/lowering.rs)                          │
│  - Reads LLVM IR basic blocks                                               │
│  - Stack-slot allocation for SSA values                                      │
│  - Emits PVM instructions with fixups                                        │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  PHASE 5: SPI Assembly (translate/mod.rs)                                    │
│  - Entry header generation                                                   │
│  - Call fixup resolution                                                     │
│  - Jump table construction                                                   │
│  - RO/RW data packaging                                                     │
└─────────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                           JAM Binary Output                                  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Components

### LLVM Frontend

**Location**: `crates/wasm-pvm/src/llvm_frontend/`

**Files**:
- `mod.rs` (24 lines) - Public interface
- `function_builder.rs` (~1350 lines) - Core translation

**Responsibilities**:
1. Create LLVM module and context via inkwell
2. Declare PVM intrinsics for memory operations
3. Translate WASM operators to LLVM IR
4. Handle control flow (block/if/loop) via LLVM BasicBlocks
5. Use Phi nodes for SSA form

**Key Design**:
```rust
pub struct WasmToLlvm<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    pvm_intrinsics: PvmIntrinsics<'ctx>,  // Memory operations
    operand_stack: Vec<IntValue<'ctx>>,   // Stack machine
    control_stack: Vec<ControlFrame<'ctx>>, // WASM structured control
}
```

**Status**: ✅ Fully implemented, handles all WASM MVP operators.

---

### LLVM Backend (Lowering)

**Location**: `crates/wasm-pvm/src/llvm_backend/`

**Files**:
- `mod.rs` (8 lines) - Public interface
- `lowering.rs` (~1900 lines) - Core lowering

**Responsibilities**:
1. Read LLVM IR (post-mem2reg, in SSA form)
2. Allocate stack slots for each SSA value
3. Emit PVM instructions
4. Handle calling conventions
5. Resolve jump fixups

**Key Design**:
```rust
struct PvmEmitter<'ctx> {
    instructions: Vec<Instruction>,
    block_labels: HashMap<BasicBlock<'ctx>, usize>,  // LLVM BB → PVM label
    value_slots: HashMap<ValKey, i32>,                // LLVM value → stack slot
    next_slot_offset: i32,                            // Bump allocator
}
```

**Status**: ✅ Fully implemented, handles LLVM IR → PVM.

---

### Translation Orchestration

**Location**: `crates/wasm-pvm/src/translate/`

**Files**:
- `mod.rs` (~400 lines) - Compilation pipeline
- `wasm_module.rs` (~300 lines) - WASM parsing
- `memory_layout.rs` (~93 lines) - Memory constants

**Clean Implementation**:
```rust
pub fn compile(wasm: &[u8]) -> Result<SpiProgram> {
    let module = WasmModule::parse(wasm)?;
    compile_module(&module)
}

pub fn compile_module(module: &WasmModule) -> Result<SpiProgram> {
    // Phase 1: WASM → LLVM IR
    let context = Context::create();
    let llvm_module = llvm_frontend::translate_wasm_to_llvm(&context, module)?;
    
    // Phase 2: LLVM IR → PVM for each function
    for local_func_idx in 0..module.functions.len() {
        let translation = llvm_backend::lower_function(...)?;
        all_instructions.extend(translation.instructions);
    }
    
    // Phase 3: Build SPI program
    Ok(SpiProgram::new(blob).with_ro_data(ro_data).with_rw_data(rw_data))
}
```

---

### Module Structure

```
crates/wasm-pvm/src/
├── lib.rs                    # Public API (26 lines)
├── error.rs                  # Error types (thiserror)
├── llvm_frontend/           # WASM → LLVM IR
│   ├── mod.rs              # Frontend interface
│   └── function_builder.rs # Core translation (~1350 lines)
├── llvm_backend/           # LLVM IR → PVM
│   ├── mod.rs              # Backend interface
│   └── lowering.rs         # Core lowering (~1900 lines)
├── translate/              # Parsing + orchestration
│   ├── mod.rs              # Compilation pipeline (~400 lines)
│   ├── memory_layout.rs    # Memory constants (~93 lines)
│   └── wasm_module.rs      # WASM parsing (~300 lines)
├── pvm/                   # PVM instruction definitions
│   ├── instruction.rs      # Instruction enum + encoding (~332 lines)
│   ├── opcode.rs           # Opcode constants (~114 lines)
│   └── blob.rs             # Program blob format (~143 lines)
└── spi.rs                 # SPI/JAM container format
```

**Total Lines**: ~4000 lines of Rust (excluding tests)

---

## Design Decisions

### Why LLVM?

**Pros**:
- Mature, battle-tested infrastructure
- SSA form via mem2reg
- Existing optimization passes
- Type system alignment with WASM

**Cons**:
- API coupling (inkwell/LLVM)
- Additional dependency
- Requires LLVM knowledge

**Verdict**: LLVM provides significant value for the tradeoffs.

---

### Why Stack-Slot Allocation?

**Decision**: Conservative stack-slot approach instead of register allocation

**Code**:
```rust
// "Each SSA value gets a dedicated stack slot (correctness-first, no register allocation)"
fn alloc_slot_for_key(&mut self, key: ValKey) -> i32 {
    let offset = self.next_slot_offset;
    self.value_slots.insert(key, offset);
    self.next_slot_offset += 8;
    offset
}
```

**Rationale**: 
- V1 focus on correctness over performance
- PVM has 13 registers - limited allocation space
- Simpler implementation

**Future**: V2 should implement linear scan or graph coloring register allocation.

---

### Why PVM Intrinsics?

**Design**: Memory operations via intrinsics (`__pvm_load_i32`, etc.)

**Benefits**:
- Clean separation between frontend and backend
- Frontend doesn't know PVM details
- Backend can optimize intrinsic lowering

**Example**:
```rust
// Frontend: Call intrinsic
let result = builder.build_call(pvm_intrinsics.load_i32, &[address], "load")?;

// Backend: Lower to PVM instruction
if is_pvm_load_intrinsic(call) {
    emit(Instruction::LoadIndU32 { dst, base, offset });
}
```

---

## Differential Testing

During development, differential tests compared outputs:

```rust
fn differential_compile(fixture_name: &str) {
    let wasm = load_wat_fixture(fixture_name);
    let module = WasmModule::parse(&wasm).unwrap();
    
    let legacy = compile_legacy(&module).unwrap();
    let llvm = compile_via_llvm(&module).unwrap();
    
    assert_eq!(legacy.heap_pages(), llvm.heap_pages());
    assert_eq!(legacy.rw_data(), llvm.rw_data());
    assert_eq!(legacy.ro_data().len(), llvm.ro_data().len());
}
```

**Purpose**: Verify LLVM backend matched proven legacy behavior.

**Status**: 43 differential tests passing.

---

## Testing Strategy

| Test Type | Count | Purpose |
|-----------|-------|---------|
| Integration tests | 360+ | End-to-end correctness |
| Differential tests | 43 | Backend equivalence (during transition) |
| Rust unit tests | ~50 | Component testing |
| Property-based | Planned | Fuzzing with wasm-smith |

**Coverage**: Comprehensive for WASM MVP features.

---

## What Was Implemented

| Proposal | Status | Notes |
|----------|--------|-------|
| **Add IR layer** | ✅ **DONE** | LLVM 18 via inkwell |
| **Separate phases** | ✅ **DONE** | Frontend/backend separation |
| **Memory layout abstraction** | ✅ **DONE** | `memory_layout.rs` |
| **Clean module structure** | ✅ **DONE** | No legacy code |
| **Graph coloring allocator** | ❌ **NOT DONE** | Stack-slot for V1 |
| **CFG-based control flow** | ✅ **DONE** | LLVM provides CFG |
| **Differential testing** | ✅ **DONE** | 43 tests comparing backends |
| **Single backend** | ✅ **DONE** | Legacy removed |

---

## Remaining Work

### High Priority (V1.x)

1. **Verify edge case correctness**:
   - Division overflow checks
   - Memory.copy overlapping regions
   - Import return value handling

2. **Add targeted tests** for edge cases

### Medium Priority

3. **Improve documentation**:
   - LLVM lowering strategy
   - Control flow translation

4. **Add benchmarks**:
   - Compilation time
   - Generated code size
   - Execution speed

5. **Add fuzzing**:
   - wasm-smith integration
   - Differential testing against wasmtime

### Long Term (V2)

6. **Register allocation**:
   - Replace stack-slot approach
   - Linear scan or graph coloring

7. **Custom optimizations**:
   - Peephole passes on LLVM IR
   - Strength reduction

8. **SIMD support** (if PVM adds vector instructions)

---

## Architecture Benefits

| Benefit | Status |
|---------|--------|
| **Can add optimizations** | ✅ LLVM pass system |
| **Can verify correctness** | ✅ Can inspect LLVM IR |
| **Debugging possible** | ✅ LLVM IR is inspectable |
| **Supports future features** | ✅ LLVM has SIMD, threads, etc. |
| **Testability** | ✅ Frontend/backend separated |
| **Extensibility** | ✅ LLVM ecosystem |
| **Clean codebase** | ✅ Single backend, no legacy |

---

## Conclusion

The WASM→PVM compiler architecture has been **successfully implemented** as a clean, LLVM-based compiler:

✅ **LLVM IR layer** - SSA form, mature optimizations  
✅ **Clear phase separation** - Parsing → IR → Lowering → Assembly  
✅ **Centralized abstractions** - Memory layout, instruction encoding  
✅ **Single code path** - No legacy maintenance burden  
✅ **Comprehensive tests** - 360+ integration tests passing  

**Remaining work**: Edge case verification, documentation, performance optimization (register allocation in V2).

---

*Architecture review conducted 2026-02-10*
