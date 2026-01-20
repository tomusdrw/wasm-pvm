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
    BranchGeU { reg1: u8, reg2: u8, offset: i32 },
    BranchLtU { reg1: u8, reg2: u8, offset: i32 },
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
            Self::BranchGeU { reg1, reg2, offset } => {
                encode_two_reg_one_off(Opcode::BranchGeU, *reg1, *reg2, *offset)
            }
            Self::BranchLtU { reg1, reg2, offset } => {
                encode_two_reg_one_off(Opcode::BranchLtU, *reg1, *reg2, *offset)
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
                | Self::BranchGeU { .. }
                | Self::BranchLtU { .. }
        )
    }
}

fn encode_three_reg(opcode: Opcode, dst: u8, src1: u8, src2: u8) -> Vec<u8> {
    vec![opcode as u8, (src1 & 0x0F) << 4 | (src2 & 0x0F), dst & 0x0F]
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
