# WASM to PVM Recompiler - Learnings & Knowledge Base

This document captures technical learnings, design decisions, and discoveries made during the development of the WASM to PVM recompiler.

---

## PVM (Polka Virtual Machine) Overview

### From Gray Paper v0.7.2

**Core Characteristics:**
- Based on RISC-V rv64em variant (64-bit, embedded, with multiplication)
- 13 general-purpose 64-bit registers (ω ∈ ⟦ N_R ⟧^13, indexed 0-12)
- Gas-metered execution (N_G = N_{2^64})
- Memory organized in pages (PAGE_SIZE = 4KB, SEGMENT_SIZE = 64KB)
- Little-endian byte order

**Exit Conditions:**
- `∎` (halt) - Normal termination
- `☇` (panic) - Error/trap
- `∞` (out-of-gas) - Gas exhausted
- `F × address` (page-fault) - Memory access violation
- `h̵ × id` (host-call) - External function call

---

## PVM Program Blob Format

Source: `vendor/anan-as/assembly/program.ts`

```
┌─────────────────────────────────────────┐
│ jumpTableLength    (varU32)             │
├─────────────────────────────────────────┤
│ jumpTableItemBytes (u8)                 │
├─────────────────────────────────────────┤
│ codeLength         (varU32)             │
├─────────────────────────────────────────┤
│ jumpTable          (jumpTableLength *   │
│                     jumpTableItemBytes) │
├─────────────────────────────────────────┤
│ code               (codeLength bytes)   │
├─────────────────────────────────────────┤
│ mask               ((codeLength+7)/8)   │
│                    (bit-packed, 1=opcode)│
└─────────────────────────────────────────┘
```

**Mask Encoding:**
- 1 bit per code byte
- `1` = instruction opcode (start of instruction)
- `0` = argument byte (part of previous instruction)
- Packed LSB-first in each byte

---

## SPI (Standard Program Interface) Format

Source: `vendor/anan-as/assembly/spi.ts`

**Binary Format:**
```
┌─────────────────────────────────────────┐
│ roLength           (u24 - 3 bytes)      │
├─────────────────────────────────────────┤
│ rwLength           (u24 - 3 bytes)      │
├─────────────────────────────────────────┤
│ heapPages          (u16 - 2 bytes)      │
├─────────────────────────────────────────┤
│ stackSize          (u24 - 3 bytes)      │
├─────────────────────────────────────────┤
│ roData             (roLength bytes)     │
├─────────────────────────────────────────┤
│ rwData             (rwLength bytes)     │
├─────────────────────────────────────────┤
│ codeLength         (u32 - 4 bytes)      │
├─────────────────────────────────────────┤
│ code               (PVM program blob)   │
└─────────────────────────────────────────┘
```

**Memory Layout (32-bit address space):**
```
  Address          Region                    Access
 ─────────────────────────────────────────────────────
 0x0000_0000  ┌─────────────────────────┐
              │   Reserved / Guard      │   None    (64 KB)
 0x0001_0000  ├─────────────────────────┤
              │   Read-Only Data (RO)   │   Read
 0x0002_0000+ ├─────────────────────────┤
              │   Read-Write Data (RW)  │   Write
              ├ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ┤
              │   Heap (Zero-init)      │   Write
              ├─────────────────────────┤
              │         ░░░░░░░         │
              │    Unmapped / Guard     │   None
              │         ░░░░░░░         │
 stackStart   ├─────────────────────────┤
              │        Stack            │   Write   (grows ↓)
 0xFEFE_0000  ├─────────────────────────┤  (STACK_SEGMENT_END)
              │   Guard (64 KB)         │   None
 0xFEFF_0000  ├─────────────────────────┤  (ARGS_SEGMENT_START)
              │   Arguments (RO)        │   Read    (up to 16 MB)
              ├─────────────────────────┤
              │   Guard (64 KB)         │   None
 0xFFFF_FFFF  └─────────────────────────┘
```

**Initial Register State (SPI):**
| Register | Value | Purpose |
|----------|-------|---------|
| r0 | 0xFFFF_0000 | EXIT address - jump here to HALT |
| r1 | STACK_SEGMENT_END (0xFEFE_0000) | Stack pointer |
| r2-r6 | 0 | Available for computation |
| r7 | ARGS_SEGMENT_START (0xFEFF_0000) | Arguments pointer (IN) / Result address (OUT) |
| r8 | args.length | Arguments length (IN) / Result length (OUT) |
| r9-r12 | 0 | Available for parameters/locals |

**Program Termination:**
- HALT: `LOAD_IMM r2, -65536; JUMP_IND r2, 0` → jumps to 0xFFFF0000 → status=HALT
- Note: Don't rely on r0 containing EXIT - hardcode 0xFFFF0000 (= -65536 as i32)
- PANIC: `TRAP` instruction → status=PANIC

**WASM-to-PVM Entrypoint Convention:**
```wat
(module
  (memory 1)
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    ;; args_ptr = r7 (PVM address 0xFEFF0000)
    ;; args_len = r8
    ;; Read args with i32.load (direct PVM memory access)
    ;; Write results to heap (0x20100+)
    ;; Set $result_ptr and $result_len globals
  )
)
```

**Memory Layout for WASM Programs:**
```
0x20000 - 0x200FF: Globals storage (compiler-managed)
0x20100+:          User heap (for result data, allocations)
```

**Compiler Epilogue:**
1. Read `$result_ptr` global → r7
2. Read `$result_len` global → r8
3. `LOAD_IMM r2, 0xFFFF0000` (hardcoded EXIT)
4. `JUMP_IND r2, 0` → HALT

---

## PVM Instruction Set

Source: `vendor/anan-as/assembly/instructions.ts`

### Argument Types
| Type | Description | Byte Layout |
|------|-------------|-------------|
| Zero | No arguments | - |
| OneImm | 1 immediate (up to 4 bytes) | `[imm...]` |
| TwoImm | 2 immediates | `[split_nibble, imm1..., imm2...]` |
| OneOff | 1 offset (jump target) | `[off...]` |
| OneRegOneImm | 1 register + 1 immediate | `[reg_nibble, imm...]` |
| OneRegOneExtImm | 1 register + 8-byte immediate | `[reg_nibble, imm_lo_4, imm_hi_4]` |
| OneRegTwoImm | 1 register + 2 immediates | `[reg_split_nibble, imm1..., imm2...]` |
| OneRegOneImmOneOff | 1 register + 1 imm + 1 offset | `[reg_split_nibble, imm..., off...]` |
| TwoReg | 2 registers | `[hi_reg << 4 | lo_reg]` |
| TwoRegOneImm | 2 registers + 1 immediate | `[regs_nibbles, imm...]` |
| TwoRegOneOff | 2 registers + 1 offset | `[regs_nibbles, off...]` |
| TwoRegTwoImm | 2 registers + 2 immediates | `[regs_nibbles, split_nibble, imm1..., imm2...]` |
| ThreeReg | 3 registers | `[reg1 << 4 | reg2, reg3_nibble]` |

### TwoRegOneImm Encoding Details
**Critical:** High nibble (args.a) is typically the SOURCE, low nibble (args.b) is the DESTINATION.

```
Byte layout: [opcode] [src << 4 | dst] [imm...]

Example ADD_IMM_32: regs[dst] = regs[src] + imm
  Encoding: [131] [src << 4 | dst] [imm_bytes...]

Example LOAD_IND_U32: regs[dst] = memory[regs[base] + offset]  
  Encoding: [128] [base << 4 | dst] [offset_bytes...]

Example STORE_IND_U32: memory[regs[base] + offset] = regs[src]
  Encoding: [122] [base << 4 | src] [offset_bytes...]
```

### Complete Opcode Table
```
000 TRAP              Zero           - Panic
001 FALLTHROUGH       Zero           - Basic block end (no-op)

010 ECALLI            OneImm         - Host call

020 LOAD_IMM_64       OneRegOneExtImm - Load 64-bit immediate

030 STORE_IMM_U8      TwoImm         - Store immediate byte
031 STORE_IMM_U16     TwoImm
032 STORE_IMM_U32     TwoImm
033 STORE_IMM_U64     TwoImm

040 JUMP              OneOff         - Unconditional jump

050 JUMP_IND          OneRegOneImm   - Indirect jump
051 LOAD_IMM          OneRegOneImm   - Load 32-bit immediate (sign-extended)
052 LOAD_U8           OneRegOneImm   - Load from memory
053 LOAD_I8           OneRegOneImm
054 LOAD_U16          OneRegOneImm
055 LOAD_I16          OneRegOneImm
056 LOAD_U32          OneRegOneImm
057 LOAD_I32          OneRegOneImm
058 LOAD_U64          OneRegOneImm
059 STORE_U8          OneRegOneImm   - Store to memory

060 STORE_U16         OneRegOneImm
061 STORE_U32         OneRegOneImm
062 STORE_U64         OneRegOneImm

070 STORE_IMM_IND_U8  OneRegTwoImm   - Store immediate indirect
071 STORE_IMM_IND_U16 OneRegTwoImm
072 STORE_IMM_IND_U32 OneRegTwoImm
073 STORE_IMM_IND_U64 OneRegTwoImm

080 LOAD_IMM_JUMP     OneRegOneImmOneOff - Load immediate and jump
081 BRANCH_EQ_IMM     OneRegOneImmOneOff - Branch if equal to immediate
082 BRANCH_NE_IMM     OneRegOneImmOneOff
083 BRANCH_LT_U_IMM   OneRegOneImmOneOff
084 BRANCH_LE_U_IMM   OneRegOneImmOneOff
085 BRANCH_GE_U_IMM   OneRegOneImmOneOff
086 BRANCH_GT_U_IMM   OneRegOneImmOneOff
087 BRANCH_LT_S_IMM   OneRegOneImmOneOff
088 BRANCH_LE_S_IMM   OneRegOneImmOneOff
089 BRANCH_GE_S_IMM   OneRegOneImmOneOff

090 BRANCH_GT_S_IMM   OneRegOneImmOneOff

100 MOVE_REG          TwoReg         - Copy register
101 SBRK              TwoReg         - Memory allocation
102 COUNT_SET_BITS_64 TwoReg         - popcnt
103 COUNT_SET_BITS_32 TwoReg
104 LEADING_ZERO_BITS_64 TwoReg      - clz
105 LEADING_ZERO_BITS_32 TwoReg
106 TRAILING_ZERO_BITS_64 TwoReg     - ctz
107 TRAILING_ZERO_BITS_32 TwoReg
108 SIGN_EXTEND_8     TwoReg
109 SIGN_EXTEND_16    TwoReg

110 ZERO_EXTEND_16    TwoReg
111 REVERSE_BYTES     TwoReg

120-130 STORE_IND/LOAD_IND variants (TwoRegOneImm)
131-161 Immediate arithmetic (TwoRegOneImm)
        ADD_IMM, AND_IMM, XOR_IMM, OR_IMM, MUL_IMM, SET_LT, shifts, etc.

170 BRANCH_EQ         TwoRegOneOff   - Branch if registers equal
171 BRANCH_NE         TwoRegOneOff
172 BRANCH_LT_U       TwoRegOneOff
173 BRANCH_LT_S       TwoRegOneOff
174 BRANCH_GE_U       TwoRegOneOff
175 BRANCH_GE_S       TwoRegOneOff

180 LOAD_IMM_JUMP_IND TwoRegTwoImm   - Load immediate and indirect jump

190-199 32-bit arithmetic (ThreeReg)
        ADD_32, SUB_32, MUL_32, DIV_U_32, DIV_S_32, REM_U_32, REM_S_32, shifts

200-209 64-bit arithmetic (ThreeReg)
        ADD_64, SUB_64, MUL_64, DIV_U_64, DIV_S_64, REM_U_64, REM_S_64, shifts

210-230 Logic/comparison (ThreeReg)
        AND, XOR, OR, MUL_UPPER, SET_LT, CMOV, ROT, AND_INV, OR_INV, XNOR, MAX, MIN
```

---

## WASM to PVM Mapping Strategy

### Arithmetic Operations
| WASM | PVM |
|------|-----|
| i32.add | ADD_32 rD, rA, rB |
| i64.add | ADD_64 rD, rA, rB |
| i32.const N | LOAD_IMM rD, N (or LOAD_IMM_64 for large) |

### Control Flow
| WASM | PVM Strategy |
|------|--------------|
| block | Label for forward branch |
| loop | Label for backward branch |
| br N | JUMP to Nth enclosing label |
| br_if N | BRANCH_NE_IMM + condition check |
| if/else/end | BRANCH + JUMP combination |

### Locals/Stack
**Challenge:** WASM has unlimited stack + locals, PVM has 13 registers.

**Strategy:**
1. Use r2-r6, r9-r12 for operand stack and locals (9 registers)
2. Spill to stack memory when needed (use r1 as stack pointer)
3. Track stack depth at each instruction

### Memory Operations
| WASM | PVM |
|------|-----|
| i32.load offset=N | LOAD_IND_U32 rD, rBase, N |
| i32.store offset=N | STORE_IND_U32 rBase, rVal, N |
| memory.grow | SBRK (or host call) |

**Address Translation:**
- WASM addresses start at 0
- PVM addresses < 0x10000 cause panic
- Solution: Add base offset (0x20000 for RW data in SPI)

---

## Design Decisions

### 1. Float Rejection
**Decision:** Reject WASM modules containing float instructions.
**Reason:** PVM has no floating point support.

### 2. Output Format
**Decision:** Target SPI format for JAM compatibility.
**Reason:** User requirement - JAM programs are the priority.

### 3. Register Allocation (Proposed)
| Registers | Usage |
|-----------|-------|
| r0 | Reserved (SPI convention) |
| r1 | Stack pointer (SPI convention) |
| r2-r6 | Operand stack / temporaries |
| r7-r8 | Reserved (args in SPI) |
| r9-r12 | Local variables / callee-saved |

### 4. Calling Convention (Proposed)
- Arguments: r2-r6 (first 5 args), then stack
- Return value: r2
- Callee-saved: r9-r12
- Caller-saved: r2-r6

---

## Open Questions

1. ~~What are PVM's calling conventions?~~ → See SPI format above
2. How to handle WASM globals? → Store in RW data section
3. What's the best strategy for br_table? → Jump table
4. ~~Should we support floating point?~~ → No, reject
5. How to handle memory.grow? → SBRK instruction or host call

---

## Tooling Notes

### anan-as
- AssemblyScript PVM implementation
- Can be used as reference interpreter
- Has test vectors compatibility

### Zink Compiler
- WASM → EVM compiler
- Uses wasmparser's VisitOperator trait
- Note: EVM is also stack-based, so doesn't solve our stack→register problem
- Good reference for visitor pattern and control flow handling

---

## References

- [Gray Paper v0.7.2](./gp-0.7.2.md) - JAM protocol specification
- [Ananas PVM](./vendor/anan-as) - PVM reference implementation (submodule)
- [Zink Compiler](./vendor/zink) - WASM→EVM compiler inspiration (submodule)
- [WebAssembly Spec](https://webassembly.github.io/spec/) - WASM specification
