/// IR instruction set: a clean, self-contained representation of WASM operations.
///
/// Each variant maps 1:1 to a WASM operator that the compiler supports.
/// This decouples the compiler from `wasmparser` types and enables
/// inspection, testing, and future optimization passes.
#[derive(Debug, Clone, PartialEq)]
pub enum IrInstruction {
    // === Constants ===
    I32Const(i32),
    I64Const(i64),

    // === Locals / Globals ===
    LocalGet(u32),
    LocalSet(u32),
    LocalTee(u32),
    GlobalGet(u32),
    GlobalSet(u32),

    // === Arithmetic i32 ===
    I32Add,
    I32Sub,
    I32Mul,
    I32DivU,
    I32DivS,
    I32RemU,
    I32RemS,

    // === Arithmetic i64 ===
    I64Add,
    I64Sub,
    I64Mul,
    I64DivU,
    I64DivS,
    I64RemU,
    I64RemS,

    // === Bitwise ===
    I32And,
    I32Or,
    I32Xor,
    I64And,
    I64Or,
    I64Xor,

    // === Shifts ===
    I32Shl,
    I32ShrU,
    I32ShrS,
    I64Shl,
    I64ShrU,
    I64ShrS,

    // === Rotations ===
    I32Rotl,
    I32Rotr,
    I64Rotl,
    I64Rotr,

    // === Bit counting ===
    I32Clz,
    I32Ctz,
    I32Popcnt,
    I64Clz,
    I64Ctz,
    I64Popcnt,

    // === Comparisons ===
    I32Eqz,
    I32Eq,
    I32Ne,
    I32LtU,
    I32LtS,
    I32GtU,
    I32GtS,
    I32LeU,
    I32LeS,
    I32GeU,
    I32GeS,
    I64Eqz,
    I64Eq,
    I64Ne,
    I64LtU,
    I64LtS,
    I64GtU,
    I64GtS,
    I64LeU,
    I64LeS,
    I64GeU,
    I64GeS,

    // === Memory loads ===
    I32Load { offset: u64 },
    I64Load { offset: u64 },
    I32Load8U { offset: u64 },
    I32Load8S { offset: u64 },
    I32Load16U { offset: u64 },
    I32Load16S { offset: u64 },
    I64Load8U { offset: u64 },
    I64Load8S { offset: u64 },
    I64Load16U { offset: u64 },
    I64Load16S { offset: u64 },
    I64Load32U { offset: u64 },
    I64Load32S { offset: u64 },

    // === Memory stores ===
    I32Store { offset: u64 },
    I64Store { offset: u64 },
    I32Store8 { offset: u64 },
    I32Store16 { offset: u64 },
    I64Store8 { offset: u64 },
    I64Store16 { offset: u64 },
    I64Store32 { offset: u64 },

    // === Memory management ===
    MemorySize,
    MemoryGrow,
    MemoryFill,
    MemoryCopy,

    // === Control flow ===
    Block { has_result: bool },
    Loop,
    If { has_result: bool },
    Else,
    End,
    Br(u32),
    BrIf(u32),
    BrTable { targets: Vec<u32>, default: u32 },
    Return,

    // === Calls ===
    Call(u32),
    CallIndirect { type_idx: u32, table_idx: u32 },

    // === Stack manipulation ===
    Drop,
    Select,

    // === Misc ===
    Unreachable,
    Nop,

    // === Conversions ===
    I32WrapI64,
    I64ExtendI32S,
    I64ExtendI32U,
    I32Extend8S,
    I32Extend16S,
    I64Extend8S,
    I64Extend16S,
    I64Extend32S,

    // === Float truncation stubs ===
    I32TruncSatF64U,
    I32TruncSatF64S,
    I32TruncSatF32U,
    I32TruncSatF32S,
    I64TruncSatF64U,
    I64TruncSatF64S,
    I64TruncSatF32U,
    I64TruncSatF32S,
}
