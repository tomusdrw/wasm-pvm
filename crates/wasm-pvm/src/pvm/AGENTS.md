# PVM Instruction Module

**Purpose**: PolkaVM (PVM) instruction definitions, opcodes, and encoding

## Files

| File | Lines | Role |
|------|-------|------|
| `instruction.rs` | ~700 | Instruction enum, encoding/decoding logic |
| `opcode.rs` | ~130 | Opcode constants (~100 opcodes) |
| `blob.rs` | 143 | Program blob format with jump table |
| `peephole.rs` | ~290 | Post-codegen peephole optimizer (Fallthroughs, truncation NOPs, dead stores) |

## Key Patterns

### Instruction Encoding
```rust
pub enum Instruction {
    Add32 { dst: u8, src1: u8, src2: u8 },
    LoadIndU32 { dst: u8, base: u8, offset: i32 },
    MoveReg { dst: u8, src: u8 },
    BranchLtUImm { reg: u8, value: i32, offset: i32 },
    BranchEq { reg1: u8, reg2: u8, offset: i32 },
    CmovIzImm { dst: u8, cond: u8, value: i32 },  // TwoRegOneImm encoding
    StoreImmU32 { address: i32, value: i32 },  // TwoImm encoding
    StoreImmIndU32 { base: u8, offset: i32, value: i32 },  // OneRegTwoImm encoding
    AndImm { dst: u8, src: u8, value: i32 },
    ShloLImm32 { dst: u8, src: u8, value: i32 },
    NegAddImm32 { dst: u8, src: u8, value: i32 },
    SetGtUImm { dst: u8, src: u8, value: i32 },
    // ... ~100 variants total
}
```

### Encoding Helpers

- `encode_three_reg(opcode, dst, src1, src2)` - ALU ops (3 regs)
- `encode_two_reg(opcode, dst, src)` - Moves/conversions (2 regs)
- `encode_two_reg_one_imm(opcode, dst, src, value)` - ALU immediate ops (2 regs + imm)
- `encode_two_imm(opcode, imm1, imm2)` - TwoImm format (StoreImm*)
- `encode_one_reg_one_imm_one_off(opcode, reg, imm, offset)` - Branch-immediate ops
- `encode_one_reg_two_imm(opcode, base, offset, value)` - Store immediate indirect
- `encode_two_reg_one_off(opcode, reg1, reg2, offset)` - Branch-register ops
- `encode_two_reg_two_imm(opcode, reg1, reg2, imm1, imm2)` - Compound indirect jump (`LoadImmJumpInd`)
- `encode_imm(value)` - Variable-length signed immediate (0-4 bytes)
- `encode_uimm(value)` - Variable-length unsigned immediate (0-4 bytes)
- `encode_var_u32(value)` - LEB128-style variable int

### Decoding Helpers

- `Instruction::decode(bytes)` dispatches by opcode and returns `(instruction, consumed_bytes)`
- `Opcode::from_u8` / `Opcode::try_from` provide explicit opcode-byte to enum conversion
- `decode_imm_signed` / `decode_imm_unsigned` handle 0-4 byte immediate expansion
- `decode_offset_at` reads fixed 4-byte branch/jump offsets
- For formats where the trailing immediate has no explicit length (`OneImm`, `OneRegOneImm`, `TwoRegOneImm`, `TwoImm`, `OneRegTwoImm`, `TwoRegTwoImm`), decode consumes the remaining bytes as that immediate

### Terminating Instructions
Instructions that end a basic block:
```rust
pub fn is_terminating(&self) -> bool {
    matches!(self,
        Trap | Fallthrough | Jump {..} | LoadImmJump {..} | JumpInd {..} | LoadImmJumpInd {..} |
        BranchNeImm {..} | BranchEqImm {..} | ...)
}
```

### Destination Register Query
Used by the register cache in `emitter.rs` to auto-invalidate stale cache entries:
```rust
pub fn dest_reg(&self) -> Option<u8> {
    // Returns Some(reg) for instructions that write to a register
    // Returns None for stores, branches, traps, ecalli
}
```

### Peephole Notes
- Dead-code elimination runs only when a function has no labels (single-block code). Multi-block
  functions skip DCE to avoid incorrect liveness across control flow.
- DCE must track side-effects for all store variants: `StoreIndU8/U16/U32/U64`, `StoreImmIndU8/U16/U32/U64`, `StoreImmU8/U16/U32/U64`, `StoreU8/U16/U32/U64`
- DCE must track memory loads (can-trap, track dst) for all load variants: `LoadIndU8/I8/U16/I16/U32/I32/U64`, `LoadU8/I8/U16/I16/U32/I32/U64`


## Where to Look

| Task | Location |
|------|----------|
| Add new PVM instruction | `opcode.rs` (add enum variant) + `instruction.rs` (encoding) |
| Change instruction encoding | `instruction.rs:impl Instruction` |
| Check opcode exists | `opcode.rs` (~100 opcodes defined) |
| Build program blob | `blob.rs:ProgramBlob::with_jump_table()` |
| Variable int encoding | `blob.rs:encode_var_u32()` |

## Anti-Patterns

1. **Don't change opcode numbers** - Would break existing JAM files
2. **Preserve register field order** - `(dst, src1, src2)` convention
3. **Keep encoding compact** - Variable-length immediates save space

## Testing

Unit tests in same files under `#[cfg(test)]`:
- `instruction.rs`: Tests encoding and decode(encode) roundtrip coverage for all variants
- `blob.rs`: Tests mask packing, varint encoding

## Gray Paper Reference

See `gp-0.7.2.md` Appendix A for PVM spec:
- Gas costs per instruction (ϱ∆)
- Semantics for each opcode
- This module implements the encoding, not semantics
