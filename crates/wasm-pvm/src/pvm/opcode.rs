#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    Trap = 0,
    Fallthrough = 1,
    Ecalli = 10,
    LoadImm64 = 20,
    // Store immediate to absolute address (TwoImm)
    StoreImmU8 = 30,
    StoreImmU16 = 31,
    StoreImmU32 = 32,
    StoreImmU64 = 33,
    Jump = 40,
    JumpInd = 50,
    LoadImm = 51,
    // Load/store absolute address (OneRegOneImm)
    LoadU8 = 52,
    LoadI8 = 53,
    LoadU16 = 54,
    LoadI16 = 55,
    LoadU32 = 56,
    LoadI32 = 57,
    LoadU64 = 58,
    StoreU8 = 59,
    StoreU16 = 60,
    StoreU32 = 61,
    StoreU64 = 62,
    // Store immediate indirect (OneRegTwoImm)
    StoreImmIndU8 = 70,
    StoreImmIndU16 = 71,
    StoreImmIndU32 = 72,
    StoreImmIndU64 = 73,
    // Compound jump (OneRegOneImmOneOff)
    LoadImmJump = 80,
    // Branch with immediate comparison (OneRegOneImmOneOff)
    BranchEqImm = 81,
    BranchNeImm = 82,
    BranchLtUImm = 83,
    BranchLeUImm = 84,
    BranchGeUImm = 85,
    BranchGtUImm = 86,
    BranchLtSImm = 87,
    BranchLeSImm = 88,
    BranchGeSImm = 89,
    BranchGtSImm = 90,
    MoveReg = 100,
    Sbrk = 101,
    CountSetBits64 = 102,
    CountSetBits32 = 103,
    LeadingZeroBits64 = 104,
    LeadingZeroBits32 = 105,
    TrailingZeroBits64 = 106,
    TrailingZeroBits32 = 107,
    SignExtend8 = 108,
    SignExtend16 = 109,
    ZeroExtend16 = 110,
    ReverseBytes = 111,
    StoreIndU8 = 120,
    StoreIndU16 = 121,
    StoreIndU32 = 122,
    StoreIndU64 = 123,
    LoadIndU8 = 124,
    LoadIndI8 = 125,
    LoadIndU16 = 126,
    LoadIndI16 = 127,
    LoadIndU32 = 128,
    LoadIndI32 = 129,
    LoadIndU64 = 130,
    AddImm32 = 131,
    AndImm = 132,
    XorImm = 133,
    OrImm = 134,
    MulImm32 = 135,
    // Set if less/greater than immediate (TwoRegOneImm)
    SetLtUImm = 136,
    SetLtSImm = 137,
    // Shift by immediate (TwoRegOneImm)
    ShloLImm32 = 138,
    ShloRImm32 = 139,
    SharRImm32 = 140,
    NegAddImm32 = 141,
    SetGtUImm = 142,
    SetGtSImm = 143,
    // Alternate shift immediates: dst = imm OP src (reversed operands, 32-bit)
    ShloLImmAlt32 = 144,
    ShloRImmAlt32 = 145,
    SharRImmAlt32 = 146,
    // Conditional move with immediate (TwoRegOneImm)
    CmovIzImm = 147,
    CmovNzImm = 148,
    AddImm64 = 149,
    MulImm64 = 150,
    ShloLImm64 = 151,
    ShloRImm64 = 152,
    SharRImm64 = 153,
    NegAddImm64 = 154,
    // Alternate shift immediates: dst = imm OP src (reversed operands, 64-bit)
    ShloLImmAlt64 = 155,
    ShloRImmAlt64 = 156,
    SharRImmAlt64 = 157,
    // Rotate right by immediate (TwoRegOneImm)
    RotRImm64 = 158,
    RotRImmAlt64 = 159,
    RotRImm32 = 160,
    RotRImmAlt32 = 161,
    // Branch with two registers (TwoRegOneOff)
    BranchEq = 170,
    BranchNe = 171,
    BranchLtU = 172,
    BranchLtS = 173,
    BranchGeU = 174,
    BranchGeS = 175,
    // Compound indirect jump (TwoRegTwoImm)
    LoadImmJumpInd = 180,
    // 32-bit three-register arithmetic
    Add32 = 190,
    Sub32 = 191,
    Mul32 = 192,
    DivU32 = 193,
    DivS32 = 194,
    RemU32 = 195,
    RemS32 = 196,
    // 32-bit shift operations (ThreeReg)
    ShloL32 = 197,
    ShloR32 = 198,
    SharR32 = 199,
    // 64-bit three-register arithmetic
    Add64 = 200,
    Sub64 = 201,
    Mul64 = 202,
    DivU64 = 203,
    DivS64 = 204,
    RemU64 = 205,
    RemS64 = 206,
    ShloL64 = 207,
    ShloR64 = 208,
    SharR64 = 209,
    // Bitwise operations (ThreeReg)
    And = 210,
    Xor = 211,
    Or = 212,
    // Upper multiply (ThreeReg)
    MulUpperSS = 213,
    MulUpperUU = 214,
    MulUpperSU = 215,
    // Comparison (ThreeReg) - result is 0 or 1
    SetLtU = 216,
    SetLtS = 217,
    // Conditional move (ThreeReg)
    CmovIz = 218,
    CmovNz = 219,
    // Rotate (ThreeReg)
    RotL64 = 220,
    RotL32 = 221,
    RotR64 = 222,
    RotR32 = 223,
    // Inverted bitwise (ThreeReg)
    AndInv = 224,
    OrInv = 225,
    Xnor = 226,
    // Min/Max (ThreeReg)
    Max = 227,
    MaxU = 228,
    Min = 229,
    MinU = 230,
}

impl Opcode {
    #[must_use]
    pub const fn from_u8(opcode: u8) -> Option<Self> {
        match opcode {
            0 => Some(Self::Trap),
            1 => Some(Self::Fallthrough),
            10 => Some(Self::Ecalli),
            20 => Some(Self::LoadImm64),
            30 => Some(Self::StoreImmU8),
            31 => Some(Self::StoreImmU16),
            32 => Some(Self::StoreImmU32),
            33 => Some(Self::StoreImmU64),
            40 => Some(Self::Jump),
            50 => Some(Self::JumpInd),
            51 => Some(Self::LoadImm),
            52 => Some(Self::LoadU8),
            53 => Some(Self::LoadI8),
            54 => Some(Self::LoadU16),
            55 => Some(Self::LoadI16),
            56 => Some(Self::LoadU32),
            57 => Some(Self::LoadI32),
            58 => Some(Self::LoadU64),
            59 => Some(Self::StoreU8),
            60 => Some(Self::StoreU16),
            61 => Some(Self::StoreU32),
            62 => Some(Self::StoreU64),
            70 => Some(Self::StoreImmIndU8),
            71 => Some(Self::StoreImmIndU16),
            72 => Some(Self::StoreImmIndU32),
            73 => Some(Self::StoreImmIndU64),
            80 => Some(Self::LoadImmJump),
            81 => Some(Self::BranchEqImm),
            82 => Some(Self::BranchNeImm),
            83 => Some(Self::BranchLtUImm),
            84 => Some(Self::BranchLeUImm),
            85 => Some(Self::BranchGeUImm),
            86 => Some(Self::BranchGtUImm),
            87 => Some(Self::BranchLtSImm),
            88 => Some(Self::BranchLeSImm),
            89 => Some(Self::BranchGeSImm),
            90 => Some(Self::BranchGtSImm),
            100 => Some(Self::MoveReg),
            101 => Some(Self::Sbrk),
            102 => Some(Self::CountSetBits64),
            103 => Some(Self::CountSetBits32),
            104 => Some(Self::LeadingZeroBits64),
            105 => Some(Self::LeadingZeroBits32),
            106 => Some(Self::TrailingZeroBits64),
            107 => Some(Self::TrailingZeroBits32),
            108 => Some(Self::SignExtend8),
            109 => Some(Self::SignExtend16),
            110 => Some(Self::ZeroExtend16),
            111 => Some(Self::ReverseBytes),
            120 => Some(Self::StoreIndU8),
            121 => Some(Self::StoreIndU16),
            122 => Some(Self::StoreIndU32),
            123 => Some(Self::StoreIndU64),
            124 => Some(Self::LoadIndU8),
            125 => Some(Self::LoadIndI8),
            126 => Some(Self::LoadIndU16),
            127 => Some(Self::LoadIndI16),
            128 => Some(Self::LoadIndU32),
            129 => Some(Self::LoadIndI32),
            130 => Some(Self::LoadIndU64),
            131 => Some(Self::AddImm32),
            132 => Some(Self::AndImm),
            133 => Some(Self::XorImm),
            134 => Some(Self::OrImm),
            135 => Some(Self::MulImm32),
            136 => Some(Self::SetLtUImm),
            137 => Some(Self::SetLtSImm),
            138 => Some(Self::ShloLImm32),
            139 => Some(Self::ShloRImm32),
            140 => Some(Self::SharRImm32),
            141 => Some(Self::NegAddImm32),
            142 => Some(Self::SetGtUImm),
            143 => Some(Self::SetGtSImm),
            144 => Some(Self::ShloLImmAlt32),
            145 => Some(Self::ShloRImmAlt32),
            146 => Some(Self::SharRImmAlt32),
            147 => Some(Self::CmovIzImm),
            148 => Some(Self::CmovNzImm),
            149 => Some(Self::AddImm64),
            150 => Some(Self::MulImm64),
            151 => Some(Self::ShloLImm64),
            152 => Some(Self::ShloRImm64),
            153 => Some(Self::SharRImm64),
            154 => Some(Self::NegAddImm64),
            155 => Some(Self::ShloLImmAlt64),
            156 => Some(Self::ShloRImmAlt64),
            157 => Some(Self::SharRImmAlt64),
            158 => Some(Self::RotRImm64),
            159 => Some(Self::RotRImmAlt64),
            160 => Some(Self::RotRImm32),
            161 => Some(Self::RotRImmAlt32),
            170 => Some(Self::BranchEq),
            171 => Some(Self::BranchNe),
            172 => Some(Self::BranchLtU),
            173 => Some(Self::BranchLtS),
            174 => Some(Self::BranchGeU),
            175 => Some(Self::BranchGeS),
            180 => Some(Self::LoadImmJumpInd),
            190 => Some(Self::Add32),
            191 => Some(Self::Sub32),
            192 => Some(Self::Mul32),
            193 => Some(Self::DivU32),
            194 => Some(Self::DivS32),
            195 => Some(Self::RemU32),
            196 => Some(Self::RemS32),
            197 => Some(Self::ShloL32),
            198 => Some(Self::ShloR32),
            199 => Some(Self::SharR32),
            200 => Some(Self::Add64),
            201 => Some(Self::Sub64),
            202 => Some(Self::Mul64),
            203 => Some(Self::DivU64),
            204 => Some(Self::DivS64),
            205 => Some(Self::RemU64),
            206 => Some(Self::RemS64),
            207 => Some(Self::ShloL64),
            208 => Some(Self::ShloR64),
            209 => Some(Self::SharR64),
            210 => Some(Self::And),
            211 => Some(Self::Xor),
            212 => Some(Self::Or),
            213 => Some(Self::MulUpperSS),
            214 => Some(Self::MulUpperUU),
            215 => Some(Self::MulUpperSU),
            216 => Some(Self::SetLtU),
            217 => Some(Self::SetLtS),
            218 => Some(Self::CmovIz),
            219 => Some(Self::CmovNz),
            220 => Some(Self::RotL64),
            221 => Some(Self::RotL32),
            222 => Some(Self::RotR64),
            223 => Some(Self::RotR32),
            224 => Some(Self::AndInv),
            225 => Some(Self::OrInv),
            226 => Some(Self::Xnor),
            227 => Some(Self::Max),
            228 => Some(Self::MaxU),
            229 => Some(Self::Min),
            230 => Some(Self::MinU),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_terminating(self) -> bool {
        matches!(
            self,
            Self::Trap
                | Self::Fallthrough
                | Self::Jump
                | Self::LoadImmJump
                | Self::JumpInd
                | Self::BranchEqImm
                | Self::BranchNeImm
                | Self::BranchLtUImm
                | Self::BranchLeUImm
                | Self::BranchGeUImm
                | Self::BranchGtUImm
                | Self::BranchLtSImm
                | Self::BranchLeSImm
                | Self::BranchGeSImm
                | Self::BranchGtSImm
                | Self::BranchEq
                | Self::BranchNe
                | Self::BranchLtU
                | Self::BranchLtS
                | Self::BranchGeU
                | Self::BranchGeS
                | Self::LoadImmJumpInd
        )
    }
}

impl TryFrom<u8> for Opcode {
    type Error = ();

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        Self::from_u8(value).ok_or(())
    }
}
