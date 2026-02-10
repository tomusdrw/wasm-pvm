# WASM to PVM Recompiler - Technical Reference

**Date**: 2026-02-10

Technical learnings, PVM architecture details, and conventions used in the compiler.

---

## PVM (Polka Virtual Machine) Overview

### From Gray Paper v0.7.2

**Core Characteristics:**
- Based on RISC-V rv64em variant (64-bit, embedded, with multiplication)
- 13 general-purpose 64-bit registers (indexed 0-12)
- Gas-metered execution
- Memory organized in pages (PAGE_SIZE = 4KB, SEGMENT_SIZE = 64KB)
- Little-endian byte order

**Exit Conditions:**
- halt — Normal termination
- panic — Error/trap
- out-of-gas — Gas exhausted
- page-fault — Memory access violation
- host-call — External function call

---

## Register Convention

| Register | Usage |
|----------|-------|
| r0 | Return address (jump table index) |
| r1 | Stack pointer |
| r2-r6 | Scratch registers |
| r7 | Return value from calls / SPI args pointer (in main) |
| r8 | SPI args length (in main) |
| r9-r12 | Local variables (first 4) / callee-saved across calls |

---

## Calling Convention

**Before call (caller-side):**
1. Calculate `new_sp = sp - frame_size`
2. Stack overflow check: `new_sp >= stack_limit` (unsigned comparison)
3. Save return address (r0) to `[sp+0]`
4. Save locals r9-r12 to `[sp+8..40]`
5. Save any additional state to stack
6. Place arguments in r9+ (first 4 args) and PARAM_OVERFLOW_BASE (5th+)
7. Load return address (jump table index) into r0
8. Jump to callee entry point

**After return (caller-side):**
1. Restore return address (r0) from `[sp+0]`
2. Restore locals r9-r12 from `[sp+8..40]`
3. Restore additional state from stack
4. Increment stack pointer by frame size
5. Copy return value from r7

**Stack overflow detection:**
- Default stack size: 64KB (configurable in SPI format up to 16MB)
- Stack grows downward from `0xFEFE0000`
- With ~40-byte frames, overflow occurs at ~1600 recursion depth

---

## Jump Table Mechanism

PVM's `JUMP_IND` instruction uses a **jump table** lookup, NOT direct address jumping:

```
JUMP_IND rA, offset
  target_address = jumpTable[(rA + offset) / 2 - 1]
  jump to target_address
```

- Value `2` refers to `jumpTable[0]`
- Value `4` refers to `jumpTable[1]`
- Value `2*(N+1)` refers to `jumpTable[N]`
- Value `0xFFFF0000` (EXIT address) is special-cased for program termination

---

## PVM Instruction Encoding

### ThreeReg Instructions
For ThreeReg instructions (ADD_32, SUB_32, etc.):
- Encoding: `[opcode, src1<<4 | src2, dst]`
- **Execution: reg[c] = op(reg[b], reg[a])** — Note the swap!

This means for `SET_LT_U`, it computes `dst = (src2 < src1)`, NOT `dst = (src1 < src2)`.

### TwoRegOneImm Encoding
High nibble (args.a) is typically the SOURCE, low nibble (args.b) is the DESTINATION:

```
Byte layout: [opcode] [src << 4 | dst] [imm...]

Example ADD_IMM_32: regs[dst] = regs[src] + imm
Example LOAD_IND_U32: regs[dst] = memory[regs[base] + offset]
Example STORE_IND_U32: memory[regs[base] + offset] = regs[src]
```

---

## SPI (Standard Program Interface) Format

**Binary Format:**
```
+------------------------------------------+
| roLength           (u24 - 3 bytes)       |
| rwLength           (u24 - 3 bytes)       |
| heapPages          (u16 - 2 bytes)       |
| stackSize          (u24 - 3 bytes)       |
| roData             (roLength bytes)      |
| rwData             (rwLength bytes)      |
| codeLength         (u32 - 4 bytes)       |
| code               (PVM program blob)    |
+------------------------------------------+
```

**Initial Register State:**
| Register | Value | Purpose |
|----------|-------|---------|
| r0 | 0xFFFF_0000 | EXIT address — jump here to HALT |
| r1 | 0xFEFE_0000 | Stack pointer (STACK_SEGMENT_END) |
| r2-r6 | 0 | Available for computation |
| r7 | 0xFEFF_0000 | Arguments pointer (IN) / Result address (OUT) |
| r8 | args.length | Arguments length (IN) / Result length (OUT) |
| r9-r12 | 0 | Available for parameters/locals |

**Program Termination:**
- HALT: `LOAD_IMM r2, -65536; JUMP_IND r2, 0` → jumps to 0xFFFF0000 → status=HALT
- Note: Don't rely on r0 containing EXIT — hardcode 0xFFFF0000 (= -65536 as i32)
- PANIC: `TRAP` instruction → status=PANIC

---

## Memory Layout

```
  Address          Region                    Access
 ----------------------------------------------------------
 0x0000_0000  +-------------------------+
              |   Reserved / Guard      |   None    (64 KB)
 0x0001_0000  +-------------------------+
              |   Read-Only Data (RO)   |   Read
 0x0002_0000+ +-------------------------+
              |   Read-Write Data (RW)  |   Write
              + - - - - - - - - - - - - +
              |   Heap (Zero-init)      |   Write
              +-------------------------+
              |                         |
              |    Unmapped / Guard     |   None
              |                         |
 stackStart   +-------------------------+
              |        Stack            |   Write   (grows down)
 0xFEFE_0000  +-------------------------+  (STACK_SEGMENT_END)
              |   Guard (64 KB)         |   None
 0xFEFF_0000  +-------------------------+  (ARGS_SEGMENT_START)
              |   Arguments (RO)        |   Read    (up to 16 MB)
              +-------------------------+
              |   Guard (64 KB)         |   None
 0xFFFF_FFFF  +-------------------------+
```

### Compiler Memory Regions

| Address | Purpose |
|---------|---------|
| `0x10000` | Read-only data (dispatch table for `call_indirect`) |
| `0x30000` | Globals storage |
| `0x30100` | User heap (result storage) |
| `0x40000` | Spilled locals (512 bytes per function) |
| `0x50000+` | WASM linear memory base (data sections placed here) |

Spilled local address: `0x40000 + (func_idx * 512) + ((local_idx - 4) * 8)`

---

## Indirect Calls (`call_indirect`)

### Dispatch Table (RO Memory)
At `0x10000`, dispatch table entries are 8 bytes:
```
[0..3]  jump address (u32)
[4..7]  type index (u32)
```

At runtime, `call_indirect` loads the type index from offset 4 and traps if it doesn't match the expected type.

### Jump Table Extension
```
jumpTable = [ret_addr_0, ret_addr_1, ..., func_offset_0, func_offset_1, ...]
              ^- for return from calls    ^- for indirect calls (func_entry_base)
```

---

## Division Edge Cases

PVM follows RISC-V semantics for division (returns specific values instead of trapping). WASM requires traps for:
- **Division by zero**: All div/rem operations
- **Signed overflow**: `INT_MIN / -1` for `i32.div_s`

The compiler currently relies on PVM hardware behavior for these edge cases (trap sequences planned).

For i64 signed overflow: `i64::MIN` doesn't fit in a 32-bit immediate, so use `LoadImm64 + Xor` approach with fast path.

---

## AssemblyScript Note

AssemblyScript applies `& 0xFF` mask to the result of u8 arithmetic, even when assigning to a u32 variable:

```typescript
// BUG: 128 + 159 = 287 & 0xFF = 31
result = arr[0] + arr[1];

// FIX: cast to u32 first
result = <u32>arr[0] + <u32>arr[1];  // = 287
```

When summing `Uint8Array` elements where the result may exceed 255, always cast to u32/i32 first.

---

## References

- [Gray Paper v0.7.2](./gp-0.7.2.md) — JAM protocol specification
- [Ananas PVM](./vendor/anan-as) — PVM reference implementation (submodule)
- [WebAssembly Spec](https://webassembly.github.io/spec/) — WASM specification
