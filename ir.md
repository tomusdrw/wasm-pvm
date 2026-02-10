# Migration Plan: Replace Custom IR with LLVM IR

## Context

The current wasm-pvm compiler has a custom stack-based IR that is a 1:1 transliteration of WASM opcodes. This provides no abstraction benefit and blocks future use of LLVM's optimization infrastructure. The goal is to replace the custom IR with proper LLVM IR:

```
WASM → [inkwell] → LLVM IR → [mem2reg + optional passes] → [Rust PVM backend] → PVM bytecode
```

The PVM backend is written in Rust (reading LLVM IR via inkwell's iteration API), NOT as a native LLVM C++ target. This gives us LLVM's SSA/CFG representation and optimization passes without the enormous effort of writing a TableGen-based LLVM backend.

**Focus**: Correctness only. No optimization work beyond `mem2reg` (which is required for usable SSA form).

**Constraint**: `unsafe_code = "deny"` at workspace level. We will use PVM-specific intrinsic functions for memory operations to avoid inkwell's `unsafe` GEP/inttoptr APIs entirely.

---

## Phase 1: Infrastructure & Feature Flag Setup -- DONE

**Goal**: Both pipelines coexist; existing tests unaffected.

**Changes**:
- Add `inkwell` to workspace dependencies in `/Cargo.toml`:
  ```toml
  inkwell = { version = "0.8", features = ["llvm18-0"] }
  ```
- Add optional dependency + feature flag in `crates/wasm-pvm/Cargo.toml`:
  ```toml
  [features]
  llvm-backend = ["dep:inkwell"]

  [dependencies]
  inkwell = { workspace = true, optional = true }
  ```
- Create module stubs: `src/llvm_frontend/mod.rs`, `src/llvm_backend/mod.rs`
- Gate with `#[cfg(feature = "llvm-backend")]` in `src/lib.rs`
- Rename current `compile()` to `compile_legacy()`, add dispatch:
  ```rust
  pub fn compile(wasm: &[u8]) -> Result<SpiProgram> {
      #[cfg(feature = "llvm-backend")]
      return compile_via_llvm(wasm);
      #[cfg(not(feature = "llvm-backend"))]
      return compile_legacy(wasm);
  }
  ```
- Extract WASM section parsing from `translate/mod.rs` into a shared `WasmModule` struct usable by both pipelines

**Files**: `Cargo.toml`, `crates/wasm-pvm/Cargo.toml`, `src/lib.rs`, `src/translate/mod.rs` (refactor), new `src/translate/wasm_module.rs`, new `src/llvm_frontend/mod.rs`, new `src/llvm_backend/mod.rs`

**Verify**: `cargo test` passes (legacy path unchanged). `cargo check --features llvm-backend` compiles.

---

## Phase 2: WASM → LLVM IR — Scaffolding, Constants & Arithmetic -- DONE

**Goal**: Generate valid LLVM IR for trivial WASM functions (no control flow, no memory).

**What to build**:
- `llvm_frontend/function_builder.rs` with `WasmToLlvm<'ctx>` struct holding:
  - `context`, `module`, `builder` (inkwell trinity)
  - `operand_stack: Vec<IntValue<'ctx>>` — simulates WASM value stack during translation
  - `locals: Vec<PointerValue<'ctx>>` — alloca slots for WASM locals
  - Cached `i32_type`, `i64_type`
- Translate WASM function signatures → LLVM function types (all params/returns as i64 for uniform 64-bit PVM registers)
- For each local: `alloca i64` in entry block, zero-init non-params
- Translate directly from `wasmparser::Operator` (skip old IR layer entirely):
  - `I32Const(v)` / `I64Const(v)` → const int, push stack
  - `LocalGet(i)` → load from alloca, push
  - `LocalSet(i)` → pop, store to alloca
  - `LocalTee(i)` → peek + store (no pop)
  - `GlobalGet(i)` / `GlobalSet(i)` → load/store LLVM global variables
  - `I32Add/Sub/Mul/DivU/DivS/RemU/RemS` → `build_int_add` etc., push result
  - `I64Add/Sub/Mul/...` → same with i64
  - All bitwise: `And/Or/Xor/Shl/ShrU/ShrS` → corresponding builder calls
  - All comparisons: `Eq/Ne/LtU/LtS/...` → `build_int_compare` + `zext i1 to i32/i64`
  - `Eqz` → compare with 0
  - `Clz/Ctz/Popcnt` → LLVM `ctlz`/`cttz`/`ctpop` intrinsics
  - `Rotl/Rotr` → LLVM `fshl`/`fshr` intrinsics
  - Type conversions: `I32WrapI64` → trunc, `I64ExtendI32S/U` → sext/zext, sign-extends → trunc+sext
  - `Select` → `build_select`
  - `Drop` → pop and discard
  - `Return` → `build_return`
  - `Nop` → nothing
  - `Unreachable` → `build_unreachable`
- Run `mem2reg` pass after building each module

**Files**: new `src/llvm_frontend/function_builder.rs`, `src/llvm_frontend/mod.rs`

**Verify**: Unit tests compile WAT like `(func (param i32 i32) (result i32) local.get 0 local.get 1 i32.add)`, print LLVM IR text, verify it contains expected `add i32` instructions. Call `module.verify()` to confirm validity.

---

## Phase 3: WASM → LLVM IR — Control Flow -- DONE

**Goal**: Translate WASM structured control flow into LLVM basic blocks, branches, and phi nodes.

**This was the hardest phase.** WASM's nested block/loop/if structure becomes a flat CFG.

**What to build** — add to `WasmToLlvm`:
- `control_stack: Vec<ControlFrame<'ctx>>`:
  ```rust
  enum ControlFrame<'ctx> {
      Block  { merge_bb: BasicBlock<'ctx>, result_phi: Option<PhiValue<'ctx>>, stack_depth: usize },
      Loop   { header_bb: BasicBlock<'ctx>, merge_bb: BasicBlock<'ctx>, stack_depth: usize },
      If     { else_bb: BasicBlock<'ctx>, merge_bb: BasicBlock<'ctx>,
               result_phi: Option<PhiValue<'ctx>>, stack_depth: usize, else_seen: bool },
  }
  ```
- Translation rules:
  - **Block { has_result }**: Create `merge_bb`. If has_result, add phi in merge_bb. Push frame. Continue emitting in current block.
  - **Loop**: Create `header_bb` and `merge_bb`. Branch current → header. Push frame. Emit into header.
  - **If { has_result }**: Pop condition. Create `then_bb`, `else_bb`, `merge_bb`. Conditional branch. Push frame. Emit into then_bb.
  - **Else**: Branch current → merge (add phi incoming if result). Switch to else_bb.
  - **End**: Branch current → merge (add phi incoming if result). Position at merge_bb. Restore stack to frame's depth (+1 if result, pushing phi value).
  - **Br(depth)**: Find target frame. If Block/If → branch to merge_bb (add phi incoming). If Loop → branch to header_bb. Create new unreachable BB for dead code after branch.
  - **BrIf(depth)**: Pop condition. Create continue_bb. Conditional branch: true→target, false→continue. Position at continue_bb.
  - **BrTable { targets, default }**: Pop index. Emit LLVM `switch` instruction. Create unreachable BB for continuation.
- Stack depth management: on branch to target, unwind operand_stack to frame's entry depth (grab result value first if target block has_result).

**Files**: `src/llvm_frontend/function_builder.rs` (extend)

**Verify**: Test blocks with results, nested if/else, loops with br/br_if, br_table. Verify phi nodes present in LLVM IR text. `module.verify()` passes.

---

## Phase 4: WASM → LLVM IR — Memory Operations & Calls -- DONE

**Goal**: Complete the LLVM IR frontend with memory access, function calls, and indirect calls.

**Memory operations** — use intrinsic function declarations (avoids `unsafe` GEP):
- Declare intrinsics in `llvm_frontend/intrinsics.rs`:
  - `@__pvm_load_i32(i64 addr) -> i64`
  - `@__pvm_load_i64(i64 addr) -> i64`
  - `@__pvm_load_i8u(i64 addr) -> i64` (etc. for all load widths)
  - `@__pvm_store_i32(i64 addr, i64 val) -> void` (etc.)
  - `@__pvm_memory_size() -> i64`
  - `@__pvm_memory_grow(i64 pages) -> i64`
  - `@__pvm_memory_fill(i64 dst, i64 val, i64 len) -> void`
  - `@__pvm_memory_copy(i64 dst, i64 src, i64 len) -> void`
- For each WASM load: pop addr, compute `addr + offset`, call `@__pvm_load_*`, push result
- For each WASM store: pop value, pop addr, compute `addr + offset`, call `@__pvm_store_*`
- The PVM backend will recognize these intrinsic names and lower them directly

**Direct calls**:
- `Call(func_idx)` → pop args, emit `builder.build_call(target_fn, &args)`, push return value
- All WASM functions are already declared in the LLVM module from Phase 2

**Indirect calls**:
- Declare `@__pvm_call_indirect(i64 type_idx, i64 table_idx, ...) -> i64`
- Or model with: load function pointer from global table, type-check via global type table, indirect call
- Simpler approach for now: use a single intrinsic that the backend recognizes

**Files**: new `src/llvm_frontend/intrinsics.rs`, extend `src/llvm_frontend/function_builder.rs`

**Verify**: Full WASM → LLVM IR pipeline works for all supported opcodes. Every WAT test fixture produces valid LLVM IR (`module.verify()` passes).

---

## Phase 5: LLVM IR → PVM Backend — Core Lowering -- DONE

**Goal**: Read LLVM IR and emit PVM bytecode for arithmetic, comparisons, memory, constants, and basic control flow.

**Architecture** — `llvm_backend/lowering.rs` with `LlvmToPvm<'ctx>`:
- **Value mapping**: `HashMap<InstructionValue, StackSlot>` — each SSA value gets a fixed memory offset relative to SP. Simple bump allocator.
- **Instruction lowering pattern** (for each LLVM instruction):
  1. Load input operands from their stack slots into temp registers (r2, r3)
  2. Emit PVM instruction using temp registers, result into r4
  3. Store result from r4 to the output's stack slot
- **Block mapping**: `HashMap<BasicBlock, PvmLabel>` — each LLVM block gets a PVM label, resolved via fixups (same mechanism as current `CodeEmitter`)

**What to lower**:
- `Add/Sub/Mul/UDiv/SDiv/URem/SRem` → PVM `Add32/64`, `Sub32/64`, etc.
- `And/Or/Xor/Shl/LShr/AShr` → PVM bitwise ops
- `ICmp` → PVM comparison sequences (same patterns as current codegen)
- `ZExt/SExt/Trunc` → PVM extension/truncation ops
- `Br` (unconditional) → PVM `Jump`
- `Br` (conditional) → PVM `BranchEqImm`/`BranchNeImm` with fixup
- `Switch` → PVM branch chain (same as current BrTable lowering)
- `Return` → PVM return sequence (via r0 or exit for main)
- `Alloca` → stack slot allocation (bump SP)
- `Load/Store` → should not appear if we use intrinsics (but handle as fallback)
- Constants → PVM `LoadImm`/`LoadImm64`
- **Intrinsic calls** (`@__pvm_load_*`, `@__pvm_store_*`, etc.) → emit PVM `LoadIndU32`/`StoreIndU32`/etc. with `wasm_memory_base` offset added. Reuse logic from current `translate_ir_op` for memory ops.

**Function prologue/epilogue**: Port directly from current `emit_prologue()`/`emit_epilogue()` in `codegen.rs`.

**Files**: new `src/llvm_backend/lowering.rs`, new `src/llvm_backend/slot_allocator.rs`

**Verify**: Simplest WAT programs compile through full new pipeline and produce working PVM bytecode. Compare output against legacy pipeline execution.

---

## Phase 6: LLVM IR → PVM Backend — Phi Nodes & Function Calls -- DONE

**Goal**: Handle phi nodes and the full PVM calling convention.

**Phi node elimination**:
- Each phi gets a dedicated stack slot (already assigned in Phase 5)
- Before each terminator in a basic block, for every phi in each successor that has an incoming from this block: store the incoming value to the phi's slot
- Handle phi cycles: always copy through a temp slot to avoid clobbering

**Call lowering** — port from existing `emit_call()` / `emit_call_indirect()` in `codegen.rs`:
1. Stack overflow check: `SP - frame_size >= stack_limit`
2. Save return address (r0), locals (r9-r12), spilled locals to PVM stack
3. Place args in r9-r12 (first 4) and PARAM_OVERFLOW_BASE (5th+)
4. Emit `LoadImm64` + `Jump` to callee (register fixup for address)
5. On return: restore r0, r9-r12 from stack. Copy return value from r7 to call's stack slot.

**Indirect calls**: Dispatch table lookup + type check. Port from current `emit_call_indirect()`.

**Intrinsic lowering for memory_grow/fill/copy**: Port inline sequences from current codegen (the MemoryGrow/Fill/Copy arms of `translate_ir_op`).

**Files**: extend `src/llvm_backend/lowering.rs`, new `src/llvm_backend/call_lowering.rs`

**Verify**: Functions with calls (factorial, fibonacci), indirect calls. Full call convention exercised.

---

## Phase 7: Full Pipeline Integration -- DONE

**Goal**: Wire everything together in `compile_via_llvm()` and pass all existing tests.

**What to build** — `translate/compile_llvm.rs`:
1. Parse WASM sections via shared `WasmModule` (from Phase 1)
2. Create inkwell `Context` + `Module`
3. Declare all LLVM functions and globals
4. For each function body: call `llvm_frontend::translate_function()`
5. Run `mem2reg` pass: `module.run_passes("mem2reg", &machine, opts)`
6. For each function: call `llvm_backend::lower_function()`
7. Collect PVM instructions + fixups
8. Build entry header, resolve fixups, build dispatch tables, ro_data/rw_data
9. Return `SpiProgram`

**Reuse from current `translate/mod.rs`**: Entry header emission, data segment copying, dispatch table construction, SPI assembly — all PVM-side concerns that are independent of the IR.

**Files**: new `src/translate/compile_llvm.rs`, modify `src/translate/mod.rs`

**Verify**:
- `cargo test --features llvm-backend` — all existing tests pass
- `cd tests && bun test` — TypeScript integration tests pass
- Build a differential test comparing legacy vs LLVM pipeline outputs

---

## Phase 8: Differential Testing, Bug Fixing & Legacy Removal -- IN PROGRESS

**Goal**: Achieve full parity, then remove the old code.

**Current status**: All existing Rust tests and all 360 TypeScript integration tests pass with `--features llvm-backend`. Some instruction-pattern tests are gated with `#[cfg(not(feature = "llvm-backend"))]` because the LLVM backend produces different instruction patterns (e.g. constant folding, no explicit div-by-zero trap sequences yet).

**Known gaps**:
- Division-by-zero and signed overflow trap checks not yet emitted by LLVM backend (relies on PVM hardware behavior)
- Import function calls emit Trap (same as legacy for unsupported imports)
- Multi-value returns (`entry_returns_ptr_len`) not yet handled via LLVM multi-value return

**Differential testing**:
- Create `tests/differential.rs` that compiles every WAT fixture through both pipelines
- Execute both SPI programs and compare outputs (not bytecode, since instruction sequences will differ)
- Fix discrepancies — common issues will be:
  - 32-bit value normalization (current codegen uses `AddImm32 { value: 0 }` hacks)
  - Division overflow trapping sequences
  - Signed vs unsigned edge cases
  - Spilled locals across recursive calls

**Once parity achieved**:
- Make `llvm-backend` the default feature
- Gate old pipeline behind `legacy-backend` feature
- In a follow-up PR: remove `src/ir/`, old codegen paths in `src/translate/codegen.rs`, `src/translate/stack.rs`

**Verify**: All tests pass with only `llvm-backend`. Legacy feature still works as fallback.

---

## Key Design Decisions

| Decision | Rationale |
|----------|-----------|
| Skip old IR, go `wasmparser::Operator` → LLVM IR directly | The old IR is a 1:1 WASM copy; adding a hop through it has no value |
| Use intrinsic functions for PVM memory ops | Avoids `unsafe` code (workspace denies it), prevents LLVM from making wrong assumptions about PVM's memory model |
| Every SSA value gets a stack slot in PVM backend | Correctness-first; register allocation is a future optimization |
| Use `alloca` + `mem2reg` for WASM locals | Standard LLVM frontend pattern; avoids manual SSA/phi construction for locals |
| All values as i64 internally | PVM registers are 64-bit; simplifies the translation |
| Feature-flag both pipelines | Safe migration; can diff-test and fall back |

## Risks

| Risk | Mitigation |
|------|------------|
| LLVM system dependency on all dev machines + CI | Document install steps; pin LLVM 18; use `LLVM_SYS_180_PREFIX` env var |
| `unsafe_code = "deny"` conflicts with inkwell | Intrinsic approach avoids unsafe; if needed, `#[allow(unsafe_code)]` on specific fns |
| Control flow translation bugs (Phase 3) | Most complex phase — invest in thorough unit tests for each control flow pattern |
| Phi node cycle bugs | Always copy through temp slot; add targeted diamond-CFG tests |
| Performance regression from stack-slot backend | Acceptable for correctness-first; register allocator is planned follow-up |
| inkwell iteration API gaps | Prototype Phase 5 early to verify all needed inspection APIs exist |

## Critical Files Reference

| File | Role |
|------|------|
| `src/translate/codegen.rs` (3101 lines) | **Port from**: all PVM lowering logic, call convention, prologue/epilogue |
| `src/translate/mod.rs` (759 lines) | **Refactor**: extract `WasmModule`, keep SPI assembly |
| `src/translate/memory_layout.rs` (93 lines) | **Keep as-is**: PVM memory constants used by new backend |
| `src/pvm/instruction.rs` | **Keep as-is**: PVM instruction encoding, output target |
| `src/ir/builder.rs` (252 lines) | **Reference**: complete list of supported WASM opcodes to translate |
