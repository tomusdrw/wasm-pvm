use super::Opcode;

#[derive(Debug, Clone)]
pub enum Instruction {
    Trap,
    Fallthrough,
    LoadImm64 { reg: u8, value: u64 },
    LoadImm { reg: u8, value: i32 },
    Add32 { dst: u8, src1: u8, src2: u8 },
    Add64 { dst: u8, src1: u8, src2: u8 },
    Jump { offset: i32 },
    /// Indirect jump: jump to address in register + offset. Jump to r0 (0xFFFF0000) to HALT.
    JumpInd { reg: u8, offset: i32 },
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
            Self::Add32 { dst, src1, src2 } => {
                vec![
                    Opcode::Add32 as u8,
                    (*src1 & 0x0F) << 4 | (*src2 & 0x0F),
                    *dst & 0x0F,
                ]
            }
            Self::Add64 { dst, src1, src2 } => {
                vec![
                    Opcode::Add64 as u8,
                    (*src1 & 0x0F) << 4 | (*src2 & 0x0F),
                    *dst & 0x0F,
                ]
            }
            Self::Jump { offset } => {
                let mut bytes = vec![Opcode::Jump as u8];
                bytes.extend_from_slice(&encode_imm(*offset));
                bytes
            }
            Self::JumpInd { reg, offset } => {
                let mut bytes = vec![Opcode::JumpInd as u8, *reg & 0x0F];
                bytes.extend_from_slice(&encode_imm(*offset));
                bytes
            }
        }
    }

    #[must_use]
    pub const fn is_terminating(&self) -> bool {
        matches!(self, Self::Trap | Self::Fallthrough | Self::Jump { .. } | Self::JumpInd { .. })
    }
}

fn encode_imm(value: i32) -> Vec<u8> {
    let bytes = value.to_le_bytes();
    let len = if value == 0 {
        0
    } else if value >= -128 && value <= 127 {
        1
    } else if value >= -32768 && value <= 32767 {
        2
    } else if value >= -8_388_608 && value <= 8_388_607 {
        3
    } else {
        4
    };
    bytes[..len].to_vec()
}
