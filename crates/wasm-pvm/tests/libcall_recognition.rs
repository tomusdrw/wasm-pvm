//! Tests for compiler-builtins libcall recognition.
//!
//! Verifies that:
//! - A WASM function named `__multi3` with the canonical signature gets its
//!   body replaced with the hand-crafted PVM-friendly version (uses
//!   `MulUpperUU`).
//! - Disabling `libcall_recognition` keeps the original body intact.
//! - Functions with the wrong signature are not replaced.
//! - Functions with a different name are not affected.

use wasm_pvm::test_harness::*;
use wasm_pvm::{CompileOptions, Instruction, OptimizationFlags};

/// A minimal WAT that declares a function named `__multi3` matching the
/// compiler-builtins signature `(i32 sret, i64 a_lo, i64 a_hi, i64 b_lo,
/// i64 b_hi) -> void`. The body here is a stub (stores zeros to the sret
/// area); recognition replaces it with the real 128-bit multiplication.
///
/// `main` calls `__multi3` with `a = a_lo` and `b = b_lo` (both zero-
/// extended), then loads the high 64 bits of the product from `sret + 8`
/// and packs it into the unified ABI return.
const MULTI3_WAT: &str = r#"
(module
  (memory 1)

  (func $__multi3 (param i32 i64 i64 i64 i64)
    ;; Stub body — recognition replaces this with `Mul64 + MulUpperUU + ...`.
    ;; A trap here would also work, but `i32.const` + `i64.store` is real
    ;; WASM that survives validation regardless of what recognition does.
    local.get 0
    i64.const 0
    i64.store
    local.get 0
    i32.const 8
    i32.add
    i64.const 0
    i64.store
  )

  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $sret i32)
    ;; Stack-allocate 16 bytes at address 0 for the sret area.
    i32.const 0
    local.set $sret

    ;; Read a_lo and b_lo from args memory (offsets 0 and 8).
    local.get $sret
    local.get $args_ptr
    i64.load offset=0
    i64.const 0
    local.get $args_ptr
    i64.load offset=8
    i64.const 0
    call $__multi3

    ;; Read result high half from sret + 8 and store into result memory at
    ;; address 32. Return (ptr=32, len=8) per the unified entry ABI.
    i32.const 32
    local.get $sret
    i64.load offset=8
    i64.store

    ;; (ptr=32, len=8) → (8 << 32) | 32 = 0x00000008_00000020
    i64.const 0x800000020
  )
)
"#;

fn compile_with_libcall_recognition(wat: &str, enabled: bool) -> Vec<Instruction> {
    let opts = CompileOptions {
        optimizations: OptimizationFlags {
            libcall_recognition: enabled,
            ..OptimizationFlags::default()
        },
        ..CompileOptions::default()
    };
    let program = compile_wat_with_options(wat, &opts).expect("compile");
    extract_instructions(&program)
}

fn contains_mul_upper_uu(instructions: &[Instruction]) -> bool {
    instructions
        .iter()
        .any(|i| matches!(i, Instruction::MulUpperUU { .. }))
}

#[test]
fn multi3_replaced_by_default() {
    let instructions = compile_with_libcall_recognition(MULTI3_WAT, true);
    assert!(
        contains_mul_upper_uu(&instructions),
        "expected MulUpperUU in output when libcall_recognition is enabled"
    );
}

#[test]
fn multi3_kept_when_disabled() {
    let instructions = compile_with_libcall_recognition(MULTI3_WAT, false);
    assert!(
        !contains_mul_upper_uu(&instructions),
        "expected no MulUpperUU when libcall_recognition is disabled"
    );
}

/// Verifies that a function literally named `__multi3` but with the
/// **wrong signature** (e.g. fewer parameters) is *not* replaced — its
/// stub body survives and no `MulUpperUU` is emitted.
///
/// This is the safety check: a user function that happens to share the
/// reserved compiler-builtins name must not be silently mis-translated.
#[test]
fn wrong_signature_not_replaced() {
    let wat = r#"
        (module
          (memory 1)

          ;; Two i64 params (not the canonical (i32, i64, i64, i64, i64) → void)
          (func $__multi3 (param i64 i64) (result i64)
            local.get 0
            local.get 1
            i64.mul
          )

          (func (export "main") (param i32 i32) (result i64)
            i64.const 7
            i64.const 11
            call $__multi3
            drop
            i64.const 0x100000000
          )
        )
    "#;
    let instructions = compile_with_libcall_recognition(wat, true);
    assert!(
        !contains_mul_upper_uu(&instructions),
        "expected no MulUpperUU — signature mismatch should skip recognition"
    );
}

/// A function named something else (not `__multi3`) is never replaced
/// regardless of its signature or body shape.
#[test]
fn unrelated_function_unaffected() {
    let wat = r#"
        (module
          (memory 1)

          ;; Same signature as __multi3 but a totally unrelated name.
          (func $my_helper (param i32 i64 i64 i64 i64)
            local.get 0
            i64.const 0
            i64.store
          )

          (func (export "main") (param i32 i32) (result i64)
            i32.const 0
            i64.const 1
            i64.const 0
            i64.const 2
            i64.const 0
            call $my_helper
            i64.const 0x100000000
          )
        )
    "#;
    let instructions = compile_with_libcall_recognition(wat, true);
    assert!(
        !contains_mul_upper_uu(&instructions),
        "function with non-libcall name must not be replaced"
    );
}

// -----------------------------------------------------------------------------
// __udivti3 recognition
//
// The recognition gate is name + signature + body scan: the function must be
// named `__udivti3`, take `(i32 sret, i64 a_lo, i64 a_hi, i64 b_lo, i64 b_hi)`,
// AND its body must contain at least one `GlobalGet` (the stack pointer) and
// one `Call` (the slow-path callee). Without both, recognition silently
// no-ops — the synthesized body's slow path can't forward anywhere safely.
// -----------------------------------------------------------------------------

/// WAT matching the canonical compiler-builtins `__udivti3` shape:
/// stack-allocate 32 bytes, call `specialized_div_rem`, copy quotient back,
/// restore stack. The stub `specialized_div_rem` writes sentinel values to
/// its 32-byte sret so integration tests can verify the slow path works.
const UDIVTI3_WAT: &str = r#"
(module
  (memory 1)
  (global $__stack_pointer (mut i32) (i32.const 65536))

  ;; Stub slow-path. Writes [a_lo+1, a_lo+2, a_lo+3, a_lo+4] as the 32-byte
  ;; result. Real specialized_div_rem writes [q_lo, q_hi, r_lo, r_hi].
  (func $sdr (param $sret i32) (param $a_lo i64) (param $a_hi i64) (param $b_lo i64) (param $b_hi i64)
    local.get $sret
    local.get $a_lo
    i64.const 1
    i64.add
    i64.store offset=0
    local.get $sret
    local.get $a_lo
    i64.const 2
    i64.add
    i64.store offset=8
    local.get $sret
    local.get $a_lo
    i64.const 3
    i64.add
    i64.store offset=16
    local.get $sret
    local.get $a_lo
    i64.const 4
    i64.add
    i64.store offset=24
  )

  (func $__udivti3 (param i32 i64 i64 i64 i64)
    (local i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 5
    global.set $__stack_pointer
    local.get 5
    local.get 1
    local.get 2
    local.get 3
    local.get 4
    call $sdr
    local.get 0
    local.get 5
    i64.load
    i64.store
    local.get 0
    local.get 5
    i64.load offset=8
    i64.store offset=8
    local.get 5
    i32.const 32
    i32.add
    global.set $__stack_pointer
  )

  (func (export "main") (param i32 i32) (result i64)
    i32.const 0
    i64.const 0
    i64.const 0
    i64.const 0
    i64.const 0
    call $__udivti3
    i64.const 0x100000000
  )
)
"#;

fn contains_divu64(instructions: &[Instruction]) -> bool {
    instructions
        .iter()
        .any(|i| matches!(i, Instruction::DivU64 { .. }))
}

/// The synthesized `__udivti3` fast path emits a `DivU64`. With
/// recognition enabled, that instruction must appear somewhere in the
/// output (specifically in `__udivti3`'s body; we don't assert location).
#[test]
fn udivti3_replaced_by_default() {
    let instructions = compile_with_libcall_recognition(UDIVTI3_WAT, true);
    assert!(
        contains_divu64(&instructions),
        "expected DivU64 (from the udivti3 fast path) when recognition is enabled"
    );
}

/// With recognition disabled, the original `__udivti3` body has no
/// `DivU64` — it just stack-allocates and calls the slow path. The only
/// way `DivU64` appears in the output is via our synthesized fast path.
#[test]
fn udivti3_kept_when_disabled() {
    let instructions = compile_with_libcall_recognition(UDIVTI3_WAT, false);
    assert!(
        !contains_divu64(&instructions),
        "expected no DivU64 when recognition is disabled (original body has no native divide)"
    );
}

/// A function literally named `__udivti3` but with the wrong arity must
/// not be replaced.
#[test]
fn udivti3_wrong_signature_not_replaced() {
    let wat = r#"
        (module
          (memory 1)
          (global $__stack_pointer (mut i32) (i32.const 65536))

          ;; 2 i64 params — not the canonical 5.
          (func $__udivti3 (param i64 i64) (result i64)
            local.get 0
            local.get 1
            i64.div_u
          )

          (func (export "main") (param i32 i32) (result i64)
            i64.const 10
            i64.const 3
            call $__udivti3
            drop
            i64.const 0x100000000
          )
        )
    "#;
    let instructions = compile_with_libcall_recognition(wat, true);
    // The wrong-arity __udivti3 will still lower its i64.div_u to DivU64, so
    // we can't use DivU64 presence as the signal here. Instead, check that
    // the function's structure remains a single direct div (no fast/slow
    // dispatch branch).
    let n_divs = instructions
        .iter()
        .filter(|i| matches!(i, Instruction::DivU64 { .. }))
        .count();
    assert_eq!(
        n_divs, 1,
        "wrong-arity __udivti3 should keep its own DivU64 unchanged (got {n_divs} DivU64s)"
    );
}

/// `__udivti3` with no body-internal Call (e.g. someone inlined the
/// slow path away) must skip recognition — we'd have nowhere safe to
/// forward to.
#[test]
fn udivti3_no_slow_path_call_skipped() {
    let wat = r#"
        (module
          (memory 1)
          (global $__stack_pointer (mut i32) (i32.const 65536))

          ;; Has the right name, signature, and GlobalGet — but NO Call.
          ;; Recognition should skip this entirely (silently).
          (func $__udivti3 (param i32 i64 i64 i64 i64)
            global.get $__stack_pointer
            drop
            local.get 0
            i64.const 0
            i64.store
            local.get 0
            i64.const 0
            i64.store offset=8
          )

          (func (export "main") (param i32 i32) (result i64)
            i32.const 0
            i64.const 0
            i64.const 0
            i64.const 0
            i64.const 0
            call $__udivti3
            i64.const 0x100000000
          )
        )
    "#;
    let instructions = compile_with_libcall_recognition(wat, true);
    assert!(
        !contains_divu64(&instructions),
        "expected no DivU64: __udivti3 has no Call so recognition must skip"
    );
}

/// `__udivti3` whose body has a Call but no `GlobalGet` must skip
/// recognition — the synthesized slow path needs to identify
/// `__stack_pointer` (32-byte WASM stack frame for the slow-path
/// callee), and without a body-internal `GlobalGet` we can't.
#[test]
fn udivti3_no_global_get_skipped() {
    let wat = r#"
        (module
          (memory 1)

          ;; A 5-i64-param helper for the Call target, so the *target*
          ;; signature matches `EXPECTED_SIG`. The point of this test is
          ;; the missing `GlobalGet`, not a target-signature mismatch.
          (func $helper (param i32 i64 i64 i64 i64)
            local.get 0
            i64.const 0
            i64.store
          )

          ;; Right name + signature + Call — but no `GlobalGet` anywhere.
          (func $__udivti3 (param i32 i64 i64 i64 i64)
            local.get 0
            local.get 1
            local.get 2
            local.get 3
            local.get 4
            call $helper
          )

          (func (export "main") (param i32 i32) (result i64)
            i32.const 0
            i64.const 0
            i64.const 0
            i64.const 0
            i64.const 0
            call $__udivti3
            i64.const 0x100000000
          )
        )
    "#;
    let instructions = compile_with_libcall_recognition(wat, true);
    assert!(
        !contains_divu64(&instructions),
        "expected no DivU64: __udivti3 has no GlobalGet so recognition must skip"
    );
}

/// `__udivti3` whose first Call points to a function with a *different*
/// signature must skip recognition. Otherwise the synthesized body's
/// slow path would call that function with five i64 args, fail LLVM's
/// type checker, and bring down compilation.
#[test]
fn udivti3_wrong_slow_path_signature_skipped() {
    let wat = r#"
        (module
          (memory 1)
          (global $__stack_pointer (mut i32) (i32.const 65536))

          ;; Wrong-arity Call target (1 i32 param instead of 5 i64). If
          ;; recognition fires anyway, the synthesized __udivti3 body
          ;; would call this with 5 i64 args and LLVM verify would fail.
          (func $wrong_arity (param i32))

          (func $__udivti3 (param i32 i64 i64 i64 i64)
            (local i32)
            global.get $__stack_pointer
            i32.const 32
            i32.sub
            local.tee 5
            global.set $__stack_pointer
            local.get 5
            call $wrong_arity
            local.get 0
            i64.const 0
            i64.store
            local.get 0
            i64.const 0
            i64.store offset=8
            local.get 5
            i32.const 32
            i32.add
            global.set $__stack_pointer
          )

          (func (export "main") (param i32 i32) (result i64)
            i32.const 0
            i64.const 0
            i64.const 0
            i64.const 0
            i64.const 0
            call $__udivti3
            i64.const 0x100000000
          )
        )
    "#;
    let instructions = compile_with_libcall_recognition(wat, true);
    assert!(
        !contains_divu64(&instructions),
        "expected no DivU64: slow-path target has wrong signature so recognition must skip"
    );
}
