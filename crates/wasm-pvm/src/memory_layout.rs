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
//!   0x30000 / 0x30004+  User globals (per-global width: 4 B for i32/f32, 8 B for i64/f64;
//!                       packed in declaration order, no padding)
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
/// Each user global's offset is precomputed from `global_widths` (see
/// `WasmModule::global_offsets`); i32/f32 globals occupy 4 bytes and i64/f64
/// globals occupy 8 bytes, packed in declaration order with no padding.
pub const GLOBAL_MEMORY_BASE: i32 = 0x30000;

/// Bytes occupied by the compiler-managed memory-size slot, when present.
pub const MEM_SIZE_SLOT_BYTES: usize = 4;

/// Bytes per passive data segment effective-length record.
pub const PASSIVE_SEG_LEN_BYTES: usize = 4;

/// Storage width (bytes) needed for a single WASM global of the given type.
///
/// Only `i32`/`i64` globals reach `WasmModule`'s layout pipeline in practice —
/// `f32`/`f64`/`v128`/ref globals are rejected by `WasmModule::parse` before
/// `global_storage_width` is called. The match is still **exhaustive over all
/// `ValType` variants** with each variant's actual width (not a one-size-fits-all
/// fallback): if the parse-time rejection guard is ever bypassed or relaxed,
/// the backend's `lower_wasm_global_load`/`lower_wasm_global_store` width
/// dispatch only accepts 4/8 byte slots and will fail with
/// `Error::Internal("wasm_global_N has unexpected width …")` rather than
/// silently miscompile a `v128` (16 B) global by truncating to 8 bytes.
///
/// Gated on the `compiler` feature because it consumes `wasmparser::ValType`;
/// the rest of `memory_layout` stays usable without the compiler toolchain.
#[cfg(feature = "compiler")]
#[must_use]
pub fn global_storage_width(ty: wasmparser::ValType) -> u32 {
    match ty {
        wasmparser::ValType::I32 | wasmparser::ValType::F32 => 4,
        // i64/f64 are 8 bytes by definition. `Ref(_)` (funcref/externref)
        // is 1 abstract WASM slot, sized as i64 (8 B) at our i64-uniform
        // ABI — listed here for exhaustiveness even though `WasmModule::parse`
        // rejects ref globals before this function is called.
        wasmparser::ValType::I64 | wasmparser::ValType::F64 | wasmparser::ValType::Ref(_) => 8,
        // v128 is 16 bytes — its actual WASM width. Reaching the backend
        // with a 16-byte slot would surface as
        // `Error::Internal("wasm_global_N has unexpected width 16 bytes")`
        // at lowering rather than a silent 8-byte truncation.
        wasmparser::ValType::V128 => 16,
    }
}

/// Size of the parameter overflow area in bytes.
/// Supports up to 32 overflow parameters (5th+ args) during `call_indirect`.
/// Each overflow parameter occupies 8 bytes.
pub const PARAM_OVERFLOW_SIZE: usize = 256;

/// Maximum total parameter count supported by a single function signature.
/// First `MAX_LOCAL_REGS` (4) go into r9-r12; the next
/// `PARAM_OVERFLOW_SIZE / 8` (32) into the overflow window. Signatures beyond
/// this would have call sites writing past the overflow reservation into
/// WASM linear memory, so `WasmModule::parse` rejects them.
pub const MAX_TOTAL_PARAMS: usize = crate::abi::MAX_LOCAL_REGS + PARAM_OVERFLOW_SIZE / 8;

/// Compute the base address for the parameter overflow area.
/// Placed right after the globals region, 8-byte aligned.
#[must_use]
pub fn compute_param_overflow_base(
    global_widths: &[u32],
    num_passive_segments: usize,
    has_mem_size_global: bool,
) -> i32 {
    let globals_end = GLOBAL_MEMORY_BASE as usize
        + globals_region_size(global_widths, num_passive_segments, has_mem_size_global);
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
    global_widths: &[u32],
    num_passive_segments: usize,
    has_mem_size_global: bool,
    needs_param_overflow: bool,
) -> i32 {
    let globals_end = GLOBAL_MEMORY_BASE as usize
        + globals_region_size(global_widths, num_passive_segments, has_mem_size_global);
    let region_end = if needs_param_overflow {
        compute_param_overflow_base(global_widths, num_passive_segments, has_mem_size_global)
            as usize
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
///
/// Layout: `[mem_size_slot? (4B)] [user_globals (per-global width)] [passive_lens (4B each)]`.
#[must_use]
pub fn globals_region_size(
    global_widths: &[u32],
    num_passive_segments: usize,
    has_mem_size_global: bool,
) -> usize {
    let mem_size_slot = usize::from(has_mem_size_global) * MEM_SIZE_SLOT_BYTES;
    let user_globals: usize = global_widths.iter().map(|w| *w as usize).sum();
    let passive_lens = num_passive_segments * PASSIVE_SEG_LEN_BYTES;
    mem_size_slot + user_globals + passive_lens
}

/// Precompute absolute PVM addresses for every WASM global from their widths.
///
/// Pairs with `WasmModule::global_offsets`. The output has the same length as
/// `widths`; entry `i` is the address of the i-th global. Layout invariant:
/// `[mem_size_slot? (4B)] [global_0 (widths[0])] [global_1 (widths[1])] ...`.
#[must_use]
pub fn compute_global_offsets(widths: &[u32], has_mem_size_global: bool) -> Vec<i32> {
    let mut offsets = Vec::with_capacity(widths.len());
    let mut cur = GLOBAL_MEMORY_BASE + i32::from(has_mem_size_global) * MEM_SIZE_SLOT_BYTES as i32;
    for &w in widths {
        offsets.push(cur);
        cur += w as i32;
    }
    offsets
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
    global_widths: &[u32],
    ordinal: usize,
    has_mem_size_global: bool,
) -> i32 {
    let mem_size_slot = i32::from(has_mem_size_global) * MEM_SIZE_SLOT_BYTES as i32;
    let user_globals: i32 = global_widths.iter().map(|w| *w as i32).sum();
    GLOBAL_MEMORY_BASE
        + mem_size_slot
        + user_globals
        + (ordinal as i32) * PASSIVE_SEG_LEN_BYTES as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: build a `Vec<u32>` of N 4-byte (i32) global widths — the common
    /// case for tests that don't care about per-global width variation.
    fn i32_widths(n: usize) -> Vec<u32> {
        vec![4u32; n]
    }

    #[test]
    fn wasm_memory_base_typical_program() {
        // 5 i32 globals (4B each), mem-size slot (4B), no passive, overflow needed:
        // globals_end = 0x30000 + 4 + 5*4 = 0x30018
        // param_overflow_base = align8(0x30018) = 0x30018
        // region_end = 0x30018 + 256 = 0x30118
        let base = compute_wasm_memory_base(&i32_widths(5), 0, true, true);
        assert_eq!(base, 0x30118);
    }

    #[test]
    fn wasm_memory_base_mixed_i32_i64_globals() {
        // 2 i32 + 2 i64 + mem-size + overflow:
        // globals_end = 0x30000 + 4 + 2*4 + 2*8 = 0x3001C
        // param_overflow_base = align8(0x3001C) = 0x30020
        // region_end = 0x30020 + 256 = 0x30120
        let widths = [4u32, 4, 8, 8];
        let base = compute_wasm_memory_base(&widths, 0, true, true);
        assert_eq!(base, 0x30120);
    }

    #[test]
    fn wasm_memory_base_zero_globals_with_memgrow() {
        // Zero user globals, mem-size slot present, no passive, overflow needed:
        // globals_end = 0x30000 + 4 = 0x30004
        // param_overflow_base = align8(0x30004) = 0x30008
        // region_end = 0x30008 + 256 = 0x30108
        let base = compute_wasm_memory_base(&[], 0, true, true);
        assert_eq!(base, 0x30108);
    }

    #[test]
    fn wasm_memory_base_zero_globals_memgrow_no_overflow() {
        // The sweet spot: memory.grow-using program with nothing else.
        // Only 4 bytes of mem-size slot; base sits at 0x30004.
        let base = compute_wasm_memory_base(&[], 0, true, false);
        assert_eq!(base, GLOBAL_MEMORY_BASE + 4);
    }

    #[test]
    fn wasm_memory_base_bare_minimum_lands_at_globals_base() {
        // No globals, no passive segs, no mem-size global, no param overflow:
        // region_end = 0x30000. Base stays at GLOBAL_MEMORY_BASE.
        let base = compute_wasm_memory_base(&[], 0, false, false);
        assert_eq!(base, GLOBAL_MEMORY_BASE);
    }

    #[test]
    fn wasm_memory_base_many_globals_no_4kb_jump() {
        // 2000 i32 globals (4B each) + mem-size (4B) + overflow:
        // globals_end = 0x30000 + 4 + 2000*4 = 0x31F44
        // param_overflow_base = align8(0x31F44) = 0x31F48
        // region_end = 0x31F48 + 256 = 0x32048
        let base = compute_wasm_memory_base(&i32_widths(2000), 0, true, true);
        assert_eq!(base, 0x32048);
    }

    #[test]
    fn wasm_memory_base_globals_no_overflow() {
        // Globals only, no overflow: base sits right after globals — no 4KB jump.
        let base = compute_wasm_memory_base(&i32_widths(5), 0, true, false);
        // globals_end = 0x30000 + 4 + 5*4 = 0x30018.
        assert_eq!(base, 0x30018);
    }

    #[test]
    fn param_overflow_base_after_globals() {
        // 5 i32 globals (4B) + mem_size (4B) → globals_end = 0x30018, 8-aligned = 0x30018
        assert_eq!(
            compute_param_overflow_base(&i32_widths(5), 0, true),
            0x30018
        );
        // 0 globals + mem_size → globals_end = 0x30004, 8-aligned = 0x30008
        assert_eq!(compute_param_overflow_base(&[], 0, true), 0x30008);
        // 5 i32 + 3 passive + mem_size → globals_end = 0x30000 + 4 + 20 + 12 = 0x30024, 8-aligned = 0x30028
        assert_eq!(
            compute_param_overflow_base(&i32_widths(5), 3, true),
            0x30028
        );
        // 5 i32, no mem_size → globals_end = 0x30000 + 20 = 0x30014, 8-aligned = 0x30018
        assert_eq!(
            compute_param_overflow_base(&i32_widths(5), 0, false),
            0x30018
        );
    }

    #[test]
    fn param_overflow_does_not_overlap_globals() {
        // With many globals, the overflow area must start after all of them.
        let widths = i32_widths(1000);
        let globals_end = GLOBAL_MEMORY_BASE as usize + globals_region_size(&widths, 0, true);
        let overflow_base = compute_param_overflow_base(&widths, 0, true) as usize;
        assert!(
            overflow_base >= globals_end,
            "overflow base 0x{overflow_base:X} overlaps globals_end 0x{globals_end:X}"
        );
    }

    #[test]
    fn globals_region_size_formula() {
        assert_eq!(globals_region_size(&[], 0, true), 4); // just memory_size slot (4B)
        assert_eq!(globals_region_size(&[], 0, false), 0); // nothing
        assert_eq!(globals_region_size(&i32_widths(5), 0, true), 24); // 4 + 5*4
        assert_eq!(globals_region_size(&i32_widths(5), 0, false), 20); // 5*4
        assert_eq!(globals_region_size(&i32_widths(5), 3, true), 36); // 4 + 5*4 + 3*4
        assert_eq!(globals_region_size(&i32_widths(5), 3, false), 32); // 5*4 + 3*4
        // Mixed: 2 i32 + 2 i64 + mem-size = 4 + 2*4 + 2*8 = 28
        assert_eq!(globals_region_size(&[4, 4, 8, 8], 0, true), 28);
    }

    #[test]
    fn compute_global_offsets_packed_i32() {
        let offsets = compute_global_offsets(&i32_widths(3), true);
        // mem-size at 0x30000; globals start at 0x30004; each 4B.
        assert_eq!(offsets, vec![0x30004, 0x30008, 0x3000C]);
    }

    #[test]
    fn compute_global_offsets_packed_i64() {
        let offsets = compute_global_offsets(&[8, 8, 8], false);
        // No mem-size; globals at 0x30000, +8, +16.
        assert_eq!(offsets, vec![0x30000, 0x30008, 0x30010]);
    }

    #[test]
    fn compute_global_offsets_mixed_widths() {
        // i32, i64, i32, i64 with mem-size: offsets at 0x30004, +4, +8, +4
        let offsets = compute_global_offsets(&[4, 8, 4, 8], true);
        assert_eq!(offsets, vec![0x30004, 0x30008, 0x30010, 0x30014]);
    }

    #[test]
    fn compute_global_offsets_empty() {
        assert!(compute_global_offsets(&[], true).is_empty());
        assert!(compute_global_offsets(&[], false).is_empty());
    }

    #[test]
    fn memory_size_global_is_stable() {
        // Always at GLOBAL_MEMORY_BASE regardless of global count.
        assert_eq!(memory_size_global_offset(), 0x30000);
    }

    #[test]
    fn data_segment_length_after_mem_size_and_globals() {
        // With mem-size (4B) + 5 i32 globals (4B each): lens start at 0x30000 + 4 + 20 = 0x30018.
        assert_eq!(data_segment_length_offset(&i32_widths(5), 0, true), 0x30018);
        assert_eq!(data_segment_length_offset(&i32_widths(5), 1, true), 0x3001C);
        // Without mem-size: lens start at 0x30000 + 5*4 = 0x30014.
        assert_eq!(
            data_segment_length_offset(&i32_widths(5), 0, false),
            0x30014
        );
        assert_eq!(
            data_segment_length_offset(&i32_widths(5), 1, false),
            0x30018
        );
    }

    #[cfg(feature = "compiler")]
    #[test]
    fn global_storage_width_per_type() {
        assert_eq!(global_storage_width(wasmparser::ValType::I32), 4);
        assert_eq!(global_storage_width(wasmparser::ValType::F32), 4);
        assert_eq!(global_storage_width(wasmparser::ValType::I64), 8);
        assert_eq!(global_storage_width(wasmparser::ValType::F64), 8);
        // v128 is 16 bytes (its actual WASM width). These types are rejected
        // by `WasmModule::parse` before reaching the layout code, but if that
        // guard were bypassed the backend's `4/8` width dispatch would
        // surface an `Error::Internal("unexpected width …")` rather than
        // silently miscompile a v128 global by truncating to 8 bytes.
        assert_eq!(global_storage_width(wasmparser::ValType::V128), 16);
        assert_eq!(
            global_storage_width(wasmparser::ValType::Ref(wasmparser::RefType::FUNCREF)),
            8
        );
    }

    #[test]
    fn stack_limit_formula() {
        assert_eq!(
            stack_limit(DEFAULT_STACK_SIZE),
            (0xFEFE_0000u32 - 64 * 1024) as i32
        );
    }
}
