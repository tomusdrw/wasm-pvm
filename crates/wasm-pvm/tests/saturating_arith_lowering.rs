//! Tests that LLVM instcombine folds canonical saturating-arithmetic WAT
//! patterns into @llvm.{u,s}{add,sub}.sat.* intrinsics, and that our
//! backend lowers each intrinsic without errors.
//!
//! Issue: https://github.com/tomusdrw/wasm-pvm/issues/217

use wasm_pvm::test_harness::*;

#[test]
fn dump_llvm_ir_helper_returns_textual_ir() {
    let wat = r#"
        (module
            (func (export "main") (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.add))
    "#;
    let ir = dump_llvm_ir(wat).expect("dump");
    assert!(ir.contains("define"), "IR should contain a function definition, got:\n{ir}");
}

// =============================================================================
// usub.sat
// =============================================================================

/// Canonical WAT pattern for `usub.sat.i32`.
///
/// NOTE: Our WASM frontend represents all values as i64 (PVM ABI), so the
/// WASM `select` instruction is emitted as a 64-bit select over the
/// zero-extended i32 sub result. LLVM's `usub.sat` pattern matcher requires
/// the select and the sub to share the same type, so it does not fold when
/// the select is i64 and the sub is i32. The intrinsic can still appear from
/// Rust code (via `saturating_sub`) when inlined into a wider context;
/// the backend lowering handles all four widths regardless.
const USUB_SAT_I32_WAT: &str = r#"
    (module
        (func (export "main") (param $a i32) (param $b i32) (result i32)
            (select
                (i32.sub (local.get $a) (local.get $b))
                (i32.const 0)
                (i32.gt_u (local.get $a) (local.get $b)))))
"#;

#[test]
fn usub_sat_i32_compiles() {
    compile_wat(USUB_SAT_I32_WAT).expect("backend should lower llvm.usub.sat.i32");
}

const USUB_SAT_I64_WAT: &str = r#"
    (module
        (func (export "main") (param $a i64) (param $b i64) (result i64)
            (select
                (i64.sub (local.get $a) (local.get $b))
                (i64.const 0)
                (i64.gt_u (local.get $a) (local.get $b)))))
"#;

#[test]
fn usub_sat_i64_folds_to_intrinsic() {
    let ir = dump_llvm_ir(USUB_SAT_I64_WAT).expect("dump");
    assert!(
        ir.contains("@llvm.usub.sat.i64"),
        "expected @llvm.usub.sat.i64 in IR, got:\n{ir}"
    );
}

#[test]
fn usub_sat_i64_compiles() {
    compile_wat(USUB_SAT_I64_WAT).expect("backend should lower llvm.usub.sat.i64");
}

const USUB_SAT_I8_WAT: &str = r#"
    (module
        (memory 1)
        (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
            (local $a i32)
            (local $b i32)
            (local.set $a (i32.load8_u (local.get $args_ptr)))
            (local.set $b (i32.load8_u (i32.add (local.get $args_ptr) (i32.const 1))))
            (i32.store8 (i32.const 0)
                (select
                    (i32.sub (local.get $a) (local.get $b))
                    (i32.const 0)
                    (i32.gt_u (local.get $a) (local.get $b))))
            (i64.const 4294967296)))
"#;

// NOTE: i8 fold test omitted — see USUB_SAT_I32_WAT comment above for why
// i8/i16/i32 WAT patterns cannot produce @llvm.usub.sat.iN through our pipeline.

#[test]
fn usub_sat_i8_compiles() {
    // The WAT doesn't fold to usub.sat.i8 via our pipeline (i64-based select),
    // but compiling it must still succeed (the result is just handled via the
    // standard select/sub lowering path without the intrinsic).
    compile_wat(USUB_SAT_I8_WAT).expect("compile");
}

const USUB_SAT_I16_WAT: &str = r#"
    (module
        (memory 1)
        (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
            (local $a i32)
            (local $b i32)
            (local.set $a (i32.load16_u (local.get $args_ptr)))
            (local.set $b (i32.load16_u (i32.add (local.get $args_ptr) (i32.const 2))))
            (i32.store16 (i32.const 0)
                (select
                    (i32.sub (local.get $a) (local.get $b))
                    (i32.const 0)
                    (i32.gt_u (local.get $a) (local.get $b))))
            (i64.const 8589934592)))
"#;

// NOTE: i16 fold test omitted — see USUB_SAT_I32_WAT comment above.

#[test]
fn usub_sat_i16_compiles() {
    compile_wat(USUB_SAT_I16_WAT).expect("compile");
}

#[test]
fn usub_sat_i64_emits_setltu_and_cmovnzimm() {
    use wasm_pvm::Opcode;
    let program = compile_wat(USUB_SAT_I64_WAT).expect("compile");
    let instructions = extract_instructions(&program);
    assert!(
        has_opcode(&instructions, Opcode::SetLtU)
            && has_opcode(&instructions, Opcode::CmovNzImm),
        "usub.sat.i64 lowering must emit SetLtU + CmovNzImm; got opcodes: {:?}",
        instructions.iter().map(|i| format!("{i:?}")).collect::<Vec<_>>()
    );
}
