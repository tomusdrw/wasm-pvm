use std::fmt;

use super::IrInstruction;

impl fmt::Display for IrInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Constants
            IrInstruction::I32Const(v) => write!(f, "i32.const {v}"),
            IrInstruction::I64Const(v) => write!(f, "i64.const {v}"),

            // Locals / Globals
            IrInstruction::LocalGet(i) => write!(f, "local.get {i}"),
            IrInstruction::LocalSet(i) => write!(f, "local.set {i}"),
            IrInstruction::LocalTee(i) => write!(f, "local.tee {i}"),
            IrInstruction::GlobalGet(i) => write!(f, "global.get {i}"),
            IrInstruction::GlobalSet(i) => write!(f, "global.set {i}"),

            // Arithmetic i32
            IrInstruction::I32Add => write!(f, "i32.add"),
            IrInstruction::I32Sub => write!(f, "i32.sub"),
            IrInstruction::I32Mul => write!(f, "i32.mul"),
            IrInstruction::I32DivU => write!(f, "i32.div_u"),
            IrInstruction::I32DivS => write!(f, "i32.div_s"),
            IrInstruction::I32RemU => write!(f, "i32.rem_u"),
            IrInstruction::I32RemS => write!(f, "i32.rem_s"),

            // Arithmetic i64
            IrInstruction::I64Add => write!(f, "i64.add"),
            IrInstruction::I64Sub => write!(f, "i64.sub"),
            IrInstruction::I64Mul => write!(f, "i64.mul"),
            IrInstruction::I64DivU => write!(f, "i64.div_u"),
            IrInstruction::I64DivS => write!(f, "i64.div_s"),
            IrInstruction::I64RemU => write!(f, "i64.rem_u"),
            IrInstruction::I64RemS => write!(f, "i64.rem_s"),

            // Bitwise
            IrInstruction::I32And => write!(f, "i32.and"),
            IrInstruction::I32Or => write!(f, "i32.or"),
            IrInstruction::I32Xor => write!(f, "i32.xor"),
            IrInstruction::I64And => write!(f, "i64.and"),
            IrInstruction::I64Or => write!(f, "i64.or"),
            IrInstruction::I64Xor => write!(f, "i64.xor"),

            // Shifts
            IrInstruction::I32Shl => write!(f, "i32.shl"),
            IrInstruction::I32ShrU => write!(f, "i32.shr_u"),
            IrInstruction::I32ShrS => write!(f, "i32.shr_s"),
            IrInstruction::I64Shl => write!(f, "i64.shl"),
            IrInstruction::I64ShrU => write!(f, "i64.shr_u"),
            IrInstruction::I64ShrS => write!(f, "i64.shr_s"),

            // Rotations
            IrInstruction::I32Rotl => write!(f, "i32.rotl"),
            IrInstruction::I32Rotr => write!(f, "i32.rotr"),
            IrInstruction::I64Rotl => write!(f, "i64.rotl"),
            IrInstruction::I64Rotr => write!(f, "i64.rotr"),

            // Bit counting
            IrInstruction::I32Clz => write!(f, "i32.clz"),
            IrInstruction::I32Ctz => write!(f, "i32.ctz"),
            IrInstruction::I32Popcnt => write!(f, "i32.popcnt"),
            IrInstruction::I64Clz => write!(f, "i64.clz"),
            IrInstruction::I64Ctz => write!(f, "i64.ctz"),
            IrInstruction::I64Popcnt => write!(f, "i64.popcnt"),

            // Comparisons
            IrInstruction::I32Eqz => write!(f, "i32.eqz"),
            IrInstruction::I32Eq => write!(f, "i32.eq"),
            IrInstruction::I32Ne => write!(f, "i32.ne"),
            IrInstruction::I32LtU => write!(f, "i32.lt_u"),
            IrInstruction::I32LtS => write!(f, "i32.lt_s"),
            IrInstruction::I32GtU => write!(f, "i32.gt_u"),
            IrInstruction::I32GtS => write!(f, "i32.gt_s"),
            IrInstruction::I32LeU => write!(f, "i32.le_u"),
            IrInstruction::I32LeS => write!(f, "i32.le_s"),
            IrInstruction::I32GeU => write!(f, "i32.ge_u"),
            IrInstruction::I32GeS => write!(f, "i32.ge_s"),
            IrInstruction::I64Eqz => write!(f, "i64.eqz"),
            IrInstruction::I64Eq => write!(f, "i64.eq"),
            IrInstruction::I64Ne => write!(f, "i64.ne"),
            IrInstruction::I64LtU => write!(f, "i64.lt_u"),
            IrInstruction::I64LtS => write!(f, "i64.lt_s"),
            IrInstruction::I64GtU => write!(f, "i64.gt_u"),
            IrInstruction::I64GtS => write!(f, "i64.gt_s"),
            IrInstruction::I64LeU => write!(f, "i64.le_u"),
            IrInstruction::I64LeS => write!(f, "i64.le_s"),
            IrInstruction::I64GeU => write!(f, "i64.ge_u"),
            IrInstruction::I64GeS => write!(f, "i64.ge_s"),

            // Memory loads
            IrInstruction::I32Load { offset } => write!(f, "i32.load offset={offset}"),
            IrInstruction::I64Load { offset } => write!(f, "i64.load offset={offset}"),
            IrInstruction::I32Load8U { offset } => write!(f, "i32.load8_u offset={offset}"),
            IrInstruction::I32Load8S { offset } => write!(f, "i32.load8_s offset={offset}"),
            IrInstruction::I32Load16U { offset } => write!(f, "i32.load16_u offset={offset}"),
            IrInstruction::I32Load16S { offset } => write!(f, "i32.load16_s offset={offset}"),
            IrInstruction::I64Load8U { offset } => write!(f, "i64.load8_u offset={offset}"),
            IrInstruction::I64Load8S { offset } => write!(f, "i64.load8_s offset={offset}"),
            IrInstruction::I64Load16U { offset } => write!(f, "i64.load16_u offset={offset}"),
            IrInstruction::I64Load16S { offset } => write!(f, "i64.load16_s offset={offset}"),
            IrInstruction::I64Load32U { offset } => write!(f, "i64.load32_u offset={offset}"),
            IrInstruction::I64Load32S { offset } => write!(f, "i64.load32_s offset={offset}"),

            // Memory stores
            IrInstruction::I32Store { offset } => write!(f, "i32.store offset={offset}"),
            IrInstruction::I64Store { offset } => write!(f, "i64.store offset={offset}"),
            IrInstruction::I32Store8 { offset } => write!(f, "i32.store8 offset={offset}"),
            IrInstruction::I32Store16 { offset } => write!(f, "i32.store16 offset={offset}"),
            IrInstruction::I64Store8 { offset } => write!(f, "i64.store8 offset={offset}"),
            IrInstruction::I64Store16 { offset } => write!(f, "i64.store16 offset={offset}"),
            IrInstruction::I64Store32 { offset } => write!(f, "i64.store32 offset={offset}"),

            // Memory management
            IrInstruction::MemorySize => write!(f, "memory.size"),
            IrInstruction::MemoryGrow => write!(f, "memory.grow"),
            IrInstruction::MemoryFill => write!(f, "memory.fill"),
            IrInstruction::MemoryCopy => write!(f, "memory.copy"),

            // Control flow
            IrInstruction::Block { has_result } => {
                if *has_result {
                    write!(f, "block (result)")
                } else {
                    write!(f, "block")
                }
            }
            IrInstruction::Loop => write!(f, "loop"),
            IrInstruction::If { has_result } => {
                if *has_result {
                    write!(f, "if (result)")
                } else {
                    write!(f, "if")
                }
            }
            IrInstruction::Else => write!(f, "else"),
            IrInstruction::End => write!(f, "end"),
            IrInstruction::Br(depth) => write!(f, "br {depth}"),
            IrInstruction::BrIf(depth) => write!(f, "br_if {depth}"),
            IrInstruction::BrTable { targets, default } => {
                write!(f, "br_table")?;
                for t in targets {
                    write!(f, " {t}")?;
                }
                write!(f, " {default}")
            }
            IrInstruction::Return => write!(f, "return"),

            // Calls
            IrInstruction::Call(idx) => write!(f, "call {idx}"),
            IrInstruction::CallIndirect {
                type_idx,
                table_idx,
            } => write!(f, "call_indirect (type {type_idx}) (table {table_idx})"),

            // Stack manipulation
            IrInstruction::Drop => write!(f, "drop"),
            IrInstruction::Select => write!(f, "select"),

            // Misc
            IrInstruction::Unreachable => write!(f, "unreachable"),
            IrInstruction::Nop => write!(f, "nop"),

            // Conversions
            IrInstruction::I32WrapI64 => write!(f, "i32.wrap_i64"),
            IrInstruction::I64ExtendI32S => write!(f, "i64.extend_i32_s"),
            IrInstruction::I64ExtendI32U => write!(f, "i64.extend_i32_u"),
            IrInstruction::I32Extend8S => write!(f, "i32.extend8_s"),
            IrInstruction::I32Extend16S => write!(f, "i32.extend16_s"),
            IrInstruction::I64Extend8S => write!(f, "i64.extend8_s"),
            IrInstruction::I64Extend16S => write!(f, "i64.extend16_s"),
            IrInstruction::I64Extend32S => write!(f, "i64.extend32_s"),

            // Float truncation stubs
            IrInstruction::I32TruncSatF64U => write!(f, "i32.trunc_sat_f64_u"),
            IrInstruction::I32TruncSatF64S => write!(f, "i32.trunc_sat_f64_s"),
            IrInstruction::I32TruncSatF32U => write!(f, "i32.trunc_sat_f32_u"),
            IrInstruction::I32TruncSatF32S => write!(f, "i32.trunc_sat_f32_s"),
            IrInstruction::I64TruncSatF64U => write!(f, "i64.trunc_sat_f64_u"),
            IrInstruction::I64TruncSatF64S => write!(f, "i64.trunc_sat_f64_s"),
            IrInstruction::I64TruncSatF32U => write!(f, "i64.trunc_sat_f32_u"),
            IrInstruction::I64TruncSatF32S => write!(f, "i64.trunc_sat_f32_s"),
        }
    }
}
