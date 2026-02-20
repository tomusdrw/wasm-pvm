//! PVM ABI constants (Registers, Memory Layout, Frame Layout).
//!
//! This module centralizes all definitions related to the PVM execution environment
//! to ensure consistency between the compiler frontend, backend, and tests.

// ── Register Assignments ──

/// Return address register (ra).
/// Holds the return address (jump table index) for function calls.
pub const RETURN_ADDR_REG: u8 = 0;

/// Stack pointer register (sp).
/// Points to the current top of the stack (grows downwards).
pub const STACK_PTR_REG: u8 = 1;

/// Temporary register 1 (t0).
/// Used for loading operands from stack slots or immediate values.
pub const TEMP1: u8 = 2;

/// Temporary register 2 (t1).
/// Used for loading operands from stack slots.
pub const TEMP2: u8 = 3;

/// Temporary register for computation results (t2).
/// Holds the result of ALU operations before storing back to stack slot.
pub const TEMP_RESULT: u8 = 4;

/// Scratch register 1 (s0).
/// General purpose scratch register.
pub const SCRATCH1: u8 = 5;

/// Scratch register 2 (s1).
/// General purpose scratch register.
pub const SCRATCH2: u8 = 6;

/// Return value register (a0).
/// Holds the first return value from a function call.
/// Also used as the pointer to arguments (`args_ptr`) in the entry function.
pub const RETURN_VALUE_REG: u8 = 7;
pub const ARGS_PTR_REG: u8 = 7;

/// Arguments length register (a1).
/// Holds the length of arguments in the entry function.
/// Also used as the second return value if needed (e.g. for entry function (ptr, len)).
pub const ARGS_LEN_REG: u8 = 8;

/// First local variable register (l0).
/// Start of the range of registers used for local variables (callee-saved).
pub const FIRST_LOCAL_REG: u8 = 9;

/// Number of registers dedicated to local variables (r9-r12).
pub const MAX_LOCAL_REGS: usize = 4;

// ── Stack Frame Layout ──

/// Maximum stack frame header size in bytes (used as default when shrink wrapping is disabled).
///
/// Layout (all callee-saved registers):
/// - 0: Saved r0 (ra)
/// - 8: Saved r9 (l0)
/// - 16: Saved r10 (l1)
/// - 24: Saved r11 (l2)
/// - 32: Saved r12 (l3)
///
/// Total: 5 * 8 = 40 bytes.
///
/// With shrink wrapping enabled, the actual header size is dynamic:
/// `8 (ra) + 8 * num_used_callee_regs`. Only registers that are actually
/// used by the function body are saved/restored.
pub const FRAME_HEADER_SIZE: i32 = 40;

/// Operand stack spill area base offset (relative to SP, negative direction).
/// Spilled operand stack values are stored at `SP + OPERAND_SPILL_BASE + slot*8`.
pub const OPERAND_SPILL_BASE: i32 = -0x100;

// ── Memory Layout ──

// Re-export memory layout constants from the translate module for now.
// In a future refactor, we might move the definitions here entirely.
pub use crate::translate::memory_layout::*;
