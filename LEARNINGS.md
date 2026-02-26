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

### PVM Memory Layout Optimization

- **Globals only occupy the bytes they actually need**: the compiler now tracks `globals_region_size = num_globals + 1 + num_passive_segments` and places the heap immediately after that region instead of reserving a full 64KB block. This keeps the RW data blob limited to real globals/passive-length fields plus active data segments.
- **Dynamic heap base calculation**: `compute_wasm_memory_base(num_funcs, num_globals, num_passive_segments)` compares the spill area (`SPILLED_LOCALS_BASE + num_funcs * SPILLED_LOCALS_PER_FUNC`) with the globals region end (`GLOBAL_MEMORY_BASE + globals_region_size(...)`) before rounding up to the next 4KB (PVM page) boundary. This typically gives `0x33000` instead of the old `0x40000`, saving ~52KB per program.
- **4KB alignment is sufficient**: The SPI spec only requires page-aligned (4KB) `rw_data` length. The 64KB WASM page size governs `memory.grow` granularity, not the base address. The anan-as interpreter uses `alignToPageSize(rwLength)` (4KB) not segment alignment for the heap zeros start.

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

---

## Specialized PVM Instructions for Common Patterns

### Absolute Address Load/Store (LoadU32/StoreU32)

- `LoadU32 { dst, address }` replaces `LoadImm { reg, value: addr } + LoadIndU32 { dst, base: reg, offset: 0 }` for known-address loads (globals)
- `StoreU32 { src, address }` similarly replaces the store pattern
- OneRegOneImm encoding: `[opcode, reg & 0x0F, encode_imm(address)...]`
- **PVM-in-PVM layout sensitivity**: Replacing multi-instruction sequences with single instructions changes bytecode layout (code size, jump offsets). The anan-as PVM interpreter has a pre-existing bug triggered by specific bytecode layouts. This means some LoadU32/StoreU32 optimizations can cause PVM-in-PVM test failures even though direct execution is correct. Empirically: LoadU32 for global loads is safe; StoreU32 for global stores, LoadU32 for memory_size, and StoreU32 for memory_grow can trigger failures. Test each change with full PVM-in-PVM suite (273 tests).
- Current status: Only `LoadU32` for `lower_wasm_global_load` is enabled. Other absolute address optimizations are deferred pending anan-as interpreter fix.

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

- Allocates long-lived SSA values (>1 use, spanning multiple blocks/loops) to available callee-saved registers (r9-r12) beyond parameter count.
- Operates on LLVM IR before PVM lowering; produces `ValKey` → physical register mapping
- `load_operand` checks regalloc before slot lookup: uses `MoveReg` from allocated reg instead of `LoadIndU64` from stack
- `store_to_slot` uses write-through: copies to allocated reg AND stores to stack; DSE removes the stack store if never loaded
- r5/r6 are excluded from global allocation because they are heavily reused as scratch in lowering paths
- Clobbered allocated scratch regs (when present) are handled with lazy invalidation/reload instead of eager spill+reload
- Values with ≤1 use are skipped (not worth a register)
- Loop extension: back-edges detected by successor having lower block index; live ranges extended to cover the back-edge source
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
  - Conservative non-leaf filter currently helps avoid large regressions: skip values defined inside loop bodies and require at least 3 uses before considering allocation.
  - Additional non-leaf gates that reduced remaining regressions:
    - Skip regalloc when fewer than 2 non-leaf allocatable callee registers are available (1-register allocation tended to thrash on AS decoder/array workloads).
    - Skip very small non-leaf functions (`total_values < 24`) where move/reload overhead often dominates.
- Post-fix benchmark shape: consistent JAM size reductions from regalloc, but gas/time gains are workload-dependent and often near-noise on current microbenchmarks.

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
