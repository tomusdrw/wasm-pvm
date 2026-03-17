# Technical Reference

Accumulated technical knowledge from development — LLVM pass behavior, PVM instruction semantics, code generation patterns, and optimization details.

---

## Entry Function ABI — Unified Packed i64 Convention

All entry functions (both WAT and AssemblyScript) must use `main(args_ptr: i32, args_len: i32) -> i64`.
The i64 return value packs a WASM pointer and length: `(ptr as u64) | ((len as u64) << 32)`.
The PVM epilogue unpacks: `r7 = (ret & 0xFFFFFFFF) + wasm_memory_base`, `r8 = r7 + (ret >> 32)`.

Common constant: ptr=0, len=4 → `i64.const 17179869184` (= `4 << 32`).

Previous conventions (globals-based, multi-value `(result i32 i32)`, simple scalar) were removed.
AssemblyScript uses a `writeResult(val: i32): i64` helper that stores the value and returns `packResult(ptr, len)`.

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

### PVM Branch Operand Convention

Two-register branch instructions use **reversed operand order**: `Branch_op { reg1: a, reg2: b }` branches when `reg2 op reg1` (i.e., `b op a`). For example, `BranchLtU { reg1: 3, reg2: 2 }` branches when `reg[2] < reg[3]`. This matches the Gray Paper where `branch_lt_u(rA, rB)` branches when `ω_rB < ω_rA`. In the encoding, `reg1` = high nibble (rA), `reg2` = low nibble (rB). Immediate-form branches are straightforward: `BranchLtUImm { reg, value }` branches when `reg < value`.

### PVM Memory Layout Optimization

- **Globals only occupy the bytes they actually need**: the compiler now tracks `globals_region_size = (num_globals + 1 + num_passive_segments) * 4` bytes and places the heap immediately after that region instead of reserving a full 64KB block. This keeps the RW data blob limited to real globals/passive-length fields plus active data segments.
- **Dynamic heap base calculation**: `compute_wasm_memory_base(num_funcs, num_globals, num_passive_segments)` compares the spill area (`SPILLED_LOCALS_BASE + num_funcs * SPILLED_LOCALS_PER_FUNC`) with the globals region end (`GLOBAL_MEMORY_BASE + globals_region_size(...)`) before rounding up to the next 4KB (PVM page) boundary. This typically gives `0x33000` instead of the old `0x40000`, saving ~52KB per program.
- **4KB alignment is sufficient**: The SPI spec only requires page-aligned (4KB) `rw_data` length. The 64KB WASM page size governs `memory.grow` granularity, not the base address. The anan-as interpreter uses `alignToPageSize(rwLength)` (4KB) not segment alignment for the heap zeros start. Evidence: `vendor/anan-as/assembly/spi.ts` line 41: `heapZerosStart = heapStart + alignToPageSize(rwLength)`.
- **heap_pages headroom for rw_data trimming**: SPI `heap_pages` means "zero pages after rw_data", but `build_rw_data()` trims trailing zeros. With the tighter 4KB alignment, both rw_data and heap_pages shrink, reducing total writable memory. A 16-page (64KB) headroom is added to `calculate_heap_pages()` to compensate. This doesn't affect JAM file size (heap_pages is a 2-byte header field), it only tells the runtime to allocate more zero pages. Without this headroom, PVM-in-PVM tests fail for programs at the memory edge (e.g. `as-tests-structs` inside the anan-as interpreter).

### Code Generation

- **Leaf Functions**: Functions that make no calls don't need to save/restore the return address (`ra`/r0) because it's invariant. This optimization saves 2 instructions per leaf function.
- **Address Calculation**: Fusing `AddImm` into subsequent `LoadInd`/`StoreInd` offsets reduces instruction count.
- **Dead Code Elimination**: Basic DCE for ALU operations removes unused computations (e.g. from macro expansions).

---

## StoreImm (TwoImm Encoding)

- Opcodes 30-33: StoreImmU8/U16/U32/U64
- TwoImm encoding: `[opcode, addr_len & 0x0F, address_bytes..., value_bytes...]`
- Both address and value are variable-length signed immediates (0-4 bytes each)
- Semantics: `mem[address] = value` (no registers involved)
- Used for: `data.drop` (store 0 to segment length addr), `global.set` with constants
- Savings: 3 instructions (LoadImm + LoadImm + StoreInd) → 1 instruction

## StoreImmInd (Store Immediate Indirect)

### Encoding (OneRegTwoImm)

- Format: `[opcode, (offset_len << 4) | (base & 0x0F), offset_bytes..., value_bytes...]`
- Both offset and value use variable-length signed encoding (`encode_imm`)
- Opcodes: StoreImmIndU8=70, StoreImmIndU16=71, StoreImmIndU32=72, StoreImmIndU64=73
- Semantics: `mem[reg[base] + sign_extend(offset)] = value` (truncated/sign-extended per width)
- For U64: `value` is sign-extended from i32 to i64

### Optimization Triggers

- `emit_pvm_store`: When WASM store value is a compile-time constant fitting i32
- Saves 1 instruction (LoadImm) per constant store to WASM linear memory

## ALU Immediate Opcode Folding

### Immediate folding for binary operations
- When one operand of a binary ALU op is a constant that fits in i32, use the *Imm variant (e.g., `And` + const → `AndImm`)
- Saves 1 gas per folded instruction (no separate `LoadImm`/`LoadImm64` needed) + code size reduction
- Available for: Add, Mul, And, Or, Xor, ShloL, ShloR, SharR (both 32-bit and 64-bit)
- Sub with const RHS → `AddImm` with negated value; Sub with const LHS → `NegAddImm`
- ICmp UGT/SGT with const RHS → `SetGtUImm`/`SetGtSImm` (avoids swap trick)
- LLVM often constant-folds before reaching the PVM backend, so benefits are most visible in complex programs

---

## Instruction Decoder (`Instruction::decode`)

- `instruction.rs` now has `Instruction::decode(&[u8]) -> Result<(Instruction, usize)>` so roundtrip tests and disassembly-style tooling can share one decode path.
- `Opcode::from_u8` / `TryFrom<u8>` are now the canonical byte→opcode conversion helpers for code and tests.
- Fixed-width formats (`Zero`, `ThreeReg`, `TwoReg`, `OneOff`, `TwoRegOneOff`, `OneRegOneExtImm`, `OneRegOneImmOneOff`) return exact consumed length.
- Formats with trailing variable-length immediates but no explicit terminal length marker (`OneImm`, `OneRegOneImm`, `TwoRegOneImm`, `TwoImm`, `OneRegTwoImm`, `TwoRegTwoImm`) are decoded by consuming the remaining bytes for that trailing immediate.
- Unknown opcode passthrough is explicit: decode returns `Instruction::Unknown { opcode, raw_bytes }` with original bytes preserved.

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
- Now used: optimize `select` when one operand is a compile-time constant that fits in i32

---

## LoadImmJumpInd (Opcode 180) — Implemented

- TwoRegTwoImm encoding: fuses `LoadImm + JumpInd` into one instruction.
- Semantics: `reg[dst] = sign_extend(value); jump to reg[base] + sign_extend(offset)`.
- `call_indirect` now emits `LoadImmJumpInd { base: r8, dst: r0, value: preassigned_return_addr, offset: 0 }`.
- Dispatch table address math for indirect calls can use `ShloLImm32(..., value=3)` instead of three `Add32` doublings (`idx*8`), reducing one hot-path sequence from 3 instructions to 1 with equivalent 32-bit wrap/sign-extension semantics.
- Fixups remain stable by:
  - pre-assigning return jump-table slots at emission time, and
  - recording `return_addr_instr == jump_ind_instr` for this fused call instruction.
- `return_addr_jump_table_idx()` accepts `LoadImmJump`, `LoadImm`, and `LoadImmJumpInd`, so mixed old/new patterns still resolve safely.
- Important semantic pitfall: do **not** assume `base == dst` is safe for absolute jumps. Using `LoadImmJumpInd` for the main epilogue (`EXIT_ADDRESS`) caused global failures because jump target evaluation does not behave like a guaranteed "write dst first, then read base" in practice.

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
- `LoadImmJump` does not read any source registers; treat it like `LoadImm`/`LoadImm64` in `Instruction::src_regs` for DCE
- PVM-in-PVM args are passed via a temp binary file; use a unique temp dir + random filename to avoid collisions under concurrent `bun test` workers. Debug knobs: `PVM_IN_PVM_DEBUG=1` for extra logging, `PVM_IN_PVM_KEEP_ARGS=1` to retain the temp args file on disk.
- DCE `src_regs`: Imm ALU ops read only `src`; `StoreImm*` reads no regs; `StoreImmInd*` reads base only.

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
- For indirect calls (`call_indirect`), `LoadImmJumpInd` is used to combine return-address setup and the indirect jump
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

### Peephole AddImm Width Safety

- `optimize_address_calculation()` must not fold address relations across `AddImm32`/`AddImm64` width boundaries.
- Track `AddImm` relation width alongside `(base, offset)` and only fold when widths match (`32→32`, `64→64`), while still allowing width-agnostic `MoveReg` alias folding.

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

---

## Specialized PVM Instructions for Common Patterns

### Absolute Address Load/Store (LoadU32/StoreU32)

- `LoadU32 { dst, address }` replaces `LoadImm { reg, value: addr } + LoadIndU32 { dst, base: reg, offset: 0 }` for known-address loads (globals)
- `StoreU32 { src, address }` similarly replaces the store pattern
- OneRegOneImm encoding: `[opcode, reg & 0x0F, encode_imm(address)...]`
- **PVM-in-PVM layout sensitivity**: Replacing multi-instruction sequences with single instructions changes bytecode layout (code size, jump offsets). Test each significant code generation change with the full PVM-in-PVM suite.
- `LoadU32` is used for `lower_wasm_global_load`. `StoreU32` is used for `lower_wasm_global_store`. Both absolute-address variants are now emitted everywhere applicable.

### LoadIndI32 (Sign-Extending Indirect Load)

- Replaces `LoadIndU32 { dst, base, offset } + AddImm32 { dst, src: dst, value: 0 }` for signed i32 loads
- Single instruction: `LoadIndI32 { dst, base, offset }` (sign-extends result to 64 bits)
- Safe for PVM-in-PVM (small layout change)

### Min/Max/MinU/MaxU (Single-Instruction Min/Max)

- Replaces `SetLt + branch + stores + jump` pattern (~5-8 instructions) with `Min`/`Max`/`MinU`/`MaxU` (1 instruction)
- For i32 signed variants, must keep `AddImm32 { value: 0 }` sign-extension before the instruction (PVM compares full 64-bit values)

### ReverseBytes (Byte Swap)

- `llvm.bswap` intrinsic lowered as `ReverseBytes { dst, src }` instead of byte-by-byte extraction
- For sub-64-bit types: add `ShloRImm64` to align bytes (48 for i16, 32 for i32)
- Savings: i16: ~10→2 instructions, i32: ~20→2, i64: ~40→1

### CmovIzImm/CmovNzImm (Conditional Move with Immediate)

- For `select` with one constant operand: `CmovNzImm { dst, cond, value }` or `CmovIzImm { dst, cond, value }`
- Load non-constant operand as default, then conditionally overwrite with immediate
- Note: LLVM may invert conditions, so `select(cond, true_const, false_runtime)` may emit CmovIzImm instead of CmovNzImm

### RotL/RotR (Rotate Instructions)

- `llvm.fshl(a, b, amt)` / `llvm.fshr(a, b, amt)` when a == b (same SSA value) → rotation
- Detected via `val_key_basic(a) == val_key_basic(b)` identity check
- fshl with same operands → `RotL32`/`RotL64`, fshr → `RotR32`/`RotR64`
- Falls back to existing shift+or sequence when operands differ

### Linear-Scan Register Allocation

- Allocates SSA values to physical registers using spill-weight eviction (`use_count × 10^loop_depth`).
- Operates on LLVM IR before PVM lowering; produces `ValKey` → physical register mapping
- `load_operand` checks regalloc before slot lookup: uses `MoveReg` from allocated reg instead of `LoadIndU64` from stack
- `store_to_slot` uses write-through: copies to allocated reg AND stores to stack; DSE removes the stack store if never loaded
- r5/r6 allocatable in safe leaf functions (no bulk memory ops or funnel shifts); detected by `scratch_regs_safe()` LLVM IR scan
- r7/r8 allocatable in all leaf functions; lowering paths that use them as scratch trigger `invalidate_reg` via `emit()`
- Clobbered allocated scratch regs (when present) are handled with lazy invalidation/reload instead of eager spill+reload
- Allocates in all functions (looped and straight-line), not just loop-heavy code
- MIN_USES default=2 (aggressive=1); values with fewer uses are skipped
- Loop extension: back-edges detected by successor having lower block index; live ranges extended to cover the back-edge source
- Eviction uses spill weight (sum of `10^loop_depth` per use) instead of furthest-end heuristic
- `linear_scan` must track active assignments separately from final assignments:
  - naturally expired intervals should remain in the final `val_to_reg`/`slot_to_reg` maps (their earlier uses still benefit),
  - evicted intervals must be removed from final mapping (whole-interval mapping is no longer valid after eviction).
- Unit tests cover both interval outcomes (non-overlapping reuse and eviction dropping).
- Targeted benchmark fixture: `tests/fixtures/wat/regalloc-two-loops.jam.wat` (`regalloc two loops(500)` row).
- Regalloc instrumentation:
  - `regalloc::run()` logs candidate/assignment stats at target `wasm_pvm::regalloc` (enable via `RUST_LOG=wasm_pvm::regalloc=debug`).
  - `lower_function()` logs per-function summary including allocation usage counters (`alloc_load_hits`, `alloc_store_hits`).
- Instrumentation root cause and fix:
  - Root cause was `allocatable_regs=0` in non-leaf functions because only leaf functions exposed r9-r12 to regalloc.
  - Fix: expose available r9-r12 registers in both leaf and non-leaf functions; reserve outgoing argument registers (`r9..r9+max_call_args-1`) from non-leaf allocation and invalidate local-register mappings after calls.
  - Example (`regalloc-two-loops`): `allocatable_regs=2`, `allocated_values=4`, `alloc_load_hits=11`, `alloc_store_hits=8`.
- Non-leaf stabilization:
  - Reserve outgoing call-argument registers (r9.. by max call arity) from the non-leaf allocatable set.
  - Initially, `alloc_reg_valid` was reset at label boundaries (`define_label` / `define_label_preserving_cache`) because that validity state was not path-sensitive and `CacheSnapshot` did not yet snapshot `alloc_reg_slot` during cross-block cache propagation.
  - Without boundary reset, large workloads (notably `anan-as-compiler.jam`) can miscompile under pvm-in-pvm despite direct tests passing.
- Follow-up stabilization:
  - Corrective follow-up: `CacheSnapshot` now includes allocated-register slot ownership (`alloc_reg_slot`), which replaced the earlier label-boundary `alloc_reg_valid` reset approach by restoring allocation state path-sensitively across propagated edges.
  - `alloc_reg_valid` was removed; slot identity (`alloc_reg_slot == Some(slot)`) is sufficient to decide whether a lazy reload is needed.
  - Non-leaf gate: skip when no allocatable registers remain (all r9-r12 used by params/call args). Previously skipped at <2 regs and <24 SSA values, but these conservative gates were removed in Phase 2 (#165).
- Post-fix benchmark shape: consistent JAM size reductions from regalloc, but gas/time gains are workload-dependent and often near-noise on current microbenchmarks.
- **Leaf detection fix**: PVM intrinsics (`__pvm_load_i32`, `__pvm_store_i32`, etc.) are LLVM `Call` instructions but are NOT real function calls — they're lowered inline using temp registers only. The `is_real_call()` function in `emitter.rs` distinguishes real calls (`wasm_func_*`, `__pvm_call_indirect`) from intrinsics (`__pvm_*`, `llvm.*`). Before this fix, ALL functions with memory access were classified as non-leaf, causing unnecessary callee-save prologue/epilogue overhead.
- **Cross-block alloc_reg_slot propagation**: In leaf functions (no real calls), `alloc_reg_slot` is preserved across all block boundaries because allocated registers are never clobbered. In non-leaf functions with multi-predecessor blocks, predecessor exit snapshots are intersected — only entries where ALL processed predecessors agree are kept. Back-edges (unprocessed predecessors) are treated conservatively.
- **Phi node allocation is a gas regression in PVM**: Allocating phi nodes at loop headers adds +1 MoveReg per iteration per phi (write-through to allocated reg) with 0 gas savings (MoveReg replaces LoadIndU64, both cost 1 gas). Net: +1 gas per iteration per allocated phi. Only beneficial when loads are cheaper than stores, when allocated regs can be used directly by instructions (avoiding MoveReg to temps), or when code size matters more than gas.

### Fused Inverted Bitwise (AndInv / OrInv / Xnor)

- `and(a, xor(b, -1))` → `AndInv(a, b)` (bit clear): saves 1 instruction (eliminates separate Xor for NOT)
- `or(a, xor(b, -1))` → `OrInv(a, b)` (or-not): same pattern
- `xor(a, xor(b, -1))` → `Xnor(a, b)` (equivalence): note that LLVM instcombine may reassociate `xor(a, xor(b, -1))` to `xor(xor(a,b), -1)`, which makes Xnor fire less often in practice
- Detection is commutative: checks both LHS and RHS for the NOT pattern
- All three use ThreeReg encoding: `[opcode, (src2<<4)|src1, dst]`

### CmovIz Register Form for Inverted Select

- `select(!cond, true_val, false_val)` now uses `CmovIz` instead of computing the inversion + `CmovNz`
- Detected patterns: `xor(cond, 1)` (boolean flip) and `icmp eq cond, 0` (i32.eqz)
- Saves 2-3 instructions by avoiding the boolean inversion sequence
- Note: LLVM instcombine often folds `select(icmp eq x, 0, tv, fv)` → `select(x, fv, tv)`, so the pattern fires mainly in edge cases or with specific IR shapes

### Intentionally Not Emitted Opcodes

- **MulUpperSS/UU/SU (213-215)**: No WASM operator produces 128-bit multiply upper halves
- **Alt shift immediates (reversed)**: `dst = imm OP src` form — no WASM pattern generates this (LLVM canonicalizes register on LHS)
- **Absolute address non-32-bit sizes**: All WASM globals use 4-byte (i32) slots; no need for U8/U16/U64 absolute address variants

### RW Data Trimming

- `translate::build_rw_data()` now trims trailing zero bytes before SPI encoding.
- Semantics remain correct because heap pages are zero-initialized; omitted high-address zero tail bytes are equivalent.
- This is a low-risk blob-size optimization and does not materially affect gas.

### Fallthrough Jump Elimination

- When LLVM block N ends with an unconditional branch to block N+1 (next in layout order), the `Jump` can be skipped — execution falls through naturally.
- Controlled by `fallthrough_jumps` optimization flag (`--no-fallthrough-jumps` to disable).
- Implementation: `PvmEmitter.next_block_label` tracks the label of the next block. `emit_jump_to_label()` skips the `Jump` when the target matches `next_block_label`.
- **Critical pitfall — phi node trampolines**: When conditional branches target blocks with phi nodes, the codegen emits per-edge trampoline code (phi copies + Jump) between blocks. The `emit_jump_to_label()` in trampoline code must NOT be eliminated, because the jump is not the last instruction before the next block's `define_label`. Fix: `lower_br` and `lower_switch` temporarily clear `next_block_label` during trampoline emission.
- Entry header shrunk from 10 to 6 bytes when no secondary entry (removed 4 Fallthrough padding after Trap).
- Main function emitted first (right after entry header) to minimize Jump distance.

### Memory Layout Sensitivity (PVM-in-PVM)

- Moving the globals/overflow/spill region around directly affects the base address that the interpreter loads as the WASM heap, so every change still requires a full pvm-in-pvm validation. Direct/unit runs may look fine, but the outer interpreter can panic if the linear memory isn't page-aligned or overlaps reserved slots.
- **Critical**: `PARAM_OVERFLOW_BASE` and `SPILLED_LOCALS_BASE` must be >= `GLOBAL_MEMORY_BASE` (0x30000) because the SPI rw_data zone starts at 0x30000. The gap zone (0x20000-0x2FFFF) between ro_data and rw_data is unmapped. Placing constants in the gap zone causes PVM panics.
- The layout keeps overflow/spill inside the rw_data zone (0x32000+) after the globals window, which preserves compatibility while trimming the RW data blob size.

### Benchmark Comparison Parsing

- `tests/utils/benchmark.sh` emits two different result tables:
  - Direct: `Benchmark | WASM Size | JAM Size | Gas Used | Time`
  - PVM-in-PVM: `Benchmark | JAM Size | Outer Gas Used | Time`
- Branch comparison must parse JAM size and gas from the correct columns per table header (direct rows use columns 3/4; PiP rows use 2/3).
- With `set -u`, EXIT trap handlers must not depend on function-local variables at exit time; expand local values when installing the trap.

### Peephole Immediate Chain Fusion (2026-03)

- **LoadImm + AddImm fusion**: `LoadImm r1, A; AddImm r1, r1, B` → `LoadImm r1, A+B`
  - Saves 1 instruction when loading a value then adjusting it
  - Only applies when combined result fits in i32
- **Chained AddImm fusion**: `AddImm r1, r1, A; AddImm r1, r1, B` → `AddImm r1, r1, A+B`
  - Collapses sequences of incremental adjustments
  - Common in address calculations and loop induction variables
- **MoveReg self-elimination**: `MoveReg r1, r1` → removed entirely (no-op)
  - Can appear after register allocation or phi lowering
- Implementation in `peephole.rs::optimize_immediate_chains()`

### Comparison Code Size Optimizations (2026-03)

### PVM-in-PVM Ecalli Forwarding (2026-03)

- **Dynamic ecalli index is not supported by PVM**: The `ecalli` instruction takes a static u32 immediate. To forward inner program ecalli with dynamic indices, either use a per-ecalli dispatch table in the adapter or use a fixed "proxy" ecalli with a data buffer protocol.
- **Adapter import resolution against main exports**: `adapter_merge.rs` resolves adapter imports matching main export names internally. Key use case: adapter importing `host_read_memory` / `host_write_memory` (exported by the compiler module) to access inner PVM memory during ecalli handling.
- **Scratch buffer protocol for trace replay**: The replay adapter allocates a single WASM memory page (`memory.grow(1)`) on the first ecalli call and caches the address at a sentinel location (`0xFFFF0`) for reuse on subsequent calls. The outer handler writes the ecalli response (`[8:new_r7][8:new_r8][4:num_memwrites][8:new_gas][memwrites...]`) to the buffer at the PVM address obtained via `pvm_ptr`. The adapter reads the response, applies memwrites via `host_write_memory`, and returns the new register values.
- **Adapter globals not supported**: `adapter_merge` only merges function-related sections (types, imports, functions, code) from the adapter. Globals, data sections, and memory declarations from the adapter are NOT included in the merged module. Workaround: use main module memory with fixed addresses or `memory.grow`.
- **host_call_N requires compile-time constant ecalli index**: The first argument to `host_call_N` imports must be a compile-time constant because it becomes the immediate operand of the PVM `ecalli` instruction. Runtime ecalli indices (e.g., forwarded from inner programs) cause compilation failure.

- **NE comparison optimization was reverted for correctness in PVM-in-PVM**:
  `Xor + SetGtUImm(0)` looked equivalent to `Xor + LoadImm(0) + SetLtU`, but it regressed
  `as-decoder-subarray-test` in layer5 (inner run returned empty `Result: [0x]`).
  Keep the conservative `LoadImm(0) + SetLtU` lowering for `icmp ne`.
- **i1→i64 sign-extension**: `LoadImm(0) + Sub64` → `NegAddImm64(0)`
  - Original: 2 instructions to compute `0 - val` (negate boolean to 0/-1)
  - Optimized: 1 instruction using `NegAddImm64` which computes `val = imm - src`
  - `NegAddImm64(dst, src, 0)` = `dst = 0 - src` = `-src`
  - Saves 1 instruction per boolean sign-extension

### Register-Aware Phi Resolution (Phase 5, 2026-03)

- **Ordering dependencies between reg→reg and reg→stack phi copies**: When phi copies include both register-to-register copies and copies involving stack, they must be treated as a single set of parallel moves. An initial implementation separated them into two independent phases, but this caused incorrect results when a reg→reg copy clobbered a source register that a reg→stack copy also needed. The fix: use a unified two-pass approach (load ALL incoming values into temp registers first, then store all to destinations).
- **Phi destinations must be restored after `define_label`**: After `define_label` clears all alloc state at a block boundary, blocks with phi nodes must call `restore_phi_alloc_reg_slots` to re-establish `alloc_reg_slot` for phi destinations. Without this, `load_operand` falls back to stack loads, missing the values that the phi copy placed in registers.
- **Dirty phi values and block exit**: After `restore_phi_alloc_reg_slots` marks phi destinations as dirty, the before-terminator `spill_all_dirty_regs()` writes them to the stack. This is essential: non-phi successor blocks (like loop exit blocks) clear alloc state and read from the stack. Without the spill, exit paths read stale stack values. This limits the code-size benefit of lazy spill — each iteration still writes phi values to the stack once via the before-terminator spill.
- **`alloc_reg_slot` shared between phi destination and incoming value**: The same SSA value can be both a phi destination (in the header) and an incoming value (from the body). After mem2reg, phi incoming values from the loop body ARE the phi results from the current iteration. The regalloc may assign them the same physical register. When `phi_reg == incoming_reg`, the phi copy is a no-op (the value is already in the right register).

### Load-Side Coalescing (Phase 8, 2026-03)

- **Eliminating MoveReg by reading directly from allocated registers**: `operand_reg()` checks if a value is currently live in its allocated register and returns that register directly. Lowering code uses the allocated register as the instruction's source operand instead of loading into TEMP1/TEMP2, eliminating the `MoveReg` that `load_operand()` would have emitted. This complements store-side coalescing — together they eliminate moves on both sides of instructions.
- **Dst-conflict safety**: When an operand's allocated register equals the instruction's destination register (`result_reg`), the operand must fall back to a temp register. Otherwise, `emit() → invalidate_reg(dst)` auto-spills the old value and clears alloc tracking before the instruction reads the operand. While the PVM instruction itself would execute correctly (read-before-write at hardware level), the conservative approach avoids subtle alloc-state corruption in edge cases.
- **Div/rem excluded from coalescing**: Signed division/remainder trap code (`emit_wasm_signed_overflow_trap`) uses SCRATCH1 (r5) as scratch for sign-extending 32-bit operands. If the LHS operand is in r5, the trap code clobbers it before the div instruction can read it. Rather than adding per-operation conflict checks, div/rem operations always load into TEMP1/TEMP2.
- **Immediate-folding paths coalesced**: The `commutative_imm_instruction` helper was parameterized to accept a `src` register instead of hardcoding TEMP1. This allows immediate-folding paths (the most common for LLVM-optimized code) to use the allocated register directly. Shift/sub immediate paths were similarly updated.
- **Store instructions have no dst conflict**: PVM store instructions (`StoreIndU8`, etc.) write to memory, not to a register, so they have no destination register. Both address and value operands can freely use allocated registers without conflict checks.
- **Impact**: The fib(20) benchmark dropped from 613 to 511 gas (17%), regalloc two loops from 23,334 to 16,776 gas (28%), and the anan-as PVM interpreter JAM size from 164.9 KB to 158.9 KB (3.6%).

### Store-Side Coalescing (Phase 7, 2026-03)

- **Avoiding MoveReg by computing directly into allocated registers**: `result_reg()` returns the allocated register for the current instruction's result slot, allowing ALU/memory-load/intrinsic lowering to use it as the output destination. This eliminates the `MoveReg` that `store_to_slot` would otherwise emit to copy from TEMP_RESULT into the allocated register. On the anan-as compiler, this reduced store_moves by 54% (2720 to 1262) and total instructions by 4%.
- **`lower_select` cannot be coalesced**: The select lowering loads the false (default) value first, then loads the true value and condition. If the default value is loaded into the allocated register via `load_operand(val, alloc_reg)`, this updates the register cache to associate `alloc_reg` with `val`'s slot. When subsequent `load_operand` calls try to load the true value or condition, they may find stale cache hits pointing at the allocated register (now holding the wrong value). The fix is to keep using TEMP_RESULT for select lowering.
- **`result_reg_or()` needed for zext/sext/trunc**: These lowering paths use TEMP1 (not TEMP_RESULT) as the working register in the non-allocated case, because the source operand is already in TEMP1 and the in-place truncation/extension writes back to the same register. Using TEMP_RESULT would require an extra `MoveReg`. `result_reg_or(TEMP1)` returns the allocated register when available, or TEMP1 as fallback, preserving the existing efficient non-allocated codepath.
- **Control-flow-spanning TEMP_RESULT uses cannot be coalesced**: `emit_pvm_memory_grow` and `lower_abs` both use TEMP_RESULT across branches (grow success/failure, positive/negative paths). Computing into the allocated register would corrupt it if the branch takes the alternative path. These remain uncoalesced.

### Non-Leaf r5-r8 Allocation and load_operand Reload Bug (Phase 6, 2026-03)

- **Removing the leaf-only restriction for r5-r8**: Previously r5/r6 (`allocate_scratch_regs`) and r7/r8 (`allocate_caller_saved_regs`) were only available in leaf functions. Phase 6 makes them available in all functions. The existing non-leaf call lowering infrastructure (`spill_allocated_regs` before calls, `clear_reg_cache` after calls, lazy reload on next access) handles caller-saved register spill/reload automatically, so no new mechanism was needed.
- **Removing the `calls_in_loops` gate**: Previously, non-leaf functions with calls inside loop bodies were skipped entirely by the register allocator (the theory being that reload traffic outweighs savings). Phase 6 removes this restriction. The lazy spill + per-call-site arity-aware invalidation makes allocation beneficial even with calls in loops, since only registers actually clobbered by a specific call's arity are invalidated rather than all registers.
- **`load_operand` reload-into-allocated-register bug**: When an allocated register is invalidated (e.g., after a call) and `load_operand` is asked to reload the value into a *different* target register (e.g., TEMP1 for a binary operation), the original code would reload into the allocated register first, then copy to the target. This is incorrect when the allocated register is being used for call argument setup -- writing to the allocated register corrupts the argument being prepared. The fix: when the allocated register is invalidated and the target register differs, load directly from the stack into the target register, bypassing the allocated register entirely. This prevents corruption during call argument setup sequences where multiple allocated values are being moved into argument registers (r9, r10, etc.).
- **r7/r8 invalidation after calls**: The `reload_allocated_regs_after_call_with_arity` predicate was extended to also invalidate r7/r8 after calls (not just r9-r12), since r7/r8 are now allocatable in non-leaf functions and are always clobbered by call return values.
- **Impact**: 79 non-leaf functions now receive allocation in the anan-as compiler (up from 0), bringing the total to 205 out of 210 functions allocated.