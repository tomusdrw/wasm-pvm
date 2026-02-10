//! PVM memory address layout constants.
//!
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
pub const PARAM_OVERFLOW_BASE: i32 = 0x3FF00;

/// Base address for spilled locals in memory.
/// Layout: 0x30000-0x300FF globals, 0x30100+ user heap, 0x40000+ spilled locals.
/// User heap can use up to ~64KB (0x30100 to 0x3FFFF) before colliding with spilled locals.
pub const SPILLED_LOCALS_BASE: i32 = 0x40000;

/// Bytes allocated per function for spilled locals (64 locals * 8 bytes).
pub const SPILLED_LOCALS_PER_FUNC: i32 = 512;

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
pub fn compute_wasm_memory_base(num_local_funcs: usize) -> i32 {
    let spilled_locals_end =
        SPILLED_LOCALS_BASE + (num_local_funcs as i32) * SPILLED_LOCALS_PER_FUNC;
    // Align up to 64KB boundary (0x10000) for clean page alignment
    let aligned = (spilled_locals_end + 0xFFFF) & !0xFFFF;
    // Minimum of 0x50000 to maintain backward compatibility for small modules
    aligned.max(0x50000)
}

/// Offset within `GLOBAL_MEMORY_BASE` for the compiler-managed memory size global.
/// This is stored AFTER all user globals: address = 0x30000 + (`num_globals` * 4).
/// Value is the current memory size in 64KB pages (u32).
#[must_use]
pub fn memory_size_global_offset(num_globals: usize) -> i32 {
    GLOBAL_MEMORY_BASE + (num_globals as i32 * 4)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_memory_base_zero_funcs() {
        assert_eq!(compute_wasm_memory_base(0), 0x50000);
    }

    #[test]
    fn wasm_memory_base_one_func() {
        assert_eq!(compute_wasm_memory_base(1), 0x50000);
    }

    #[test]
    fn wasm_memory_base_many_funcs() {
        // 200 funcs: 0x40000 + 200*512 = 0x40000 + 0x19000 = 0x59000
        // Aligned up to 64KB: 0x60000
        assert_eq!(compute_wasm_memory_base(200), 0x60000);
    }

    #[test]
    fn stack_limit_zero() {
        assert_eq!(stack_limit(0), STACK_SEGMENT_END);
    }

    #[test]
    fn stack_limit_default() {
        let limit = stack_limit(DEFAULT_STACK_SIZE);
        let expected = (STACK_SEGMENT_END as u32).wrapping_sub(DEFAULT_STACK_SIZE) as i32;
        assert_eq!(limit, expected);
        // Limit should be below STACK_SEGMENT_END (in unsigned terms)
        assert!((limit as u32) < (STACK_SEGMENT_END as u32));
    }

    #[test]
    fn memory_size_global_offset_zero_globals() {
        assert_eq!(memory_size_global_offset(0), GLOBAL_MEMORY_BASE);
    }

    #[test]
    fn memory_size_global_offset_five_globals() {
        assert_eq!(memory_size_global_offset(5), GLOBAL_MEMORY_BASE + 20);
    }

    #[test]
    fn spilled_local_addr_func0_local0() {
        assert_eq!(spilled_local_addr(0, 0), SPILLED_LOCALS_BASE);
    }

    #[test]
    fn spilled_local_addr_func1_local8() {
        assert_eq!(
            spilled_local_addr(1, 8),
            SPILLED_LOCALS_BASE + SPILLED_LOCALS_PER_FUNC + 8
        );
    }

    #[test]
    fn global_addr_zero() {
        assert_eq!(global_addr(0), GLOBAL_MEMORY_BASE);
    }

    #[test]
    fn global_addr_three() {
        assert_eq!(global_addr(3), GLOBAL_MEMORY_BASE + 12);
    }

    #[test]
    fn non_overlap_invariant() {
        for n in [0, 1, 10, 100, 200] {
            let wasm_base = compute_wasm_memory_base(n);
            assert!(
                global_addr(0) < SPILLED_LOCALS_BASE,
                "globals must be below spilled locals"
            );
            assert!(
                SPILLED_LOCALS_BASE < wasm_base,
                "spilled locals must be below wasm memory base for {n} funcs"
            );
        }
    }
}
