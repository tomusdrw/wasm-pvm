// LLVM IR backend: lowers LLVM IR → PVM bytecode.

mod lowering;

pub use lowering::{
    lower_function, LlvmCallFixup, LlvmFunctionTranslation, LlvmIndirectCallFixup,
    LoweringContext,
};
