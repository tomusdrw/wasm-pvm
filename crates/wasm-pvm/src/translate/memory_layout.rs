//! PVM memory address layout constants.

// Memory layout constants often use negative i32s or large u32s that wrap.
#![allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]

//! All WASM-to-PVM memory regions are defined here so the layout can be
//! understood and modified in one place.
//!
//! ```text
//! PVM Address Space:
//!   0x00000 - 0x0FFFF   Reserved (fault on access)
//!   0x10000 - 0x1FFFF   Read-only data segment (RO_DATA_BASE)
//!   0x30000 - 0x3FEFF   Globals + user heap (GLOBAL_MEMORY_BASE)
//!   0x3FF00 - 0x3FFFF   Parameter overflow area (PARAM_OVERFLOW_BASE)
//!   0x40000 - 0x4FFFF+  Spilled locals (SPILLED_LOCALS_BASE)
//!   0x50000+            WASM linear memory (computed dynamically)
//!   ...
//!   0xFEFE0000          Stack segment end (stack grows downward)
//!   0xFFFF0000          Exit address (EXIT_ADDRESS)
//! ```

/// Base address for the read-only data segment (dispatch tables, constant data).
pub const RO_DATA_BASE: i32 = 0x10000;

/// Base address for WASM globals in PVM memory.
/// Each global occupies 4 bytes at `GLOBAL_MEMORY_BASE + index * 4`.
pub const GLOBAL_MEMORY_BASE: i32 = 0x30000;

/// Temporary area for passing overflow parameters (5th+ args) during `call_indirect`.
/// The caller writes here, and the callee's prologue copies to its spilled local addresses.
/// Supports up to 8 overflow parameters (64 bytes).
/// Reduced from 0x3FF00 to save space (allows 8KB for globals).
pub const PARAM_OVERFLOW_BASE: i32 = 0x32000;

/// Base address for spilled locals in memory.
/// Layout: 0x30000+ globals, 0x32000 overflow, 0x32100+ spilled locals.
/// User programs should use `heap.alloc()` (AS) or `memory.grow` (WASM) for dynamic allocation.
pub const SPILLED_LOCALS_BASE: i32 = 0x32100;

/// Bytes allocated per function for spilled locals.
/// Set to 0 as modern compiler spills to stack (r1-relative).
pub const SPILLED_LOCALS_PER_FUNC: i32 = 0;

/// Stack segment end address (where the stack pointer starts, grows downward).
pub const STACK_SEGMENT_END: i32 = 0xFEFE_0000u32 as i32;

/// Default stack size limit (64KB, matching SPI default).
pub const DEFAULT_STACK_SIZE: u32 = 64 * 1024;

/// Exit address: jumping here terminates the program.
/// This is `0xFFFF0000` interpreted as a signed i32.
pub const EXIT_ADDRESS: i32 = -65536;

/// Operand stack spill area base offset (relative to SP, negative direction).
/// Spilled operand stack values are stored at `SP + OPERAND_SPILL_BASE + slot*8`.
pub const OPERAND_SPILL_BASE: i32 = -0x100;

/// Minimum address the stack pointer can reach (`STACK_SEGMENT_END - stack_size`).
/// If SP goes below this, we have a stack overflow.
#[must_use]
pub fn stack_limit(stack_size: u32) -> i32 {
    (STACK_SEGMENT_END as u32).wrapping_sub(stack_size) as i32
}

/// Compute the base address for WASM linear memory in PVM address space.
/// This must be placed after the spilled locals region to avoid overlap.
/// WASM memory address 0 maps to this PVM address.
/// All `i32.load`/`i32.store` operations add this offset to the WASM address.
#[must_use]
pub fn compute_wasm_memory_base(_num_local_funcs: usize) -> i32 {
    // No per-function allocation (spills are on stack).
    // No 64KB alignment required for base address.
    SPILLED_LOCALS_BASE
}

/// Offset within `GLOBAL_MEMORY_BASE` for the compiler-managed memory size global.
/// This is stored AFTER all user globals: address = 0x30000 + (`num_globals` * 4).
/// Value is the current memory size in 64KB pages (u32).
#[must_use]
pub fn memory_size_global_offset(num_globals: usize) -> i32 {
    GLOBAL_MEMORY_BASE + (num_globals as i32 * 4)
}

/// Offset within `GLOBAL_MEMORY_BASE` for a passive data segment's effective length.
/// Stored after the memory size global: `memory_size_offset + 4 + ordinal * 4`.
/// Used for bounds checking in `memory.init` and zeroed by `data.drop`.
#[must_use]
pub fn data_segment_length_offset(num_globals: usize, ordinal: usize) -> i32 {
    memory_size_global_offset(num_globals) + 4 + (ordinal as i32 * 4)
}

/// Compute the spilled local address for a given function and local index.
#[must_use]
pub fn spilled_local_addr(func_idx: usize, local_offset: i32) -> i32 {
    SPILLED_LOCALS_BASE + (func_idx as i32) * SPILLED_LOCALS_PER_FUNC + local_offset
}

/// Compute the global variable address for a given global index.
#[must_use]
pub fn global_addr(idx: u32) -> i32 {
    GLOBAL_MEMORY_BASE + (idx as i32) * 4
}
