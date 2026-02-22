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
