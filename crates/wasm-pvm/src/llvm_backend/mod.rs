// LLVM IR backend: lowers LLVM IR → PVM bytecode.

mod lowering;

pub use lowering::{
    LlvmCallFixup, LlvmFunctionTranslation, LlvmIndirectCallFixup, LoweringContext, lower_function,
};
