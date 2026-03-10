#![allow(
    clippy::too_many_lines, // TODO: Remove after refactoring lowering.rs (Task #30)
    clippy::missing_errors_doc // TODO: Add docs in V2 (Task #34/Documentation)
)]

pub mod abi;
pub mod error;
pub mod memory_layout;
pub mod pvm;
pub mod spi;

#[cfg(feature = "compiler")]
pub mod llvm_backend;
#[cfg(feature = "compiler")]
pub mod llvm_frontend;
#[cfg(feature = "compiler")]
pub mod translate;

/// Test harness module for writing unit and integration tests.
///
/// This module is only available when running tests or when the
/// `test-harness` feature is enabled.
#[cfg(any(test, feature = "test-harness"))]
pub mod test_harness;

pub use error::{Error, Result};
pub use pvm::{Instruction, Opcode, ProgramBlob};
pub use spi::SpiProgram;

#[cfg(feature = "compiler")]
pub use translate::{
    CompileOptions, ImportAction, OptimizationFlags, compile, compile_with_options,
};
