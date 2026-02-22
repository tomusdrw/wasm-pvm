# Technical Learnings

Accumulated knowledge from development. Update after every task.

---

## LLVM New Pass Manager (inkwell 0.8.0 / LLVM 18)

### Pass Pipeline Syntax

- `Module::run_passes()` accepts a pipeline string parsed as a **module-level** pipeline
- Function passes (like `mem2reg`, `instcombine`) auto-wrap as `module(function(...))`
- CGSCC passes (like `inline`) **cannot** be mixed with function passes in a single string
- To run the inliner: use a separate `run_passes("cgscc(inline)")` call
- Pass parameters use angle brackets: `instcombine<max-iterations=2>`

### instcombine Convergence

- `instcombine` defaults to `max-iterations=1`, which can cause `LLVM ERROR: Instruction Combining did not reach a fixpoint` on complex IR (e.g., after aggressive inlining)
- Fix: use `instcombine<max-iterations=2>` to give it a second iteration
- Running `instcombine,simplifycfg` before inlining also helps by simplifying the IR first

### Inlining Creates New LLVM Intrinsics

- After inlining, `instcombine` may transform patterns into LLVM intrinsics that weren't present before:
  - `if x < 0 then -x else x` becomes `llvm.abs.i64`
  - Similar patterns may produce `llvm.smax`, `llvm.smin`, `llvm.umax`, `llvm.umin`
- The PVM backend must handle these intrinsics (see `llvm_backend/intrinsics.rs`)

### PassBuilderOptions

- `set_inliner_threshold()` is on `PassManagerBuilder`, NOT on `PassBuilderOptions`
- `PassBuilderOptions` has no direct way to set the inline threshold
- The inline pass uses LLVM's default threshold (225) when invoked via `cgscc(inline)`

---

## CmovIzImm / CmovNzImm (TwoRegOneImm Encoding)

- Opcodes 147-148: Conditional move with immediate value
- TwoRegOneImm encoding: `[opcode, (cond << 4) | dst, imm_bytes...]`
- CmovIzImm: `if reg[cond] == 0 then reg[dst] = sign_extend(imm)`
- CmovNzImm: `if reg[cond] != 0 then reg[dst] = sign_extend(imm)`
- Future use: optimize `select` when one operand is a compile-time constant (depends on CmovIz/CmovNz from PR #98 merging first)

---

## LoadImmJumpInd (Opcode 180) — Not Yet Implemented

- TwoRegTwoImm encoding: fuses `LoadImm + JumpInd` into one instruction
- Semantics: `reg[dst] = sign_extend(value); jump to reg[base] + sign_extend(offset)`
- **Blocker**: The fixup system computes byte offsets from instruction encodings, then patches values which changes variable-length encoding sizes. LoadImm64 has fixed 10-byte encoding, so patching its value doesn't change byte offsets. LoadImmJumpInd uses variable-length TwoImm encoding, creating a chicken-and-egg problem: the return address offset depends on the encoding size, which depends on the patched value.
- **To implement**: Either (a) use a fixed-size encoding variant for fixup placeholders, or (b) rework fixup resolution to iterate to a fixed point after patching, or (c) pre-reserve maximum encoding size and pad with Fallthroughs.

---

## PVM Intrinsic Lowering

### llvm.abs (absolute value)

- Signature: `llvm.abs.i32(x, is_int_min_poison)` / `llvm.abs.i64(x, is_int_min_poison)`
- Lowered as: `if x >= 0 then x else 0 - x`
- For i32: must sign-extend first (zero-extension from load_operand makes negatives look positive in i64 comparisons)
