# PVM Instruction Module

**Purpose**: PolkaVM (PVM) instruction definitions, opcodes, and encoding

## Files

| File | Lines | Role |
|------|-------|------|
| `instruction.rs` | 332 | Instruction enum, encoding logic |
| `opcode.rs` | 114 | Opcode constants (84 opcodes) |
| `blob.rs` | 143 | Program blob format with jump table |

## Key Patterns

### Instruction Encoding
```rust
pub enum Instruction {
    Add32 { dst: u8, src1: u8, src2: u8 },
    LoadIndU32 { dst: u8, base: u8, offset: i32 },
    // ... 64 variants total
}
```

### Encoding Helpers
- `encode_three_reg(opcode, dst, src1, src2)` - ALU ops
- `encode_two_reg(opcode, dst, src)` - Moves/conversions
- `encode_imm(value)` - Variable-length immediate (0-4 bytes)
- `encode_var_u32(value)` - LEB128-style variable int

### Terminating Instructions
Instructions that end a basic block:
```rust
pub fn is_terminating(&self) -> bool {
    matches!(self,
        Trap | Fallthrough | Jump {..} | JumpInd {..} |
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

## Where to Look

| Task | Location |
|------|----------|
| Add new PVM instruction | `opcode.rs` (add enum variant) + `instruction.rs` (encoding) |
| Change instruction encoding | `instruction.rs:impl Instruction` |
| Check opcode exists | `opcode.rs` (84 opcodes defined) |
| Build program blob | `blob.rs:ProgramBlob::with_jump_table()` |
| Variable int encoding | `blob.rs:encode_var_u32()` |

## Anti-Patterns

1. **Don't change opcode numbers** - Would break existing JAM files
2. **Preserve register field order** - `(dst, src1, src2)` convention
3. **Keep encoding compact** - Variable-length immediates save space

## Testing

Unit tests in same files under `#[cfg(test)]`:
- `instruction.rs`: Tests encoding roundtrip
- `blob.rs`: Tests mask packing, varint encoding

## Gray Paper Reference

See `gp-0.7.2.md` Appendix A for PVM spec:
- Gas costs per instruction (ϱ∆)
- Semantics for each opcode
- This module implements the encoding, not semantics
