use super::Opcode;

#[derive(Debug, Clone)]
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
    Ecalli {
        index: u32,
    },
    Unknown {
        opcode: u8,
        raw_bytes: Vec<u8>,
    },
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
        )
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
                if let Some(&last) = imm_bytes.last() {
                    if last & 0x80 != 0 {
                        for b in buf.iter_mut().skip(imm_bytes.len()) {
                            *b = 0xFF;
                        }
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
                "Wrong opcode for {:?}",
                instr
            );
            // Verify encoding is at least 2 bytes (opcode + reg byte)
            assert!(encoded.len() >= 2, "Encoding too short for {:?}", instr);
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
                "Wrong dest_reg for {:?}",
                instr
            );
        }
    }
}
