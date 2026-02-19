// PVM encoding utilities use explicit 'as' casts for byte packing and serialization.
#![allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]

mod blob;
mod instruction;
mod opcode;
pub(crate) mod peephole;

pub use blob::ProgramBlob;
pub(crate) use blob::encode_var_u32;
pub use instruction::Instruction;
pub use opcode::Opcode;
