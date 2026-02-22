#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    Trap = 0,
    Fallthrough = 1,
    Ecalli = 10,
    LoadImm64 = 20,
    Jump = 40,
    JumpInd = 50,
    LoadImm = 51,
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
    StoreIndU8 = 120,
    StoreIndU16 = 121,
    StoreIndU32 = 122,
    StoreIndU64 = 123,
    LoadIndU8 = 124,
    LoadIndI8 = 125,
    LoadIndU16 = 126,
    LoadIndI16 = 127,
    LoadIndU32 = 128,
    LoadIndU64 = 130,
    AddImm32 = 131,
    // Set if less than immediate (TwoRegOneImm)
    SetLtUImm = 136,
    SetLtSImm = 137,
    // Conditional move with immediate (TwoRegOneImm)
    CmovIzImm = 147,
    CmovNzImm = 148,
    AddImm64 = 149,
    // Branch with two registers (TwoRegOneOff)
    BranchEq = 170,
    BranchNe = 171,
    BranchLtU = 172,
    BranchLtS = 173,
    BranchGeU = 174,
    BranchGeS = 175,
    // 32-bit three-register arithmetic
    Add32 = 190,
    Sub32 = 191,
    Mul32 = 192,
    DivU32 = 193,
    DivS32 = 194,
    RemU32 = 195,
    RemS32 = 196,
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
    // Shift operations (ThreeReg)
    ShloL32 = 197,
    ShloR32 = 198,
    SharR32 = 199,
    // Comparison (ThreeReg) - result is 0 or 1
    SetLtU = 216,
    SetLtS = 217,
}

impl Opcode {
    #[must_use]
    pub const fn is_terminating(self) -> bool {
        matches!(
            self,
            Self::Trap
                | Self::Fallthrough
                | Self::Jump
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
        )
    }
}
