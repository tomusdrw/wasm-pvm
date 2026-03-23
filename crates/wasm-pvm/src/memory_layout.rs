//! PVM memory address layout constants.

// Memory layout constants often use negative i32s or large u32s that wrap.
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

//! All WASM-to-PVM memory regions are defined here so the layout can be
//! understood and modified in one place.
//!
//! ```text
//! PVM Address Space:
//!   0x00000 - 0x0FFFF   Reserved (fault on access)
//!   0x10000 - 0x1FFFF   Read-only data segment (RO_DATA_BASE)
//!   0x20000 - 0x2FFFF   Gap zone (unmapped, guard between RO and RW)
//!   0x30000+            Globals window (GLOBAL_MEMORY_BASE; actual = globals_region_size(...))
//!   globals_end+        Parameter overflow area (PARAM_OVERFLOW_SIZE bytes, 8-byte aligned)
//!   next 4KB boundary   WASM linear memory (4KB-aligned, computed dynamically)
//!   ...
//!   0xFEFE0000          Stack segment end (stack grows downward)
//!   0xFFFF0000          Exit address (EXIT_ADDRESS)
//! ```

/// Base address for the read-only data segment (dispatch tables, constant data).
pub const RO_DATA_BASE: i32 = 0x10000;

/// Base address for WASM globals in PVM memory.
/// Each global occupies 4 bytes at `GLOBAL_MEMORY_BASE + index * 4`.
pub const GLOBAL_MEMORY_BASE: i32 = 0x30000;

/// Size of the parameter overflow area in bytes.
/// Supports up to 32 overflow parameters (5th+ args) during `call_indirect`.
/// Each overflow parameter occupies 8 bytes.
pub const PARAM_OVERFLOW_SIZE: usize = 256;

/// Compute the base address for the parameter overflow area.
/// Placed right after the globals region, 8-byte aligned.
#[must_use]
pub fn compute_param_overflow_base(num_globals: usize, num_passive_segments: usize) -> i32 {
    let globals_end =
        GLOBAL_MEMORY_BASE as usize + globals_region_size(num_globals, num_passive_segments);
    // Align to 8 bytes for clean parameter access.
    ((globals_end + 7) & !7) as i32
}

/// Stack segment end address (where the stack pointer starts, grows downward).
pub const STACK_SEGMENT_END: i32 = 0xFEFE_0000u32 as i32;

/// Default stack size limit (64KB, matching SPI default).
pub const DEFAULT_STACK_SIZE: u32 = 64 * 1024;

/// Exit address: jumping here terminates the program.
/// This is `0xFFFF0000` interpreted as a signed i32.
pub const EXIT_ADDRESS: i32 = -65536;

/// Minimum address the stack pointer can reach (`STACK_SEGMENT_END - stack_size`).
/// If SP goes below this, we have a stack overflow.
#[must_use]
pub fn stack_limit(stack_size: u32) -> i32 {
    (STACK_SEGMENT_END as u32).wrapping_sub(stack_size) as i32
}

/// Compute the base address for WASM linear memory in the PVM address space.
/// Globals, the compiler-managed memory size slot, passive segment lengths,
/// and the parameter overflow area are laid out starting at `GLOBAL_MEMORY_BASE`.
/// The heap begins after all of these regions, aligned to a 4KB PVM page boundary.
///
/// # Why 4KB alignment (not 64KB)
///
/// The result is aligned to the PVM page size (4KB = 0x1000). This is correct
/// because:
/// - The SPI spec requires page-aligned (4KB) `rw_data` lengths, not 64KB.
/// - The anan-as interpreter (`vendor/anan-as/assembly/spi.ts`) uses
///   `alignToPageSize(rwLength)` (4KB) for the heap zeros start, not
///   `alignToSegmentSize` (64KB).
/// - The WASM page size (64KB) governs `memory.grow` granularity only — it
///   controls how much memory grows per step, not where the base address must
///   sit.
/// - Using 4KB alignment saves ~52KB per program (the old 64KB alignment
///   wasted up to 60KB of padding between `globals_end` and the heap start).
#[must_use]
pub fn compute_wasm_memory_base(num_globals: usize, num_passive_segments: usize) -> i32 {
    let param_overflow_end = compute_param_overflow_base(num_globals, num_passive_segments)
        as usize
        + PARAM_OVERFLOW_SIZE;
    // Align to PVM page size (4KB = 0x1000).
    ((param_overflow_end + 0xFFF) & !0xFFF) as i32
}

/// Bytes reserved for globals, the compiler-managed memory size global, and
/// passive data segment lengths.
#[must_use]
pub fn globals_region_size(num_globals: usize, num_passive_segments: usize) -> usize {
    (num_globals + 1 + num_passive_segments) * 4
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

/// Compute the global variable address for a given global index.
#[must_use]
pub fn global_addr(idx: u32) -> i32 {
    GLOBAL_MEMORY_BASE + (idx as i32) * 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_memory_base_typical_program() {
        // Few globals, no passive segments:
        // globals_end = 0x30000 + 24 = 0x30018
        // param_overflow_base = align8(0x30018) = 0x30018
        // param_overflow_end = 0x30018 + 256 = 0x30118
        // aligned to 4KB = 0x31000
        let base = compute_wasm_memory_base(5, 0);
        assert_eq!(base, 0x31000);
    }

    #[test]
    fn wasm_memory_base_zero_globals() {
        let base = compute_wasm_memory_base(0, 0);
        // globals_end = 0x30000 + 4 = 0x30004
        // param_overflow_base = 0x30008 (8-aligned)
        // param_overflow_end = 0x30108
        // aligned = 0x31000
        assert_eq!(base, 0x31000);
    }

    #[test]
    fn wasm_memory_base_many_globals_pushes_base() {
        // 2000 globals + 1 memory_size = 2001 * 4 = 8004 bytes
        // globals_end = 0x30000 + 8004 = 0x31F44
        // param_overflow_base = 0x31F48 (8-aligned)
        // param_overflow_end = 0x32048
        // aligned = 0x33000
        let base = compute_wasm_memory_base(2000, 0);
        assert_eq!(base, 0x33000);
    }

    #[test]
    fn wasm_memory_base_is_4kb_aligned() {
        for globals in [0, 1, 100, 500, 1000] {
            for passive in [0, 1, 5] {
                let base = compute_wasm_memory_base(globals, passive);
                assert_eq!(
                    base & 0xFFF,
                    0,
                    "base 0x{base:X} not 4KB-aligned for {globals} globals, {passive} passive"
                );
            }
        }
    }

    #[test]
    fn param_overflow_base_after_globals() {
        // 5 globals → globals_end = 0x30018, 8-aligned = 0x30018
        assert_eq!(compute_param_overflow_base(5, 0), 0x30018);
        // 0 globals → globals_end = 0x30004, 8-aligned = 0x30008
        assert_eq!(compute_param_overflow_base(0, 0), 0x30008);
        // 5 globals + 3 passive → globals_end = 0x30000 + 36 = 0x30024, 8-aligned = 0x30028
        assert_eq!(compute_param_overflow_base(5, 3), 0x30028);
    }

    #[test]
    fn param_overflow_does_not_overlap_globals() {
        // With many globals, the overflow area must start after all of them.
        let globals = 1000;
        let globals_end = GLOBAL_MEMORY_BASE as usize + globals_region_size(globals, 0);
        let overflow_base = compute_param_overflow_base(globals, 0) as usize;
        assert!(
            overflow_base >= globals_end,
            "overflow base 0x{overflow_base:X} overlaps globals_end 0x{globals_end:X}"
        );
    }

    #[test]
    fn globals_region_size_formula() {
        assert_eq!(globals_region_size(0, 0), 4); // just memory_size slot
        assert_eq!(globals_region_size(5, 0), 24); // 5 globals + 1 mem_size = 6 * 4
        assert_eq!(globals_region_size(5, 3), 36); // (5 + 1 + 3) * 4
    }

    #[test]
    fn global_addr_formula() {
        assert_eq!(global_addr(0), 0x30000);
        assert_eq!(global_addr(1), 0x30004);
        assert_eq!(global_addr(10), 0x30028);
    }

    #[test]
    fn memory_size_global_after_user_globals() {
        assert_eq!(memory_size_global_offset(0), 0x30000);
        assert_eq!(memory_size_global_offset(5), 0x30014);
    }

    #[test]
    fn data_segment_length_after_memory_size() {
        assert_eq!(data_segment_length_offset(5, 0), 0x30018); // mem_size + 4
        assert_eq!(data_segment_length_offset(5, 1), 0x3001C); // + 4
    }

    #[test]
    fn stack_limit_formula() {
        assert_eq!(
            stack_limit(DEFAULT_STACK_SIZE),
            (0xFEFE_0000u32 - 64 * 1024) as i32
        );
    }
}
