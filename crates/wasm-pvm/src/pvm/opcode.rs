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
    MoveReg = 100,
    AddImm32 = 131,
    AddImm64 = 149,
    Add32 = 190,
    Add64 = 200,
}

impl Opcode {
    #[must_use]
    pub const fn is_terminating(self) -> bool {
        matches!(self, Self::Trap | Self::Fallthrough | Self::Jump | Self::JumpInd)
    }
}
