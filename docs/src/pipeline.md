# Compiler Pipeline

The compiler translates WebAssembly to PVM bytecode in five stages:

```text
  ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
  │ Adapter  │     │  WASM →  │     │   LLVM   │     │ LLVM IR  │     │   SPI    │
  │  Merge   │────►│  LLVM IR │────►│  Passes  │────►│  → PVM   │────►│ Assembly │
  └──────────┘     └──────────┘     └──────────┘     └──────────┘     └──────────┘
   (optional)       inkwell          mem2reg,etc      Rust backend     JAM blob
```

## Stage 1: Adapter Merge (Optional)

**File**: `crates/wasm-pvm/src/translate/adapter_merge.rs`

When a WAT adapter module is provided (`--adapter`), it is merged into the main WASM binary. Adapter exports replace matching WASM imports, enabling complex import resolution logic (pointer conversion, memory reads, host calls). Uses `wasm-encoder` to build the merged binary.

## Stage 2: WASM → LLVM IR

**File**: `crates/wasm-pvm/src/llvm_frontend/function_builder.rs` (~1350 lines)

Each WASM function is translated to LLVM IR using [inkwell](https://github.com/TheDan64/inkwell) (LLVM 18 bindings). PVM-specific intrinsics (`@__pvm_load_i32`, `@__pvm_store_i32`, etc.) are used for memory operations instead of direct pointer arithmetic, avoiding `unsafe` GEP/inttoptr patterns.

All values are treated as i64 (matching PVM's 64-bit registers).

## Stage 3: LLVM Optimization Passes

**File**: `crates/wasm-pvm/src/llvm_frontend/function_builder.rs`

Three optimization phases run sequentially:

1. **Pre-inline cleanup**: `mem2reg` (SSA promotion), `instcombine`, `simplifycfg`
2. **Inlining** (optional): `cgscc(inline)` — function inlining for small callees
3. **Post-inline cleanup**: `instcombine<max-iterations=2>`, `simplifycfg`, `gvn` (redundancy elimination), `simplifycfg`, `dce` (dead code removal)

## Stage 4: LLVM IR → PVM Bytecode

**Files**: `crates/wasm-pvm/src/llvm_backend/` (7 modules)

A custom Rust backend reads LLVM IR and emits PVM instructions:

| Module | Responsibility |
|--------|---------------|
| `emitter.rs` | Core emitter, value slot management, register cache |
| `alu.rs` | Arithmetic, logic, comparisons, conversions, fused bitwise |
| `memory.rs` | Load/store, memory intrinsics, word-sized bulk ops |
| `control_flow.rs` | Branches, phi nodes, switch, return |
| `calls.rs` | Direct/indirect calls, import stubs |
| `intrinsics.rs` | PVM + LLVM intrinsic lowering |
| `regalloc.rs` | Linear-scan register allocator |

Key optimizations at this stage:
- **Per-block register cache**: eliminates redundant loads (~50% gas reduction)
- **Cross-block cache propagation**: for single-predecessor blocks
- **ICmp+Branch fusion**: combines compare and branch into one PVM instruction
- **Linear-scan register allocation**: assigns loop values to callee-saved registers
- **Peephole optimizer**: fuses immediate chains, eliminates dead stores

## Stage 5: SPI Assembly

**File**: `crates/wasm-pvm/src/translate/mod.rs`

Packages everything into a JAM/SPI program blob:

1. Build entry header (jump to main function, optional secondary entry)
2. Build dispatch table (for `call_indirect`) → `ro_data`
3. Build globals + WASM memory initial data → `rw_data` (with trailing zero trim)
4. Encode PVM program blob (jump table + bytecode + instruction mask)
5. Write SPI header (ro_data_len, rw_data_len, heap_pages, stack_size)
