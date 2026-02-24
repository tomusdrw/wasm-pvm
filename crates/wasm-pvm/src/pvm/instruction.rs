use super::Opcode;
use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Instruction {
    Trap,
    Fallthrough,
    LoadImm64 {
        reg: u8,
        value: u64,
    },
    LoadImm {
        reg: u8,
        value: i32,
    },
    Add32 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    Sub32 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    Mul32 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    DivU32 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    DivS32 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    RemU32 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    RemS32 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    Add64 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    Sub64 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    Mul64 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    DivU64 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    DivS64 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    RemU64 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    RemS64 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    ShloL64 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    ShloR64 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    SharR64 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    AddImm32 {
        dst: u8,
        src: u8,
        value: i32,
    },
    AddImm64 {
        dst: u8,
        src: u8,
        value: i32,
    },
    AndImm {
        dst: u8,
        src: u8,
        value: i32,
    },
    XorImm {
        dst: u8,
        src: u8,
        value: i32,
    },
    OrImm {
        dst: u8,
        src: u8,
        value: i32,
    },
    MulImm32 {
        dst: u8,
        src: u8,
        value: i32,
    },
    MulImm64 {
        dst: u8,
        src: u8,
        value: i32,
    },
    ShloLImm32 {
        dst: u8,
        src: u8,
        value: i32,
    },
    ShloRImm32 {
        dst: u8,
        src: u8,
        value: i32,
    },
    SharRImm32 {
        dst: u8,
        src: u8,
        value: i32,
    },
    ShloLImm64 {
        dst: u8,
        src: u8,
        value: i32,
    },
    ShloRImm64 {
        dst: u8,
        src: u8,
        value: i32,
    },
    SharRImm64 {
        dst: u8,
        src: u8,
        value: i32,
    },
    NegAddImm32 {
        dst: u8,
        src: u8,
        value: i32,
    },
    NegAddImm64 {
        dst: u8,
        src: u8,
        value: i32,
    },
    SetGtUImm {
        dst: u8,
        src: u8,
        value: i32,
    },
    SetGtSImm {
        dst: u8,
        src: u8,
        value: i32,
    },
    Jump {
        offset: i32,
    },
    /// Combined load-immediate + jump: `reg = sign_extend(value); goto(PC + offset)`
    LoadImmJump {
        reg: u8,
        value: i32,
        offset: i32,
    },
    JumpInd {
        reg: u8,
        offset: i32,
    },
    LoadIndU32 {
        dst: u8,
        base: u8,
        offset: i32,
    },
    StoreIndU32 {
        base: u8,
        src: u8,
        offset: i32,
    },
    LoadIndU64 {
        dst: u8,
        base: u8,
        offset: i32,
    },
    StoreIndU64 {
        base: u8,
        src: u8,
        offset: i32,
    },
    BranchNeImm {
        reg: u8,
        value: i32,
        offset: i32,
    },
    BranchEqImm {
        reg: u8,
        value: i32,
        offset: i32,
    },
    BranchGeSImm {
        reg: u8,
        value: i32,
        offset: i32,
    },
    BranchLtUImm {
        reg: u8,
        value: i32,
        offset: i32,
    },
    BranchLeUImm {
        reg: u8,
        value: i32,
        offset: i32,
    },
    BranchGeUImm {
        reg: u8,
        value: i32,
        offset: i32,
    },
    BranchGtUImm {
        reg: u8,
        value: i32,
        offset: i32,
    },
    BranchLtSImm {
        reg: u8,
        value: i32,
        offset: i32,
    },
    BranchLeSImm {
        reg: u8,
        value: i32,
        offset: i32,
    },
    BranchGtSImm {
        reg: u8,
        value: i32,
        offset: i32,
    },
    MoveReg {
        dst: u8,
        src: u8,
    },
    BranchEq {
        reg1: u8,
        reg2: u8,
        offset: i32,
    },
    BranchNe {
        reg1: u8,
        reg2: u8,
        offset: i32,
    },
    BranchGeU {
        reg1: u8,
        reg2: u8,
        offset: i32,
    },
    BranchLtU {
        reg1: u8,
        reg2: u8,
        offset: i32,
    },
    BranchLtS {
        reg1: u8,
        reg2: u8,
        offset: i32,
    },
    BranchGeS {
        reg1: u8,
        reg2: u8,
        offset: i32,
    },
    SetLtU {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    SetLtS {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    /// Conditional move if zero: `if reg[cond] == 0 then reg[dst] = reg[src]`
    CmovIz {
        dst: u8,
        src: u8,
        cond: u8,
    },
    /// Conditional move if non-zero: `if reg[cond] != 0 then reg[dst] = reg[src]`
    CmovNz {
        dst: u8,
        src: u8,
        cond: u8,
    },
    And {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    Xor {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    Or {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    SetLtUImm {
        dst: u8,
        src: u8,
        value: i32,
    },
    SetLtSImm {
        dst: u8,
        src: u8,
        value: i32,
    },
    ShloL32 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    ShloR32 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    SharR32 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    Sbrk {
        dst: u8,
        src: u8,
    },
    CountSetBits64 {
        dst: u8,
        src: u8,
    },
    CountSetBits32 {
        dst: u8,
        src: u8,
    },
    LeadingZeroBits64 {
        dst: u8,
        src: u8,
    },
    LeadingZeroBits32 {
        dst: u8,
        src: u8,
    },
    TrailingZeroBits64 {
        dst: u8,
        src: u8,
    },
    TrailingZeroBits32 {
        dst: u8,
        src: u8,
    },
    SignExtend8 {
        dst: u8,
        src: u8,
    },
    SignExtend16 {
        dst: u8,
        src: u8,
    },
    ZeroExtend16 {
        dst: u8,
        src: u8,
    },
    LoadIndU8 {
        dst: u8,
        base: u8,
        offset: i32,
    },
    LoadIndI8 {
        dst: u8,
        base: u8,
        offset: i32,
    },
    StoreIndU8 {
        base: u8,
        src: u8,
        offset: i32,
    },
    LoadIndU16 {
        dst: u8,
        base: u8,
        offset: i32,
    },
    LoadIndI16 {
        dst: u8,
        base: u8,
        offset: i32,
    },
    StoreIndU16 {
        base: u8,
        src: u8,
        offset: i32,
    },
    /// Conditional move if zero with immediate: if reg[cond] == 0 then reg[dst] = `sign_extend(value)`
    CmovIzImm {
        dst: u8,
        cond: u8,
        value: i32,
    },
    /// Conditional move if non-zero with immediate: if reg[cond] != 0 then reg[dst] = `sign_extend(value)`
    CmovNzImm {
        dst: u8,
        cond: u8,
        value: i32,
    },
    /// Store immediate to absolute address: mem[address] = value (u8)
    StoreImmU8 {
        address: i32,
        value: i32,
    },
    /// Store immediate to absolute address: mem[address] = value (u16)
    StoreImmU16 {
        address: i32,
        value: i32,
    },
    /// Store immediate to absolute address: mem[address] = value (u32)
    StoreImmU32 {
        address: i32,
        value: i32,
    },
    /// Store immediate to absolute address: mem[address] = `sign_extend(value)` (u64)
    StoreImmU64 {
        address: i32,
        value: i32,
    },
    /// Store immediate to [base + offset]: mem[reg[base] + offset] = value (u8)
    StoreImmIndU8 {
        base: u8,
        offset: i32,
        value: i32,
    },
    /// Store immediate to [base + offset]: mem[reg[base] + offset] = value (u16)
    StoreImmIndU16 {
        base: u8,
        offset: i32,
        value: i32,
    },
    /// Store immediate to [base + offset]: mem[reg[base] + offset] = value (u32)
    StoreImmIndU32 {
        base: u8,
        offset: i32,
        value: i32,
    },
    /// Store immediate to [base + offset]: mem[reg[base] + offset] = `sign_extend(value)` (u64)
    StoreImmIndU64 {
        base: u8,
        offset: i32,
        value: i32,
    },
    /// Load from absolute address: reg[dst] = `zero_extend(mem[address])` (u8)
    LoadU8 {
        dst: u8,
        address: i32,
    },
    /// Load from absolute address: reg[dst] = `sign_extend(mem[address])` (i8)
    LoadI8 {
        dst: u8,
        address: i32,
    },
    /// Load from absolute address: reg[dst] = `zero_extend(mem[address])` (u16)
    LoadU16 {
        dst: u8,
        address: i32,
    },
    /// Load from absolute address: reg[dst] = `sign_extend(mem[address])` (i16)
    LoadI16 {
        dst: u8,
        address: i32,
    },
    /// Load from absolute address: reg[dst] = `zero_extend(mem[address])` (u32)
    LoadU32 {
        dst: u8,
        address: i32,
    },
    /// Load from absolute address: reg[dst] = `sign_extend(mem[address])` (i32)
    LoadI32 {
        dst: u8,
        address: i32,
    },
    /// Load from absolute address: reg[dst] = mem[address] (u64)
    LoadU64 {
        dst: u8,
        address: i32,
    },
    /// Store to absolute address: mem[address] = reg[src] (u8)
    StoreU8 {
        src: u8,
        address: i32,
    },
    /// Store to absolute address: mem[address] = reg[src] (u16)
    StoreU16 {
        src: u8,
        address: i32,
    },
    /// Store to absolute address: mem[address] = reg[src] (u32)
    StoreU32 {
        src: u8,
        address: i32,
    },
    /// Store to absolute address: mem[address] = reg[src] (u64)
    StoreU64 {
        src: u8,
        address: i32,
    },
    /// Load indirect with sign extension: reg[dst] = `sign_extend(mem[reg[base] + offset])` (i32)
    LoadIndI32 {
        dst: u8,
        base: u8,
        offset: i32,
    },
    /// Reverse byte order: reg[dst] = bswap(reg[src])
    ReverseBytes {
        dst: u8,
        src: u8,
    },
    /// Alternate shift left immediate (32-bit): reg[dst] = `sign_extend(value)` << reg[src]
    ShloLImmAlt32 {
        dst: u8,
        src: u8,
        value: i32,
    },
    /// Alternate shift logical right immediate (32-bit): reg[dst] = `sign_extend(value)` >> reg[src]
    ShloRImmAlt32 {
        dst: u8,
        src: u8,
        value: i32,
    },
    /// Alternate shift arithmetic right immediate (32-bit): reg[dst] = `sign_extend(value)` >>> reg[src]
    SharRImmAlt32 {
        dst: u8,
        src: u8,
        value: i32,
    },
    /// Alternate shift left immediate (64-bit): reg[dst] = `sign_extend(value)` << reg[src]
    ShloLImmAlt64 {
        dst: u8,
        src: u8,
        value: i32,
    },
    /// Alternate shift logical right immediate (64-bit): reg[dst] = `sign_extend(value)` >> reg[src]
    ShloRImmAlt64 {
        dst: u8,
        src: u8,
        value: i32,
    },
    /// Alternate shift arithmetic right immediate (64-bit): reg[dst] = `sign_extend(value)` >>> reg[src]
    SharRImmAlt64 {
        dst: u8,
        src: u8,
        value: i32,
    },
    /// Rotate right by immediate (64-bit): reg[dst] = reg[src] ROR value
    RotRImm64 {
        dst: u8,
        src: u8,
        value: i32,
    },
    /// Alternate rotate right (64-bit): reg[dst] = `sign_extend(value)` ROR reg[src]
    RotRImmAlt64 {
        dst: u8,
        src: u8,
        value: i32,
    },
    /// Rotate right by immediate (32-bit): reg[dst] = reg[src] ROR value
    RotRImm32 {
        dst: u8,
        src: u8,
        value: i32,
    },
    /// Alternate rotate right (32-bit): reg[dst] = `sign_extend(value)` ROR reg[src]
    RotRImmAlt32 {
        dst: u8,
        src: u8,
        value: i32,
    },
    /// Combined load-immediate + indirect jump: `reg[dst] = sign_extend(value); jump to reg[base] + sign_extend(offset)`
    LoadImmJumpInd {
        base: u8,
        dst: u8,
        value: i32,
        offset: i32,
    },
    /// Upper 64 bits of signed*signed 128-bit multiply
    MulUpperSS {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    /// Upper 64 bits of unsigned*unsigned 128-bit multiply
    MulUpperUU {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    /// Upper 64 bits of signed*unsigned 128-bit multiply
    MulUpperSU {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    /// Rotate left (64-bit): reg[dst] = reg[src1] ROL reg[src2]
    RotL64 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    /// Rotate left (32-bit): reg[dst] = reg[src1] ROL reg[src2]
    RotL32 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    /// Rotate right (64-bit): reg[dst] = reg[src1] ROR reg[src2]
    RotR64 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    /// Rotate right (32-bit): reg[dst] = reg[src1] ROR reg[src2]
    RotR32 {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    /// AND with inverted second operand: reg[dst] = reg[src1] & ~reg[src2]
    AndInv {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    /// OR with inverted second operand: reg[dst] = reg[src1] | ~reg[src2]
    OrInv {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    /// XNOR: reg[dst] = ~(reg[src1] ^ reg[src2])
    Xnor {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    /// Signed maximum: reg[dst] = max(reg[src1], reg[src2])
    Max {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    /// Unsigned maximum: reg[dst] = `max_u(reg[src1], reg[src2])`
    MaxU {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    /// Signed minimum: reg[dst] = min(reg[src1], reg[src2])
    Min {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    /// Unsigned minimum: reg[dst] = `min_u(reg[src1], reg[src2])`
    MinU {
        dst: u8,
        src1: u8,
        src2: u8,
    },
    Ecalli {
        index: u32,
    },
    Unknown {
        opcode: u8,
        raw_bytes: Vec<u8>,
    },
}

impl Instruction {
    /// Decode a single instruction from a byte slice.
    ///
    /// Returns `(instruction, consumed_bytes)`.
    ///
    /// For encodings where the last immediate does not carry an explicit length
    /// (`OneImm`, `OneRegOneImm`, `TwoRegOneImm`, `TwoImm`, `OneRegTwoImm`,
    /// `TwoRegTwoImm`), this decoder consumes the remaining bytes as that
    /// immediate.
    pub fn decode(bytes: &[u8]) -> Result<(Self, usize)> {
        if bytes.is_empty() {
            return Err(Error::Internal(
                "cannot decode instruction from an empty slice".to_string(),
            ));
        }

        let opcode = bytes[0];

        match opcode {
            op if op == Opcode::Trap as u8 => Ok((Self::Trap, 1)),
            op if op == Opcode::Fallthrough as u8 => Ok((Self::Fallthrough, 1)),

            op if op == Opcode::LoadImm64 as u8 => {
                ensure_min_len(bytes, 10, "LoadImm64")?;
                let reg = bytes[1] & 0x0F;
                let mut raw = [0u8; 8];
                raw.copy_from_slice(&bytes[2..10]);
                Ok((
                    Self::LoadImm64 {
                        reg,
                        value: u64::from_le_bytes(raw),
                    },
                    10,
                ))
            }

            op if op == Opcode::Jump as u8 => {
                let offset = decode_offset_at(bytes, 1, "Jump")?;
                Ok((Self::Jump { offset }, 5))
            }

            op if op == Opcode::Ecalli as u8 => {
                let index = decode_imm_unsigned(&bytes[1..], "Ecalli index")?;
                Ok((Self::Ecalli { index }, bytes.len()))
            }

            op if op == Opcode::LoadImm as u8
                || op == Opcode::JumpInd as u8
                || op == Opcode::LoadU8 as u8
                || op == Opcode::LoadI8 as u8
                || op == Opcode::LoadU16 as u8
                || op == Opcode::LoadI16 as u8
                || op == Opcode::LoadU32 as u8
                || op == Opcode::LoadI32 as u8
                || op == Opcode::LoadU64 as u8
                || op == Opcode::StoreU8 as u8
                || op == Opcode::StoreU16 as u8
                || op == Opcode::StoreU32 as u8
                || op == Opcode::StoreU64 as u8 =>
            {
                ensure_min_len(bytes, 2, "OneRegOneImm")?;
                let reg = bytes[1] & 0x0F;
                let value = decode_imm_signed(&bytes[2..], "OneRegOneImm immediate")?;

                let instruction = match op {
                    op if op == Opcode::LoadImm as u8 => Self::LoadImm { reg, value },
                    op if op == Opcode::JumpInd as u8 => Self::JumpInd { reg, offset: value },
                    op if op == Opcode::LoadU8 as u8 => Self::LoadU8 {
                        dst: reg,
                        address: value,
                    },
                    op if op == Opcode::LoadI8 as u8 => Self::LoadI8 {
                        dst: reg,
                        address: value,
                    },
                    op if op == Opcode::LoadU16 as u8 => Self::LoadU16 {
                        dst: reg,
                        address: value,
                    },
                    op if op == Opcode::LoadI16 as u8 => Self::LoadI16 {
                        dst: reg,
                        address: value,
                    },
                    op if op == Opcode::LoadU32 as u8 => Self::LoadU32 {
                        dst: reg,
                        address: value,
                    },
                    op if op == Opcode::LoadI32 as u8 => Self::LoadI32 {
                        dst: reg,
                        address: value,
                    },
                    op if op == Opcode::LoadU64 as u8 => Self::LoadU64 {
                        dst: reg,
                        address: value,
                    },
                    op if op == Opcode::StoreU8 as u8 => Self::StoreU8 {
                        src: reg,
                        address: value,
                    },
                    op if op == Opcode::StoreU16 as u8 => Self::StoreU16 {
                        src: reg,
                        address: value,
                    },
                    op if op == Opcode::StoreU32 as u8 => Self::StoreU32 {
                        src: reg,
                        address: value,
                    },
                    op if op == Opcode::StoreU64 as u8 => Self::StoreU64 {
                        src: reg,
                        address: value,
                    },
                    _ => {
                        return Err(Error::Internal(format!(
                            "OneRegOneImm decode dispatch mismatch for opcode {op}"
                        )));
                    }
                };

                Ok((instruction, bytes.len()))
            }

            op if op == Opcode::LoadImmJump as u8
                || op == Opcode::BranchEqImm as u8
                || op == Opcode::BranchNeImm as u8
                || op == Opcode::BranchLtUImm as u8
                || op == Opcode::BranchLeUImm as u8
                || op == Opcode::BranchGeUImm as u8
                || op == Opcode::BranchGtUImm as u8
                || op == Opcode::BranchLtSImm as u8
                || op == Opcode::BranchLeSImm as u8
                || op == Opcode::BranchGeSImm as u8
                || op == Opcode::BranchGtSImm as u8 =>
            {
                ensure_min_len(bytes, 2, "OneRegOneImmOneOff")?;
                let reg = bytes[1] & 0x0F;
                let imm_len = (bytes[1] >> 4) as usize;
                if imm_len > 4 {
                    return Err(Error::Internal(format!(
                        "OneRegOneImmOneOff immediate length must be <= 4, got {imm_len}"
                    )));
                }

                let imm_end = 2 + imm_len;
                ensure_min_len(bytes, imm_end + 4, "OneRegOneImmOneOff")?;
                let value = decode_imm_signed(&bytes[2..imm_end], "OneRegOneImmOneOff immediate")?;
                let offset = decode_offset_at(bytes, imm_end, "OneRegOneImmOneOff offset")?;

                let instruction = match op {
                    op if op == Opcode::LoadImmJump as u8 => {
                        Self::LoadImmJump { reg, value, offset }
                    }
                    op if op == Opcode::BranchEqImm as u8 => {
                        Self::BranchEqImm { reg, value, offset }
                    }
                    op if op == Opcode::BranchNeImm as u8 => {
                        Self::BranchNeImm { reg, value, offset }
                    }
                    op if op == Opcode::BranchLtUImm as u8 => {
                        Self::BranchLtUImm { reg, value, offset }
                    }
                    op if op == Opcode::BranchLeUImm as u8 => {
                        Self::BranchLeUImm { reg, value, offset }
                    }
                    op if op == Opcode::BranchGeUImm as u8 => {
                        Self::BranchGeUImm { reg, value, offset }
                    }
                    op if op == Opcode::BranchGtUImm as u8 => {
                        Self::BranchGtUImm { reg, value, offset }
                    }
                    op if op == Opcode::BranchLtSImm as u8 => {
                        Self::BranchLtSImm { reg, value, offset }
                    }
                    op if op == Opcode::BranchLeSImm as u8 => {
                        Self::BranchLeSImm { reg, value, offset }
                    }
                    op if op == Opcode::BranchGeSImm as u8 => {
                        Self::BranchGeSImm { reg, value, offset }
                    }
                    op if op == Opcode::BranchGtSImm as u8 => {
                        Self::BranchGtSImm { reg, value, offset }
                    }
                    _ => {
                        return Err(Error::Internal(format!(
                            "OneRegOneImmOneOff decode dispatch mismatch for opcode {op}"
                        )));
                    }
                };

                Ok((instruction, imm_end + 4))
            }

            op if op == Opcode::StoreImmU8 as u8
                || op == Opcode::StoreImmU16 as u8
                || op == Opcode::StoreImmU32 as u8
                || op == Opcode::StoreImmU64 as u8 =>
            {
                ensure_min_len(bytes, 2, "TwoImm")?;
                let imm1_len = (bytes[1] & 0x0F) as usize;
                if imm1_len > 4 {
                    return Err(Error::Internal(format!(
                        "TwoImm first immediate length must be <= 4, got {imm1_len}"
                    )));
                }

                let imm1_end = 2 + imm1_len;
                ensure_min_len(bytes, imm1_end, "TwoImm")?;

                let address = decode_imm_signed(&bytes[2..imm1_end], "TwoImm address")?;
                let value = decode_imm_signed(&bytes[imm1_end..], "TwoImm value")?;

                let instruction = match op {
                    op if op == Opcode::StoreImmU8 as u8 => Self::StoreImmU8 { address, value },
                    op if op == Opcode::StoreImmU16 as u8 => Self::StoreImmU16 { address, value },
                    op if op == Opcode::StoreImmU32 as u8 => Self::StoreImmU32 { address, value },
                    op if op == Opcode::StoreImmU64 as u8 => Self::StoreImmU64 { address, value },
                    _ => {
                        return Err(Error::Internal(format!(
                            "TwoImm decode dispatch mismatch for opcode {op}"
                        )));
                    }
                };

                Ok((instruction, bytes.len()))
            }

            op if op == Opcode::StoreImmIndU8 as u8
                || op == Opcode::StoreImmIndU16 as u8
                || op == Opcode::StoreImmIndU32 as u8
                || op == Opcode::StoreImmIndU64 as u8 =>
            {
                ensure_min_len(bytes, 2, "OneRegTwoImm")?;
                let base = bytes[1] & 0x0F;
                let imm1_len = (bytes[1] >> 4) as usize;
                if imm1_len > 4 {
                    return Err(Error::Internal(format!(
                        "OneRegTwoImm first immediate length must be <= 4, got {imm1_len}"
                    )));
                }

                let imm1_end = 2 + imm1_len;
                ensure_min_len(bytes, imm1_end, "OneRegTwoImm")?;
                let offset = decode_imm_signed(&bytes[2..imm1_end], "OneRegTwoImm offset")?;
                let value = decode_imm_signed(&bytes[imm1_end..], "OneRegTwoImm value")?;

                let instruction = match op {
                    op if op == Opcode::StoreImmIndU8 as u8 => Self::StoreImmIndU8 {
                        base,
                        offset,
                        value,
                    },
                    op if op == Opcode::StoreImmIndU16 as u8 => Self::StoreImmIndU16 {
                        base,
                        offset,
                        value,
                    },
                    op if op == Opcode::StoreImmIndU32 as u8 => Self::StoreImmIndU32 {
                        base,
                        offset,
                        value,
                    },
                    op if op == Opcode::StoreImmIndU64 as u8 => Self::StoreImmIndU64 {
                        base,
                        offset,
                        value,
                    },
                    _ => {
                        return Err(Error::Internal(format!(
                            "OneRegTwoImm decode dispatch mismatch for opcode {op}"
                        )));
                    }
                };

                Ok((instruction, bytes.len()))
            }

            op if op == Opcode::MoveReg as u8
                || op == Opcode::Sbrk as u8
                || op == Opcode::CountSetBits64 as u8
                || op == Opcode::CountSetBits32 as u8
                || op == Opcode::LeadingZeroBits64 as u8
                || op == Opcode::LeadingZeroBits32 as u8
                || op == Opcode::TrailingZeroBits64 as u8
                || op == Opcode::TrailingZeroBits32 as u8
                || op == Opcode::SignExtend8 as u8
                || op == Opcode::SignExtend16 as u8
                || op == Opcode::ZeroExtend16 as u8
                || op == Opcode::ReverseBytes as u8 =>
            {
                ensure_min_len(bytes, 2, "TwoReg")?;
                let src = (bytes[1] >> 4) & 0x0F;
                let dst = bytes[1] & 0x0F;

                let instruction = match op {
                    op if op == Opcode::MoveReg as u8 => Self::MoveReg { dst, src },
                    op if op == Opcode::Sbrk as u8 => Self::Sbrk { dst, src },
                    op if op == Opcode::CountSetBits64 as u8 => Self::CountSetBits64 { dst, src },
                    op if op == Opcode::CountSetBits32 as u8 => Self::CountSetBits32 { dst, src },
                    op if op == Opcode::LeadingZeroBits64 as u8 => {
                        Self::LeadingZeroBits64 { dst, src }
                    }
                    op if op == Opcode::LeadingZeroBits32 as u8 => {
                        Self::LeadingZeroBits32 { dst, src }
                    }
                    op if op == Opcode::TrailingZeroBits64 as u8 => {
                        Self::TrailingZeroBits64 { dst, src }
                    }
                    op if op == Opcode::TrailingZeroBits32 as u8 => {
                        Self::TrailingZeroBits32 { dst, src }
                    }
                    op if op == Opcode::SignExtend8 as u8 => Self::SignExtend8 { dst, src },
                    op if op == Opcode::SignExtend16 as u8 => Self::SignExtend16 { dst, src },
                    op if op == Opcode::ZeroExtend16 as u8 => Self::ZeroExtend16 { dst, src },
                    op if op == Opcode::ReverseBytes as u8 => Self::ReverseBytes { dst, src },
                    _ => {
                        return Err(Error::Internal(format!(
                            "TwoReg decode dispatch mismatch for opcode {op}"
                        )));
                    }
                };

                Ok((instruction, 2))
            }

            op if op == Opcode::AddImm32 as u8
                || op == Opcode::AddImm64 as u8
                || op == Opcode::AndImm as u8
                || op == Opcode::XorImm as u8
                || op == Opcode::OrImm as u8
                || op == Opcode::MulImm32 as u8
                || op == Opcode::MulImm64 as u8
                || op == Opcode::SetLtUImm as u8
                || op == Opcode::SetLtSImm as u8
                || op == Opcode::ShloLImm32 as u8
                || op == Opcode::ShloRImm32 as u8
                || op == Opcode::SharRImm32 as u8
                || op == Opcode::ShloLImm64 as u8
                || op == Opcode::ShloRImm64 as u8
                || op == Opcode::SharRImm64 as u8
                || op == Opcode::NegAddImm32 as u8
                || op == Opcode::NegAddImm64 as u8
                || op == Opcode::SetGtUImm as u8
                || op == Opcode::SetGtSImm as u8
                || op == Opcode::CmovIzImm as u8
                || op == Opcode::CmovNzImm as u8
                || op == Opcode::LoadIndU8 as u8
                || op == Opcode::LoadIndI8 as u8
                || op == Opcode::LoadIndU16 as u8
                || op == Opcode::LoadIndI16 as u8
                || op == Opcode::LoadIndU32 as u8
                || op == Opcode::LoadIndI32 as u8
                || op == Opcode::LoadIndU64 as u8
                || op == Opcode::StoreIndU8 as u8
                || op == Opcode::StoreIndU16 as u8
                || op == Opcode::StoreIndU32 as u8
                || op == Opcode::StoreIndU64 as u8
                || op == Opcode::ShloLImmAlt32 as u8
                || op == Opcode::ShloRImmAlt32 as u8
                || op == Opcode::SharRImmAlt32 as u8
                || op == Opcode::ShloLImmAlt64 as u8
                || op == Opcode::ShloRImmAlt64 as u8
                || op == Opcode::SharRImmAlt64 as u8
                || op == Opcode::RotRImm64 as u8
                || op == Opcode::RotRImmAlt64 as u8
                || op == Opcode::RotRImm32 as u8
                || op == Opcode::RotRImmAlt32 as u8 =>
            {
                ensure_min_len(bytes, 2, "TwoRegOneImm")?;
                let hi = (bytes[1] >> 4) & 0x0F;
                let lo = bytes[1] & 0x0F;
                let value = decode_imm_signed(&bytes[2..], "TwoRegOneImm immediate")?;

                let instruction = match op {
                    op if op == Opcode::AddImm32 as u8 => Self::AddImm32 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::AddImm64 as u8 => Self::AddImm64 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::AndImm as u8 => Self::AndImm {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::XorImm as u8 => Self::XorImm {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::OrImm as u8 => Self::OrImm {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::MulImm32 as u8 => Self::MulImm32 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::MulImm64 as u8 => Self::MulImm64 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::SetLtUImm as u8 => Self::SetLtUImm {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::SetLtSImm as u8 => Self::SetLtSImm {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::ShloLImm32 as u8 => Self::ShloLImm32 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::ShloRImm32 as u8 => Self::ShloRImm32 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::SharRImm32 as u8 => Self::SharRImm32 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::ShloLImm64 as u8 => Self::ShloLImm64 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::ShloRImm64 as u8 => Self::ShloRImm64 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::SharRImm64 as u8 => Self::SharRImm64 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::NegAddImm32 as u8 => Self::NegAddImm32 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::NegAddImm64 as u8 => Self::NegAddImm64 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::SetGtUImm as u8 => Self::SetGtUImm {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::SetGtSImm as u8 => Self::SetGtSImm {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::CmovIzImm as u8 => Self::CmovIzImm {
                        dst: lo,
                        cond: hi,
                        value,
                    },
                    op if op == Opcode::CmovNzImm as u8 => Self::CmovNzImm {
                        dst: lo,
                        cond: hi,
                        value,
                    },
                    op if op == Opcode::LoadIndU8 as u8 => Self::LoadIndU8 {
                        dst: lo,
                        base: hi,
                        offset: value,
                    },
                    op if op == Opcode::LoadIndI8 as u8 => Self::LoadIndI8 {
                        dst: lo,
                        base: hi,
                        offset: value,
                    },
                    op if op == Opcode::LoadIndU16 as u8 => Self::LoadIndU16 {
                        dst: lo,
                        base: hi,
                        offset: value,
                    },
                    op if op == Opcode::LoadIndI16 as u8 => Self::LoadIndI16 {
                        dst: lo,
                        base: hi,
                        offset: value,
                    },
                    op if op == Opcode::LoadIndU32 as u8 => Self::LoadIndU32 {
                        dst: lo,
                        base: hi,
                        offset: value,
                    },
                    op if op == Opcode::LoadIndI32 as u8 => Self::LoadIndI32 {
                        dst: lo,
                        base: hi,
                        offset: value,
                    },
                    op if op == Opcode::LoadIndU64 as u8 => Self::LoadIndU64 {
                        dst: lo,
                        base: hi,
                        offset: value,
                    },
                    op if op == Opcode::StoreIndU8 as u8 => Self::StoreIndU8 {
                        base: hi,
                        src: lo,
                        offset: value,
                    },
                    op if op == Opcode::StoreIndU16 as u8 => Self::StoreIndU16 {
                        base: hi,
                        src: lo,
                        offset: value,
                    },
                    op if op == Opcode::StoreIndU32 as u8 => Self::StoreIndU32 {
                        base: hi,
                        src: lo,
                        offset: value,
                    },
                    op if op == Opcode::StoreIndU64 as u8 => Self::StoreIndU64 {
                        base: hi,
                        src: lo,
                        offset: value,
                    },
                    op if op == Opcode::ShloLImmAlt32 as u8 => Self::ShloLImmAlt32 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::ShloRImmAlt32 as u8 => Self::ShloRImmAlt32 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::SharRImmAlt32 as u8 => Self::SharRImmAlt32 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::ShloLImmAlt64 as u8 => Self::ShloLImmAlt64 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::ShloRImmAlt64 as u8 => Self::ShloRImmAlt64 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::SharRImmAlt64 as u8 => Self::SharRImmAlt64 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::RotRImm64 as u8 => Self::RotRImm64 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::RotRImmAlt64 as u8 => Self::RotRImmAlt64 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::RotRImm32 as u8 => Self::RotRImm32 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    op if op == Opcode::RotRImmAlt32 as u8 => Self::RotRImmAlt32 {
                        dst: lo,
                        src: hi,
                        value,
                    },
                    _ => {
                        return Err(Error::Internal(format!(
                            "TwoRegOneImm decode dispatch mismatch for opcode {op}"
                        )));
                    }
                };

                Ok((instruction, bytes.len()))
            }

            op if op == Opcode::BranchEq as u8
                || op == Opcode::BranchNe as u8
                || op == Opcode::BranchLtU as u8
                || op == Opcode::BranchLtS as u8
                || op == Opcode::BranchGeU as u8
                || op == Opcode::BranchGeS as u8 =>
            {
                ensure_min_len(bytes, 6, "TwoRegOneOff")?;
                let reg1 = (bytes[1] >> 4) & 0x0F;
                let reg2 = bytes[1] & 0x0F;
                let offset = decode_offset_at(bytes, 2, "TwoRegOneOff offset")?;

                let instruction = match op {
                    op if op == Opcode::BranchEq as u8 => Self::BranchEq { reg1, reg2, offset },
                    op if op == Opcode::BranchNe as u8 => Self::BranchNe { reg1, reg2, offset },
                    op if op == Opcode::BranchLtU as u8 => Self::BranchLtU { reg1, reg2, offset },
                    op if op == Opcode::BranchLtS as u8 => Self::BranchLtS { reg1, reg2, offset },
                    op if op == Opcode::BranchGeU as u8 => Self::BranchGeU { reg1, reg2, offset },
                    op if op == Opcode::BranchGeS as u8 => Self::BranchGeS { reg1, reg2, offset },
                    _ => {
                        return Err(Error::Internal(format!(
                            "TwoRegOneOff decode dispatch mismatch for opcode {op}"
                        )));
                    }
                };

                Ok((instruction, 6))
            }

            op if op == Opcode::LoadImmJumpInd as u8 => {
                ensure_min_len(bytes, 3, "TwoRegTwoImm")?;
                let base = (bytes[1] >> 4) & 0x0F;
                let dst = bytes[1] & 0x0F;
                let imm1_len = (bytes[2] & 0x0F) as usize;
                if imm1_len > 4 {
                    return Err(Error::Internal(format!(
                        "TwoRegTwoImm first immediate length must be <= 4, got {imm1_len}"
                    )));
                }

                let imm1_end = 3 + imm1_len;
                ensure_min_len(bytes, imm1_end, "TwoRegTwoImm")?;

                let value = decode_imm_signed(&bytes[3..imm1_end], "TwoRegTwoImm value")?;
                let offset = decode_imm_signed(&bytes[imm1_end..], "TwoRegTwoImm offset")?;

                Ok((
                    Self::LoadImmJumpInd {
                        base,
                        dst,
                        value,
                        offset,
                    },
                    bytes.len(),
                ))
            }

            op if op == Opcode::Add32 as u8
                || op == Opcode::Sub32 as u8
                || op == Opcode::Mul32 as u8
                || op == Opcode::DivU32 as u8
                || op == Opcode::DivS32 as u8
                || op == Opcode::RemU32 as u8
                || op == Opcode::RemS32 as u8
                || op == Opcode::ShloL32 as u8
                || op == Opcode::ShloR32 as u8
                || op == Opcode::SharR32 as u8
                || op == Opcode::Add64 as u8
                || op == Opcode::Sub64 as u8
                || op == Opcode::Mul64 as u8
                || op == Opcode::DivU64 as u8
                || op == Opcode::DivS64 as u8
                || op == Opcode::RemU64 as u8
                || op == Opcode::RemS64 as u8
                || op == Opcode::ShloL64 as u8
                || op == Opcode::ShloR64 as u8
                || op == Opcode::SharR64 as u8
                || op == Opcode::SetLtU as u8
                || op == Opcode::SetLtS as u8
                || op == Opcode::CmovIz as u8
                || op == Opcode::CmovNz as u8
                || op == Opcode::And as u8
                || op == Opcode::Xor as u8
                || op == Opcode::Or as u8
                || op == Opcode::MulUpperSS as u8
                || op == Opcode::MulUpperUU as u8
                || op == Opcode::MulUpperSU as u8
                || op == Opcode::RotL64 as u8
                || op == Opcode::RotL32 as u8
                || op == Opcode::RotR64 as u8
                || op == Opcode::RotR32 as u8
                || op == Opcode::AndInv as u8
                || op == Opcode::OrInv as u8
                || op == Opcode::Xnor as u8
                || op == Opcode::Max as u8
                || op == Opcode::MaxU as u8
                || op == Opcode::Min as u8
                || op == Opcode::MinU as u8 =>
            {
                ensure_min_len(bytes, 3, "ThreeReg")?;
                let src1 = bytes[1] & 0x0F;
                let src2 = (bytes[1] >> 4) & 0x0F;
                let dst = bytes[2] & 0x0F;

                let instruction = match op {
                    op if op == Opcode::Add32 as u8 => Self::Add32 { dst, src1, src2 },
                    op if op == Opcode::Sub32 as u8 => Self::Sub32 { dst, src1, src2 },
                    op if op == Opcode::Mul32 as u8 => Self::Mul32 { dst, src1, src2 },
                    op if op == Opcode::DivU32 as u8 => Self::DivU32 { dst, src1, src2 },
                    op if op == Opcode::DivS32 as u8 => Self::DivS32 { dst, src1, src2 },
                    op if op == Opcode::RemU32 as u8 => Self::RemU32 { dst, src1, src2 },
                    op if op == Opcode::RemS32 as u8 => Self::RemS32 { dst, src1, src2 },
                    op if op == Opcode::ShloL32 as u8 => Self::ShloL32 { dst, src1, src2 },
                    op if op == Opcode::ShloR32 as u8 => Self::ShloR32 { dst, src1, src2 },
                    op if op == Opcode::SharR32 as u8 => Self::SharR32 { dst, src1, src2 },
                    op if op == Opcode::Add64 as u8 => Self::Add64 { dst, src1, src2 },
                    op if op == Opcode::Sub64 as u8 => Self::Sub64 { dst, src1, src2 },
                    op if op == Opcode::Mul64 as u8 => Self::Mul64 { dst, src1, src2 },
                    op if op == Opcode::DivU64 as u8 => Self::DivU64 { dst, src1, src2 },
                    op if op == Opcode::DivS64 as u8 => Self::DivS64 { dst, src1, src2 },
                    op if op == Opcode::RemU64 as u8 => Self::RemU64 { dst, src1, src2 },
                    op if op == Opcode::RemS64 as u8 => Self::RemS64 { dst, src1, src2 },
                    op if op == Opcode::ShloL64 as u8 => Self::ShloL64 { dst, src1, src2 },
                    op if op == Opcode::ShloR64 as u8 => Self::ShloR64 { dst, src1, src2 },
                    op if op == Opcode::SharR64 as u8 => Self::SharR64 { dst, src1, src2 },
                    op if op == Opcode::SetLtU as u8 => Self::SetLtU { dst, src1, src2 },
                    op if op == Opcode::SetLtS as u8 => Self::SetLtS { dst, src1, src2 },
                    op if op == Opcode::CmovIz as u8 => Self::CmovIz {
                        dst,
                        src: src1,
                        cond: src2,
                    },
                    op if op == Opcode::CmovNz as u8 => Self::CmovNz {
                        dst,
                        src: src1,
                        cond: src2,
                    },
                    op if op == Opcode::And as u8 => Self::And { dst, src1, src2 },
                    op if op == Opcode::Xor as u8 => Self::Xor { dst, src1, src2 },
                    op if op == Opcode::Or as u8 => Self::Or { dst, src1, src2 },
                    op if op == Opcode::MulUpperSS as u8 => Self::MulUpperSS { dst, src1, src2 },
                    op if op == Opcode::MulUpperUU as u8 => Self::MulUpperUU { dst, src1, src2 },
                    op if op == Opcode::MulUpperSU as u8 => Self::MulUpperSU { dst, src1, src2 },
                    op if op == Opcode::RotL64 as u8 => Self::RotL64 { dst, src1, src2 },
                    op if op == Opcode::RotL32 as u8 => Self::RotL32 { dst, src1, src2 },
                    op if op == Opcode::RotR64 as u8 => Self::RotR64 { dst, src1, src2 },
                    op if op == Opcode::RotR32 as u8 => Self::RotR32 { dst, src1, src2 },
                    op if op == Opcode::AndInv as u8 => Self::AndInv { dst, src1, src2 },
                    op if op == Opcode::OrInv as u8 => Self::OrInv { dst, src1, src2 },
                    op if op == Opcode::Xnor as u8 => Self::Xnor { dst, src1, src2 },
                    op if op == Opcode::Max as u8 => Self::Max { dst, src1, src2 },
                    op if op == Opcode::MaxU as u8 => Self::MaxU { dst, src1, src2 },
                    op if op == Opcode::Min as u8 => Self::Min { dst, src1, src2 },
                    op if op == Opcode::MinU as u8 => Self::MinU { dst, src1, src2 },
                    _ => {
                        return Err(Error::Internal(format!(
                            "ThreeReg decode dispatch mismatch for opcode {op}"
                        )));
                    }
                };

                Ok((instruction, 3))
            }

            _ => Ok((
                Self::Unknown {
                    opcode,
                    raw_bytes: bytes.to_vec(),
                },
                bytes.len(),
            )),
        }
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::Trap => vec![Opcode::Trap as u8],
            Self::Fallthrough => vec![Opcode::Fallthrough as u8],
            Self::LoadImm64 { reg, value } => {
                let mut bytes = vec![Opcode::LoadImm64 as u8, *reg & 0x0F];
                bytes.extend_from_slice(&value.to_le_bytes());
                bytes
            }
            Self::LoadImm { reg, value } => {
                let mut bytes = vec![Opcode::LoadImm as u8, *reg & 0x0F];
                bytes.extend_from_slice(&encode_imm(*value));
                bytes
            }
            Self::Add32 { dst, src1, src2 } => encode_three_reg(Opcode::Add32, *dst, *src1, *src2),
            Self::Sub32 { dst, src1, src2 } => encode_three_reg(Opcode::Sub32, *dst, *src1, *src2),
            Self::Mul32 { dst, src1, src2 } => encode_three_reg(Opcode::Mul32, *dst, *src1, *src2),
            Self::DivU32 { dst, src1, src2 } => {
                encode_three_reg(Opcode::DivU32, *dst, *src1, *src2)
            }
            Self::DivS32 { dst, src1, src2 } => {
                encode_three_reg(Opcode::DivS32, *dst, *src1, *src2)
            }
            Self::RemU32 { dst, src1, src2 } => {
                encode_three_reg(Opcode::RemU32, *dst, *src1, *src2)
            }
            Self::RemS32 { dst, src1, src2 } => {
                encode_three_reg(Opcode::RemS32, *dst, *src1, *src2)
            }
            Self::Add64 { dst, src1, src2 } => encode_three_reg(Opcode::Add64, *dst, *src1, *src2),
            Self::Sub64 { dst, src1, src2 } => encode_three_reg(Opcode::Sub64, *dst, *src1, *src2),
            Self::Mul64 { dst, src1, src2 } => encode_three_reg(Opcode::Mul64, *dst, *src1, *src2),
            Self::DivU64 { dst, src1, src2 } => {
                encode_three_reg(Opcode::DivU64, *dst, *src1, *src2)
            }
            Self::DivS64 { dst, src1, src2 } => {
                encode_three_reg(Opcode::DivS64, *dst, *src1, *src2)
            }
            Self::RemU64 { dst, src1, src2 } => {
                encode_three_reg(Opcode::RemU64, *dst, *src1, *src2)
            }
            Self::RemS64 { dst, src1, src2 } => {
                encode_three_reg(Opcode::RemS64, *dst, *src1, *src2)
            }
            Self::ShloL64 { dst, src1, src2 } => {
                encode_three_reg(Opcode::ShloL64, *dst, *src1, *src2)
            }
            Self::ShloR64 { dst, src1, src2 } => {
                encode_three_reg(Opcode::ShloR64, *dst, *src1, *src2)
            }
            Self::SharR64 { dst, src1, src2 } => {
                encode_three_reg(Opcode::SharR64, *dst, *src1, *src2)
            }
            Self::SetLtU { dst, src1, src2 } => {
                encode_three_reg(Opcode::SetLtU, *dst, *src1, *src2)
            }
            Self::SetLtS { dst, src1, src2 } => {
                encode_three_reg(Opcode::SetLtS, *dst, *src1, *src2)
            }
            Self::CmovIz { dst, src, cond } => encode_three_reg(Opcode::CmovIz, *dst, *src, *cond),
            Self::CmovNz { dst, src, cond } => encode_three_reg(Opcode::CmovNz, *dst, *src, *cond),
            Self::And { dst, src1, src2 } => encode_three_reg(Opcode::And, *dst, *src1, *src2),
            Self::Xor { dst, src1, src2 } => encode_three_reg(Opcode::Xor, *dst, *src1, *src2),
            Self::Or { dst, src1, src2 } => encode_three_reg(Opcode::Or, *dst, *src1, *src2),
            Self::Jump { offset } => {
                let mut bytes = vec![Opcode::Jump as u8];
                bytes.extend_from_slice(&offset.to_le_bytes());
                bytes
            }
            Self::LoadImmJump { reg, value, offset } => {
                encode_one_reg_one_imm_one_off(Opcode::LoadImmJump, *reg, *value, *offset)
            }
            Self::JumpInd { reg, offset } => {
                let mut bytes = vec![Opcode::JumpInd as u8, *reg & 0x0F];
                bytes.extend_from_slice(&encode_imm(*offset));
                bytes
            }
            Self::AddImm32 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::AddImm32, *dst, *src, *value)
            }
            Self::AddImm64 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::AddImm64, *dst, *src, *value)
            }
            Self::AndImm { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::AndImm, *dst, *src, *value)
            }
            Self::XorImm { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::XorImm, *dst, *src, *value)
            }
            Self::OrImm { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::OrImm, *dst, *src, *value)
            }
            Self::MulImm32 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::MulImm32, *dst, *src, *value)
            }
            Self::MulImm64 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::MulImm64, *dst, *src, *value)
            }
            Self::ShloLImm32 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::ShloLImm32, *dst, *src, *value)
            }
            Self::ShloRImm32 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::ShloRImm32, *dst, *src, *value)
            }
            Self::SharRImm32 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::SharRImm32, *dst, *src, *value)
            }
            Self::ShloLImm64 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::ShloLImm64, *dst, *src, *value)
            }
            Self::ShloRImm64 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::ShloRImm64, *dst, *src, *value)
            }
            Self::SharRImm64 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::SharRImm64, *dst, *src, *value)
            }
            Self::NegAddImm32 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::NegAddImm32, *dst, *src, *value)
            }
            Self::NegAddImm64 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::NegAddImm64, *dst, *src, *value)
            }
            Self::SetGtUImm { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::SetGtUImm, *dst, *src, *value)
            }
            Self::SetGtSImm { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::SetGtSImm, *dst, *src, *value)
            }
            Self::LoadIndU32 { dst, base, offset } => {
                let mut bytes = vec![
                    Opcode::LoadIndU32 as u8,
                    (*base & 0x0F) << 4 | (*dst & 0x0F),
                ];
                bytes.extend_from_slice(&encode_imm(*offset));
                bytes
            }
            Self::StoreIndU32 { base, src, offset } => {
                let mut bytes = vec![
                    Opcode::StoreIndU32 as u8,
                    (*base & 0x0F) << 4 | (*src & 0x0F),
                ];
                bytes.extend_from_slice(&encode_imm(*offset));
                bytes
            }
            Self::LoadIndU64 { dst, base, offset } => {
                let mut bytes = vec![
                    Opcode::LoadIndU64 as u8,
                    (*base & 0x0F) << 4 | (*dst & 0x0F),
                ];
                bytes.extend_from_slice(&encode_imm(*offset));
                bytes
            }
            Self::StoreIndU64 { base, src, offset } => {
                let mut bytes = vec![
                    Opcode::StoreIndU64 as u8,
                    (*base & 0x0F) << 4 | (*src & 0x0F),
                ];
                bytes.extend_from_slice(&encode_imm(*offset));
                bytes
            }
            Self::BranchNeImm { reg, value, offset } => {
                encode_one_reg_one_imm_one_off(Opcode::BranchNeImm, *reg, *value, *offset)
            }
            Self::BranchEqImm { reg, value, offset } => {
                encode_one_reg_one_imm_one_off(Opcode::BranchEqImm, *reg, *value, *offset)
            }
            Self::BranchGeSImm { reg, value, offset } => {
                encode_one_reg_one_imm_one_off(Opcode::BranchGeSImm, *reg, *value, *offset)
            }
            Self::BranchLtUImm { reg, value, offset } => {
                encode_one_reg_one_imm_one_off(Opcode::BranchLtUImm, *reg, *value, *offset)
            }
            Self::BranchLeUImm { reg, value, offset } => {
                encode_one_reg_one_imm_one_off(Opcode::BranchLeUImm, *reg, *value, *offset)
            }
            Self::BranchGeUImm { reg, value, offset } => {
                encode_one_reg_one_imm_one_off(Opcode::BranchGeUImm, *reg, *value, *offset)
            }
            Self::BranchGtUImm { reg, value, offset } => {
                encode_one_reg_one_imm_one_off(Opcode::BranchGtUImm, *reg, *value, *offset)
            }
            Self::BranchLtSImm { reg, value, offset } => {
                encode_one_reg_one_imm_one_off(Opcode::BranchLtSImm, *reg, *value, *offset)
            }
            Self::BranchLeSImm { reg, value, offset } => {
                encode_one_reg_one_imm_one_off(Opcode::BranchLeSImm, *reg, *value, *offset)
            }
            Self::BranchGtSImm { reg, value, offset } => {
                encode_one_reg_one_imm_one_off(Opcode::BranchGtSImm, *reg, *value, *offset)
            }
            Self::MoveReg { dst, src } => encode_two_reg(Opcode::MoveReg, *dst, *src),
            Self::BranchEq { reg1, reg2, offset } => {
                encode_two_reg_one_off(Opcode::BranchEq, *reg1, *reg2, *offset)
            }
            Self::BranchNe { reg1, reg2, offset } => {
                encode_two_reg_one_off(Opcode::BranchNe, *reg1, *reg2, *offset)
            }
            Self::BranchGeU { reg1, reg2, offset } => {
                encode_two_reg_one_off(Opcode::BranchGeU, *reg1, *reg2, *offset)
            }
            Self::BranchLtU { reg1, reg2, offset } => {
                encode_two_reg_one_off(Opcode::BranchLtU, *reg1, *reg2, *offset)
            }
            Self::BranchLtS { reg1, reg2, offset } => {
                encode_two_reg_one_off(Opcode::BranchLtS, *reg1, *reg2, *offset)
            }
            Self::BranchGeS { reg1, reg2, offset } => {
                encode_two_reg_one_off(Opcode::BranchGeS, *reg1, *reg2, *offset)
            }
            Self::SetLtUImm { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::SetLtUImm, *dst, *src, *value)
            }
            Self::SetLtSImm { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::SetLtSImm, *dst, *src, *value)
            }
            Self::ShloL32 { dst, src1, src2 } => {
                encode_three_reg(Opcode::ShloL32, *dst, *src1, *src2)
            }
            Self::ShloR32 { dst, src1, src2 } => {
                encode_three_reg(Opcode::ShloR32, *dst, *src1, *src2)
            }
            Self::SharR32 { dst, src1, src2 } => {
                encode_three_reg(Opcode::SharR32, *dst, *src1, *src2)
            }
            Self::Sbrk { dst, src } => encode_two_reg(Opcode::Sbrk, *dst, *src),
            Self::CountSetBits64 { dst, src } => encode_two_reg(Opcode::CountSetBits64, *dst, *src),
            Self::CountSetBits32 { dst, src } => encode_two_reg(Opcode::CountSetBits32, *dst, *src),
            Self::LeadingZeroBits64 { dst, src } => {
                encode_two_reg(Opcode::LeadingZeroBits64, *dst, *src)
            }
            Self::LeadingZeroBits32 { dst, src } => {
                encode_two_reg(Opcode::LeadingZeroBits32, *dst, *src)
            }
            Self::TrailingZeroBits64 { dst, src } => {
                encode_two_reg(Opcode::TrailingZeroBits64, *dst, *src)
            }
            Self::TrailingZeroBits32 { dst, src } => {
                encode_two_reg(Opcode::TrailingZeroBits32, *dst, *src)
            }
            Self::SignExtend8 { dst, src } => encode_two_reg(Opcode::SignExtend8, *dst, *src),
            Self::SignExtend16 { dst, src } => encode_two_reg(Opcode::SignExtend16, *dst, *src),
            Self::ZeroExtend16 { dst, src } => encode_two_reg(Opcode::ZeroExtend16, *dst, *src),
            Self::LoadIndU8 { dst, base, offset } => {
                let mut bytes = vec![Opcode::LoadIndU8 as u8, (*base & 0x0F) << 4 | (*dst & 0x0F)];
                bytes.extend_from_slice(&encode_imm(*offset));
                bytes
            }
            Self::LoadIndI8 { dst, base, offset } => {
                let mut bytes = vec![Opcode::LoadIndI8 as u8, (*base & 0x0F) << 4 | (*dst & 0x0F)];
                bytes.extend_from_slice(&encode_imm(*offset));
                bytes
            }
            Self::StoreIndU8 { base, src, offset } => {
                let mut bytes = vec![
                    Opcode::StoreIndU8 as u8,
                    (*base & 0x0F) << 4 | (*src & 0x0F),
                ];
                bytes.extend_from_slice(&encode_imm(*offset));
                bytes
            }
            Self::LoadIndU16 { dst, base, offset } => {
                let mut bytes = vec![
                    Opcode::LoadIndU16 as u8,
                    (*base & 0x0F) << 4 | (*dst & 0x0F),
                ];
                bytes.extend_from_slice(&encode_imm(*offset));
                bytes
            }
            Self::LoadIndI16 { dst, base, offset } => {
                let mut bytes = vec![
                    Opcode::LoadIndI16 as u8,
                    (*base & 0x0F) << 4 | (*dst & 0x0F),
                ];
                bytes.extend_from_slice(&encode_imm(*offset));
                bytes
            }
            Self::StoreIndU16 { base, src, offset } => {
                let mut bytes = vec![
                    Opcode::StoreIndU16 as u8,
                    (*base & 0x0F) << 4 | (*src & 0x0F),
                ];
                bytes.extend_from_slice(&encode_imm(*offset));
                bytes
            }
            Self::CmovIzImm { dst, cond, value } => {
                let mut bytes = encode_two_reg(Opcode::CmovIzImm, *dst, *cond);
                bytes.extend_from_slice(&encode_imm(*value));
                bytes
            }
            Self::CmovNzImm { dst, cond, value } => {
                let mut bytes = encode_two_reg(Opcode::CmovNzImm, *dst, *cond);
                bytes.extend_from_slice(&encode_imm(*value));
                bytes
            }
            Self::StoreImmU8 { address, value } => {
                encode_two_imm(Opcode::StoreImmU8, *address, *value)
            }
            Self::StoreImmU16 { address, value } => {
                encode_two_imm(Opcode::StoreImmU16, *address, *value)
            }
            Self::StoreImmU32 { address, value } => {
                encode_two_imm(Opcode::StoreImmU32, *address, *value)
            }
            Self::StoreImmU64 { address, value } => {
                encode_two_imm(Opcode::StoreImmU64, *address, *value)
            }
            Self::StoreImmIndU8 {
                base,
                offset,
                value,
            } => encode_one_reg_two_imm(Opcode::StoreImmIndU8, *base, *offset, *value),
            Self::StoreImmIndU16 {
                base,
                offset,
                value,
            } => encode_one_reg_two_imm(Opcode::StoreImmIndU16, *base, *offset, *value),
            Self::StoreImmIndU32 {
                base,
                offset,
                value,
            } => encode_one_reg_two_imm(Opcode::StoreImmIndU32, *base, *offset, *value),
            Self::StoreImmIndU64 {
                base,
                offset,
                value,
            } => encode_one_reg_two_imm(Opcode::StoreImmIndU64, *base, *offset, *value),
            // Load/store absolute (OneRegOneImm encoding)
            Self::LoadU8 { dst, address } => encode_one_reg_one_imm(Opcode::LoadU8, *dst, *address),
            Self::LoadI8 { dst, address } => encode_one_reg_one_imm(Opcode::LoadI8, *dst, *address),
            Self::LoadU16 { dst, address } => {
                encode_one_reg_one_imm(Opcode::LoadU16, *dst, *address)
            }
            Self::LoadI16 { dst, address } => {
                encode_one_reg_one_imm(Opcode::LoadI16, *dst, *address)
            }
            Self::LoadU32 { dst, address } => {
                encode_one_reg_one_imm(Opcode::LoadU32, *dst, *address)
            }
            Self::LoadI32 { dst, address } => {
                encode_one_reg_one_imm(Opcode::LoadI32, *dst, *address)
            }
            Self::LoadU64 { dst, address } => {
                encode_one_reg_one_imm(Opcode::LoadU64, *dst, *address)
            }
            Self::StoreU8 { src, address } => {
                encode_one_reg_one_imm(Opcode::StoreU8, *src, *address)
            }
            Self::StoreU16 { src, address } => {
                encode_one_reg_one_imm(Opcode::StoreU16, *src, *address)
            }
            Self::StoreU32 { src, address } => {
                encode_one_reg_one_imm(Opcode::StoreU32, *src, *address)
            }
            Self::StoreU64 { src, address } => {
                encode_one_reg_one_imm(Opcode::StoreU64, *src, *address)
            }
            Self::LoadIndI32 { dst, base, offset } => {
                let mut bytes = vec![
                    Opcode::LoadIndI32 as u8,
                    (*base & 0x0F) << 4 | (*dst & 0x0F),
                ];
                bytes.extend_from_slice(&encode_imm(*offset));
                bytes
            }
            Self::ReverseBytes { dst, src } => encode_two_reg(Opcode::ReverseBytes, *dst, *src),
            // Alternate shift immediates (TwoRegOneImm)
            Self::ShloLImmAlt32 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::ShloLImmAlt32, *dst, *src, *value)
            }
            Self::ShloRImmAlt32 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::ShloRImmAlt32, *dst, *src, *value)
            }
            Self::SharRImmAlt32 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::SharRImmAlt32, *dst, *src, *value)
            }
            Self::ShloLImmAlt64 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::ShloLImmAlt64, *dst, *src, *value)
            }
            Self::ShloRImmAlt64 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::ShloRImmAlt64, *dst, *src, *value)
            }
            Self::SharRImmAlt64 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::SharRImmAlt64, *dst, *src, *value)
            }
            // Rotate immediates (TwoRegOneImm)
            Self::RotRImm64 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::RotRImm64, *dst, *src, *value)
            }
            Self::RotRImmAlt64 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::RotRImmAlt64, *dst, *src, *value)
            }
            Self::RotRImm32 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::RotRImm32, *dst, *src, *value)
            }
            Self::RotRImmAlt32 { dst, src, value } => {
                encode_two_reg_one_imm(Opcode::RotRImmAlt32, *dst, *src, *value)
            }
            // LoadImmJumpInd (TwoRegTwoImm)
            Self::LoadImmJumpInd {
                base,
                dst,
                value,
                offset,
            } => encode_two_reg_two_imm(Opcode::LoadImmJumpInd, *base, *dst, *value, *offset),
            // Upper multiply (ThreeReg)
            Self::MulUpperSS { dst, src1, src2 } => {
                encode_three_reg(Opcode::MulUpperSS, *dst, *src1, *src2)
            }
            Self::MulUpperUU { dst, src1, src2 } => {
                encode_three_reg(Opcode::MulUpperUU, *dst, *src1, *src2)
            }
            Self::MulUpperSU { dst, src1, src2 } => {
                encode_three_reg(Opcode::MulUpperSU, *dst, *src1, *src2)
            }
            // Rotate (ThreeReg)
            Self::RotL64 { dst, src1, src2 } => {
                encode_three_reg(Opcode::RotL64, *dst, *src1, *src2)
            }
            Self::RotL32 { dst, src1, src2 } => {
                encode_three_reg(Opcode::RotL32, *dst, *src1, *src2)
            }
            Self::RotR64 { dst, src1, src2 } => {
                encode_three_reg(Opcode::RotR64, *dst, *src1, *src2)
            }
            Self::RotR32 { dst, src1, src2 } => {
                encode_three_reg(Opcode::RotR32, *dst, *src1, *src2)
            }
            // Inverted bitwise (ThreeReg)
            Self::AndInv { dst, src1, src2 } => {
                encode_three_reg(Opcode::AndInv, *dst, *src1, *src2)
            }
            Self::OrInv { dst, src1, src2 } => encode_three_reg(Opcode::OrInv, *dst, *src1, *src2),
            Self::Xnor { dst, src1, src2 } => encode_three_reg(Opcode::Xnor, *dst, *src1, *src2),
            // Min/Max (ThreeReg)
            Self::Max { dst, src1, src2 } => encode_three_reg(Opcode::Max, *dst, *src1, *src2),
            Self::MaxU { dst, src1, src2 } => encode_three_reg(Opcode::MaxU, *dst, *src1, *src2),
            Self::Min { dst, src1, src2 } => encode_three_reg(Opcode::Min, *dst, *src1, *src2),
            Self::MinU { dst, src1, src2 } => encode_three_reg(Opcode::MinU, *dst, *src1, *src2),
            Self::Ecalli { index } => {
                let mut bytes = vec![Opcode::Ecalli as u8];
                bytes.extend_from_slice(&encode_uimm(*index));
                bytes
            }
            Self::Unknown { raw_bytes, .. } => raw_bytes.clone(),
        }
    }

    /// Returns the destination register written by this instruction, if any.
    /// Used by the register cache to invalidate stale entries.
    #[must_use]
    pub const fn dest_reg(&self) -> Option<u8> {
        match self {
            Self::Add32 { dst, .. }
            | Self::Add64 { dst, .. }
            | Self::Sub32 { dst, .. }
            | Self::Sub64 { dst, .. }
            | Self::Mul32 { dst, .. }
            | Self::Mul64 { dst, .. }
            | Self::DivU32 { dst, .. }
            | Self::DivS32 { dst, .. }
            | Self::DivU64 { dst, .. }
            | Self::DivS64 { dst, .. }
            | Self::RemU32 { dst, .. }
            | Self::RemS32 { dst, .. }
            | Self::RemU64 { dst, .. }
            | Self::RemS64 { dst, .. }
            | Self::ShloL32 { dst, .. }
            | Self::ShloR32 { dst, .. }
            | Self::SharR32 { dst, .. }
            | Self::ShloL64 { dst, .. }
            | Self::ShloR64 { dst, .. }
            | Self::SharR64 { dst, .. }
            | Self::And { dst, .. }
            | Self::Or { dst, .. }
            | Self::Xor { dst, .. }
            | Self::SetLtU { dst, .. }
            | Self::SetLtS { dst, .. }
            | Self::CmovIz { dst, .. }
            | Self::CmovNz { dst, .. }
            | Self::SetLtUImm { dst, .. }
            | Self::SetLtSImm { dst, .. }
            | Self::CountSetBits32 { dst, .. }
            | Self::CountSetBits64 { dst, .. }
            | Self::LeadingZeroBits32 { dst, .. }
            | Self::LeadingZeroBits64 { dst, .. }
            | Self::TrailingZeroBits32 { dst, .. }
            | Self::TrailingZeroBits64 { dst, .. }
            | Self::SignExtend8 { dst, .. }
            | Self::SignExtend16 { dst, .. }
            | Self::ZeroExtend16 { dst, .. }
            | Self::Sbrk { dst, .. }
            | Self::LoadIndU8 { dst, .. }
            | Self::LoadIndI8 { dst, .. }
            | Self::LoadIndU16 { dst, .. }
            | Self::LoadIndI16 { dst, .. }
            | Self::LoadIndU32 { dst, .. }
            | Self::LoadIndU64 { dst, .. }
            | Self::AddImm32 { dst, .. }
            | Self::AddImm64 { dst, .. }
            | Self::CmovIzImm { dst, .. }
            | Self::CmovNzImm { dst, .. }
            | Self::AndImm { dst, .. }
            | Self::XorImm { dst, .. }
            | Self::OrImm { dst, .. }
            | Self::MulImm32 { dst, .. }
            | Self::MulImm64 { dst, .. }
            | Self::ShloLImm32 { dst, .. }
            | Self::ShloRImm32 { dst, .. }
            | Self::SharRImm32 { dst, .. }
            | Self::ShloLImm64 { dst, .. }
            | Self::ShloRImm64 { dst, .. }
            | Self::SharRImm64 { dst, .. }
            | Self::NegAddImm32 { dst, .. }
            | Self::NegAddImm64 { dst, .. }
            | Self::SetGtUImm { dst, .. }
            | Self::SetGtSImm { dst, .. }
            | Self::LoadIndI32 { dst, .. }
            | Self::ReverseBytes { dst, .. }
            | Self::ShloLImmAlt32 { dst, .. }
            | Self::ShloRImmAlt32 { dst, .. }
            | Self::SharRImmAlt32 { dst, .. }
            | Self::ShloLImmAlt64 { dst, .. }
            | Self::ShloRImmAlt64 { dst, .. }
            | Self::SharRImmAlt64 { dst, .. }
            | Self::RotRImm64 { dst, .. }
            | Self::RotRImmAlt64 { dst, .. }
            | Self::RotRImm32 { dst, .. }
            | Self::RotRImmAlt32 { dst, .. }
            | Self::MulUpperSS { dst, .. }
            | Self::MulUpperUU { dst, .. }
            | Self::MulUpperSU { dst, .. }
            | Self::RotL64 { dst, .. }
            | Self::RotL32 { dst, .. }
            | Self::RotR64 { dst, .. }
            | Self::RotR32 { dst, .. }
            | Self::AndInv { dst, .. }
            | Self::OrInv { dst, .. }
            | Self::Xnor { dst, .. }
            | Self::Max { dst, .. }
            | Self::MaxU { dst, .. }
            | Self::Min { dst, .. }
            | Self::MinU { dst, .. }
            | Self::LoadU8 { dst, .. }
            | Self::LoadI8 { dst, .. }
            | Self::LoadU16 { dst, .. }
            | Self::LoadI16 { dst, .. }
            | Self::LoadU32 { dst, .. }
            | Self::LoadI32 { dst, .. }
            | Self::LoadU64 { dst, .. }
            | Self::LoadImmJumpInd { dst, .. }
            | Self::MoveReg { dst, .. } => Some(*dst),
            // LoadImmJump writes to a register AND jumps, but since it's
            // terminating, the dest_reg is used only by peephole for cache
            // invalidation. We report the register it writes to.
            Self::LoadImm { reg, .. }
            | Self::LoadImm64 { reg, .. }
            | Self::LoadImmJump { reg, .. } => Some(*reg),
            // No destination register:
            Self::Trap
            | Self::Fallthrough
            | Self::Jump { .. }
            | Self::JumpInd { .. }
            | Self::BranchNeImm { .. }
            | Self::BranchEqImm { .. }
            | Self::BranchGeSImm { .. }
            | Self::BranchLtUImm { .. }
            | Self::BranchLeUImm { .. }
            | Self::BranchGeUImm { .. }
            | Self::BranchGtUImm { .. }
            | Self::BranchLtSImm { .. }
            | Self::BranchLeSImm { .. }
            | Self::BranchGtSImm { .. }
            | Self::BranchEq { .. }
            | Self::BranchNe { .. }
            | Self::BranchGeU { .. }
            | Self::BranchLtU { .. }
            | Self::BranchLtS { .. }
            | Self::BranchGeS { .. }
            | Self::StoreIndU8 { .. }
            | Self::StoreIndU16 { .. }
            | Self::StoreIndU32 { .. }
            | Self::StoreIndU64 { .. }
            | Self::StoreU8 { .. }
            | Self::StoreU16 { .. }
            | Self::StoreU32 { .. }
            | Self::StoreU64 { .. }
            | Self::StoreImmU8 { .. }
            | Self::StoreImmU16 { .. }
            | Self::StoreImmU32 { .. }
            | Self::StoreImmU64 { .. }
            | Self::StoreImmIndU8 { .. }
            | Self::StoreImmIndU16 { .. }
            | Self::StoreImmIndU32 { .. }
            | Self::StoreImmIndU64 { .. }
            | Self::Ecalli { .. }
            | Self::Unknown { .. } => None,
        }
    }

    #[must_use]
    pub const fn is_terminating(&self) -> bool {
        matches!(
            self,
            Self::Trap
                | Self::Fallthrough
                | Self::Jump { .. }
                | Self::LoadImmJump { .. }
                | Self::JumpInd { .. }
                | Self::BranchNeImm { .. }
                | Self::BranchEqImm { .. }
                | Self::BranchGeSImm { .. }
                | Self::BranchLtUImm { .. }
                | Self::BranchLeUImm { .. }
                | Self::BranchGeUImm { .. }
                | Self::BranchGtUImm { .. }
                | Self::BranchLtSImm { .. }
                | Self::BranchLeSImm { .. }
                | Self::BranchGtSImm { .. }
                | Self::BranchEq { .. }
                | Self::BranchNe { .. }
                | Self::BranchGeU { .. }
                | Self::BranchLtU { .. }
                | Self::BranchLtS { .. }
                | Self::BranchGeS { .. }
                | Self::LoadImmJumpInd { .. }
        )
    }

    /// Returns the source registers read by this instruction.
    /// Used by dead code elimination to determine liveness.
    /// Returns up to 3 registers (most instructions use 0-2, some 3).
    #[must_use]
    pub const fn src_regs(&self) -> [Option<u8>; 3] {
        match self {
            Self::Trap
            | Self::Fallthrough
            | Self::Jump { .. }
            | Self::Ecalli { .. }
            | Self::Unknown { .. }
            | Self::LoadImm { .. }
            | Self::LoadImm64 { .. }
            | Self::LoadImmJump { .. }
            | Self::StoreImmU8 { .. }
            | Self::StoreImmU16 { .. }
            | Self::StoreImmU32 { .. }
            | Self::StoreImmU64 { .. }
            | Self::LoadU8 { .. }
            | Self::LoadI8 { .. }
            | Self::LoadU16 { .. }
            | Self::LoadI16 { .. }
            | Self::LoadU32 { .. }
            | Self::LoadI32 { .. }
            | Self::LoadU64 { .. } => [None, None, None],

            Self::MoveReg { src, .. }
            | Self::Sbrk { src, .. }
            | Self::CountSetBits64 { src, .. }
            | Self::CountSetBits32 { src, .. }
            | Self::LeadingZeroBits64 { src, .. }
            | Self::LeadingZeroBits32 { src, .. }
            | Self::TrailingZeroBits64 { src, .. }
            | Self::TrailingZeroBits32 { src, .. }
            | Self::SignExtend8 { src, .. }
            | Self::SignExtend16 { src, .. }
            | Self::ZeroExtend16 { src, .. }
            | Self::JumpInd { reg: src, .. }
            | Self::LoadImmJumpInd { base: src, .. }
            | Self::LoadIndU8 { base: src, .. }
            | Self::LoadIndI8 { base: src, .. }
            | Self::LoadIndU16 { base: src, .. }
            | Self::LoadIndI16 { base: src, .. }
            | Self::LoadIndU32 { base: src, .. }
            | Self::LoadIndI32 { base: src, .. }
            | Self::LoadIndU64 { base: src, .. }
            | Self::StoreImmIndU8 { base: src, .. }
            | Self::StoreImmIndU16 { base: src, .. }
            | Self::StoreImmIndU32 { base: src, .. }
            | Self::StoreImmIndU64 { base: src, .. }
            | Self::BranchEqImm { reg: src, .. }
            | Self::BranchNeImm { reg: src, .. }
            | Self::BranchLtUImm { reg: src, .. }
            | Self::BranchLeUImm { reg: src, .. }
            | Self::BranchGeUImm { reg: src, .. }
            | Self::BranchGtUImm { reg: src, .. }
            | Self::BranchLtSImm { reg: src, .. }
            | Self::BranchLeSImm { reg: src, .. }
            | Self::BranchGeSImm { reg: src, .. }
            | Self::BranchGtSImm { reg: src, .. }
            | Self::AddImm32 { src, .. }
            | Self::AddImm64 { src, .. }
            | Self::SetLtUImm { src, .. }
            | Self::SetLtSImm { src, .. }
            | Self::AndImm { src, .. }
            | Self::XorImm { src, .. }
            | Self::OrImm { src, .. }
            | Self::MulImm32 { src, .. }
            | Self::MulImm64 { src, .. }
            | Self::ShloLImm32 { src, .. }
            | Self::ShloRImm32 { src, .. }
            | Self::SharRImm32 { src, .. }
            | Self::ShloLImm64 { src, .. }
            | Self::ShloRImm64 { src, .. }
            | Self::SharRImm64 { src, .. }
            | Self::NegAddImm32 { src, .. }
            | Self::NegAddImm64 { src, .. }
            | Self::SetGtUImm { src, .. }
            | Self::SetGtSImm { src, .. }
            | Self::StoreU8 { src, .. }
            | Self::StoreU16 { src, .. }
            | Self::StoreU32 { src, .. }
            | Self::StoreU64 { src, .. }
            | Self::ReverseBytes { src, .. }
            | Self::ShloLImmAlt32 { src, .. }
            | Self::ShloRImmAlt32 { src, .. }
            | Self::SharRImmAlt32 { src, .. }
            | Self::ShloLImmAlt64 { src, .. }
            | Self::ShloRImmAlt64 { src, .. }
            | Self::SharRImmAlt64 { src, .. }
            | Self::RotRImm64 { src, .. }
            | Self::RotRImmAlt64 { src, .. }
            | Self::RotRImm32 { src, .. }
            | Self::RotRImmAlt32 { src, .. } => [Some(*src), None, None],

            Self::StoreIndU8 { base, src, .. }
            | Self::StoreIndU16 { base, src, .. }
            | Self::StoreIndU32 { base, src, .. }
            | Self::StoreIndU64 { base, src, .. }
            | Self::BranchEq {
                reg1: base,
                reg2: src,
                ..
            }
            | Self::BranchNe {
                reg1: base,
                reg2: src,
                ..
            }
            | Self::BranchLtU {
                reg1: base,
                reg2: src,
                ..
            }
            | Self::BranchLtS {
                reg1: base,
                reg2: src,
                ..
            }
            | Self::BranchGeU {
                reg1: base,
                reg2: src,
                ..
            }
            | Self::BranchGeS {
                reg1: base,
                reg2: src,
                ..
            }
            | Self::CmovIz {
                src: base,
                cond: src,
                ..
            }
            | Self::CmovNz {
                src: base,
                cond: src,
                ..
            } => [Some(*base), Some(*src), None],

            Self::Add32 { src1, src2, .. }
            | Self::Sub32 { src1, src2, .. }
            | Self::Mul32 { src1, src2, .. }
            | Self::DivU32 { src1, src2, .. }
            | Self::DivS32 { src1, src2, .. }
            | Self::RemU32 { src1, src2, .. }
            | Self::RemS32 { src1, src2, .. }
            | Self::Add64 { src1, src2, .. }
            | Self::Sub64 { src1, src2, .. }
            | Self::Mul64 { src1, src2, .. }
            | Self::DivU64 { src1, src2, .. }
            | Self::DivS64 { src1, src2, .. }
            | Self::RemU64 { src1, src2, .. }
            | Self::RemS64 { src1, src2, .. }
            | Self::ShloL64 { src1, src2, .. }
            | Self::ShloR64 { src1, src2, .. }
            | Self::SharR64 { src1, src2, .. }
            | Self::SetLtU { src1, src2, .. }
            | Self::SetLtS { src1, src2, .. }
            | Self::And { src1, src2, .. }
            | Self::Xor { src1, src2, .. }
            | Self::Or { src1, src2, .. }
            | Self::ShloL32 { src1, src2, .. }
            | Self::ShloR32 { src1, src2, .. }
            | Self::SharR32 { src1, src2, .. }
            | Self::MulUpperSS { src1, src2, .. }
            | Self::MulUpperUU { src1, src2, .. }
            | Self::MulUpperSU { src1, src2, .. }
            | Self::RotL64 { src1, src2, .. }
            | Self::RotL32 { src1, src2, .. }
            | Self::RotR64 { src1, src2, .. }
            | Self::RotR32 { src1, src2, .. }
            | Self::AndInv { src1, src2, .. }
            | Self::OrInv { src1, src2, .. }
            | Self::Xnor { src1, src2, .. }
            | Self::Max { src1, src2, .. }
            | Self::MaxU { src1, src2, .. }
            | Self::Min { src1, src2, .. }
            | Self::MinU { src1, src2, .. } => [Some(*src1), Some(*src2), None],

            // CmovIzImm/CmovNzImm read from the cond register only (value is an immediate).
            Self::CmovIzImm { cond, .. } | Self::CmovNzImm { cond, .. } => {
                [Some(*cond), None, None]
            }
        }
    }
}

fn encode_three_reg(opcode: Opcode, dst: u8, src1: u8, src2: u8) -> Vec<u8> {
    // PVM three-reg encoding: [opcode, rB_hi | rA_lo, rD]
    // Semantics: reg[rD] = reg[rA] OP reg[rB]
    // We want: reg[dst] = reg[src1] OP reg[src2], so rA=src1, rB=src2
    vec![opcode as u8, (src2 & 0x0F) << 4 | (src1 & 0x0F), dst & 0x0F]
}

fn encode_two_reg_one_imm(opcode: Opcode, dst: u8, src: u8, value: i32) -> Vec<u8> {
    let mut bytes = vec![opcode as u8, (src & 0x0F) << 4 | (dst & 0x0F)];
    bytes.extend_from_slice(&encode_imm(value));
    bytes
}

fn encode_two_reg(opcode: Opcode, dst: u8, src: u8) -> Vec<u8> {
    vec![opcode as u8, (src & 0x0F) << 4 | (dst & 0x0F)]
}

fn encode_one_reg_one_imm_one_off(opcode: Opcode, reg: u8, imm: i32, offset: i32) -> Vec<u8> {
    let imm_enc = encode_imm(imm);
    let imm_len = imm_enc.len() as u8;
    let mut bytes = vec![opcode as u8, (imm_len << 4) | (reg & 0x0F)];
    bytes.extend_from_slice(&imm_enc);
    bytes.extend_from_slice(&offset.to_le_bytes());
    bytes
}

fn encode_one_reg_two_imm(opcode: Opcode, reg: u8, imm1: i32, imm2: i32) -> Vec<u8> {
    let imm1_enc = encode_imm(imm1);
    let imm1_len = imm1_enc.len() as u8;
    let imm2_enc = encode_imm(imm2);
    let mut bytes = vec![opcode as u8, (imm1_len << 4) | (reg & 0x0F)];
    bytes.extend_from_slice(&imm1_enc);
    bytes.extend_from_slice(&imm2_enc);
    bytes
}

fn encode_one_reg_one_imm(opcode: Opcode, reg: u8, value: i32) -> Vec<u8> {
    let mut bytes = vec![opcode as u8, reg & 0x0F];
    bytes.extend_from_slice(&encode_imm(value));
    bytes
}

fn encode_two_reg_two_imm(opcode: Opcode, reg1: u8, reg2: u8, imm1: i32, imm2: i32) -> Vec<u8> {
    let imm1_enc = encode_imm(imm1);
    let imm1_len = imm1_enc.len() as u8;
    let imm2_enc = encode_imm(imm2);
    let mut bytes = vec![
        opcode as u8,
        (reg1 & 0x0F) << 4 | (reg2 & 0x0F),
        imm1_len & 0x0F,
    ];
    bytes.extend_from_slice(&imm1_enc);
    bytes.extend_from_slice(&imm2_enc);
    bytes
}

fn encode_two_reg_one_off(opcode: Opcode, reg1: u8, reg2: u8, offset: i32) -> Vec<u8> {
    let mut bytes = vec![opcode as u8, (reg1 & 0x0F) << 4 | (reg2 & 0x0F)];
    bytes.extend_from_slice(&offset.to_le_bytes());
    bytes
}

fn encode_two_imm(opcode: Opcode, imm1: i32, imm2: i32) -> Vec<u8> {
    let imm1_enc = encode_imm(imm1);
    let imm1_len = imm1_enc.len() as u8;
    let imm2_enc = encode_imm(imm2);
    let mut bytes = vec![opcode as u8, imm1_len & 0x0F];
    bytes.extend_from_slice(&imm1_enc);
    bytes.extend_from_slice(&imm2_enc);
    bytes
}

fn encode_uimm(value: u32) -> Vec<u8> {
    let bytes = value.to_le_bytes();
    let len = if value == 0 {
        0
    } else if value <= 0xFF {
        1
    } else if value <= 0xFFFF {
        2
    } else if value <= 0xFF_FFFF {
        3
    } else {
        4
    };
    bytes[..len].to_vec()
}

fn encode_imm(value: i32) -> Vec<u8> {
    let bytes = value.to_le_bytes();
    let len = if value == 0 {
        0
    } else if (-128..=127).contains(&value) {
        1
    } else if (-32768..=32767).contains(&value) {
        2
    } else if (-8_388_608..=8_388_607).contains(&value) {
        3
    } else {
        4
    };
    bytes[..len].to_vec()
}

fn ensure_min_len(bytes: &[u8], min_len: usize, context: &str) -> Result<()> {
    if bytes.len() < min_len {
        return Err(Error::Internal(format!(
            "{context}: expected at least {min_len} bytes, got {}",
            bytes.len()
        )));
    }
    Ok(())
}

fn decode_offset_at(bytes: &[u8], start: usize, context: &str) -> Result<i32> {
    let end = start
        .checked_add(4)
        .ok_or_else(|| Error::Internal(format!("{context}: offset index overflow")))?;
    ensure_min_len(bytes, end, context)?;
    let mut raw = [0u8; 4];
    raw.copy_from_slice(&bytes[start..end]);
    Ok(i32::from_le_bytes(raw))
}

fn decode_imm_signed(bytes: &[u8], context: &str) -> Result<i32> {
    if bytes.len() > 4 {
        return Err(Error::Internal(format!(
            "{context}: signed immediate must be 0-4 bytes, got {}",
            bytes.len()
        )));
    }
    if bytes.is_empty() {
        return Ok(0);
    }

    let fill = if bytes[bytes.len() - 1] & 0x80 != 0 {
        0xFF
    } else {
        0x00
    };
    let mut raw = [fill; 4];
    raw[..bytes.len()].copy_from_slice(bytes);
    Ok(i32::from_le_bytes(raw))
}

fn decode_imm_unsigned(bytes: &[u8], context: &str) -> Result<u32> {
    if bytes.len() > 4 {
        return Err(Error::Internal(format!(
            "{context}: unsigned immediate must be 0-4 bytes, got {}",
            bytes.len()
        )));
    }

    let mut raw = [0u8; 4];
    raw[..bytes.len()].copy_from_slice(bytes);
    Ok(u32::from_le_bytes(raw))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_decode_roundtrip(instr: &Instruction) {
        let encoded = instr.encode();
        let (decoded, consumed) = Instruction::decode(&encoded).expect("decode should succeed");
        assert_eq!(
            consumed,
            encoded.len(),
            "wrong consumed length for {instr:?}"
        );
        assert_eq!(&decoded, instr, "decode(encode(instr)) mismatch");
    }

    fn sample_instructions() -> Vec<Instruction> {
        vec![
            Instruction::Trap,
            Instruction::Fallthrough,
            Instruction::LoadImm64 {
                reg: 12,
                value: 0x1234_5678_9ABC_DEF0,
            },
            Instruction::LoadImm {
                reg: 0,
                value: -12_345,
            },
            Instruction::Add32 {
                dst: 12,
                src1: 0,
                src2: 11,
            },
            Instruction::Sub32 {
                dst: 3,
                src1: 2,
                src2: 1,
            },
            Instruction::Mul32 {
                dst: 4,
                src1: 5,
                src2: 6,
            },
            Instruction::DivU32 {
                dst: 7,
                src1: 8,
                src2: 9,
            },
            Instruction::DivS32 {
                dst: 1,
                src1: 10,
                src2: 11,
            },
            Instruction::RemU32 {
                dst: 2,
                src1: 3,
                src2: 4,
            },
            Instruction::RemS32 {
                dst: 5,
                src1: 6,
                src2: 7,
            },
            Instruction::Add64 {
                dst: 8,
                src1: 9,
                src2: 10,
            },
            Instruction::Sub64 {
                dst: 11,
                src1: 12,
                src2: 0,
            },
            Instruction::Mul64 {
                dst: 1,
                src1: 2,
                src2: 3,
            },
            Instruction::DivU64 {
                dst: 4,
                src1: 5,
                src2: 6,
            },
            Instruction::DivS64 {
                dst: 7,
                src1: 8,
                src2: 9,
            },
            Instruction::RemU64 {
                dst: 10,
                src1: 11,
                src2: 12,
            },
            Instruction::RemS64 {
                dst: 0,
                src1: 1,
                src2: 2,
            },
            Instruction::ShloL64 {
                dst: 3,
                src1: 4,
                src2: 5,
            },
            Instruction::ShloR64 {
                dst: 6,
                src1: 7,
                src2: 8,
            },
            Instruction::SharR64 {
                dst: 9,
                src1: 10,
                src2: 11,
            },
            Instruction::AddImm32 {
                dst: 12,
                src: 0,
                value: i32::MIN,
            },
            Instruction::AddImm64 {
                dst: 1,
                src: 2,
                value: i32::MAX,
            },
            Instruction::AndImm {
                dst: 3,
                src: 4,
                value: 0x7FFF,
            },
            Instruction::XorImm {
                dst: 5,
                src: 6,
                value: -1,
            },
            Instruction::OrImm {
                dst: 7,
                src: 8,
                value: 0,
            },
            Instruction::MulImm32 {
                dst: 9,
                src: 10,
                value: 42,
            },
            Instruction::MulImm64 {
                dst: 11,
                src: 12,
                value: -128,
            },
            Instruction::ShloLImm32 {
                dst: 0,
                src: 1,
                value: 3,
            },
            Instruction::ShloRImm32 {
                dst: 2,
                src: 3,
                value: 4,
            },
            Instruction::SharRImm32 {
                dst: 4,
                src: 5,
                value: 5,
            },
            Instruction::ShloLImm64 {
                dst: 6,
                src: 7,
                value: 6,
            },
            Instruction::ShloRImm64 {
                dst: 8,
                src: 9,
                value: 7,
            },
            Instruction::SharRImm64 {
                dst: 10,
                src: 11,
                value: 8,
            },
            Instruction::NegAddImm32 {
                dst: 12,
                src: 0,
                value: -9,
            },
            Instruction::NegAddImm64 {
                dst: 1,
                src: 2,
                value: 10,
            },
            Instruction::SetGtUImm {
                dst: 3,
                src: 4,
                value: 11,
            },
            Instruction::SetGtSImm {
                dst: 5,
                src: 6,
                value: -12,
            },
            Instruction::Jump { offset: -13 },
            Instruction::LoadImmJump {
                reg: 0,
                value: 14,
                offset: -15,
            },
            Instruction::JumpInd {
                reg: 12,
                offset: 16,
            },
            Instruction::LoadIndU32 {
                dst: 1,
                base: 2,
                offset: -17,
            },
            Instruction::StoreIndU32 {
                base: 3,
                src: 4,
                offset: 18,
            },
            Instruction::LoadIndU64 {
                dst: 5,
                base: 6,
                offset: -19,
            },
            Instruction::StoreIndU64 {
                base: 7,
                src: 8,
                offset: 20,
            },
            Instruction::BranchNeImm {
                reg: 9,
                value: -21,
                offset: 22,
            },
            Instruction::BranchEqImm {
                reg: 10,
                value: 23,
                offset: -24,
            },
            Instruction::BranchGeSImm {
                reg: 11,
                value: -25,
                offset: 26,
            },
            Instruction::BranchLtUImm {
                reg: 12,
                value: 27,
                offset: -28,
            },
            Instruction::BranchLeUImm {
                reg: 0,
                value: -29,
                offset: 30,
            },
            Instruction::BranchGeUImm {
                reg: 1,
                value: 31,
                offset: -32,
            },
            Instruction::BranchGtUImm {
                reg: 2,
                value: -33,
                offset: 34,
            },
            Instruction::BranchLtSImm {
                reg: 3,
                value: 35,
                offset: -36,
            },
            Instruction::BranchLeSImm {
                reg: 4,
                value: -37,
                offset: 38,
            },
            Instruction::BranchGtSImm {
                reg: 5,
                value: 39,
                offset: -40,
            },
            Instruction::MoveReg { dst: 6, src: 7 },
            Instruction::BranchEq {
                reg1: 8,
                reg2: 9,
                offset: -41,
            },
            Instruction::BranchNe {
                reg1: 10,
                reg2: 11,
                offset: 42,
            },
            Instruction::BranchGeU {
                reg1: 12,
                reg2: 0,
                offset: -43,
            },
            Instruction::BranchLtU {
                reg1: 1,
                reg2: 2,
                offset: 44,
            },
            Instruction::BranchLtS {
                reg1: 3,
                reg2: 4,
                offset: -45,
            },
            Instruction::BranchGeS {
                reg1: 5,
                reg2: 6,
                offset: 46,
            },
            Instruction::SetLtU {
                dst: 7,
                src1: 8,
                src2: 9,
            },
            Instruction::SetLtS {
                dst: 10,
                src1: 11,
                src2: 12,
            },
            Instruction::CmovIz {
                dst: 0,
                src: 1,
                cond: 2,
            },
            Instruction::CmovNz {
                dst: 3,
                src: 4,
                cond: 5,
            },
            Instruction::And {
                dst: 6,
                src1: 7,
                src2: 8,
            },
            Instruction::Xor {
                dst: 9,
                src1: 10,
                src2: 11,
            },
            Instruction::Or {
                dst: 12,
                src1: 0,
                src2: 1,
            },
            Instruction::SetLtUImm {
                dst: 2,
                src: 3,
                value: -47,
            },
            Instruction::SetLtSImm {
                dst: 4,
                src: 5,
                value: 48,
            },
            Instruction::ShloL32 {
                dst: 6,
                src1: 7,
                src2: 8,
            },
            Instruction::ShloR32 {
                dst: 9,
                src1: 10,
                src2: 11,
            },
            Instruction::SharR32 {
                dst: 12,
                src1: 0,
                src2: 1,
            },
            Instruction::Sbrk { dst: 2, src: 3 },
            Instruction::CountSetBits64 { dst: 4, src: 5 },
            Instruction::CountSetBits32 { dst: 6, src: 7 },
            Instruction::LeadingZeroBits64 { dst: 8, src: 9 },
            Instruction::LeadingZeroBits32 { dst: 10, src: 11 },
            Instruction::TrailingZeroBits64 { dst: 12, src: 0 },
            Instruction::TrailingZeroBits32 { dst: 1, src: 2 },
            Instruction::SignExtend8 { dst: 3, src: 4 },
            Instruction::SignExtend16 { dst: 5, src: 6 },
            Instruction::ZeroExtend16 { dst: 7, src: 8 },
            Instruction::LoadIndU8 {
                dst: 9,
                base: 10,
                offset: -49,
            },
            Instruction::LoadIndI8 {
                dst: 11,
                base: 12,
                offset: 50,
            },
            Instruction::StoreIndU8 {
                base: 0,
                src: 1,
                offset: -51,
            },
            Instruction::LoadIndU16 {
                dst: 2,
                base: 3,
                offset: 52,
            },
            Instruction::LoadIndI16 {
                dst: 4,
                base: 5,
                offset: -53,
            },
            Instruction::StoreIndU16 {
                base: 6,
                src: 7,
                offset: 54,
            },
            Instruction::CmovIzImm {
                dst: 8,
                cond: 9,
                value: -55,
            },
            Instruction::CmovNzImm {
                dst: 10,
                cond: 11,
                value: 56,
            },
            Instruction::StoreImmU8 {
                address: -57,
                value: 58,
            },
            Instruction::StoreImmU16 {
                address: 59,
                value: -60,
            },
            Instruction::StoreImmU32 {
                address: -61,
                value: 62,
            },
            Instruction::StoreImmU64 {
                address: 63,
                value: -64,
            },
            Instruction::StoreImmIndU8 {
                base: 12,
                offset: -65,
                value: 66,
            },
            Instruction::StoreImmIndU16 {
                base: 0,
                offset: 67,
                value: -68,
            },
            Instruction::StoreImmIndU32 {
                base: 1,
                offset: -69,
                value: 70,
            },
            Instruction::StoreImmIndU64 {
                base: 2,
                offset: 71,
                value: -72,
            },
            Instruction::LoadU8 {
                dst: 3,
                address: -73,
            },
            Instruction::LoadI8 {
                dst: 4,
                address: 74,
            },
            Instruction::LoadU16 {
                dst: 5,
                address: -75,
            },
            Instruction::LoadI16 {
                dst: 6,
                address: 76,
            },
            Instruction::LoadU32 {
                dst: 7,
                address: -77,
            },
            Instruction::LoadI32 {
                dst: 8,
                address: 78,
            },
            Instruction::LoadU64 {
                dst: 9,
                address: -79,
            },
            Instruction::StoreU8 {
                src: 10,
                address: 80,
            },
            Instruction::StoreU16 {
                src: 11,
                address: -81,
            },
            Instruction::StoreU32 {
                src: 12,
                address: 82,
            },
            Instruction::StoreU64 {
                src: 0,
                address: -83,
            },
            Instruction::LoadIndI32 {
                dst: 1,
                base: 2,
                offset: 84,
            },
            Instruction::ReverseBytes { dst: 3, src: 4 },
            Instruction::ShloLImmAlt32 {
                dst: 5,
                src: 6,
                value: -85,
            },
            Instruction::ShloRImmAlt32 {
                dst: 7,
                src: 8,
                value: 86,
            },
            Instruction::SharRImmAlt32 {
                dst: 9,
                src: 10,
                value: -87,
            },
            Instruction::ShloLImmAlt64 {
                dst: 11,
                src: 12,
                value: 88,
            },
            Instruction::ShloRImmAlt64 {
                dst: 0,
                src: 1,
                value: -89,
            },
            Instruction::SharRImmAlt64 {
                dst: 2,
                src: 3,
                value: 90,
            },
            Instruction::RotRImm64 {
                dst: 4,
                src: 5,
                value: -91,
            },
            Instruction::RotRImmAlt64 {
                dst: 6,
                src: 7,
                value: 92,
            },
            Instruction::RotRImm32 {
                dst: 8,
                src: 9,
                value: -93,
            },
            Instruction::RotRImmAlt32 {
                dst: 10,
                src: 11,
                value: 94,
            },
            Instruction::LoadImmJumpInd {
                base: 12,
                dst: 0,
                value: -95,
                offset: 96,
            },
            Instruction::MulUpperSS {
                dst: 1,
                src1: 2,
                src2: 3,
            },
            Instruction::MulUpperUU {
                dst: 4,
                src1: 5,
                src2: 6,
            },
            Instruction::MulUpperSU {
                dst: 7,
                src1: 8,
                src2: 9,
            },
            Instruction::RotL64 {
                dst: 10,
                src1: 11,
                src2: 12,
            },
            Instruction::RotL32 {
                dst: 0,
                src1: 1,
                src2: 2,
            },
            Instruction::RotR64 {
                dst: 3,
                src1: 4,
                src2: 5,
            },
            Instruction::RotR32 {
                dst: 6,
                src1: 7,
                src2: 8,
            },
            Instruction::AndInv {
                dst: 9,
                src1: 10,
                src2: 11,
            },
            Instruction::OrInv {
                dst: 12,
                src1: 0,
                src2: 1,
            },
            Instruction::Xnor {
                dst: 2,
                src1: 3,
                src2: 4,
            },
            Instruction::Max {
                dst: 5,
                src1: 6,
                src2: 7,
            },
            Instruction::MaxU {
                dst: 8,
                src1: 9,
                src2: 10,
            },
            Instruction::Min {
                dst: 11,
                src1: 12,
                src2: 0,
            },
            Instruction::MinU {
                dst: 1,
                src1: 2,
                src2: 3,
            },
            Instruction::Ecalli { index: 0xDEAD_BEEF },
            Instruction::Unknown {
                opcode: 0xFE,
                raw_bytes: vec![0xFE, 0xAA, 0x55],
            },
        ]
    }

    #[test]
    fn test_decode_roundtrip_all_variants() {
        for instr in sample_instructions() {
            assert_decode_roundtrip(&instr);
        }
    }

    #[test]
    fn test_decode_roundtrip_immediate_edges() {
        let edge_values = [
            i32::MIN,
            -8_388_608,
            -32_768,
            -128,
            -1,
            0,
            1,
            127,
            32_767,
            8_388_607,
            i32::MAX,
        ];

        for &value in &edge_values {
            assert_decode_roundtrip(&Instruction::LoadImm { reg: 0, value });
            assert_decode_roundtrip(&Instruction::AddImm32 {
                dst: 12,
                src: 0,
                value,
            });
            assert_decode_roundtrip(&Instruction::LoadU32 {
                dst: 12,
                address: value,
            });
            assert_decode_roundtrip(&Instruction::StoreImmU32 {
                address: value,
                value,
            });
            assert_decode_roundtrip(&Instruction::StoreImmIndU32 {
                base: 12,
                offset: value,
                value,
            });
            assert_decode_roundtrip(&Instruction::LoadImmJump {
                reg: 0,
                value,
                offset: value,
            });
            assert_decode_roundtrip(&Instruction::LoadImmJumpInd {
                base: 12,
                dst: 0,
                value,
                offset: value,
            });
        }

        for index in [0u32, 1, 0xFF, 0x100, 0xFFFF, 0x1_0000, 0xFF_FFFF, u32::MAX] {
            assert_decode_roundtrip(&Instruction::Ecalli { index });
        }
    }

    #[test]
    fn test_decode_rejects_empty_slice() {
        let error = Instruction::decode(&[]).expect_err("empty input should fail");
        assert!(matches!(error, Error::Internal(_)));
    }

    #[test]
    fn test_three_reg_encoding() {
        // Test Add32 with dst=3, src1=1, src2=2
        // Expected: [opcode, (src2 << 4) | src1, dst] = [opcode, 0x21, 0x03]
        let instr = Instruction::Add32 {
            dst: 3,
            src1: 1,
            src2: 2,
        };
        let encoded = instr.encode();
        assert_eq!(encoded[0], Opcode::Add32 as u8);
        assert_eq!(encoded[1], 0x21); // src2=2 in high nibble, src1=1 in low nibble
        assert_eq!(encoded[2], 0x03); // dst
    }

    #[test]
    fn test_three_reg_encoding_symmetric() {
        // Verify src1 and src2 are distinguishable (not commutative in encoding)
        let instr1 = Instruction::Sub32 {
            dst: 0,
            src1: 5,
            src2: 7,
        };
        let instr2 = Instruction::Sub32 {
            dst: 0,
            src1: 7,
            src2: 5,
        };
        assert_ne!(instr1.encode(), instr2.encode());
    }

    #[test]
    fn test_three_reg_encoding_edge_registers() {
        // Test with register 0 and register 12 (max used in PVM)
        let instr = Instruction::Add64 {
            dst: 12,
            src1: 0,
            src2: 12,
        };
        let encoded = instr.encode();
        assert_eq!(encoded[0], Opcode::Add64 as u8);
        assert_eq!(encoded[1], 0xC0); // src2=12 in high nibble, src1=0 in low nibble
        assert_eq!(encoded[2], 0x0C); // dst=12
    }

    #[test]
    fn test_cmov_imm_encoding() {
        // CmovNzImm with cond=3, dst=5, value=42
        let instr = Instruction::CmovNzImm {
            cond: 3,
            dst: 5,
            value: 42,
        };
        let encoded = instr.encode();
        assert_eq!(encoded[0], Opcode::CmovNzImm as u8);
        assert_eq!(encoded[1], 0x35); // cond=3 in high nibble, dst=5 in low nibble
        assert_eq!(encoded[2], 42); // value as 1-byte immediate
    }

    #[test]
    fn test_cmov_iz_imm_encoding() {
        // CmovIzImm with cond=0, dst=7, value=0
        let instr = Instruction::CmovIzImm {
            cond: 0,
            dst: 7,
            value: 0,
        };
        let encoded = instr.encode();
        assert_eq!(encoded[0], Opcode::CmovIzImm as u8);
        assert_eq!(encoded[1], 0x07); // cond=0 in high nibble, dst=7 in low nibble
        assert_eq!(encoded.len(), 2); // value=0 → no immediate bytes
    }

    #[test]
    fn test_cmov_imm_roundtrip() {
        // Roundtrip: encode CmovIzImm/CmovNzImm then decode manually and verify field values.
        for (dst, cond, value) in [(0u8, 5u8, 0i32), (7, 3, 42), (12, 1, -1), (2, 9, 8_388_607)] {
            for (opcode_byte, is_iz) in [
                (Opcode::CmovIzImm as u8, true),
                (Opcode::CmovNzImm as u8, false),
            ] {
                let instr = if is_iz {
                    Instruction::CmovIzImm { dst, cond, value }
                } else {
                    Instruction::CmovNzImm { dst, cond, value }
                };
                let encoded = instr.encode();

                // Verify opcode byte
                assert_eq!(
                    encoded[0], opcode_byte,
                    "opcode mismatch for dst={dst} cond={cond} value={value}"
                );

                // Decode the packed nibble byte: cond in high nibble, dst in low nibble
                let nibble_byte = encoded[1];
                let decoded_cond = (nibble_byte >> 4) & 0x0F;
                let decoded_dst = nibble_byte & 0x0F;
                assert_eq!(
                    decoded_dst, dst,
                    "dst mismatch for dst={dst} cond={cond} value={value}"
                );
                assert_eq!(
                    decoded_cond, cond,
                    "cond mismatch for dst={dst} cond={cond} value={value}"
                );

                // Decode the immediate (sign-extend from however many bytes were written)
                let imm_bytes = &encoded[2..];
                let mut buf = [0u8; 4];
                buf[..imm_bytes.len()].copy_from_slice(imm_bytes);
                // Sign-extend: if top bit of last written byte is set, fill with 0xFF
                if let Some(&last) = imm_bytes.last()
                    && last & 0x80 != 0
                {
                    for b in buf.iter_mut().skip(imm_bytes.len()) {
                        *b = 0xFF;
                    }
                }
                let decoded_value = i32::from_le_bytes(buf);
                assert_eq!(
                    decoded_value, value,
                    "value mismatch for dst={dst} cond={cond} value={value}"
                );
            }
        }
    }

    #[test]
    fn test_two_imm_encoding() {
        // StoreImmU32 with address=0x30000 and value=42
        let instr = Instruction::StoreImmU32 {
            address: 0x30000_i32,
            value: 42,
        };
        let encoded = instr.encode();
        assert_eq!(encoded[0], Opcode::StoreImmU32 as u8);
        // Low nibble of byte 1 = length of first immediate (address)
        let addr_len = (encoded[1] & 0x0F) as usize;
        assert_eq!(addr_len, 3); // 0x30000 fits in 3 bytes
        // Decode address from bytes 2..2+addr_len
        let mut addr_bytes = [0u8; 4];
        addr_bytes[..addr_len].copy_from_slice(&encoded[2..2 + addr_len]);
        let decoded_addr = i32::from_le_bytes(addr_bytes);
        assert_eq!(decoded_addr, 0x30000);
        // Decode value from remaining bytes
        let val_start = 2 + addr_len;
        let mut val_bytes = [0u8; 4];
        let val_len = encoded.len() - val_start;
        val_bytes[..val_len].copy_from_slice(&encoded[val_start..]);
        let decoded_val = i32::from_le_bytes(val_bytes);
        assert_eq!(decoded_val, 42);
    }

    #[test]
    fn test_two_imm_encoding_zero_value() {
        // StoreImmU32 with value=0: second immediate has 0 bytes
        let instr = Instruction::StoreImmU32 {
            address: 0x30000_i32,
            value: 0,
        };
        let encoded = instr.encode();
        let addr_len = (encoded[1] & 0x0F) as usize;
        // Total length = 1(opcode) + 1(nibble byte) + addr_len + 0(value=0)
        assert_eq!(encoded.len(), 2 + addr_len);
    }

    #[test]
    fn test_one_reg_two_imm_encoding() {
        // StoreImmIndU32 with base=3, offset=0, value=42
        let instr = Instruction::StoreImmIndU32 {
            base: 3,
            offset: 0,
            value: 42,
        };
        let encoded = instr.encode();
        assert_eq!(encoded[0], Opcode::StoreImmIndU32 as u8);
        // offset=0 → 0 bytes → offset_len=0 → high nibble=0, reg=3 → byte = 0x03
        assert_eq!(encoded[1], 0x03);
        // value=42 → 1 byte → 0x2A
        assert_eq!(encoded[2], 0x2A);
    }

    #[test]
    fn test_store_imm_ind_with_offset() {
        // StoreImmIndU64 with base=1, offset=100, value=-1
        let instr = Instruction::StoreImmIndU64 {
            base: 1,
            offset: 100,
            value: -1,
        };
        let encoded = instr.encode();
        assert_eq!(encoded[0], Opcode::StoreImmIndU64 as u8);
        // offset=100 → 1 byte → offset_len=1 → high nibble=1, reg=1 → byte = 0x11
        assert_eq!(encoded[1], 0x11);
        // offset byte: 100
        assert_eq!(encoded[2], 100);
        // value=-1 → 1 byte → 0xFF
        assert_eq!(encoded[3], 0xFF);
    }

    #[test]
    fn test_ecalli_encoding_roundtrip() {
        for index in [0u32, 100, 0x1234, 0xFFFF_FFFE] {
            let instr = Instruction::Ecalli { index };
            let encoded = instr.encode();
            assert_eq!(encoded[0], Opcode::Ecalli as u8);
            let imm = &encoded[1..];
            let mut bytes = [0u8; 4];
            bytes[..imm.len()].copy_from_slice(imm);
            let decoded = u32::from_le_bytes(bytes);
            assert_eq!(decoded, index, "roundtrip failed for index {index}");
        }
    }

    #[test]
    fn test_load_imm_jump_encoding() {
        // Typical call return address: value=2 (small), offset patched later
        let instr = Instruction::LoadImmJump {
            reg: 0,
            value: 2,
            offset: 100,
        };
        let encoded = instr.encode();
        assert_eq!(encoded[0], Opcode::LoadImmJump as u8);
        // Second byte: (imm_len << 4) | reg. value=2 fits in 1 byte, so imm_len=1.
        assert_eq!(encoded[1], 0x10); // (1 << 4) | 0
        assert_eq!(encoded[2], 2); // imm = 2
        // Offset is last 4 bytes
        let offset_bytes = &encoded[3..7];
        assert_eq!(i32::from_le_bytes(offset_bytes.try_into().unwrap()), 100);
        // Total: 7 bytes (opcode + reg/len + imm + offset)
        assert_eq!(encoded.len(), 7);
    }

    #[test]
    fn test_load_imm_jump_encoding_zero_value() {
        // value=0 → 0 imm bytes
        let instr = Instruction::LoadImmJump {
            reg: 0,
            value: 0,
            offset: -50,
        };
        let encoded = instr.encode();
        assert_eq!(encoded[0], Opcode::LoadImmJump as u8);
        assert_eq!(encoded[1], 0x00); // (0 << 4) | 0
        // No imm bytes, offset starts at byte 2
        assert_eq!(encoded.len(), 6); // opcode + reg/len + offset(4)
        let offset_bytes = &encoded[2..6];
        assert_eq!(i32::from_le_bytes(offset_bytes.try_into().unwrap()), -50);
    }

    #[test]
    fn test_load_imm_jump_is_terminating() {
        let instr = Instruction::LoadImmJump {
            reg: 0,
            value: 2,
            offset: 0,
        };
        assert!(instr.is_terminating());
    }

    #[test]
    fn test_load_imm_jump_dest_reg() {
        let instr = Instruction::LoadImmJump {
            reg: 5,
            value: 10,
            offset: 0,
        };
        assert_eq!(instr.dest_reg(), Some(5));
    }

    #[test]
    fn test_cmov_encoding() {
        // CmovIz: [opcode, (cond << 4) | src, dst]
        let instr = Instruction::CmovIz {
            dst: 3,
            src: 1,
            cond: 2,
        };
        let encoded = instr.encode();
        assert_eq!(encoded[0], Opcode::CmovIz as u8);
        assert_eq!(encoded[1], 0x21); // cond=2 high nibble, src=1 low nibble
        assert_eq!(encoded[2], 0x03); // dst

        // CmovNz
        let instr = Instruction::CmovNz {
            dst: 5,
            src: 4,
            cond: 7,
        };
        let encoded = instr.encode();
        assert_eq!(encoded[0], Opcode::CmovNz as u8);
        assert_eq!(encoded[1], 0x74); // cond=7 high nibble, src=4 low nibble
        assert_eq!(encoded[2], 0x05); // dst
    }

    #[test]
    fn test_cmov_dest_reg() {
        assert_eq!(
            Instruction::CmovIz {
                dst: 5,
                src: 1,
                cond: 2
            }
            .dest_reg(),
            Some(5)
        );
        assert_eq!(
            Instruction::CmovNz {
                dst: 8,
                src: 3,
                cond: 4
            }
            .dest_reg(),
            Some(8)
        );
    }

    #[test]
    fn test_two_reg_one_imm_encoding_all_opcodes() {
        // All TwoRegOneImm instructions use the same encoding: [opcode, (src<<4)|dst, imm...]
        let test_cases: Vec<(Instruction, Opcode)> = vec![
            (
                Instruction::AndImm {
                    dst: 2,
                    src: 3,
                    value: 0xFF,
                },
                Opcode::AndImm,
            ),
            (
                Instruction::XorImm {
                    dst: 4,
                    src: 5,
                    value: -1,
                },
                Opcode::XorImm,
            ),
            (
                Instruction::OrImm {
                    dst: 6,
                    src: 7,
                    value: 42,
                },
                Opcode::OrImm,
            ),
            (
                Instruction::MulImm32 {
                    dst: 0,
                    src: 1,
                    value: 10,
                },
                Opcode::MulImm32,
            ),
            (
                Instruction::MulImm64 {
                    dst: 2,
                    src: 3,
                    value: 100,
                },
                Opcode::MulImm64,
            ),
            (
                Instruction::ShloLImm32 {
                    dst: 4,
                    src: 5,
                    value: 3,
                },
                Opcode::ShloLImm32,
            ),
            (
                Instruction::ShloRImm32 {
                    dst: 6,
                    src: 7,
                    value: 1,
                },
                Opcode::ShloRImm32,
            ),
            (
                Instruction::SharRImm32 {
                    dst: 0,
                    src: 1,
                    value: 2,
                },
                Opcode::SharRImm32,
            ),
            (
                Instruction::ShloLImm64 {
                    dst: 2,
                    src: 3,
                    value: 32,
                },
                Opcode::ShloLImm64,
            ),
            (
                Instruction::ShloRImm64 {
                    dst: 4,
                    src: 5,
                    value: 16,
                },
                Opcode::ShloRImm64,
            ),
            (
                Instruction::SharRImm64 {
                    dst: 6,
                    src: 7,
                    value: 8,
                },
                Opcode::SharRImm64,
            ),
            (
                Instruction::NegAddImm32 {
                    dst: 0,
                    src: 1,
                    value: 0,
                },
                Opcode::NegAddImm32,
            ),
            (
                Instruction::NegAddImm64 {
                    dst: 2,
                    src: 3,
                    value: -5,
                },
                Opcode::NegAddImm64,
            ),
            (
                Instruction::SetGtUImm {
                    dst: 4,
                    src: 5,
                    value: 10,
                },
                Opcode::SetGtUImm,
            ),
            (
                Instruction::SetGtSImm {
                    dst: 6,
                    src: 7,
                    value: -10,
                },
                Opcode::SetGtSImm,
            ),
        ];

        for (instr, expected_opcode) in &test_cases {
            let encoded = instr.encode();
            assert_eq!(
                encoded[0], *expected_opcode as u8,
                "Wrong opcode for {instr:?}"
            );
            // Verify encoding is at least 2 bytes (opcode + reg byte)
            assert!(encoded.len() >= 2, "Encoding too short for {instr:?}");
        }
    }

    #[test]
    fn test_two_reg_one_imm_register_encoding() {
        // Verify dst goes in low nibble, src in high nibble
        let instr = Instruction::AndImm {
            dst: 3,
            src: 9,
            value: 0,
        };
        let encoded = instr.encode();
        assert_eq!(encoded[1], 0x93); // src=9 in high nibble, dst=3 in low nibble
    }

    #[test]
    fn test_two_reg_one_imm_zero_imm() {
        // Zero immediate should produce minimal encoding (just opcode + reg byte)
        let instr = Instruction::NegAddImm32 {
            dst: 0,
            src: 1,
            value: 0,
        };
        let encoded = instr.encode();
        assert_eq!(encoded.len(), 2); // opcode + regs, no imm bytes
    }

    #[test]
    fn test_alu_imm_dest_reg() {
        // All ALU immediate instructions should report their dest_reg
        let instructions: Vec<Instruction> = vec![
            Instruction::AndImm {
                dst: 5,
                src: 1,
                value: 0,
            },
            Instruction::XorImm {
                dst: 6,
                src: 2,
                value: 0,
            },
            Instruction::OrImm {
                dst: 7,
                src: 3,
                value: 0,
            },
            Instruction::MulImm32 {
                dst: 8,
                src: 4,
                value: 0,
            },
            Instruction::MulImm64 {
                dst: 9,
                src: 5,
                value: 0,
            },
            Instruction::ShloLImm32 {
                dst: 10,
                src: 6,
                value: 0,
            },
            Instruction::ShloRImm64 {
                dst: 11,
                src: 7,
                value: 0,
            },
            Instruction::NegAddImm32 {
                dst: 12,
                src: 8,
                value: 0,
            },
            Instruction::SetGtUImm {
                dst: 0,
                src: 9,
                value: 0,
            },
            Instruction::SetGtSImm {
                dst: 1,
                src: 10,
                value: 0,
            },
        ];
        let expected_dsts: Vec<u8> = vec![5, 6, 7, 8, 9, 10, 11, 12, 0, 1];
        for (instr, expected) in instructions.iter().zip(expected_dsts.iter()) {
            assert_eq!(
                instr.dest_reg(),
                Some(*expected),
                "Wrong dest_reg for {instr:?}"
            );
        }
    }

    #[test]
    fn test_one_reg_one_imm_encoding_layout() {
        // encode_one_reg_one_imm: [opcode, reg & 0x0F, imm...]
        let encoded = encode_one_reg_one_imm(Opcode::LoadU32, 7, 0x30000);
        assert_eq!(encoded[0], Opcode::LoadU32 as u8);
        assert_eq!(encoded[1], 0x07); // reg=7, low nibble only
        assert_eq!(&encoded[2..], &encode_imm(0x30000));
    }

    #[test]
    fn test_one_reg_one_imm_zero_value() {
        let encoded = encode_one_reg_one_imm(Opcode::LoadU64, 3, 0);
        assert_eq!(encoded.len(), 2); // opcode + reg, no imm bytes for value=0
        assert_eq!(encoded[0], Opcode::LoadU64 as u8);
        assert_eq!(encoded[1], 0x03);
    }

    #[test]
    fn test_one_reg_one_imm_negative_value() {
        let encoded = encode_one_reg_one_imm(Opcode::StoreU8, 12, -1);
        assert_eq!(encoded[0], Opcode::StoreU8 as u8);
        assert_eq!(encoded[1], 0x0C); // reg=12
        assert_eq!(&encoded[2..], &encode_imm(-1));
        assert_eq!(encoded[2], 0xFF); // -1 as 1-byte signed
    }

    #[test]
    fn test_one_reg_one_imm_high_nibble_masked() {
        // Register 0xF7 should be masked to 0x07
        let encoded = encode_one_reg_one_imm(Opcode::LoadI8, 0xF7, 42);
        assert_eq!(encoded[1], 0x07); // only low nibble kept
    }

    #[test]
    fn test_one_reg_one_imm_roundtrip() {
        // Roundtrip: encode then manually decode and verify fields match
        for (opcode, reg, value) in [
            (Opcode::LoadU8, 0u8, 0i32),
            (Opcode::LoadI16, 5, 42),
            (Opcode::LoadU32, 12, 0x30000),
            (Opcode::StoreU64, 7, -128),
            (Opcode::LoadU64, 3, 8_388_607), // max 3-byte signed
        ] {
            let encoded = encode_one_reg_one_imm(opcode, reg, value);

            // Decode opcode
            assert_eq!(
                encoded[0], opcode as u8,
                "opcode mismatch for reg={reg} value={value}"
            );

            // Decode register
            let decoded_reg = encoded[1] & 0x0F;
            assert_eq!(
                decoded_reg,
                reg & 0x0F,
                "reg mismatch for reg={reg} value={value}"
            );

            // Decode immediate (sign-extend from variable-length encoding)
            let imm_bytes = &encoded[2..];
            let decoded_value = decode_sign_extended_imm(imm_bytes);
            assert_eq!(
                decoded_value, value,
                "value mismatch for reg={reg} value={value}"
            );
        }
    }

    #[test]
    fn test_one_reg_one_imm_via_instruction_encode() {
        // Verify Instruction::encode() for LoadU32 matches direct helper call
        let instr = Instruction::LoadU32 {
            dst: 5,
            address: 0x30000,
        };
        let from_instr = instr.encode();
        let from_helper = encode_one_reg_one_imm(Opcode::LoadU32, 5, 0x30000);
        assert_eq!(from_instr, from_helper);
    }

    #[test]
    fn test_two_reg_two_imm_encoding_layout() {
        // encode_two_reg_two_imm: [opcode, (reg1<<4)|reg2, imm1_len, imm1..., imm2...]
        let encoded = encode_two_reg_two_imm(Opcode::LoadImmJumpInd, 3, 7, 42, 100);
        assert_eq!(encoded[0], Opcode::LoadImmJumpInd as u8);
        assert_eq!(encoded[1], 0x37); // reg1=3 high nibble, reg2=7 low nibble
        let imm1_enc = encode_imm(42);
        let imm2_enc = encode_imm(100);
        assert_eq!(encoded[2], imm1_enc.len() as u8); // imm1 length
        assert_eq!(&encoded[3..3 + imm1_enc.len()], &imm1_enc);
        assert_eq!(&encoded[3 + imm1_enc.len()..], &imm2_enc);
    }

    #[test]
    fn test_two_reg_two_imm_zero_immediates() {
        let encoded = encode_two_reg_two_imm(Opcode::LoadImmJumpInd, 0, 5, 0, 0);
        // [opcode, 0x05, 0x00] — no imm bytes for either
        assert_eq!(encoded.len(), 3);
        assert_eq!(encoded[0], Opcode::LoadImmJumpInd as u8);
        assert_eq!(encoded[1], 0x05);
        assert_eq!(encoded[2], 0x00); // imm1_len = 0
    }

    #[test]
    fn test_two_reg_two_imm_register_masking() {
        // High bits should be masked off
        let encoded = encode_two_reg_two_imm(Opcode::LoadImmJumpInd, 0xA3, 0xB7, 1, 2);
        assert_eq!(encoded[1], 0x37); // only low nibbles: 3 and 7
    }

    #[test]
    fn test_two_reg_two_imm_roundtrip() {
        for (reg1, reg2, imm1, imm2) in [
            (0u8, 0u8, 0i32, 0i32),
            (3, 7, 42, 100),
            (12, 1, -1, 0x30000),
            (5, 9, 8_388_607, -8_388_608), // max/min 3-byte signed
        ] {
            let encoded = encode_two_reg_two_imm(Opcode::LoadImmJumpInd, reg1, reg2, imm1, imm2);

            // Decode opcode
            assert_eq!(encoded[0], Opcode::LoadImmJumpInd as u8);

            // Decode registers
            let decoded_reg1 = (encoded[1] >> 4) & 0x0F;
            let decoded_reg2 = encoded[1] & 0x0F;
            assert_eq!(decoded_reg1, reg1 & 0x0F, "reg1 mismatch");
            assert_eq!(decoded_reg2, reg2 & 0x0F, "reg2 mismatch");

            // Decode imm1 length + imm1
            let imm1_len = (encoded[2] & 0x0F) as usize;
            let imm1_bytes = &encoded[3..3 + imm1_len];
            let decoded_imm1 = decode_sign_extended_imm(imm1_bytes);
            assert_eq!(
                decoded_imm1, imm1,
                "imm1 mismatch for ({reg1},{reg2},{imm1},{imm2})"
            );

            // Decode imm2 (remaining bytes)
            let imm2_bytes = &encoded[3 + imm1_len..];
            let decoded_imm2 = decode_sign_extended_imm(imm2_bytes);
            assert_eq!(
                decoded_imm2, imm2,
                "imm2 mismatch for ({reg1},{reg2},{imm1},{imm2})"
            );
        }
    }

    #[test]
    fn test_two_reg_two_imm_via_instruction_encode() {
        // Verify Instruction::encode() for LoadImmJumpInd matches direct helper call
        let instr = Instruction::LoadImmJumpInd {
            base: 3,
            dst: 7,
            value: 42,
            offset: 100,
        };
        let from_instr = instr.encode();
        let from_helper = encode_two_reg_two_imm(Opcode::LoadImmJumpInd, 3, 7, 42, 100);
        assert_eq!(from_instr, from_helper);
    }

    /// Helper: decode a variable-length sign-extended immediate (same logic as `test_cmov_imm_roundtrip`)
    fn decode_sign_extended_imm(imm_bytes: &[u8]) -> i32 {
        let mut buf = [0u8; 4];
        buf[..imm_bytes.len()].copy_from_slice(imm_bytes);
        if let Some(&last) = imm_bytes.last()
            && last & 0x80 != 0
        {
            for b in buf.iter_mut().skip(imm_bytes.len()) {
                *b = 0xFF;
            }
        }
        i32::from_le_bytes(buf)
    }
}
