#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines,
    clippy::missing_errors_doc
)]

pub mod error;
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
