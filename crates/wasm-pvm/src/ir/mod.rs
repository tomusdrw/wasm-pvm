mod builder;
mod display;
mod instruction;
mod optimizer;

pub use builder::build_ir;
pub use instruction::IrInstruction;
pub use optimizer::optimize;
