# WASM-PVM Architecture: Register & Calling Conventions

This document describes the ABI (Application Binary Interface) used by the WASM-to-PVM
recompiler, including register assignments, calling conventions, stack frame layout,
memory layout, and the SPI/JAM program format.

**Canonical source**: The constants behind this document live in
[`crates/wasm-pvm/src/abi.rs`](crates/wasm-pvm/src/abi.rs) and
[`crates/wasm-pvm/src/translate/memory_layout.rs`](crates/wasm-pvm/src/translate/memory_layout.rs).

---

## Register Assignments

PVM provides 13 general-purpose 64-bit registers (r0–r12). The compiler assigns
them as follows:

| Register | Alias | Purpose | Saved by |
|----------|-------|---------|----------|
| r0 | ra | Return address (jump table index) | Callee |
| r1 | sp | Stack pointer (grows downward) | Callee |
| r2 | t0 | Temp: load operand 1 / immediates | Caller |
| r3 | t1 | Temp: load operand 2 | Caller |
| r4 | t2 | Temp: ALU result | Caller |
| r5 | s0 | Scratch | Caller |
| r6 | s1 | Scratch | Caller |
| r7 | a0 | Return value / SPI `args_ptr` | Caller |
| r8 | a1 | SPI `args_len` / second result | Caller |
| r9 | l0 | Local 0 / param 0 | Callee |
| r10 | l1 | Local 1 / param 1 | Callee |
| r11 | l2 | Local 2 / param 2 | Callee |
| r12 | l3 | Local 3 / param 3 | Callee |

**Callee-saved** (r0, r1, r9–r12): the callee must preserve these across calls.
**Caller-saved** (r2–r8): the caller must assume these are clobbered by any call.

---

## Stack Frame Layout

Every function allocates a stack frame. The stack grows **downward** (SP decreases).

```text
                Higher addresses
          ┌─────────────────────────┐
          │   caller's frame ...    │
old SP →  ├─────────────────────────┤
          │  Saved r0  (ra)    +0   │  8 bytes
          │  Saved r9  (l0)    +8   │  8 bytes
          │  Saved r10 (l1)   +16   │  8 bytes
          │  Saved r11 (l2)   +24   │  8 bytes
          │  Saved r12 (l3)   +32   │  8 bytes
          ├ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┤  FRAME_HEADER_SIZE = 40
          │  SSA value slot 0  +40  │  8 bytes
          │  SSA value slot 1  +48  │  8 bytes
          │  ...                    │  8 bytes per SSA value
new SP →  ├─────────────────────────┤
          │  (operand spill area)   │  SP - 0x100 .. SP
          └─────────────────────────┘
                Lower addresses
```

**Frame size** = `FRAME_HEADER_SIZE (40) + num_ssa_values * 8`

The operand spill area at `SP + OPERAND_SPILL_BASE` (i.e. `SP - 0x100`) is used for
temporary storage during phi-node copies and indirect calls. The frame grows *upward*
from SP (toward higher addresses), while the spill area is *below* SP, so the two
regions never overlap regardless of frame size. However, a callee's frame allocation
must not reach into the caller's spill area — this is protected by the stack overflow
check which ensures `SP - frame_size >= stack_limit`.

### Stack-Slot Approach

Every LLVM SSA value gets a dedicated 8-byte stack slot (no register allocation).
The typical instruction sequence is:

1. Load operands from stack slots into temp registers (t0, t1)
2. Execute ALU operation, result in t2
3. Store t2 back to the result's stack slot

This is a correctness-first design; a proper register allocator is future work.

### Per-Block Register Cache (Store-Load Forwarding)

`PvmEmitter` maintains a per-basic-block register cache (`slot_cache: HashMap<i32, u8>`,
`reg_to_slot: [Option<i32>; 13]`) that tracks which stack slot values are currently live
in registers. This eliminates redundant `LoadIndU64` instructions:

- **Cache hit, same register**: Skip entirely (0 instructions emitted)
- **Cache hit, different register**: Emit `AddImm64 dst, cached_reg, 0` (register copy)
- **Cache miss**: Emit normal `LoadIndU64`, then record in cache

The cache is **invalidated**:
- When a register is overwritten (auto-detected via `Instruction::dest_reg()`)
- At **block boundaries** (`define_label()` clears the entire cache)
- After **function calls** (`clear_reg_cache()` after `Fallthrough` return points)
- After **ecalli** host calls (`clear_reg_cache()` after `Ecalli`)

Impact: ~50% gas reduction, ~15-40% code size reduction across benchmarks.

---

## Calling Convention

### Parameter Passing

| Parameter | Location |
|-----------|----------|
| 1st–4th | r9–r12 |
| 5th+ | `PARAM_OVERFLOW_BASE` (`0x32000 + (i-4)*8`) in global memory |

Return value: **r7** (single i64).

### Caller Sequence

```text
1. Load arguments into r9–r12 (first 4)
2. Store overflow arguments to PARAM_OVERFLOW_BASE
3. LoadImm64  r0, <return_jump_table_index>
4. Jump       <callee_code_offset>
   ── callee executes ──
5. (fallthrough) Store r7 to result slot if function returns a value
```

### Callee Prologue

```asm
1. Stack overflow check (skipped for entry function):
     LoadImm64  t1, stack_limit        ; unsigned comparison!
     AddImm64   t2, sp, -frame_size
     BranchGeU  t1, t2, continue
     Trap                              ; stack overflow → panic
2. Allocate frame:
     AddImm64   sp, sp, -frame_size
3. Save callee-saved registers:
     StoreIndU64  [sp+0],  r0
     StoreIndU64  [sp+8],  r9
     StoreIndU64  [sp+16], r10
     StoreIndU64  [sp+24], r11
     StoreIndU64  [sp+32], r12
4. Copy parameters to SSA value slots:
     - First 4 from r9–r12
     - 5th+ loaded from PARAM_OVERFLOW_BASE
```

### Callee Epilogue (return)

```asm
1. Load return value into r7 (if returning a value)
2. Restore callee-saved registers:
     LoadIndU64  r9,  [sp+8]
     LoadIndU64  r10, [sp+16]
     LoadIndU64  r11, [sp+24]
     LoadIndU64  r12, [sp+32]
3. Restore return address:
     LoadIndU64  r0, [sp+0]
4. Deallocate frame:
     AddImm64   sp, sp, +frame_size
5. Return:
     JumpInd    r0, 0
```

---

## Jump Table & Return Addresses

PVM's `JUMP_IND` instruction uses a **jump table** — it is not a direct address jump:

```text
JUMP_IND rA, offset
  target_address = jumpTable[(rA + offset) / 2 - 1]
```

Return addresses stored in r0 are therefore **jump-table indices**, not code offsets:

```text
r0 = (jump_table_index + 1) * 2
```

The jump table is laid out as:

```text
[ return_addr_0, return_addr_1, ...,   // for call return sites
  func_0_entry,  func_1_entry,  ... ]  // for indirect calls
```

Each entry is a 4-byte code offset (u32). Jump table entries for `call_indirect`
encode function entry points used by the dispatch table.

---

## Indirect Calls (`call_indirect`)

A **dispatch table** at `RO_DATA_BASE` (`0x10000`) maps WASM table indices to
function entry points:

```text
Dispatch table entry (8 bytes each):
  [0–3]  Jump address (u32, byte offset → jump table index)
  [4–7]  Type signature index (u32)
```

The indirect call sequence:

```asm
 1. Compute dispatch_addr = RO_DATA_BASE + table_index * 8
 2. Load type_idx from [dispatch_addr + 4]
 3. Compare type_idx with expected_type_idx
 4. Trap if mismatch (signature validation)
 5. Load jump_addr from [dispatch_addr + 0]
 6. LoadImm64  r0, <return_jump_table_index>
 7. JumpInd    jump_addr, 0
```

---

## Import Calls

### `host_call(ecalli_index, r7, r8, r9, r10, r11)` → `ecalli`

The first argument must be a compile-time constant (the ecalli index). The remaining
arguments (up to 5) are loaded into r7–r11, and the `Ecalli <index>` instruction
is emitted. The result is returned in r7.

### `pvm_ptr(wasm_addr) -> pvm_addr`

Converts a WASM-space address to a PVM-space address by zero-extending to 64 bits
and adding `wasm_memory_base`.

### Other imports

All other imported functions emit `Trap`. If the import signature has a return
value, a dummy zero is pushed (dead code after the trap) to keep the stack
consistent.

---

## Entry Function (SPI Convention)

The entry function is special — it follows SPI conventions rather than the normal
calling convention.

**Initial register state** (set by the PVM runtime):

| Register | Value | Purpose |
|----------|-------|---------|
| r0 | `0xFFFF0000` | EXIT address — jump here to HALT |
| r1 | `0xFEFE0000` | Stack pointer (`STACK_SEGMENT_END`) |
| r7 | `0xFEFF0000` | Arguments pointer (PVM address) |
| r8 | `args.length` | Arguments length in bytes |
| r2–r6, r9–r12 | 0 | Available |

**Entry prologue differences** from a normal function:

1. **No stack overflow check** (main function starts with full stack)
2. Allocates frame and stores SSA slots
3. **No callee-saved register saves** (no caller to return to)
4. **Adjusts args_ptr**: `r7 = r7 - wasm_memory_base` (convert PVM address to WASM address)
5. Stores r7 and r8 to parameter slots

**Entry return** — three conventions are supported:

| Convention | r7 (out) | r8 (out) | When used |
|------------|----------|----------|-----------|
| Globals | `global[ptr_idx] + wasm_memory_base` | `r7 + global[len_idx]` | Legacy AS convention |
| Packed (ptr, len) | `(ret & 0xFFFFFFFF) + wasm_memory_base` | `r7 + (ret >> 32)` | Multi-value return |
| Simple | return value directly | unchanged | Raw i32/i64 return |

All three end by jumping to `EXIT_ADDRESS` (`0xFFFF0000`).

### Start Function

If a WASM start function exists, the entry function calls it before processing
arguments. r7/r8 are saved to the stack, the start function is called (no arguments),
then r7/r8 are restored.

---

## Memory Layout

```text
PVM Address Space:
  0x00000 - 0x0FFFF   Reserved / guard (fault on access)
  0x10000 - 0x1FFFF   Read-only data (RO_DATA_BASE) — dispatch tables
  0x20000 - 0x2FFFF   Gap zone (unmapped, guard between RO and RW)
  0x30000 - 0x31FFF   Globals (GLOBAL_MEMORY_BASE, 8KB)
  0x32000 - 0x320FF   Parameter overflow area (5th+ function arguments)
  0x32100+            Spilled locals (per-function metadata, typically unused)
  0x33000+             WASM linear memory (4KB-aligned, computed dynamically via `compute_wasm_memory_base`)
  ...                  (unmapped gap until stack)
  0xFEFE0000           STACK_SEGMENT_END (initial SP)
  0xFEFF0000           Arguments segment (input data, read-only)
  0xFFFF0000           EXIT_ADDRESS (jump here → HALT)
```

**Key formulas** (see `memory_layout.rs`):

- Global address: `0x30000 + global_index * 4`
- Memory size global: `0x30000 + num_globals * 4`
- Spilled local: `0x32100 + func_idx * SPILLED_LOCALS_PER_FUNC + local_offset`
- WASM memory base: `align_up(max(SPILLED_LOCALS_BASE + num_funcs * SPILLED_LOCALS_PER_FUNC, GLOBAL_MEMORY_BASE + globals_region_size(num_globals, num_passive_segments)), 4KB)` — the heap starts immediately after the globals/passive-length region, aligned to PVM page size (4KB). This is typically `0x33000` for programs with few globals.
- Stack limit: `0xFEFE0000 - stack_size`

---

## SPI/JAM Program Format

The compiled output is a JAM file in the SPI (Standard Program Interface) format:

```text
Offset  Size    Field
──────  ──────  ─────────────────────
0       3       ro_data_len (u24 LE)
3       3       rw_data_len (u24 LE)
6       2       heap_pages  (u16 LE)
8       3       stack_size  (u24 LE)
11      N       ro_data     (dispatch table)
11+N    M       rw_data     (globals + WASM memory initial data)
11+N+M  4       code_len    (u32 LE)
15+N+M  K       code        (PVM program blob)
```

**`heap_pages`** is computed from the WASM module's `initial_pages` (not `max_pages`).
It represents the number of 4KB PVM pages pre-allocated as zero-initialized writable memory
at program start. Additional memory beyond this is allocated on demand via `sbrk`/`memory.grow`.
Programs declaring `(memory 0)` get a minimum of 16 WASM pages (1MB) to accommodate
AssemblyScript runtime memory accesses.

### PVM Code Blob

Inside the `code` section, the PVM blob format is:

```text
- jump_table_len  (varint u32)
- item_len        (u8, always 4)
- code_len        (varint u32)
- jump_table      (4 bytes per entry, code offsets)
- instructions    (PVM bytecode)
- mask            (bit-packed instruction start markers)
```

### Entry Header

The first 10 bytes of code are the entry header:

```text
[0–4]   Jump  <main_function_offset>        (5 bytes)
[5–9]   Jump  <secondary_entry_offset>      (5 bytes, or Trap + padding)
```

The secondary entry is for future use (e.g. is_authorized). If unused, it emits
`Trap` followed by 4 `Fallthrough` instructions as padding.

---

## Phi Node Handling

Phi nodes (SSA merge points) use a two-pass approach to avoid clobbering:

1. **Load pass**: Load all incoming phi values into temp registers (t0, t1, t2, s0, s1)
2. **Store pass**: Store all temps to their destination phi result slots

This supports up to 5 simultaneous phi values. The two-pass design prevents cycles
where storing one phi value would overwrite a source needed by another phi.

---

## Design Trade-offs

| Decision | Rationale |
|----------|-----------|
| Stack-slot for every SSA value | Correctness-first; no register allocator needed. Per-block register cache mitigates the cost |
| Spill area below SP | Frame grows up from SP, spill area grows down — no overlap |
| Global `PARAM_OVERFLOW_BASE` | Avoids stack frame complexity for overflow params |
| Jump-table indices as return addresses | Required by PVM's `JUMP_IND` semantics |
| Entry function has no stack check | Starts with full stack, nothing to overflow into |
| Unsigned stack limit comparison | `LoadImm64` avoids sign-extension bugs with large addresses |
| `unsafe` forbidden | Workspace-level `deny(unsafe_code)` lint |

---

## References

- [`abi.rs`](crates/wasm-pvm/src/abi.rs) — Register and frame constants
- [`memory_layout.rs`](crates/wasm-pvm/src/translate/memory_layout.rs) — Memory address constants
- [`emitter.rs`](crates/wasm-pvm/src/llvm_backend/emitter.rs) — PvmEmitter and value management
- [`calls.rs`](crates/wasm-pvm/src/llvm_backend/calls.rs) — Calling convention implementation
- [`control_flow.rs`](crates/wasm-pvm/src/llvm_backend/control_flow.rs) — Prologue/epilogue/return
- [`spi.rs`](crates/wasm-pvm/src/spi.rs) — JAM/SPI format encoder
- [LEARNINGS.md](LEARNINGS.md) — Technical reference and debugging journal
- [gp-0.7.2.md](gp-0.7.2.md) — Gray Paper (JAM/PVM specification)
