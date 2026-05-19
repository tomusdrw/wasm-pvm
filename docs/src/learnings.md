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

- `instcombine` defaults to `max-iterations=1`, which can cause `LLVM ERROR: Instruction Combining did not reach a fixpoint` on complex IR (e.g., after aggressive inlining). The error is a hard `report_fatal_error` (process abort), not a recoverable Rust error — it bypasses `Error::Located` diagnostics
- Fix: use `instcombine<max-iterations=N>` for a higher cap. We currently use `N=20`
- A cap of 2 is enough for typical IR shapes but not for `--trap-floats` on large modules: every float operator emits a `@llvm.trap()`+`unreachable` cluster, and propagating those through real control flow takes more iterations to fold (issue #212 — observed on the polkadot-fellows v2.2.2 relay-chain runtimes)
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

- **Globals only occupy the bytes they actually need**: the compiler tracks `globals_region_size = (num_globals + (1 if memory.size/grow/init used else 0) + num_passive_segments) * 4` bytes. The heap usually starts right after this region, but when the compiler also reserves a 256-byte parameter-overflow area (any module type signature has > `MAX_LOCAL_REGS` params), `wasm_memory_base` moves to `compute_param_overflow_base(...) + 256`. The mem-size slot is elided for programs that never read/grow memory size or use `memory.init`, saving 4 bytes of `rw_data`.
- **Leading-zero rw_data trim (issue #195 Option 2A, extended)**: anan-as places `rw_data` at `0x30000` via a fixed memcpy, so leading zero bytes can't be dropped without a format change. Two moves together collapse the 4KB structural-padding page that would otherwise prefix `rw_data` for every memory-using program:
  1. **Stable mem-size slot at `0x30000`**: the compiler-managed memory-size global is placed at a fixed offset (`GLOBAL_MEMORY_BASE` itself) independent of `num_globals`. User globals shift to `0x30004+` when the slot is present. Memory-op lowering (`memory.size`/`grow`/`init`) reads a constant address, unaware of the program's global count.
  2. **No 4KB alignment on `wasm_memory_base`**: anan-as allocates `rw_data` a page at a time via `setData` and computes `heapZerosStart = heapStart + alignToPageSize(rwLength)` independently, so the base can land at any byte offset inside the first page without leaving holes. Dropping the alignment places `wasm_memory_base` just past the globals/passive/overflow regions — typically `0x30004` to `0x30018` — so the first data-segment byte sits almost at `rw_data[0]`. Saves ~4 KB per fixture that declares `(memory N)` with data segments, including AS-runtime programs (verified: -3.7 KB on `anan-as-compiler.jam`, -4 KB on most AS fixtures). Note: the WASM-side `args_ptr` value (`ARGS_SEGMENT_START - wasm_memory_base`) shifts with the base, which is an observable ABI change for tests that hard-coded it.
- **`heap_pages` is computed after `build_rw_data()`**: uses the actual (trimmed) `rw_data` length to cover WASM memory from `GLOBAL_MEMORY_BASE` to `wasm_memory_base + initial_pages * 64KB`. A single-page (`+1`) headroom at the heap boundary is reserved so the first `memory.grow`/sbrk call has a pre-allocated page — required for PVM-in-PVM execution to propagate correctly.

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

### llvm.bitreverse vs llvm.bswap

Two distinct LLVM intrinsics easy to confuse:

- `llvm.bswap.iN` — reverses **byte order** (`0xAABBCCDD → 0xDDCCBBAA`). Lowers directly to PVM `ReverseBytes` (opcode 111). For widths < 64, `ReverseBytes` leaves the result in the high bytes of the 64-bit register, so the bswap path follows up with a `ShloRImm64` to recover (shift by `64 - bits`).

- `llvm.bitreverse.iN` — reverses **bit order** within the value (`0x80000001` is a palindrome — bitreverse maps it to itself). PVM has no native bit-reverse, so this is software-emulated via the standard "swap odd/even bits, swap pairs, swap nibbles, swap bytes" algorithm. Supported widths: `i8`, `i16`, `i32`, `i64`.
  - **i8**: 3 mask phases (masks `0x55`/`0x33`/`0x0F`) using `AndImm` + `ShloLImm32`/`ShloRImm32` — no byte-swap step needed for a single byte (the running value stays clean within the low 8 bits).
  - **i16**: same shape with masks `0x5555`/`0x3333`/`0x0F0F`, then `ReverseBytes` + `ShloRImm64` by **48** to recover (matches the bswap path's i16 recovery shift).
  - **i32**: masks `0x55555555`/`0x33333333`/`0x0F0F0F0F`, then `ReverseBytes` + `ShloRImm64` by **32**.
  - **i64**: masks must be loaded via `LoadImm64` into `TEMP_RESULT` and combined with the register-form `And` (since 64-bit masks don't fit in `AndImm`'s i32 immediate); 64-bit shift variants throughout; no post-shift after `ReverseBytes`.

Substrate / polkadot-fellows runtimes hit `llvm.bitreverse.i32` regularly (shared codec/hashing code). LLVM 18's `recognizeBSwapOrBitReverseIdiom` pass folds the canonical open-coded pattern (at any width — we verified i8/i16/i32/i64) into the matching intrinsic before our lowering sees it, so writing the algorithm in WAT is sufficient to exercise every path in tests. For i8/i16 the trick is to load/store with narrow ops (`i32.load8_u` / `i32.store8` etc.) so LLVM's demanded-bits analysis narrows the width of the bitreverse intrinsic from the default i32.

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
- The peephole is still valuable for `--debug-skip-llvm-passes` mode and as defense-in-depth
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
- **Critical**: The parameter overflow area must be >= `GLOBAL_MEMORY_BASE` (0x30000) because the SPI rw_data zone starts at 0x30000. The gap zone (0x20000-0x2FFFF) between ro_data and rw_data is unmapped. Placing constants in the gap zone causes PVM panics.
- The compact layout places the parameter overflow area dynamically right after globals (no fixed address), and `SPILLED_LOCALS_BASE`/`SPILLED_LOCALS_PER_FUNC` have been removed. This reduces the gap between globals and WASM linear memory, saving ~8KB RW data for typical programs (WASM memory base moves from ~0x33000 to ~0x31000 for a program with 5 globals).

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

### Rematerialization — Not Feasible (Phase 8 investigation, 2026-03)

Reloading values with `LoadImm` instead of `LoadIndU64` from stack has **zero practical impact** in this architecture. Three approaches (LLVM IR constant detection, PVM emitter `reg_to_const` tracking at `store_to_slot` time, regalloc-level `val_constants` map) all failed for the same reason: every value reaching the regalloc reload path is a non-constant instruction result. LLVM's `IRBuilder` constant-folds at instruction creation time, so no all-constant-operand instruction survives into the IR; LLVM constants that *do* exist are intercepted by `get_sign_extended_constant()` at the top of `load_operand()`, before the alloc code path. There is no gap between "LLVM knows it's constant" and "the emitter needs to reload it".

Prerequisite for retrying: PVM-level constant propagation that tracks results across `AddImm32` etc., not just `LoadImm`/`LoadImm64`. Significant feature, uncertain ROI.

### Store-Side Coalescing (Phase 7, 2026-03)

- **Avoiding MoveReg by computing directly into allocated registers**: `result_reg()` returns the allocated register for the current instruction's result slot, allowing ALU/memory-load/intrinsic lowering to use it as the output destination. This eliminates the `MoveReg` that `store_to_slot` would otherwise emit to copy from TEMP_RESULT into the allocated register. On the anan-as compiler, this reduced store_moves by 54% (2720 to 1262) and total instructions by 4%.
- **`lower_select` store-side coalescing cannot be used**: Loading the default value into the allocated register via `load_operand(val, alloc_reg)` triggers `invalidate_reg(alloc_reg)` in `emit()`, which corrupts register cache state for subsequent operand loads. However, **load-side coalescing works** (Phase 9): `operand_reg()` is used for all Cmov operands so values already in their allocated registers are used directly without MoveReg copies. This is safe because all select operands are simultaneously live (the allocator guarantees different registers) and the Cmov instruction's `dst` register is only invalidated by `emit()`, not by `load_operand()` on the other operands.
- **`result_reg_or()` needed for zext/sext/trunc**: These lowering paths use TEMP1 (not TEMP_RESULT) as the working register in the non-allocated case, because the source operand is already in TEMP1 and the in-place truncation/extension writes back to the same register. Using TEMP_RESULT would require an extra `MoveReg`. `result_reg_or(TEMP1)` returns the allocated register when available, or TEMP1 as fallback, preserving the existing efficient non-allocated codepath.
- **Control-flow-spanning TEMP_RESULT uses cannot be coalesced**: `emit_pvm_memory_grow` and `lower_abs` both use TEMP_RESULT across branches (grow success/failure, positive/negative paths). Computing into the allocated register would corrupt it if the branch takes the alternative path. These remain uncoalesced.

### Spill Weight Refinement and Call Return Hints (Phase 9, 2026-03)

- **Spill weight call penalty**: Values whose live ranges span real call instructions receive a penalty of 2.0 per spanning call to their spill weight. This represents the cost of the spill+reload pair required when a register is allocated across a call boundary. Binary search on sorted call positions enables efficient counting. Trade-off: a tiny regression in very small functions with a single call (e.g., host-call-log: +3 gas) for consistent improvements in larger functions (e.g., AS fib: -2 gas, aslan-fib: -28 gas).
- **Call return value register hints**: The linear scan allocator accepts `preferred_reg` hints on live intervals. Values defined by real call instructions get a hint for r7 (`RETURN_VALUE_REG`), since the return value is already in r7 after a call. If r7 is free, it's used; otherwise, a different register is allocated. This eliminates the `MoveReg` from r7 to the allocated register in `store_to_slot`.
- **`is_real_call()` made `pub(super)`**: The function distinguishing real calls from PVM/LLVM intrinsics was made module-visible so `regalloc.rs` can use it for call position collection without code duplication.

### Loop Phi Early Interval Expiration (Phase 10, 2026-03)

- **Post-allocation coalescing doesn't work**: Three approaches were tried and all failed due to the emitter's per-register `alloc_reg_slot` tracking disagreeing with the allocator's per-value liveness model. See git history for details.
- **Early interval expiration works**: Modifying the linear scan to expire loop phi destination intervals at their actual last use (before loop extension) frees the register earlier. The incoming back-edge value naturally gets the freed register via the free pool. Since the linear scan's `slot_to_reg` maps reflect both assignments from the start, the emitter handles transitions correctly.
- **Pressure guard**: When `intervals.len() > allocatable_regs.len() * 2`, early expiration is disabled. Under high pressure, freed phi registers get taken by unrelated values, causing reload traffic that outweighs the MoveReg savings.
- **Phi copy no-op**: When incoming_reg == phi_reg AND the register currently holds the incoming value (verified by `is_alloc_reg_valid`), the phi copy is skipped — just update `alloc_reg_slot`. The `is_alloc_reg_valid` check is critical: without it, a third value that overwrote the register between the incoming's store and the phi copy would cause silent data corruption.
- **store_to_slot safety**: When storing to a slot whose allocated register currently holds a DIFFERENT dirty slot, spill the dirty value first. Prevents data loss when multiple slots share a register via early expiration.
- **Impact**: fib(20) -15.7% gas / -7.2% code, factorial -5.6% gas. No regressions.

### Cross-Block Alloc State Propagation (Phase 11, 2026-03)

- **Back-edge dominator propagation instead of clearing**: At loop headers with unprocessed predecessor back-edges, instead of clearing all `alloc_reg_slot` entries, the dominator predecessor's alloc state is propagated through `set_alloc_reg_slot_filtered()`. This avoids unnecessary reloads at loop entry for values that remain valid across the back-edge.
- **Register class filtering for safety**: Non-leaf functions only propagate callee-saved registers beyond `max_call_args` — these are the only registers guaranteed safe across all paths (never clobbered by calls). Caller-saved registers (r5-r8) are excluded because other paths may invalidate them. Leaf functions with lazy spill propagate all registers since no calls exist.
- **Leaf+lazy_spill intersection**: Multi-predecessor blocks in leaf functions with lazy spill now use the same intersection logic as non-leaf functions. Previously, leaf+lazy_spill blocks used `define_label` (clear all) at every block boundary. With the pred_map now available, the intersection approach keeps entries that all processed predecessors agree on.
- **pred_map condition expanded**: The predecessor map was previously built only for non-leaf functions. It is now built whenever `has_regalloc && (!is_leaf || lazy_spill_enabled)`, enabling alloc state propagation for leaf functions with lazy spill.
- **Impact**: fib(20) -5.1% gas, factorial(10) -7.1% gas, is_prime(25) -4.6% gas, PiP aslan-fib -0.52% gas.

### Callee-Saved Preference for Call-Spanning Intervals (Phase 12, 2026-03)

- **Problem**: The linear scan's default `free_regs.pop()` behavior assigns callee-saved registers (added last to `allocatable_regs`) to the FIRST intervals processed. Call-spanning intervals, penalized by `CALL_SPANNING_PENALTY`, sort later and get caller-saved registers that are invalidated after every call — the opposite of what's optimal.
- **Solution**: `LiveInterval.spans_calls` flag marks intervals whose live range contains at least one real call. In non-leaf functions, call-spanning intervals explicitly prefer callee-saved registers (r9-r12 beyond `max_call_args`), while non-call-spanning intervals prefer caller-saved (r5-r8). In leaf functions, all registers are equal (no preference applied). The `preferred_reg` hint (e.g., r7 for call return values) takes priority over the class preference.
- **Impact**: Modest — primarily benefits non-leaf functions with call-spanning values. anan-as PVM interpreter -0.2% code size. Most benchmarks are leaf-dominated.

### TEMP_RESULT Chain Coalescing (Phase 13, 2026-05)

- **Problem**: The dst-conflict fallback in load-side coalescing (Phase 8) was uniform: whenever an operand's cached register equalled the consuming instruction's `dst`, the lowering forced a fallback temp (`TEMP1` or `TEMP2`), which the per-block cache then satisfied with `MoveReg TEMP1, TEMP_RESULT`. For chains of non-allocated results (each landing in `TEMP_RESULT` = r4), this emitted ~47k redundant `r4 → r2` moves per polkadot runtime (67% of all MoveReg in glutton-kusama).
- **Observation**: PVM 3-operand instructions read `src1`/`src2` before writing `dst`. So `Add r4, r4, ?` evaluates correctly even when src1 aliases dst. The conservative fallback is only necessary when `dst` is an *allocated* register — there, alias-with-source can trip `invalidate_reg`, the `slot_cache`, or lazy-spill bookkeeping.
- **Solution**: Route every dst-conflict check through `apply_dst_conflict_fallback(op_reg, fallback, dst)` (`emitter.rs`). When `dst == TEMP_RESULT`, the helper keeps the alias; otherwise it falls back as before. Threaded through 17 lowering sites in `alu.rs`, `intrinsics.rs`, `memory.rs`.
- **Excluded**: `bitreverse` (`intrinsics.rs`) emits `LoadImm64 TEMP_RESULT, mask` mid-sequence — relaxing the alias would clobber `val_reg`. The conservative fallback is preserved with an inline comment.
- **Naturally excluded** because they bypass `operand_reg`: `lower_select`, `emit_pvm_memory_grow`, `lower_abs` use `load_operand` directly.
- **Cascade beyond MoveReg elimination**: The targeted optimization eliminates the `r4 → r2` MoveReg pattern (47k instances observed). Actual MoveReg reduction is 42,986 (70,141 → 27,155, -61%) — slightly below the targeted 47k because some `r4 → r2` instances were already covered by other paths. But total PVM instruction reduction is 50,476 (-4.02%), more than the MoveReg drop alone: eliminating each MoveReg also shortens the surrounding sequence, allowing the following block-boundary cache invalidation / Fallthrough / constant-load chain to shrink. JAM size: -1.97%.
- **Impact on polkadot/glutton-kusama**: JAM 6,573,304 → 6,444,138 bytes (-129 KB, -1.97%). Code 4,751,176 → 4,636,361 bytes (-2.42%). Full integration suite (465 tests) green; clippy clean.

### Non-Leaf r5-r8 Allocation and load_operand Reload Bug (Phase 6, 2026-03)

- **Removing the leaf-only restriction for r5-r8**: Previously r5/r6 (`allocate_scratch_regs`) and r7/r8 (`allocate_caller_saved_regs`) were only available in leaf functions. Phase 6 makes them available in all functions. The existing non-leaf call lowering infrastructure (`spill_allocated_regs` before calls, `clear_reg_cache` after calls, lazy reload on next access) handles caller-saved register spill/reload automatically, so no new mechanism was needed.
- **Removing the `calls_in_loops` gate**: Previously, non-leaf functions with calls inside loop bodies were skipped entirely by the register allocator (the theory being that reload traffic outweighs savings). Phase 6 removes this restriction. The lazy spill + per-call-site arity-aware invalidation makes allocation beneficial even with calls in loops, since only registers actually clobbered by a specific call's arity are invalidated rather than all registers.
- **`load_operand` reload-into-allocated-register bug**: When an allocated register is invalidated (e.g., after a call) and `load_operand` is asked to reload the value into a *different* target register (e.g., TEMP1 for a binary operation), the original code would reload into the allocated register first, then copy to the target. This is incorrect when the allocated register is being used for call argument setup -- writing to the allocated register corrupts the argument being prepared. The fix: when the allocated register is invalidated and the target register differs, load directly from the stack into the target register, bypassing the allocated register entirely. This prevents corruption during call argument setup sequences where multiple allocated values are being moved into argument registers (r9, r10, etc.).
- **r7/r8 invalidation after calls**: The `reload_allocated_regs_after_call_with_arity` predicate was extended to also invalidate r7/r8 after calls (not just r9-r12), since r7/r8 are now allocatable in non-leaf functions and are always clobbered by call return values.
- **Impact**: 79 non-leaf functions now receive allocation in the anan-as compiler (up from 0), bringing the total to 205 out of 210 functions allocated.

### Callee-Saved State Preservation After Calls — Not Feasible (2026-03)

Preserving `alloc_reg_slot` for callee-saved registers (r9–r12) across calls breaks because `operand_reg()` (load-side coalescing) returns the allocated register directly as a source operand for memory lowering. The memory lowering code may then use the same register as both source AND destination when adding `wasm_memory_base` for address computation, clobbering the preserved value. Selective invalidation in `clear_reg_cache`, snapshot/restore around the call, and guarding `operand_reg` were all tried; all fail through interactions between preserved alloc state and the general register cache. Deterministic failure mode (`as-array-push-test`): wrong base register r7 with shifted +12 offset after a call, producing `result = 0` instead of `28`. Same root cause as "Non-Leaf r7/r8 Allocation" below — both would need `operand_reg()` to distinguish "data operand" (safe) from "address base" (unsafe).

### Per-Phi Early Expiration Guard — Not Feasible (2026-03)

Replacing the blanket pressure guard (`intervals.len() > allocatable_regs.len() * 2`) that disables all loop-phi early expiration with a per-phi check fails under both pressure regimes: high pressure (multiple failures + timeouts because intervening intervals steal the freed register even when the incoming-start condition holds) and low pressure (fib(20) +19.6% gas because the per-phi guard disables expiration for phis whose incoming value is defined inside the loop body). Root cause: early expiration + register reuse depends on the linear scan's allocation *order*, which can't be predicted during interval computation. A correct per-phi guard would require lookahead into allocation decisions, defeating the purpose. The blanket pressure threshold is a crude but effective proxy.

### Non-Leaf r7/r8 Allocation — Not Feasible (2026-03)

Same root cause as "Callee-Saved State Preservation After Calls" above. The `operand_reg()` hazard: any allocated register that participates in an address calculation can be corrupted when the lowering code uses it as both base and destination for in-place arithmetic. Fixing this would require `operand_reg()` to distinguish "use as data operand" from "use as address base" — a non-trivial emitter rework.

### Multi-Predecessor Cross-Block Cache Propagation — Zero Realized Impact (2026-05)

Extending single-predecessor cross-block cache forwarding to multi-predecessor blocks (intersect predecessor snapshots, invalidate phi destinations) was correct and tested green — but byte-identical to baseline on glutton-kusama and kusama. Three reasons it didn't fire:

- **`load_operand` skips `slot_cache` for allocated values**: cache lookup is gated behind `regalloc.val_to_reg`. With aggressive regalloc, virtually every live value is allocated, so reads route through `alloc_reg_slot` and never reach the cache. The propagation helps a code shape that barely exists in regalloc'd runtimes.
- **`alloc_reg_slot` intersection was already present**: the existing `all_processed` branch in `lower_function` already does `set_alloc_reg_slot_from(pred0) + intersect_alloc_reg_slot(rest)`. The "new" propagation re-derives the same end state.
- **Block layout prevents `all_processed` at most multi-pred merges**: `compute_block_layout` is greedy fallthrough-biased, not topological. For a canonical if-else `entry → {then, else} → join`, layout is `entry, else, join, then` — at `join`'s emission `then` is unprocessed and propagation skips. Instrumented: 48,473 merge candidates on glutton-kusama, ~250 entries actually propagated function-wide.

Unblocking would require RPO emission (sacrifices fallthrough), loop-body register-liveness analysis, or a two-pass exit-snapshot dataflow pre-pass — all substantially larger than a localized tweak.

---

## Hand-Crafted Blake2b WAT (2026-04)

### WAT memarg attribute order: `offset` must come before `align`

Writing `(i64.load align=1 offset=8 ...)` fails to parse in this project's WAT frontend with "unknown operator or unexpected token". Writing `(i64.load offset=8 align=1 ...)` parses cleanly. The WebAssembly text format spec permits either order, so this is a tooling quirk (likely `wat-parser` / `wasmparser`). If you're hand-writing WAT and see unexplained parse errors on `i64.load` / `i64.store` with memargs, swap the attribute order first. Example from `tests/fixtures/wat/blake2b.jam.wat`.

### Gas/size characteristics of a typical cryptographic hash on PVM

For reference when sizing new crypto workloads on PVM:

- Blake2b ("abc", 32 B output): JAM = 8269 B, PVM code = 3076 B, gas = 17,749, time ≈ 71 ms single-run.
- Blake2b (1024 B input, 32 B output): gas = 138,478 (~15k gas per 128-byte compression block, roughly 9 blocks).
- In PVM-in-PVM, the same 3-byte input costs ~16.7M outer gas — a ~944× multiplier over direct PVM execution, consistent with what other compute-heavy fixtures show.

Per-compression-block gas is dominated by the 12 rounds × 8 G calls × ~18 i64 ops. No specific compiler optimization was needed to land this — the default pipeline (mem2reg, instcombine, GVN, peephole, register allocation) produced a correct, reasonably compact output on the first run.

### Output-pointer convention for fixtures: don't rely on WASM offset 0

`blake2b.jam.wat` currently writes its hash output to WASM-relative offset 0 and returns `(ptr=0, len=out_len)`. This works today because the WAT has no globals, no prologue, and no data segments below 0x80. But this is fragile — if a future compiler change puts anything at offset 0, the hash would be silently corrupted. When writing new fixtures, prefer an explicit offset ≥ 0x100 for output buffers. Retrofitting blake2b to this convention is a cheap follow-up but was not done in the initial PR since the tests cover the output end-to-end.

### `(if COND (then (unreachable)))` guards can be silently eliminated

While adding invalid-`out_len` trap tests for blake2b, we discovered that a bare `(if COND (then (unreachable)))` guard can be **elided by the LLVM-based compiler** even when `COND` is a runtime value. The trap appeared to fire for some inputs (e.g. `out_len=0` via `i32.eqz`) but not others (`out_len > 64` via `i32.gt_u`). Adding any side-effecting instruction before `unreachable` — e.g. `(i32.store8 ...)` — restores the guard.

**Mechanism (hypothesized):** LLVM treats `unreachable` as a UB hint — "control never reaches here." The optimizer can legally conclude "if this path is UB, then COND is always false" and delete the check entirely. Which specific patterns get eliminated depends on how `instcombine` / `simplifycfg` / GVN canonicalize the condition. `i32.eqz` apparently canonicalizes into a form the optimizer preserves; `i32.gt_u` into a form it doesn't.

**Workaround:** Put at least one side-effecting operation in the `then` block. A sentinel store to an unused memory byte is sufficient:

```wat
(if (some-condition)
  (then
    (i32.store8 (i32.const 0x268) (i32.const 0xEE))
    (unreachable)))
```

**Runtime trap observation from anan-as / SPI mode:** a trapped program exits with OS exit code 0 (not an error), prints `STATUS = -1` in debug output, and produces an **empty Result: [0x]**. `runJamBytes` therefore does **not** throw on trap — it returns an empty `Uint8Array`. Test assertions for trap behavior should check `result.length === 0` rather than `expect(...).toThrow()`.

**Follow-up:** a proper compiler-level fix would be to mark `unreachable` as a true trap (non-UB) in the PVM lowering, or emit an explicit trap instruction that the optimizer can't eliminate. Until then, the sentinel-store workaround is the portable fix for WAT-level fixtures.

### anan-as SPI mode: transient "Run out of pages" failure under sustained test load

Under rapid back-to-back `bun test` runs at high iteration counts (e.g. `SHA512_RANDOM_COUNT=1000`), the anan-as PVM runtime in `--spi` mode occasionally prints:

```text
Warning: Run out of pages! Allocating.
Unhandled host call: ecalli 0. Finishing.
```

and the test result comes back empty. The default iteration count (`SHA512_RANDOM_COUNT=50`) has not reproduced the failure. The same input hex that triggered it under `bun test` succeeded on 10/10 standalone `node anan-as ... run` invocations, ruling out any problem in the SHA-512 WAT or the test harness.

This is a non-deterministic issue in the anan-as runtime itself, not a PVM compiler bug or SHA-512 correctness issue. The runtime appears to run out of pre-allocated pages and then fails to service the resulting allocation host call (shown as ecalli 0) in `--spi` mode; the exact trigger is unclear but correlates with sustained rapid test-suite execution.

**Repro (against the original SHA-512 WAT):** seed `0x0123456789abcdef`, iteration 9 (inputLen 14439), run under `bun test layer3/sha512.test.ts` with `SHA512_RANDOM_COUNT=1000`.

**WAT-level mitigation that correlated with a fix in the SHA-512 case:** copy the entire input from the PVM args region (`args_ptr`, at `0xFEFF0000`) into WASM memory in one upfront `memory.copy`, then stream from there. The hot compress loop now reads only from the pre-allocated WASM region. After this change, 1000-iter run went from 999/1000 pass in 1023 s to 1000/1000 pass in 506 s. We have only the observed correlation — the exact trigger inside anan-as remains unclear — but the scattered args-region reads are a plausible contributor to both the failure and the wall-clock overhead, and consolidating them into one contiguous read is defensible on design grounds regardless. The `+143 B` JAM-size / `~4%` gas cost is cheap for the apparent stability and speed gains.

The blake2b follow-up (see next section) gives a more mechanical explanation for the wall-clock component — misaligned cross-page u64 loads — which is very likely the same root cause.

### Cross-PVM-page `memory.copy` reads from a misaligned source blow up gas

`memory.copy`'s word loop issues one `LoadIndU64` per iteration. When the source address is 8-byte-aligned, each load sits entirely inside one PVM page (pages are 4 KB). When the source is *misaligned*, one u64 read per page will straddle two pages — and that cross-page u64 read is extremely expensive in anan-as (orders of magnitude slower than aligned reads). A WAT that streams from the PVM args region (`0xFEFF0000`, always 4 KB-aligned) via a pointer like `args_ptr + 1` (misaligned by 1) will OOG well before finishing a 32 KB input; the inflection point is around ~4 KB, right where the first cross-page straddle happens.

**Observed with `tests/fixtures/wat/blake2b.jam.wat`** while raising its differential input cap from 2 KB to 32 KB (issue #197):

- Original `[out_len: u8][input: bytes]` format placed the input at `args_ptr + 1` — misaligned, cliff at ~4 KB inputs.
- Any WAT-level "copy-into-WASM-memory-first" fix had to keep *both* the bulk copy's source and destination 8-byte-aligned, or the same cross-page cost reappeared during the upfront copy (only now hidden from the naïve "stream from WASM memory" mental model).
- Final shape: pad the header to 8 bytes (`[out_len: u8][7 zero bytes][input: bytes]`). With `args_ptr` always 4 KB-aligned and the destination at `0x1000`, the bulk copy is fully aligned, the input lands at `args_ptr + 8` (still aligned), and `data_ptr = 0x1008` keeps every downstream 128-byte stream copy aligned in WASM memory. Test-harness only sees the new 8-byte args envelope via `encodeBlake2bArgs()`.
- Gas at 32 KB went from "OOG past 1 B gas" to ~4.6 M gas. Linear scaling restored.

**Heuristic for new WAT fixtures that read args in bulk:** make the input portion of args start at an 8-byte offset from `args_ptr` (either by having *no* prefix, like SHA-512, or by padding any prefix out to 8 bytes, like the blake2b fix above). Keeping every downstream `data_ptr` / stream-`memory.copy` source 8-byte-aligned avoids the cliff regardless of which page of the args region the tail falls in.

The SHA-512 WAT happens to have no prefix (input starts at `args_ptr + 0`), which is why the earlier SHA-512 fix was sufficient for that fixture — it stayed aligned by accident of format. Blake2b needed the padding change to benefit from the same pattern.

---

## Compilation Reproducibility (2026-04)

The compiler must produce byte-identical JAM output for the same WASM input across invocations. Two subtle traps were hit and fixed; keep both in mind when adding code to the backend.

### Trap 1: `HashMap`/`HashSet` iteration order is process-randomised

Rust's default `HashMap`/`HashSet` use a per-process-randomised hasher, so iteration order changes between CLI invocations. Any iteration whose side effects reach the emitted bytes (emitting an instruction, assigning a register/offset, mutating state read by the next iteration) leaks that randomness. The mitigation is the `AGENTS.md` rule: prefer `BTreeMap`/`BTreeSet` throughout; if a key type has no `Ord` (e.g. `inkwell::BasicBlock`), keep the `HashMap` for lookups only and collect into a `Vec` sorted by a derived key before iterating.

### Trap 2: `ValKey` originally wrapped a raw LLVM pointer

`ValKey` used to wrap `Value::as_value_ref() as usize` — the raw LLVM pointer. LLVM allocates different `Value` subclasses (e.g. `Argument`, `InstructionValue`) from separate arenas at independent ASLR-randomised base addresses, so the derived `Ord` was pointer-address order: a `BTreeMap<ValKey, _>` iterated in pointer order, which flipped between process invocations whenever entries came from different arenas.

Where this bit us: `compute_live_intervals` iterated `value_slots: BTreeMap<ValKey, i32>` directly and then pushed intervals into a `Vec` in that order. The downstream linear scan is stable-sorted by `(start, spill_weight)`; ties fell back to input order, which meant pointer order, which meant non-deterministic register assignments under aggressive allocation (more ties at min_uses=1).

The fix (issue #204) replaces the raw pointer with an insertion-order ID. A per-function `ValKeyCache` on `PvmEmitter` maps the LLVM pointer to a monotonically-increasing `u32` the first time the value is observed during IR walking; subsequent observations return the same ID. Because the IR-walking order (`pre_scan_function` + regalloc linearisation) is deterministic, the IDs are too — `BTreeMap<ValKey, _>` iteration is now reproducible across runs by construction, no derived-key sort required.

### Trap 3: Order-dependent loops over `HashMap<BasicBlock, _>`

Most `HashMap<BasicBlock, _>` iteration sites in the backend are commutative (e.g. `end = end.max(...)` across loop headers, `depths[i] += 1` across positions), so the carve-out for `BasicBlock` keys (which lack `Ord`) was considered safe. Except one case wasn't commutative: the live-interval extension loop reads `end` in its predicate and mutates `end` in its body, so iteration N+1's predicate depends on iteration N's effect. Fixed by returning `Vec<(BasicBlock, usize)>` sorted by header position from `detect_loop_headers`. When adding a new iteration over a `BasicBlock`-keyed map, prove commutativity explicitly — "I think this is order-invariant" is how this one slipped in.

### Detection

`tests/utils/check-determinism.sh` compiles a diverse set of fixtures N times in separate processes and diffs the output. A single-process cargo test cannot catch these traps because the `HashMap` hasher seed and the LLVM arena addresses are both fixed for the lifetime of one process. The script is wired into the integration CI job.

---

## Trap-Floats Lowering — Don't Set `unreachable = true`, and Use `@llvm.trap`

`--trap-floats` replaces every f32/f64 operator with an LLVM-level trap (PVM backend lowers to `Trap`). Two non-obvious traps to avoid in the implementation:

### Trap A: setting `self.unreachable = true` after the float trap

The naive implementation is "emit the trap, set `self.unreachable = true`, push placeholder zeros for the operator's outputs." This is wrong on two counts:

1. **The placeholder zeros are never consumed.** The dead-code skip path at the top of `translate_operator` returns `Ok(())` for every non-control-flow op when `self.unreachable` is true — including any future op that would have consumed those zeros. Pushing them is dead work.

2. **Function-result phis end up with no incoming branches.** The function-end implicit `Block` frame's `End` handler skips the "pop result, branch to merge" path when `self.unreachable` is true. If the only path through the body trapped, the result phi at `fn_return` has zero incoming edges → LLVM verifier rejects the module. The same hazard applies to `if`-arm phis when both arms trap.

The correct lowering: emit `unreachable`, create a fresh `after_float_trap` basic block, position there, pop the operator's inputs from the operand stack, push `i64 0` placeholders for its outputs, **and leave `self.unreachable` alone**. Subsequent ops translate normally into the (provably-dead) block; `End` handlers run their reachable branch and add a placeholder-zero incoming to the merge phi; LLVM's `dce` collapses the unreachable region away. Result: valid IR + correct runtime trap + no special-case handling for trap-floats in any other translator path.

The investigation cost was non-trivial — the broken phi only manifests when both arms of a structured construct trap, which is a rare pattern in the unit tests but common in trap-floats mode (entire float-heavy functions trap on the first const). The integration test `trap_floats_inside_if_arm_compiles` pins this down.

`self.unreachable` keeps its original meaning: "WASM operand-stack-aware dead code following an explicit `unreachable`/`return`/`br` operator." The trap-floats lowering produces *LLVM*-level dead code, not WASM-level dead code, and the two abstractions must not be conflated.

### Trap B: bare `unreachable` is folded by simplifycfg as UB

The first working version emitted only `build_unreachable()` (no `@llvm.trap` call). Tests verified compilation succeeded, but a runtime-execution test caught the real bug: floats inside an `if`-arm vanished. anan-as reported `Status: 0` (clean halt) on the trap path because **LLVM's `simplifycfg` folds branches whose only path leads to `unreachable`** — it treats `unreachable` as "this code is impossible; the condition must steer away from it" and rewrites the conditional branch to always take the other arm. Float-only else-bodies were silently deleted; the JAM ran the then-arm regardless of the condition.

The fix: emit `@llvm.trap()` (a real intrinsic call) followed by `build_unreachable()`. `@llvm.trap` is `noreturn` but **not** UB-on-reach — the optimizer treats it as a side-effecting call and preserves it. The PVM backend gains a dedicated case in `lower_llvm_intrinsic` that emits `Instruction::Trap`. The bare `unreachable` after the call is fine (it's now redundant but lets the verifier see the BB has a terminator).

Detection lesson: a pure compilation test can't catch this. The Rust integration tests all checked "JAM compiles and contains a Trap instruction" — which was true (the entry-header trap is always present). Only running the JAM through anan-as with both branch inputs and asserting `Status: 1` on the trap path exposed the elimination. The bun layer1 test `trap-floats.test.ts` is the regression guard.

---

## Loop `End` Must Preserve `unreachable` When the Body Has No Fall-Through (2026-05)

The only path into a `loop`'s `merge_bb` is the fall-through branch from the body — `br N` targeting a `Loop` jumps to the *header*, never to the merge. So when the body ends in unreachable state (e.g. `loop { return …; br 0 }`), `merge_bb` is left with zero predecessors, and post-loop code is physically dead.

The original `ControlFrame::Loop` `End` handler unconditionally reset `self.unreachable = false`, which broke this invariant: subsequent operators were translated as if reachable, even though their only path was through an empty `merge_bb`. In the polkadot-fellows v2.2.2 hashbrown insert (surfaced once `--trap-floats` lets us reach it), this caused the function-level `End` to call `pop()` on an empty operand stack and fail with `Internal error: operand stack underflow`.

The fix in `function_builder.rs::translate_operator` is two parts. (1) `Loop`'s `End` now mirrors the body's fall-through: keep `self.unreachable = true` when the body didn't fall through, and terminate the empty `merge_bb` with `build_unreachable()` so the LLVM verifier still accepts it. Just toggling the flag without the terminator trips `Basic Block in function 'X' does not have terminator!`. (2) The dead-code dispatcher's "dummy" `Block`/`If` frames reuse the current — already terminated — block as `merge_bb` (and `else_bb`); their matching `End`/`Else` handlers must detect this via `merge_bb.get_terminator().is_some()` and skip the `position_at_end`/`unreachable=false` reset, otherwise the bug returns one nesting level out (a downstream operator emits past a terminator, or worse, the function-level `End` again sees a stale `unreachable=false`).

Why both fixes are needed together: with only fix (1), an inner construct (e.g. another `loop (result T)`) appearing after the unreachable loop becomes a dummy frame; its `End` handler still flipped `unreachable=false`, re-creating the same underflow at function-level `End`. The Rust test `loop_unreachable_end.rs::unreachable_loop_followed_by_result_loop_compiles` exercises both paths simultaneously and is the regression guard.

Validation note: the WASM validator does *not* propagate the loop body's unreachable state into the surrounding scope — `pop_ctrl()` pushes the frame's `end_types` onto the outer operand stack regardless of inner unreachability. So the most-minimal `loop { return; br 0 } end_function` shape is rejected upstream by `wasmparser::validate`. The bug only surfaces when the post-loop region is well-typed for the validator (e.g. a trailing `unreachable`, or a follow-up construct that pushes the function's result type) but the compiler's own `unreachable` tracking has been corrupted.

---

## LLVM `freeze` Lowers to a Value Passthrough (#218)

LLVM's `freeze` instruction takes a value that may be `poison`/`undef` and converts it into "some specific bit pattern, but we don't say which" — operationally a no-op on a concrete value. Our LLVM optimizer occasionally emits it (instcombine sinking poison-carrying ops past branches; observed on polkadot-fellows v2.2.2 `glutton-kusama_runtime` and `encointer-kusama_runtime` under `--trap-floats`).

By the time IR reaches the PVM backend, every value is a concrete i64 in a stack slot — there is no `poison`/`undef` representation. So `freeze` is implemented as a value passthrough: take the operand, materialize it into the result slot. The arm sits next to `Phi` in `lower_instruction` (`llvm_backend/mod.rs`) and uses load-side coalescing — when the operand is already in an allocated register, `store_to_slot` writes from that register directly.

Two pieces are required for the lowering to work end-to-end:

1. **The match arm in `lower_instruction`** (the visible fix).
2. **`Freeze` listed in `instruction_produces_value`** (`llvm_backend/emitter.rs`). The pre-scan walks every block and allocates a stack slot for any instruction whose result is consumed downstream; without `Freeze` in the producer set, `result_slot()` later returns `Error::Internal("no slot for Freeze result")`. Easy to miss: the `lower_instruction` arm compiles cleanly without it and the passthrough is well-defined — the failure only surfaces when the test actually runs.

Testing strategy: triggering `freeze` reliably from a small WAT input is hard. WASM produces no poison itself, our frontend never adds `nsw`/`nuw` flags, and the optimizer passes we run (`mem2reg`, `instcombine`, `simplifycfg`, `gvn`, `dce`) only emit `freeze` for specific shapes that don't reduce to small fixtures. The regression test in `llvm_backend::tests::freeze_is_lowered_as_passthrough` parses hand-written LLVM IR text via `Context::create_module_from_ir` (inkwell 0.8 doesn't expose `build_freeze`) and runs it through `lower_function` directly with a minimal `LoweringContext`. This bypasses the LLVM-version-dependent question of "what input emits freeze" and pins down the lowering arm directly.

---

## Saturating-arithmetic intrinsic lowering (#217)

Lowering `llvm.{u,s}{add,sub}.sat.iN` splits cleanly by width:

- **Narrow widths (i8/i16/i32) — clamp via wider arithmetic:**
  - Unsigned: zero-extend operands, do 64-bit add/subtract (which cannot overflow because both operands fit in 32 bits), then `MinU` (uadd) or branch + `CmovNzImm dst, cond, 0` (usub) to saturate. Result is naturally zero-extended.
  - Signed: sign-extend operands (`SignExtend8`/`SignExtend16` or `AddImm32 _, _, 0` for i32), do 64-bit add/subtract (true result fits in i64 because two iN values differ/sum to at most 2^(N+1)), then clamp to `[INT_MIN, INT_MAX]` via signed `Max`/`Min`. Result is naturally sign-extended.

- **i64 — no wider register, must detect overflow in-place:**
  - Unsigned: `Add64`, then test `sum < a` (unsigned) for wrap; `CmovNz` saturates to `UINT64_MAX`.
  - Signed: Hacker's Delight — overflow flag is bit 63 of `(a^b) & (a^sum)` (sub) or `(a^sum) & (b^sum)` (add). `SharRImm64 by 63` extracts the flag as 0 or -1; saturation value `INT_MIN`/`INT_MAX` is built from `sign(a) XOR INT_MAX`. The signed i64 paths use SCRATCH1/SCRATCH2 and bracket the sequence with `spill_allocated_regs` + `reload_allocated_regs_after_scratch_clobber` (same compromise as non-rotation `fshl`/`fshr`).

The narrow paths are 5-7 instructions; i64 paths are 4 (uadd) / 3 (usub) / 10 (ssub/sadd). All paths use `result_reg`-driven store-side coalescing so the final saturated value lands directly in the register-allocated destination.

**Critical: avoid `TEMP_RESULT` clobber after `dst` is written.** `result_reg` may return `TEMP_RESULT` (r4) when no allocated register is available. After `Add64 dst, ...` (or `Sub64`), any subsequent `LoadImm TEMP_RESULT, ...` would overwrite the sum/difference. The narrow-width sat helpers therefore load constants into `TEMP1` (which is dead after Add/Sub), not `TEMP_RESULT`. The bug surfaced under register pressure in the layer3 fixture; it doesn't show up in small unit tests where `result_reg` returns an allocated register.

**Test coverage limitation:** WAT-driven tests for narrow-width and signed sat intrinsics only fold to `@llvm.{u,s}{add,sub}.sat.i64` (not the narrow widths) because LLVM 18 `instcombine` doesn't fold the canonical `clamp` shape through outer `zext`/`sext` to i32. The narrow-width and signed-narrow backend paths are present and correct algorithmically, exercised by real-world Rust IR (verified via the polkadot-fellows v2.2.2 runtime smoke check). The `dump_llvm_ir` test-harness helper exposes the post-pass IR so unit tests can assert which intrinsics were folded.

## Phi-Copy Resolution: Slot-Based Parallel Moves (#219)

The original phi-copy lowering snapshotted every incoming value into a distinct temp register (TEMP1, TEMP2, TEMP_RESULT, SCRATCH1, SCRATCH2 — five slots) and then wrote them all to their destinations, bailing with `Unsupported("too many phi values for available temp registers")` whenever a join block produced more than five copies on a single edge. The shape is rare in MVP-style code but appears reliably in the largest polkadot-fellows runtimes (`asset-hub-{kusama,polkadot}`, `bridge-hub-polkadot`) when compiled with `--trap-floats`.

The fix replaces the bail with a slot-based parallel-move resolver in `llvm_backend/control_flow.rs::emit_phi_copies_via_slots`. Key design points:

- **Canonical state on the stack.** `spill_all_dirty_regs()` runs first, so each value's authoritative copy lives at its allocated slot. The resolver reads/writes slots directly with `LoadIndU64`/`StoreIndU64` and never depends on register-cache state.
- **Constants are detached from the dependency graph.** A phi whose incoming value is a constant has no source slot, so it cannot participate in a cycle. Constants are emitted *after* the slot-to-slot moves with `LoadImm + StoreIndU64`. If the constant-copy destination happens to be another phi's source, the slot reads have already happened, so the order is sound.
- **Topological pass for the easy case.** A copy whose destination slot isn't anyone else's source can fire immediately (2 instructions: load via TEMP1, store). Real-world phi shapes — even on hot blocks in large runtimes — are dominated by this case.
- **Single-temp cycle handling for the hard case.** Remaining copies form one or more disjoint permutation cycles. For each cycle `(d_0, s_0) … (d_{k-1}, s_{k-1})` (closed when `s_{k-1} == d_0`), the resolver: saves slot `d_0` to TEMP1, walks copies 0..k-1 via TEMP2 (2 instructions each), then finalizes the last write from TEMP1. Total `2k` PVM instructions per cycle — same as the old temp-snapshot path used to cost when it didn't bail. Two temp registers are enough for arbitrary cycle length.
- **Cache invalidation after every direct slot store.** Each raw `StoreIndU64` to a phi destination calls `PvmEmitter::invalidate_cache_for_slot`, which drops the general `slot_cache` entry *and* clears any `alloc_reg_slot[r] == Some(slot)` mapping. Without this, later `operand_reg`/`load_operand` calls in the same block could believe an allocated register still holds the (now stale) old value of the destination slot.

The two existing fast paths (≤5 copies) are kept verbatim: the regression risk is concentrated entirely in the new fallback, and benchmarks show **zero gas/size delta** across the standard benchmark suite (no benchmark hits the `>5` threshold).

**Why a stack-only resolver, not a register-based one?** The regaware (lazy-spill) phi path could in principle resolve cycles in registers (it already discovers per-copy `incoming_reg`/`phi_reg` allocations). But once the fallback triggers, the active set is large enough that the dependency graph cuts across both register- and stack-only copies; the cleanest correctness story is to drop into a uniform slot-based representation. The resolver invalidates `alloc_reg_slot` for every destination slot it writes, so the next access through `load_operand` reloads from the canonical stack copy — no special-casing needed.

**The loop-header swap as the canonical cycle.** The motivating cycle shape comes from loops whose header contains multiple phis that reference each other on the back-edge, e.g.

```llvm
header:
  %a = phi [%init1, %entry], [%b, %latch]
  %b = phi [%init2, %entry], [%a, %latch]
```

On the body→header edge this becomes two simultaneous copies — `a.slot ← b.slot` and `b.slot ← a.slot` — a 2-cycle. The test `many_phi_values_with_loop_cycle_compiles` (in `crates/wasm-pvm/tests/phi_many_values.rs`) drives a 6-cycle through this pattern.

## O(N²) Byte-Size Scans Blocked Real-World Compilation (#225)

Once #214/#215/#217/#218/#224 closed every *correctness* gap that had been bailing the backend early on Polkadot runtimes, compilation finally reached `translate/mod.rs::compile_via_llvm`'s emission loop and `resolve_call_fixups` — and **hung at 99% CPU past 10 minutes on the smallest 2 MiB runtime**. Per-pass timing showed all LLVM passes finishing in ~2 s and per-function PVM backend lowering in ~1.6 s across 1631 functions; the missing minutes were spent in two adjacent O(N²) shapes neither of which had ever been exercised on a multi-MB module before.

**The bug.** Both loops computed instruction byte offsets the same way:

```rust
// Emission loop, per function:
let func_start_offset: usize = all_instructions.iter().map(|i| i.encode().len()).sum();
function_offsets[local_func_idx] = func_start_offset;
// ...
all_instructions.extend(translation.instructions);

// resolve_call_fixups, per direct + indirect call:
let return_addr_offset: usize = instructions[..=jump_idx]
    .iter().map(|i| i.encode().len()).sum();
let jump_start_offset: usize = instructions[..jump_idx]
    .iter().map(|i| i.encode().len()).sum();
```

Each invocation re-summed every preceding instruction's encoded byte length. For F functions / C call sites / M total instructions, the work is `O(F × M) + O(C × M)`. `glutton-kusama_runtime` lands at F=1631, C≈20 000, M≈1.5 M — roughly **3 × 10¹⁰ allocating `encode()` calls** total. `Instruction::encode()` returns a fresh `Vec<u8>` whose only consumer was `.len()`, so the cost was 30 billion small Vec allocations on top of the arithmetic.

The shape had been latent for as long as the emission loop and the fixup resolver have existed. It went unnoticed because the backend used to fail early on real-world modules — every Polkadot runtime hit either `bitreverse`, `usub.sat`, `freeze`, or a "too many phi values" bail before reaching the offset-computation hot path.

**The fix.** Two O(N+M) replacements in `translate/mod.rs`:

- *Emission loop:* maintain a running `current_code_bytes: usize` seeded from the entry header (which is pushed *before* the loop), update it by summing only the *newly appended* slice after each function is lowered, and use it directly for `function_offsets[local_func_idx]`.
- *`resolve_call_fixups`:* compute a `byte_prefix: Vec<usize>` once at function entry, with `byte_prefix[i] = sum(instructions[0..i].encode().len())`. Each fixup then reads `byte_prefix[jump_idx]` / `byte_prefix[jump_idx + 1]` directly.

**Why the prefix sum stays valid through patching.** The fixup loop patches `LoadImmJump.offset` (per `encode_one_reg_one_imm_one_off`, always a fixed 4-byte little-endian field — `bytes.extend_from_slice(&offset.to_le_bytes())`) and, *after* the loop returns, the entry-header `Jump.offset` (per `Self::Jump { offset }`, also `to_le_bytes()` so 4 bytes). Neither patch changes the instruction's encoded length, so a prefix sum computed once at the top of `resolve_call_fixups` is safe to use throughout.

This is *not* true of `encode_imm` (used for plain `LoadImm`, `JumpInd`, `AddImm32`, etc.) which produces 0–4 bytes depending on the immediate's magnitude — but those instructions aren't patched anywhere in `compile_via_llvm` once emitted, so they stay constant from the prefix-sum computation onwards.

**Verified-safe seeding.** The emission loop pushes 2 entry-header instructions (one `Jump` + either another `Jump` or `Trap`) before iterating, so `current_code_bytes` is initialized from `all_instructions.iter().map(|i| i.encode().len()).sum()` — paying the one-time cost across exactly those two entries. Forgetting this offset (`= 0`) was an early version of the fix that passed glutton but broke `test_branch_fixup_resolution` (`crates/wasm-pvm/tests/emitter_unit.rs:194-220`), which compiles a single-`if` function where main is emitted first and the entry-header `Jump.offset` ends up at zero — a fast in-flight regression catch that justifies why this test was worth keeping.

**Result on glutton (2.04 MiB WASM, 1631 functions):** compile time drops from >10 min (hard timeout, never finished) to **~4 s** — ≥150× speedup. All 14 polkadot-fellows v2.2.2 runtimes now compile in 4:26 wall-clock total. Standard benchmark JAM/code/gas numbers are byte-identical across main and the fix (verified by md5sum), since this change is purely compile-time.

## Libcall Recognition for `__multi3` / `__udivti3` (2026-05)

WASM has no `i128` type, so `rustc` for `wasm32-unknown-unknown` lowers every 128-bit operation to a call into the compiler-builtins runtime, which it bakes into each binary. The two workhorses are **`__multi3`** (`i128 × i128 → i128`, ~110 bytes WASM body of Knuth-style i64 partial products) and **`__udivti3`** (`u128 / u128 → u128`, a thin wrapper over `specialized_div_rem`, ~1100 bytes total). Every `(a as u128) * (b as u128)`, `(a as u128) / (b as u128)`, and the `*_hi` helpers route through these.

After our LLVM optimization passes (with `inline_threshold = Some(5)`) these stay as separate functions — their body sizes far exceed the threshold so they're marked `noinline` and the call sites remain visible as `call wasm_func_N(sret, a_lo, a_hi, b_lo, b_hi)`. That gave us a clean intercept point.

**Recognition is name-based.** During `WasmModule::parse` we scan the local-function name table (from the WASM custom `name` section, falling back to exports), match against `__multi3` / `__udivti3`, verify the signature is exactly `(i32 sret, i64 a_lo, i64 a_hi, i64 b_lo, i64 b_hi) → void` (in our i64-uniform IR: 5 i64 params, no return), and for `__udivti3` additionally walk the body for its first `Call` (the slow-path callee) and first `GlobalGet` (the `__stack_pointer` global). Both are required for the synthesized body to have a working slow path; without them recognition silently no-ops. The signature gate prevents a user function that happens to share a reserved-by-ABI name from being silently mis-translated.

**Why not IR pattern matching.** Naive IR pattern matching on call sites would catch the post-inline case (when someone bumps `--inline-threshold` past the body size), but is fragile across rustc versions: different toolchain releases shuffle the Knuth-expansion shape and a matcher tuned for rustc 1.85 silently stops matching on 1.86. Name-based body replacement is robust as long as compiler-builtins keeps these reserved names, which is part of the C/Rust ABI.

**`__multi3` body (8 PVM instructions).** For `a × b mod 2^128` where `a = a_lo + 2^64·a_hi` and similarly for `b`:

```text
low_half  = a_lo × b_lo                                                  (Mul64)
high_half = upper64(a_lo × b_lo) + (a_lo × b_hi) + (a_hi × b_lo)         (MulUpperUU + 2×Mul64 + 2×Add64)
```

All operations are mod 2^64, which conveniently provides the i128 sign correction: when callers pass sign-extended high halves (`(a as i64) >> 63` = all-ones or all-zeros), `(-1) × b_lo = -b_lo` is exactly the correction term needed to convert the unsigned upper half into the signed upper half. So **`MulUpperUU` (opcode 214) is sufficient** — we don't need `MulUpperSS` / `MulUpperSU`.

**`__udivti3` body (fast/slow dispatch).** Compiler-builtins' `specialized_div_rem` is a polished Knuth Algorithm D implementation with CTLZ-based normalization, native `udiv i64` for the quotient digits, and dispatch on operand sizes. It compiles to ~800 PVM instructions in our pipeline. **Beating it from scratch is out of scope**: a naive binary long-division replacement would be ~3000 PVM instructions (worse on every dimension). The pragmatic win is the b_hi specialization:

```text
if (a_hi | b_hi) == 0:
    q   = a_lo / b_lo                ; native PVM DivU64
    sret = (q, 0)
    return
else:
    sp_old = __stack_pointer
    __stack_pointer = sp_old - 32    ; specialized_div_rem writes 32 bytes (q + r)
    call specialized_div_rem(sp_new, ...)
    copy quotient (16 bytes) to caller sret
    __stack_pointer = sp_old
    return
```

The slow path re-implements the original `__udivti3` wrapper verbatim — passing the caller's 16-byte `sret` directly to `specialized_div_rem` is unsafe because it writes 32 bytes (quotient + remainder).

**Measured dynamic gas impact** (microbenchmarks at 1000 iterations through anan-as, see `tests/fixtures/wat/u128-{mul,div}-bench*.jam.wat`):

| Operation | Recognition off | Recognition on | Δ Gas | Notes |
|-----------|-----------------|----------------|-------|-------|
| u128 mul | 119,029 | 75,029 | **−37%** | Body replacement, no dispatch |
| u128 div fast path (`a_hi = b_hi = 0`) | 129,029 | 76,029 | **−41%** | Native `DivU64` vs full `__udivti3 + specialized_div_rem` stub |
| u128 div slow path (`b_hi != 0`) | 129,029 | 143,029 | **+11%** | Dispatch overhead (Or + ICmp + Branch) |

**Measured static impact** (real substrate runtimes via `examples/polkadot/`, combined mul + div recognition vs `--no-libcall-recognition`):

| Runtime | `__multi3` calls | `__udivti3` calls | Δ PVM instr | Δ JAM bytes |
|---------|------------------|-------------------|-------------|-------------|
| glutton-kusama | 79 | small | -20 | -64 |
| asset-hub-kusama | 962 | 135 | -20 | -64 |

The `__multi3` body saves ~45 PVM instructions one-shot (it shrinks from ~30 to 8). The `__udivti3` body grows by ~25 PVM instructions (the original was a thin 20-instr wrapper; we now carry a fast path + slow path + dispatch). Net per-runtime is roughly **−20 instructions / −64 bytes** — static savings are minor in either direction. **The real win is dynamic gas** (microbench table above): the b_hi specialization fast path runs in ~5 PVM instructions instead of ~50 in the original. On workloads where most callers pass zero high halves (substrate's `Perbill::from_rational`, currency math fitting comfortably in u64), every `__udivti3` invocation pays a much smaller runtime cost.

**The slow-path regression is the cost of the dispatch.** For workloads dominated by full u128/u128 arithmetic, the 11% regression is real but bounded. In substrate, the pattern `(x: u64 as u128) / (y: u64 as u128)` is extremely common (`Perbill::from_rational`, currency arithmetic where balances comfortably fit in u64), so the fast path is expected to dominate. End-to-end runtime gas measurement requires running the chain, which is out of scope here — the microbench numbers above are the available signal.

**What we explicitly did *not* do.** Naive binary long division to replace `specialized_div_rem` entirely (loses ~2000 PVM instructions static, slow-path 3-4× worse). Newton-Raphson reciprocal or other algorithmic improvements (multi-week project for an uncertain win). Caller-side IR pattern matching to inline u64/u64 directly at call sites (fragile across LLVM passes, conflicts with our preference for body recognition). See `crates/wasm-pvm/src/llvm_frontend/libcall_recognition.rs` for the full design.

## Block Layout for Fallthrough Bias (with regalloc realignment)

The pre-existing `--no-fallthrough-jumps` flag elided trailing `Jump`s when the target happened to be the next block in `function.get_basic_blocks()` order. LLVM's IR order isn't picked with PVM fallthroughs in mind, so on glutton-kusama only 16,729 of 69,932 trailing branches actually fell through; the remaining 53,203 paid 5 bytes/Jump (~266 KB code) where 1 byte would do.

**`compute_block_layout(function)` in `llvm_backend/mod.rs`** chooses a per-function emission order via greedy trace from each unplaced IR block, following a "preferred successor" link per terminator:

- `br dest` → `dest`
- `br cond, then, else` → `else` (matches the trailing `Jump else_label` after `BranchIfX then_label`)
- `switch val, default, ...` → `default` (matches the trailing `Jump default_label`)
- `ret` / `unreachable` → none

Trampoline paths in `lower_br` / `lower_switch` (per-edge phi copies on both outgoing edges) emit a *different* final `Jump` target. Those blocks miss the fallthrough but stay correct.

**Critical wiring detail.** Regalloc must walk the same order the emitter does. `regalloc::run` accepts the layout as a `block_order: &[BasicBlock]` parameter; without that, live intervals were computed against IR order while the emitter executed in layout order, and downstream reads through `operand_reg` / `load_operand` picked up a register the linear scan thought still held a value but the layout had clobbered. The original symptom was the anan-as compiler's compiled-PVM interpretation losing its `r7`/`r8` mappings — the inner JAM ran fine in `Layer 3` (direct anan-as on Node) but halted with empty output under `Layer 4`/`Layer 5` PVM-in-PVM, because the *compiled* outer interpreter had the wrong live ranges. Realigning regalloc to layout order is what made `pvm-in-pvm: as-flat-ternary-test` green again.

The two pieces (block layout + jump elision) are coupled — the elision is meaningless without the layout choosing the right successor — so both sit behind the existing `OptimizationFlags::fallthrough_jumps` flag, default on.

## Phi-Copy Temp/Destination Aliasing (Pre-existing Latent Bug)

`emit_phi_copies_legacy` and `emit_phi_copies_regaware` in `control_flow.rs` use a temp pool when 2-5 phi copies fit it:

```rust
let temp_regs = [TEMP1, TEMP2, TEMP_RESULT, SCRATCH1, SCRATCH2];
```

The trap: in `llvm_backend/emitter.rs` the names `SCRATCH1` / `SCRATCH2` are **re-exported** as `ARGS_LEN_REG = r8` and `ARGS_PTR_REG = r7` — *not* the `r5` / `r6` from `crate::abi`. So `temp_regs == [r2, r3, r4, r8, r7]`. With `allocate_caller_saved_regs` (default on), `r7` and `r8` are also valid phi destinations.

When a phi copy at index `i` has `phi_reg == temp_regs[j]` for some `j != i`, the legacy "Phase 1: load all temps; Phase 2: write all destinations in `0..N` order" sequence corrupts itself: writing destination at step `i` clobbers `temp_regs[j]` before step `j` reads it. The clobbered value is silently substituted.

The `regalloc-two-loops` fixture exercised exactly this (5 phi copies, `local_5`'s phi_reg = `r7` = `temp_regs[4]`, `local_3`'s incoming value loaded into `r7`): `local_3` (the loop counter `i`) ended up holding `local_5`'s value (`b`), so the loop iterated against the wrong counter and returned the wrong sum. The test expectations were calibrated to the buggy output (72 / 154 / 328 / …) — native WASM gives (76 / 211 / 720 / 2851 / 58958 / 165809572) for n ∈ {0,1,2,3,5,10}.

**Fix in `topo_order_phase2`:** build a dependency edge `i → j` whenever `phi_regs[j] == temp_regs[i]` (`i != j`), then Kahn-sort to produce a Phase-2 emission order where every consumer of a temp is processed before any producer that overwrites it. Cycles (`phi_regs[3] = r7` AND `phi_regs[4] = r8`, etc.) drop to the slot-based `emit_phi_copies_via_slots` resolver. The temp pool and Phase-1 loads are unchanged; only Phase-2 ordering shifts.

## Cross-Block Snapshot Must Mirror Terminator-Clobber Set

The cache snapshot taken before lowering a block's terminator (`llvm_backend/mod.rs`) used to invalidate only `TEMP1` and `TEMP2`, because they're the operand-load temps for any branch/switch terminator. That was correct for branches without phi copies. But `emit_phi_copies_regaware` also uses `TEMP_RESULT` (`r4`) and the emitter-scope `SCRATCH1` / `SCRATCH2` (= `r8` / `r7`) as Phase-1 temps for the 3rd/4th/5th active copy. When a successor restored that stale snapshot, its `alloc_reg_slot` showed `r4` / `r7` / `r8` still owning whatever the predecessor's block-body had put there — but the phi-copy that ran in between had overwritten them. Downstream reads via `operand_reg` / `load_operand` took the fast path against `alloc_reg_slot` and returned the wrong value.

Fix: invalidate `TEMP1`, `TEMP2`, `TEMP_RESULT`, `SCRATCH1`, `SCRATCH2` in the snapshot — the full set of registers any terminator path may touch. This is a strict superset of what was invalidated before, so it can never make a successor read a fresher cache entry than is actually valid.

## Global Storage Width: Per-Type Slots, Not Uniform 8-Byte Widening

For most of the compiler's history each WASM global was stored in a fixed 4-byte slot at `0x30000 + (has_mem_size ? 4 : 0) + idx * 4`, and the lowering in `llvm_backend/memory.rs` emitted `LoadU32`/`StoreU32` for every `global.get` / `global.set`. That worked invisibly because:

- the WASM parser only matched `I32Const` in `eval_const_i32` (silently dropping `I64Const` initializers to 0);
- the LLVM frontend declared every global as LLVM `i64` regardless of the WASM-declared type;
- and `wasmparser::validate` enforced that any WASM operator consuming a global's value matched the global's declared type, so for `(global i32 ...)` the trailing i32 ops truncated whatever garbage was in the top 32 bits.

The combination silently corrupted `(global i64 ...)` values whose high 32 bits were non-zero — store dropped them, load zero-filled them, and no test fixture exercised i64 globals at all so the regression never surfaced.

**Rejected approach: uniform 8-byte widening.** The first cut of this fix simply widened every global slot to 8 bytes (`GLOBAL_SLOT_SIZE = 8`) and switched lowering to `LoadU64`/`StoreU64` unconditionally. That paid an i32-global-wide tax to fix a bug no current input triggers — every polkadot fellowship runtime (v2.2.2, 14 modules) has exactly 3 globals, all i32 (the standard Rust→wasm32 trio: stack pointer, `__data_end`, `__heap_base`). Rust→wasm32 effectively never emits i64 globals because pointers are 32-bit and most LLVM-managed globals live in linear memory. So uniform widening added 12 bytes of `rw_data` per polkadot runtime for zero observable benefit.

**Chosen approach: per-global widths.** Storage width matches the declared WASM type — 4 bytes for `i32`/`f32`, 8 bytes for `i64`/`f64`. Address resolution moves from a closed-form `idx * SLOT` formula to a precomputed `WasmModule::global_offsets: Vec<i32>` parallel to `globals`/`global_widths`. The LLVM frontend keeps its uniform `load i64`/`store i64` shape (unchanged from before this PR); the backend reads the per-global width from `ctx.global_widths[idx]` and selects the matching PVM opcode. Keeping the LLVM IR shape identical avoids LLVM-pass outcomes drifting for i32-only modules — an exploratory variant that issued `load i32`/`zext` and `trunc`/`store i32` regressed the anan-as PVM interpreter by ~2.5% (+2872 bytes) before being reverted.

**Implementation, layer by layer.**

1. `WasmModule::parse` only accepts `i32`/`i64` globals; `f32`/`f64`, `v128`, and ref-type globals all error out with `Error::Unsupported` at parse time. (An earlier draft tolerated `f32`/`f64` globals on the assumption that `--trap-floats` would catch reads, but `global.get`/`global.set` are lowered as plain integer loads/stores — `--trap-floats` only traps float *operators*, so a program could observe a zeroed float global via `i32.reinterpret_f32` or by forwarding the loaded i64 elsewhere. Rejecting up front avoids that footgun. No real workload uses float globals: all 14 polkadot fellowship runtimes have 3 i32 globals each and zero floats.)
2. `WasmModule` now carries `global_init_values: Vec<i64>`, `global_widths: Vec<u32>`, and `global_offsets: Vec<i32>`, all parallel to `globals`. `eval_const_global_init` accepts only a single `I32Const` / `I64Const` literal followed by `End`; multi-operator extended-const expressions (legal under wasmparser's default `EXTENDED_CONST` feature) and any other operator (`global.get` of an imported const, `ref.func`, `ref.null`) *error* — the previous pattern of silently returning `Ok(0)` for unsupported init-exprs (or only consuming the first operator of a multi-op chain) would have corrupted a program's initial state without any compile-time signal.
3. `memory_layout`: `globals_region_size`, `data_segment_length_offset`, `compute_param_overflow_base`, and `compute_wasm_memory_base` now take a `&[u32]` widths slice instead of `num_globals: usize`. New `compute_global_offsets(widths, has_mem_size)` precomputes absolute PVM addresses; new `global_storage_width(ValType)` returns 4 or 8 per type (gated on `feature = "compiler"` because it consumes `wasmparser::ValType`; the rest of `memory_layout` stays usable without the compiler toolchain). The old `global_addr(idx, has_mem_size)` closed-form helper is gone — callers index `WasmModule::global_offsets` directly.
4. `LoweringContext` gains `global_offsets: Vec<i32>` and `global_widths: Vec<u32>` (cloned from `WasmModule` at compile entry). The backend's two global-access lowerings (`lower_wasm_global_load`, `lower_wasm_global_store`) look up `ctx.global_offsets[idx]` for the address and `ctx.global_widths[idx]` for the width, then pick `LoadU32` vs `LoadU64` (and `StoreU32`/`StoreImmU32` vs `StoreU64`/`StoreImmU64`) per width. Width is *not* derived from the LLVM instruction's type — the LLVM IR is uniformly i64 and would be misleading.
5. The LLVM frontend (`function_builder.rs`) is **unchanged** from main: every global is declared as LLVM `i64`, and `global.get`/`global.set` issue `load i64`/`store i64` uniformly. The width-vs-LLVM-IR mismatch (LLVM IR claims to read/write 8 bytes from a 4-byte i32 slot) is invisible to LLVM (no pass observes raw storage widths) and resolved at the backend via `ctx.global_widths`. The "top 32 bits = 0" invariant holds because the frontend's i32 ops always zero-extend to i64 before pushing onto the operand stack.
6. `build_rw_data` takes the widths slice and writes the low `width` bytes of each `i64` init value into the appropriate slot, packed in declaration order. Returns `Result<Vec<u8>>` so layout-invariant violations (mismatched parallel arrays, unsupported widths > 8 B from a hypothetical bypassed parse guard) surface as `Error::Internal` rather than as a release-build slice panic — `debug_assert!` would have disappeared in release.

**Why i32 globals are unchanged for typical programs.** With per-global widths, an all-i32 module (every fixture, every polkadot runtime) sees byte-identical `globals_region_size`, `wasm_memory_base`, and `rw_data` layout as before this PR. The fix is invisible until someone actually compiles a module with `(global i64 ...)`.

**Verification.** `crates/wasm-pvm/tests/i64_globals.rs` (9 cases): (i) i64 `global.get` lowers to `LoadU64` (not `LoadU32`); (ii) i64 `global.set` with a small const lowers to `StoreImmU64`; (iii) i64 `global.set` with a >i32-range const lowers to `LoadImm64 + StoreU64`; (iv) i32 globals still lower to `LoadU32` / `StoreU32` (no 64-bit opcodes, no regression for the common case — split across two functions to defeat LLVM intra-function store→load forwarding); (v) mixed-width modules emit both i32 and i64 opcodes; (vi) v128 globals are rejected at parse; (vii) f32/f64 globals are rejected at parse; (viii) non-const-literal init expressions (e.g. `global.get` of an imported global) are rejected; (ix) extended-const init expressions (e.g. `i32.add` of two literals) are rejected. Plus two `build_rw_data` unit tests (`rejects_mismatched_parallel_arrays`, `rejects_unsupported_global_width`) covering the error-path replacements for the prior `debug_assert!`/slice panics. Full Rust + integration + PVM-in-PVM + differential suites stay green; benchmarks are byte-identical to main for every existing fixture (no fixture uses i64 globals).

---

## Value-Lifetime-Aware DSE + Stack-Slot Reuse — Nothing Shipped (2026-05)

A position-aware DSE extension (kill SP-relative stores whose offset is overwritten later in the same basic block with no intervening load) was hypothesized to unblock a stack-slot reuse pass for a combined ~10 % code-size win on polkadot runtimes. Measured **0.03 %** on glutton-kusama (4,636,361 → 4,634,900 B code; 6,444,121 → 6,442,477 B JAM) — two orders of magnitude below the hypothesis. Both the DSE rewrite and the slot-reuse port were reverted.

The new DSE alone is byte-identical to main: each SSA value currently owns a unique stack-slot offset, so the "two stores at the same offset, no intervening load" pattern doesn't arise within a basic block. The new pass is dormant.

Slot reuse + DSE saves 0.03 %, and the win comes from offset-encoding compression (shared offsets fit in fewer varint bytes), *not* store elimination: when V1 and V2 share offset X, the emitted sequence is `store V1@X; … load V1@X; store V2@X; … load V2@X`. The intervening `load V1@X` clears the kill-pending set before V2's overwrite, so V1's store stays. Pass 2b only fires for stores with no reload at all (lazy-spill flushes satisfied entirely by the register cache) — rare, and lazy spill already optimizes the common cases.

Slot reuse also *reduces* the original pass 1's kill rate: an SSA value held in a register with its slot otherwise unused has its store killed today (offset has no consumers); under slot reuse the offset is shared with a live value, so pass 1 keeps both stores. Pass 2b recovers most but not all.

Promising direction not pursued: attack the lazy-spill flush at the source — skip the just-in-case store at block exits when proven unreachable. Removing the store at the source also kills the matching reload.
