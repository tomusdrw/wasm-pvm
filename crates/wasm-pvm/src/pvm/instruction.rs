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

#[cfg(test)]
mod tests {
    use super::*;

    // --- encode_imm boundary tests ---

    #[test]
    fn encode_imm_zero() {
        assert_eq!(encode_imm(0), Vec::<u8>::new());
    }

    #[test]
    fn encode_imm_one_byte_boundaries() {
        assert_eq!(encode_imm(1), vec![1]);
        assert_eq!(encode_imm(-1), vec![0xFF]);
        assert_eq!(encode_imm(127), vec![0x7F]);
        assert_eq!(encode_imm(-128), vec![0x80]);
    }

    #[test]
    fn encode_imm_two_byte_boundaries() {
        assert_eq!(encode_imm(128), vec![0x80, 0x00]);
        assert_eq!(encode_imm(-129), vec![0x7F, 0xFF]);
        assert_eq!(encode_imm(32767), vec![0xFF, 0x7F]);
        assert_eq!(encode_imm(-32768), vec![0x00, 0x80]);
    }

    #[test]
    fn encode_imm_three_byte_boundaries() {
        assert_eq!(encode_imm(32768), vec![0x00, 0x80, 0x00]);
        assert_eq!(encode_imm(8_388_607), vec![0xFF, 0xFF, 0x7F]);
    }

    #[test]
    fn encode_imm_four_byte_boundaries() {
        assert_eq!(encode_imm(8_388_608), vec![0x00, 0x00, 0x80, 0x00]);
        assert_eq!(encode_imm(i32::MAX), vec![0xFF, 0xFF, 0xFF, 0x7F]);
        assert_eq!(encode_imm(i32::MIN), vec![0x00, 0x00, 0x00, 0x80]);
    }

    // --- Encoding family tests ---

    #[test]
    fn encode_trap() {
        assert_eq!(Instruction::Trap.encode(), vec![Opcode::Trap as u8]);
    }

    #[test]
    fn encode_fallthrough() {
        assert_eq!(
            Instruction::Fallthrough.encode(),
            vec![Opcode::Fallthrough as u8]
        );
    }

    #[test]
    fn encode_load_imm64() {
        let instr = Instruction::LoadImm64 {
            reg: 3,
            value: 0x0102_0304_0506_0708,
        };
        let encoded = instr.encode();
        assert_eq!(encoded.len(), 10);
        assert_eq!(encoded[0], Opcode::LoadImm64 as u8);
        assert_eq!(encoded[1], 3);
        assert_eq!(&encoded[2..], &0x0102_0304_0506_0708u64.to_le_bytes());
    }

    #[test]
    fn encode_load_imm() {
        let instr = Instruction::LoadImm { reg: 5, value: 42 };
        let encoded = instr.encode();
        assert_eq!(encoded, vec![Opcode::LoadImm as u8, 5, 42]);
    }

    #[test]
    fn encode_load_imm_zero_value() {
        let instr = Instruction::LoadImm { reg: 0, value: 0 };
        // 0 encodes to 0 imm bytes
        assert_eq!(encoded(instr), vec![Opcode::LoadImm as u8, 0]);
    }

    #[test]
    fn encode_three_reg_add32() {
        let instr = Instruction::Add32 {
            dst: 2,
            src1: 3,
            src2: 4,
        };
        let encoded = instr.encode();
        assert_eq!(encoded.len(), 3);
        assert_eq!(encoded[0], Opcode::Add32 as u8);
        assert_eq!(encoded[1], (3 << 4) | 4); // (src1<<4)|src2
        assert_eq!(encoded[2], 2); // dst
    }

    #[test]
    fn encode_two_reg_sbrk() {
        let instr = Instruction::Sbrk { dst: 1, src: 2 };
        let encoded = instr.encode();
        assert_eq!(encoded.len(), 2);
        assert_eq!(encoded[0], Opcode::Sbrk as u8);
        assert_eq!(encoded[1], (2 << 4) | 1); // (src<<4)|dst
    }

    #[test]
    fn encode_two_reg_one_imm_add_imm64() {
        let instr = Instruction::AddImm64 {
            dst: 1,
            src: 2,
            value: 10,
        };
        let encoded = instr.encode();
        assert_eq!(encoded[0], Opcode::AddImm64 as u8);
        assert_eq!(encoded[1], (2 << 4) | 1); // (src<<4)|dst
        assert_eq!(encoded[2], 10); // variable-length imm
        assert_eq!(encoded.len(), 3);
    }

    #[test]
    fn encode_two_reg_one_off_branch_geu() {
        let instr = Instruction::BranchGeU {
            reg1: 3,
            reg2: 5,
            offset: 100,
        };
        let encoded = instr.encode();
        assert_eq!(encoded.len(), 6);
        assert_eq!(encoded[0], Opcode::BranchGeU as u8);
        assert_eq!(encoded[1], (3 << 4) | 5); // (reg1<<4)|reg2
        assert_eq!(&encoded[2..6], &100i32.to_le_bytes());
    }

    #[test]
    fn encode_one_reg_one_imm_one_off_branch_ne_imm() {
        let instr = Instruction::BranchNeImm {
            reg: 2,
            value: 1,
            offset: 50,
        };
        let encoded = instr.encode();
        // imm=1 → 1 byte, so imm_len=1
        assert_eq!(encoded[0], Opcode::BranchNeImm as u8);
        assert_eq!(encoded[1], (1 << 4) | 2); // (imm_len<<4)|reg
        assert_eq!(encoded[2], 1); // imm byte
        assert_eq!(&encoded[3..7], &50i32.to_le_bytes());
        assert_eq!(encoded.len(), 7);
    }

    #[test]
    fn encode_one_reg_one_imm_one_off_zero_imm() {
        let instr = Instruction::BranchEqImm {
            reg: 0,
            value: 0,
            offset: -1,
        };
        let encoded = instr.encode();
        // imm=0 → 0 bytes, imm_len=0
        assert_eq!(encoded[0], Opcode::BranchEqImm as u8);
        assert_eq!(encoded[1], (0 << 4) | 0); // (imm_len<<4)|reg
        assert_eq!(&encoded[2..6], &(-1i32).to_le_bytes());
        assert_eq!(encoded.len(), 6);
    }

    #[test]
    fn encode_memory_load_ind_u32() {
        let instr = Instruction::LoadIndU32 {
            dst: 3,
            base: 5,
            offset: 8,
        };
        let encoded = instr.encode();
        assert_eq!(encoded[0], Opcode::LoadIndU32 as u8);
        assert_eq!(encoded[1], (5 << 4) | 3); // (base<<4)|dst
        assert_eq!(encoded[2], 8); // 1-byte imm
        assert_eq!(encoded.len(), 3);
    }

    #[test]
    fn encode_memory_store_ind_u8() {
        let instr = Instruction::StoreIndU8 {
            base: 2,
            src: 4,
            offset: 16,
        };
        let encoded = instr.encode();
        assert_eq!(encoded[0], Opcode::StoreIndU8 as u8);
        assert_eq!(encoded[1], (2 << 4) | 4); // (base<<4)|src
        assert_eq!(encoded[2], 16); // 1-byte imm
        assert_eq!(encoded.len(), 3);
    }

    #[test]
    fn encode_jump() {
        let instr = Instruction::Jump { offset: 256 };
        let encoded = instr.encode();
        assert_eq!(encoded[0], Opcode::Jump as u8);
        assert_eq!(&encoded[1..5], &256i32.to_le_bytes());
        assert_eq!(encoded.len(), 5);
    }

    #[test]
    fn encode_jump_ind() {
        let instr = Instruction::JumpInd { reg: 7, offset: 10 };
        let encoded = instr.encode();
        assert_eq!(encoded[0], Opcode::JumpInd as u8);
        assert_eq!(encoded[1], 7);
        assert_eq!(encoded[2], 10);
        assert_eq!(encoded.len(), 3);
    }

    // --- is_terminating tests ---

    #[test]
    fn is_terminating_true_cases() {
        assert!(Instruction::Trap.is_terminating());
        assert!(Instruction::Fallthrough.is_terminating());
        assert!(Instruction::Jump { offset: 0 }.is_terminating());
        assert!(Instruction::JumpInd { reg: 0, offset: 0 }.is_terminating());
        assert!(
            Instruction::BranchNeImm {
                reg: 0,
                value: 0,
                offset: 0
            }
            .is_terminating()
        );
        assert!(
            Instruction::BranchEqImm {
                reg: 0,
                value: 0,
                offset: 0
            }
            .is_terminating()
        );
        assert!(
            Instruction::BranchGeSImm {
                reg: 0,
                value: 0,
                offset: 0
            }
            .is_terminating()
        );
        assert!(
            Instruction::BranchGeU {
                reg1: 0,
                reg2: 0,
                offset: 0
            }
            .is_terminating()
        );
        assert!(
            Instruction::BranchLtU {
                reg1: 0,
                reg2: 0,
                offset: 0
            }
            .is_terminating()
        );
    }

    #[test]
    fn is_terminating_false_cases() {
        assert!(
            !Instruction::Add32 {
                dst: 0,
                src1: 0,
                src2: 0
            }
            .is_terminating()
        );
        assert!(!Instruction::LoadImm { reg: 0, value: 0 }.is_terminating());
        assert!(
            !Instruction::StoreIndU32 {
                base: 0,
                src: 0,
                offset: 0
            }
            .is_terminating()
        );
        assert!(!Instruction::Sbrk { dst: 0, src: 0 }.is_terminating());
        assert!(
            !Instruction::And {
                dst: 0,
                src1: 0,
                src2: 0
            }
            .is_terminating()
        );
        assert!(!Instruction::LoadImm64 { reg: 0, value: 0 }.is_terminating());
    }

    fn encoded(instr: Instruction) -> Vec<u8> {
        instr.encode()
    }
}
