use super::Opcode;

#[derive(Debug, Clone)]
pub enum Instruction {
    Trap,
    Fallthrough,
    LoadImm64 { reg: u8, value: u64 },
    LoadImm { reg: u8, value: i32 },
    Add32 { dst: u8, src1: u8, src2: u8 },
    Sub32 { dst: u8, src1: u8, src2: u8 },
    Mul32 { dst: u8, src1: u8, src2: u8 },
    DivU32 { dst: u8, src1: u8, src2: u8 },
    DivS32 { dst: u8, src1: u8, src2: u8 },
    RemU32 { dst: u8, src1: u8, src2: u8 },
    RemS32 { dst: u8, src1: u8, src2: u8 },
    Add64 { dst: u8, src1: u8, src2: u8 },
    Sub64 { dst: u8, src1: u8, src2: u8 },
    Mul64 { dst: u8, src1: u8, src2: u8 },
    DivU64 { dst: u8, src1: u8, src2: u8 },
    DivS64 { dst: u8, src1: u8, src2: u8 },
    RemU64 { dst: u8, src1: u8, src2: u8 },
    RemS64 { dst: u8, src1: u8, src2: u8 },
    ShloL64 { dst: u8, src1: u8, src2: u8 },
    ShloR64 { dst: u8, src1: u8, src2: u8 },
    SharR64 { dst: u8, src1: u8, src2: u8 },
    AddImm32 { dst: u8, src: u8, value: i32 },
    AddImm64 { dst: u8, src: u8, value: i32 },
    Jump { offset: i32 },
    JumpInd { reg: u8, offset: i32 },
    LoadIndU32 { dst: u8, base: u8, offset: i32 },
    StoreIndU32 { base: u8, src: u8, offset: i32 },
    LoadIndU64 { dst: u8, base: u8, offset: i32 },
    StoreIndU64 { base: u8, src: u8, offset: i32 },
    BranchNeImm { reg: u8, value: i32, offset: i32 },
    BranchEqImm { reg: u8, value: i32, offset: i32 },
    BranchGeSImm { reg: u8, value: i32, offset: i32 },
    BranchLtUImm { reg: u8, value: i32, offset: i32 },
    BranchLeUImm { reg: u8, value: i32, offset: i32 },
    BranchGeUImm { reg: u8, value: i32, offset: i32 },
    BranchGtUImm { reg: u8, value: i32, offset: i32 },
    BranchLtSImm { reg: u8, value: i32, offset: i32 },
    BranchLeSImm { reg: u8, value: i32, offset: i32 },
    BranchGtSImm { reg: u8, value: i32, offset: i32 },
    MoveReg { dst: u8, src: u8 },
    BranchEq { reg1: u8, reg2: u8, offset: i32 },
    BranchNe { reg1: u8, reg2: u8, offset: i32 },
    BranchGeU { reg1: u8, reg2: u8, offset: i32 },
    BranchLtU { reg1: u8, reg2: u8, offset: i32 },
    BranchLtS { reg1: u8, reg2: u8, offset: i32 },
    BranchGeS { reg1: u8, reg2: u8, offset: i32 },
    SetLtU { dst: u8, src1: u8, src2: u8 },
    SetLtS { dst: u8, src1: u8, src2: u8 },
    And { dst: u8, src1: u8, src2: u8 },
    Xor { dst: u8, src1: u8, src2: u8 },
    Or { dst: u8, src1: u8, src2: u8 },
    SetLtUImm { dst: u8, src: u8, value: i32 },
    SetLtSImm { dst: u8, src: u8, value: i32 },
    ShloL32 { dst: u8, src1: u8, src2: u8 },
    ShloR32 { dst: u8, src1: u8, src2: u8 },
    SharR32 { dst: u8, src1: u8, src2: u8 },
    Sbrk { dst: u8, src: u8 },
    CountSetBits64 { dst: u8, src: u8 },
    CountSetBits32 { dst: u8, src: u8 },
    LeadingZeroBits64 { dst: u8, src: u8 },
    LeadingZeroBits32 { dst: u8, src: u8 },
    TrailingZeroBits64 { dst: u8, src: u8 },
    TrailingZeroBits32 { dst: u8, src: u8 },
    SignExtend8 { dst: u8, src: u8 },
    SignExtend16 { dst: u8, src: u8 },
    ZeroExtend16 { dst: u8, src: u8 },
    LoadIndU8 { dst: u8, base: u8, offset: i32 },
    LoadIndI8 { dst: u8, base: u8, offset: i32 },
    StoreIndU8 { base: u8, src: u8, offset: i32 },
    LoadIndU16 { dst: u8, base: u8, offset: i32 },
    LoadIndI16 { dst: u8, base: u8, offset: i32 },
    StoreIndU16 { base: u8, src: u8, offset: i32 },
    /// Conditional move if zero with immediate: if reg[cond] == 0 then reg[dst] = sign_extend(value)
    CmovIzImm { cond: u8, dst: u8, value: i32 },
    /// Conditional move if non-zero with immediate: if reg[cond] != 0 then reg[dst] = sign_extend(value)
    CmovNzImm { cond: u8, dst: u8, value: i32 },
    Ecalli { index: u32 },
    Unknown { opcode: u8, raw_bytes: Vec<u8> },
}

impl Instruction {
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
            Self::And { dst, src1, src2 } => encode_three_reg(Opcode::And, *dst, *src1, *src2),
            Self::Xor { dst, src1, src2 } => encode_three_reg(Opcode::Xor, *dst, *src1, *src2),
            Self::Or { dst, src1, src2 } => encode_three_reg(Opcode::Or, *dst, *src1, *src2),
            Self::Jump { offset } => {
                let mut bytes = vec![Opcode::Jump as u8];
                bytes.extend_from_slice(&offset.to_le_bytes());
                bytes
            }
            Self::JumpInd { reg, offset } => {
                let mut bytes = vec![Opcode::JumpInd as u8, *reg & 0x0F];
                bytes.extend_from_slice(&encode_imm(*offset));
                bytes
            }
            Self::AddImm32 { dst, src, value } => {
                let mut bytes = vec![Opcode::AddImm32 as u8, (*src & 0x0F) << 4 | (*dst & 0x0F)];
                bytes.extend_from_slice(&encode_imm(*value));
                bytes
            }
            Self::AddImm64 { dst, src, value } => {
                let mut bytes = vec![Opcode::AddImm64 as u8, (*src & 0x0F) << 4 | (*dst & 0x0F)];
                bytes.extend_from_slice(&encode_imm(*value));
                bytes
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
                let mut bytes = vec![Opcode::SetLtUImm as u8, (*src & 0x0F) << 4 | (*dst & 0x0F)];
                bytes.extend_from_slice(&encode_imm(*value));
                bytes
            }
            Self::SetLtSImm { dst, src, value } => {
                let mut bytes = vec![Opcode::SetLtSImm as u8, (*src & 0x0F) << 4 | (*dst & 0x0F)];
                bytes.extend_from_slice(&encode_imm(*value));
                bytes
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
            Self::CmovIzImm { cond, dst, value } => {
                let mut bytes =
                    vec![Opcode::CmovIzImm as u8, (*cond & 0x0F) << 4 | (*dst & 0x0F)];
                bytes.extend_from_slice(&encode_imm(*value));
                bytes
            }
            Self::CmovNzImm { cond, dst, value } => {
                let mut bytes =
                    vec![Opcode::CmovNzImm as u8, (*cond & 0x0F) << 4 | (*dst & 0x0F)];
                bytes.extend_from_slice(&encode_imm(*value));
                bytes
            }
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
            | Self::MoveReg { dst, .. } => Some(*dst),
            Self::LoadImm { reg, .. } | Self::LoadImm64 { reg, .. } => Some(*reg),
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
        )
    }
}

fn encode_three_reg(opcode: Opcode, dst: u8, src1: u8, src2: u8) -> Vec<u8> {
    // PVM three-reg encoding: [opcode, rB_hi | rA_lo, rD]
    // Semantics: reg[rD] = reg[rA] OP reg[rB]
    // We want: reg[dst] = reg[src1] OP reg[src2], so rA=src1, rB=src2
    vec![opcode as u8, (src2 & 0x0F) << 4 | (src1 & 0x0F), dst & 0x0F]
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

fn encode_two_reg_one_off(opcode: Opcode, reg1: u8, reg2: u8, offset: i32) -> Vec<u8> {
    let mut bytes = vec![opcode as u8, (reg1 & 0x0F) << 4 | (reg2 & 0x0F)];
    bytes.extend_from_slice(&offset.to_le_bytes());
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
