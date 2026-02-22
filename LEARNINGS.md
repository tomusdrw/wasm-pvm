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

## StoreImm (TwoImm Encoding)

- Opcodes 30-33: StoreImmU8/U16/U32/U64
- TwoImm encoding: `[opcode, addr_len & 0x0F, address_bytes..., value_bytes...]`
- Both address and value are variable-length signed immediates (0-4 bytes each)
- Semantics: `mem[address] = value` (no registers involved)
- Used for: `data.drop` (store 0 to segment length addr), `global.set` with constants
- Savings: 3 instructions (LoadImm + LoadImm + StoreInd) → 1 instruction

## ALU Immediate Opcode Folding

### Immediate folding for binary operations
- When one operand of a binary ALU op is a constant that fits in i32, use the *Imm variant (e.g., `And` + const → `AndImm`)
- Saves 1 gas per folded instruction (no separate `LoadImm`/`LoadImm64` needed) + code size reduction
- Available for: Add, Mul, And, Or, Xor, ShloL, ShloR, SharR (both 32-bit and 64-bit)
- Sub with const RHS → `AddImm` with negated value; Sub with const LHS → `NegAddImm`
- ICmp UGT/SGT with const RHS → `SetGtUImm`/`SetGtSImm` (avoids swap trick)
- LLVM often constant-folds before reaching the PVM backend, so benefits are most visible in complex programs

---

## Conditional Move (CmovIz/CmovNz)

### Branchless select lowering

- `select i1 %cond, %true_val, %false_val` now uses `CmovNz` instead of a branch
- Old: load false_val, branch on cond==0, load true_val, define label (5-6 instructions)
- New: load false_val, load true_val, load cond, CmovNz (4 instructions, branchless)
- CmovIz/CmovNz are ThreeReg encoded: `[opcode, (cond<<4)|src, dst]`
- Semantics: `if reg[cond] == 0 (CmovIz) / != 0 (CmovNz) then reg[dst] = reg[src]`
- Note: CmovNz conditionally writes dst — the register cache must invalidate dst after CmovNz/CmovIz since the write is conditional

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

---

## LoadImmJump for Direct Calls

### Combined Instruction Replaces LoadImm64 + Jump

- Direct function calls previously used two instructions: `LoadImm64 { reg: r0, value }` (10 bytes) + `Jump { offset }` (5 bytes) = 15 bytes, 2 gas
- `LoadImmJump { reg: r0, value, offset }` (opcode 80) combines both into a single instruction: 6-10 bytes, 1 gas
- Uses `encode_one_reg_one_imm_one_off` encoding: `opcode(1) + (imm_len|reg)(1) + imm(0-4) + offset(4)`
- For typical call return addresses (small positive integers like 2, 4, 6), the imm field is 1 byte, so total is 7 bytes

### Pre-Assignment of Jump Table Addresses

- Same challenge as `LoadImm` for return addresses: `LoadImmJump` has variable-size encoding, so the value must be known at emission time
- Solution: Thread a `next_call_return_idx` counter through the compilation pipeline, pre-computing `(index + 1) * 2` at emission time
- During `resolve_call_fixups`, only the `offset` field is patched (always 4 bytes, size-stable)
- The `value` field is verified via `debug_assert!` to match the actual jump table index

### Bonus: Peephole Fallthrough Elimination

- Since `LoadImmJump` is a terminating instruction, the peephole optimizer can remove a preceding `Fallthrough`
- This saves an additional 1 byte per call site where a basic block boundary precedes the call
- Total savings per call: -8 bytes (instruction) + -1 byte (Fallthrough removal) + -1 gas

---

## Call Return Address Encoding

### LoadImm vs LoadImm64 for Call Return Addresses

- Call return addresses are jump table addresses: `(jump_table_index + 1) * 2`
- These are always small positive integers (2, 4, 6, ...) that fit in `LoadImm` (3-6 bytes)
- Previously used `LoadImm64` (10 bytes) with placeholder value 0, patched during fixup resolution
- **Problem with late patching**: `LoadImm` has variable encoding size (2 bytes for value 0, 3 bytes for value 2), so changing the value after branch fixups are resolved corrupts relative offsets
- **Solution**: Pre-assign jump table indices at emission time by threading a `next_call_return_idx` counter through the compilation pipeline. This way `LoadImm` values are known during emission, ensuring correct `byte_offset` tracking for branch fixup resolution
- For direct calls, `LoadImmJump` combines return address load + jump into one instruction, using the same pre-assigned index
- For indirect calls (`call_indirect`), `LoadImm` + `JumpInd` is used since the jump target is in a register
- **Impact**: Saves 7 bytes per indirect call site (LoadImm vs LoadImm64). Direct calls save even more via LoadImmJump fusion.

### Why LoadImm64 was originally needed

- `LoadImm64` has fixed 10-byte encoding regardless of value, so placeholder patching was safe
- `LoadImm` with value 0 encodes to 2 bytes, but after patching to value 2 becomes 3 bytes
- This size change would break branch fixups already resolved with the old instruction sizes

---

## PVM 32-bit Instruction Semantics

### Sign Extension

- All PVM 32-bit arithmetic/shift instructions produce `u32SignExtend(result)` — the lower 32 bits are computed, then sign-extended to fill the full 64-bit register
- This means `AddImm32(x, x, 0)` after a 32-bit producer is a NOP (both sign-extend identically)
- Confirmed in anan-as reference: `add_32`, `sub_32`, `mul_32`, `div_u_32`, `rem_u_32`, `shlo_l_32`, etc. all call `u32SignExtend()`

### Peephole Truncation Pattern

- The pattern `[32-bit-producer] → [AddImm32(x, x, 0)]` is eliminated by peephole when directly adjacent
- In practice with LLVM passes enabled, `instcombine` already eliminates `trunc(32-bit-op)` at the LLVM IR level, so this peephole pattern fires rarely
- The peephole is still valuable for `--no-llvm-passes` mode and as defense-in-depth
- **Known limitation**: the pattern only matches directly adjacent instructions; a `StoreIndU64` between producer and truncation breaks the match

---

## Cross-Block Register Cache

### Approach

- Pre-scan computes `block_single_pred` map by scanning terminator successors
- For each block with exactly 1 predecessor and no phi nodes, restore the predecessor's cache snapshot instead of clearing
- Snapshot is taken **before** the terminator instruction to avoid capturing path-specific phi copies

### Key Pitfall: Terminator Phi Copies

- `lower_switch` emits phi copies for the default path inline (not in a trampoline)
- These phi copies modify the register cache (storing values to phi slots)
- If the exit cache includes these entries, they are WRONG for case targets (which don't take the default path)
- Fix: snapshot before the terminator and invalidate TEMP1/TEMP2 (registers the terminator clobbers for operand loads)
- Same issue can occur with conditional branches when one path has phis and the other doesn't (trampoline case)
