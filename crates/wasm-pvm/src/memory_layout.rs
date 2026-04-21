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
//!   0x30000             Mem-size slot (4 bytes, only when memory.size/grow/init used)
//!   0x30000 / 0x30004+  User globals (4 bytes each; offset by 4 when mem-size slot present)
//!   globals_end+        Passive data segment effective-length slots (4 bytes each)
//!   passive_lens_end+   Parameter overflow area (256 bytes, 8-byte aligned, only when any module type signature has >4 params — gated by `needs_param_overflow`, which covers both local functions and `call_indirect` types)
//!   region_end          WASM linear memory (no 4KB alignment; sits immediately after last region)
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
pub fn compute_param_overflow_base(
    num_globals: usize,
    num_passive_segments: usize,
    has_mem_size_global: bool,
) -> i32 {
    let globals_end = GLOBAL_MEMORY_BASE as usize
        + globals_region_size(num_globals, num_passive_segments, has_mem_size_global);
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
///
/// Layout (from `GLOBAL_MEMORY_BASE` upward):
/// 1. The compiler-managed memory-size slot (4 bytes) — only when the module
///    uses `memory.size`/`memory.grow`/`memory.init`.
/// 2. User globals (4 bytes each).
/// 3. Passive data segment effective-length slots (4 bytes each).
/// 4. Parameter overflow area (256 bytes) — only when any module type
///    signature (local function or `call_indirect` target) has more than
///    `MAX_LOCAL_REGS` parameters (tracked by `needs_param_overflow`).
///
/// `wasm_memory_base` sits immediately after region 4 (or the last present
/// region). It is **not** 4KB-aligned: anan-as allocates `rw_data` one PVM page
/// at a time via `setData`, and `heapZerosStart` is separately computed as
/// `heapStart + alignToPageSize(rwLength)`, so the base can land at any byte
/// offset inside the first `rw_data` page without leaving unreachable memory.
/// Skipping the alignment collapses the 4KB leading-zero prefix that would
/// otherwise inflate `rw_data` for every fixture declaring linear memory.
#[must_use]
pub fn compute_wasm_memory_base(
    num_globals: usize,
    num_passive_segments: usize,
    has_mem_size_global: bool,
    needs_param_overflow: bool,
) -> i32 {
    let globals_end = GLOBAL_MEMORY_BASE as usize
        + globals_region_size(num_globals, num_passive_segments, has_mem_size_global);
    let region_end = if needs_param_overflow {
        compute_param_overflow_base(num_globals, num_passive_segments, has_mem_size_global) as usize
            + PARAM_OVERFLOW_SIZE
    } else {
        globals_end
    };
    // No 4KB alignment — anan-as page-aligns the rw_data tail (`heapZerosStart`)
    // independently, so the base can sit at any byte offset in the first page.
    region_end as i32
}

/// Bytes reserved for globals, (optionally) the compiler-managed memory-size
/// global, and passive data segment lengths.
#[must_use]
pub fn globals_region_size(
    num_globals: usize,
    num_passive_segments: usize,
    has_mem_size_global: bool,
) -> usize {
    (num_globals + usize::from(has_mem_size_global) + num_passive_segments) * 4
}

/// PVM address of the compiler-managed memory-size global.
/// Always `GLOBAL_MEMORY_BASE` (`0x30000`) when emitted — a stable, program-
/// independent slot so memory-op lowering doesn't need to know `num_globals`.
///
/// Only meaningful when the module uses `memory.size`/`memory.grow`/`memory.init`.
/// When unused, the slot is not emitted and user globals occupy position 0
/// instead — callers must gate on `needs_memory_size_global`.
#[must_use]
pub fn memory_size_global_offset() -> i32 {
    GLOBAL_MEMORY_BASE
}

/// Offset within `GLOBAL_MEMORY_BASE` for a passive data segment's effective length.
/// Stored after the mem-size slot (when present) and user globals.
/// Used for bounds checking in `memory.init` and zeroed by `data.drop`.
#[must_use]
pub fn data_segment_length_offset(
    num_globals: usize,
    ordinal: usize,
    has_mem_size_global: bool,
) -> i32 {
    let mem_size_slot = i32::from(has_mem_size_global);
    GLOBAL_MEMORY_BASE + ((mem_size_slot + num_globals as i32 + ordinal as i32) * 4)
}

/// Compute the global variable address for a given global index.
///
/// When `has_mem_size_global` is true, user globals start 4 bytes past
/// `GLOBAL_MEMORY_BASE` (the mem-size slot occupies position 0). Otherwise
/// globals start at `GLOBAL_MEMORY_BASE` itself — no wasted prefix.
#[must_use]
pub fn global_addr(idx: u32, has_mem_size_global: bool) -> i32 {
    let mem_size_slot = i32::from(has_mem_size_global);
    GLOBAL_MEMORY_BASE + (mem_size_slot + idx as i32) * 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wasm_memory_base_typical_program() {
        // 5 globals, mem-size slot, no passive, overflow needed:
        // globals_end = 0x30000 + (1 + 5) * 4 = 0x30018
        // param_overflow_base = align8(0x30018) = 0x30018
        // region_end = 0x30018 + 256 = 0x30118
        let base = compute_wasm_memory_base(5, 0, true, true);
        assert_eq!(base, 0x30118);
    }

    #[test]
    fn wasm_memory_base_zero_globals_with_memgrow() {
        // Zero user globals, mem-size slot present, no passive, overflow needed:
        // globals_end = 0x30000 + 4 = 0x30004
        // param_overflow_base = align8(0x30004) = 0x30008
        // region_end = 0x30008 + 256 = 0x30108
        let base = compute_wasm_memory_base(0, 0, true, true);
        assert_eq!(base, 0x30108);
    }

    #[test]
    fn wasm_memory_base_zero_globals_memgrow_no_overflow() {
        // The sweet spot: memory.grow-using program with nothing else.
        // Only 4 bytes of mem-size slot; base sits at 0x30004.
        let base = compute_wasm_memory_base(0, 0, true, false);
        assert_eq!(base, GLOBAL_MEMORY_BASE + 4);
    }

    #[test]
    fn wasm_memory_base_bare_minimum_lands_at_globals_base() {
        // No globals, no passive segs, no mem-size global, no param overflow:
        // region_end = 0x30000. Base stays at GLOBAL_MEMORY_BASE.
        let base = compute_wasm_memory_base(0, 0, false, false);
        assert_eq!(base, GLOBAL_MEMORY_BASE);
    }

    #[test]
    fn wasm_memory_base_many_globals_no_4kb_jump() {
        // 2000 globals + mem-size + overflow:
        // globals_end = 0x30000 + 2001 * 4 = 0x31F44
        // param_overflow_base = align8(0x31F44) = 0x31F48
        // region_end = 0x31F48 + 256 = 0x32048
        let base = compute_wasm_memory_base(2000, 0, true, true);
        assert_eq!(base, 0x32048);
    }

    #[test]
    fn wasm_memory_base_globals_no_overflow() {
        // Globals only, no overflow: base sits right after globals — no 4KB jump.
        let base = compute_wasm_memory_base(5, 0, true, false);
        // globals_end = 0x30000 + 6*4 = 0x30018.
        assert_eq!(base, 0x30018);
    }

    #[test]
    fn param_overflow_base_after_globals() {
        // 5 globals + mem_size → globals_end = 0x30018, 8-aligned = 0x30018
        assert_eq!(compute_param_overflow_base(5, 0, true), 0x30018);
        // 0 globals + mem_size → globals_end = 0x30004, 8-aligned = 0x30008
        assert_eq!(compute_param_overflow_base(0, 0, true), 0x30008);
        // 5 globals + 3 passive + mem_size → globals_end = 0x30000 + 36 = 0x30024, 8-aligned = 0x30028
        assert_eq!(compute_param_overflow_base(5, 3, true), 0x30028);
        // 5 globals, no mem_size → globals_end = 0x30014, 8-aligned = 0x30018
        assert_eq!(compute_param_overflow_base(5, 0, false), 0x30018);
    }

    #[test]
    fn param_overflow_does_not_overlap_globals() {
        // With many globals, the overflow area must start after all of them.
        let globals = 1000;
        let globals_end = GLOBAL_MEMORY_BASE as usize + globals_region_size(globals, 0, true);
        let overflow_base = compute_param_overflow_base(globals, 0, true) as usize;
        assert!(
            overflow_base >= globals_end,
            "overflow base 0x{overflow_base:X} overlaps globals_end 0x{globals_end:X}"
        );
    }

    #[test]
    fn globals_region_size_formula() {
        assert_eq!(globals_region_size(0, 0, true), 4); // just memory_size slot
        assert_eq!(globals_region_size(0, 0, false), 0); // nothing
        assert_eq!(globals_region_size(5, 0, true), 24); // 5 globals + 1 mem_size = 6 * 4
        assert_eq!(globals_region_size(5, 0, false), 20); // 5 globals = 5 * 4
        assert_eq!(globals_region_size(5, 3, true), 36); // (5 + 1 + 3) * 4
        assert_eq!(globals_region_size(5, 3, false), 32); // (5 + 3) * 4
    }

    #[test]
    fn global_addr_formula_without_mem_size() {
        // Without the mem-size slot, globals start at GLOBAL_MEMORY_BASE itself.
        assert_eq!(global_addr(0, false), 0x30000);
        assert_eq!(global_addr(1, false), 0x30004);
        assert_eq!(global_addr(10, false), 0x30028);
    }

    #[test]
    fn global_addr_formula_with_mem_size() {
        // With mem-size at 0x30000, globals start at 0x30004.
        assert_eq!(global_addr(0, true), 0x30004);
        assert_eq!(global_addr(1, true), 0x30008);
        assert_eq!(global_addr(10, true), 0x3002C);
    }

    #[test]
    fn memory_size_global_is_stable() {
        // Always at GLOBAL_MEMORY_BASE regardless of global count.
        assert_eq!(memory_size_global_offset(), 0x30000);
    }

    #[test]
    fn data_segment_length_after_mem_size_and_globals() {
        // With mem-size + 5 globals: lens start at 0x30000 + (1+5)*4 = 0x30018.
        assert_eq!(data_segment_length_offset(5, 0, true), 0x30018);
        assert_eq!(data_segment_length_offset(5, 1, true), 0x3001C);
        // Without mem-size: lens start at 0x30000 + 5*4 = 0x30014.
        assert_eq!(data_segment_length_offset(5, 0, false), 0x30014);
        assert_eq!(data_segment_length_offset(5, 1, false), 0x30018);
    }

    #[test]
    fn stack_limit_formula() {
        assert_eq!(
            stack_limit(DEFAULT_STACK_SIZE),
            (0xFEFE_0000u32 - 64 * 1024) as i32
        );
    }
}
