#![allow(
    clippy::cast_possible_truncation, // intentional: WASM uses i32/i64, PVM uses u8 registers
    clippy::cast_possible_wrap, // intentional: unsigned/signed conversions for WASM address arithmetic
    clippy::cast_sign_loss, // intentional: WASM addresses are i32 but stored as u32
    clippy::too_many_lines, // lowering.rs is intentionally monolithic for V1
    clippy::missing_errors_doc // will be addressed in V2 documentation pass
)]

pub mod error;
pub mod llvm_backend;
pub mod llvm_frontend;
pub mod pvm;
pub mod spi;
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
pub use translate::compile;
