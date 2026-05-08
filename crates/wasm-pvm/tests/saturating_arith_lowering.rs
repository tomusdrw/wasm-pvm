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
/// The WASM frontend represents all params/returns as i64, so a straight
/// `select(i32.gt_u, i32.sub, 0)` produces `trunc→select(i32)→zext`, and
/// LLVM's `usub.sat` matcher does not fold through the outer `zext`.
///
/// The working shape: zero-extend the i32 inputs to i64 locals first, then
/// perform the `select + i64.sub` at i64 level. LLVM `instcombine` folds
/// this to `@llvm.usub.sat.i64`, which our backend already lowers correctly.
/// `i32.wrap_i64` at the end is folded away by LLVM optimization.
const USUB_SAT_I32_WAT: &str = r#"
    (module
        (func (export "main") (param $a i32) (param $b i32) (result i32)
            (local $ae i64)
            (local $be i64)
            (local.set $ae (i64.extend_i32_u (local.get $a)))
            (local.set $be (i64.extend_i32_u (local.get $b)))
            (i32.wrap_i64
                (select
                    (i64.sub (local.get $ae) (local.get $be))
                    (i64.const 0)
                    (i64.gt_u (local.get $ae) (local.get $be))))))
"#;

#[test]
fn usub_sat_i32_folds_to_intrinsic() {
    let ir = dump_llvm_ir(USUB_SAT_I32_WAT).expect("dump");
    assert!(
        ir.contains("@llvm.usub.sat.i64"),
        "expected @llvm.usub.sat.i64 in IR (i32 inputs zero-extended to i64), got:\n{ir}"
    );
}

#[test]
fn usub_sat_i32_compiles() {
    compile_wat(USUB_SAT_I32_WAT).expect("backend should lower llvm.usub.sat.i64 for i32 inputs");
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

/// WAT pattern for `usub.sat.i8`.
///
/// Uses the same i64-level trick as i32: load bytes unsigned, zero-extend to
/// i64, perform the select/sub at i64 level. LLVM folds to `usub.sat.i64`.
const USUB_SAT_I8_WAT: &str = r#"
    (module
        (memory 1)
        (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
            (local $ae i64)
            (local $be i64)
            (local.set $ae (i64.extend_i32_u (i32.load8_u (local.get $args_ptr))))
            (local.set $be (i64.extend_i32_u (i32.load8_u (i32.add (local.get $args_ptr) (i32.const 1)))))
            (i32.store8 (i32.const 0)
                (i32.wrap_i64
                    (select
                        (i64.sub (local.get $ae) (local.get $be))
                        (i64.const 0)
                        (i64.gt_u (local.get $ae) (local.get $be)))))
            (i64.const 4294967296)))
"#;

#[test]
fn usub_sat_i8_folds_to_intrinsic() {
    let ir = dump_llvm_ir(USUB_SAT_I8_WAT).expect("dump");
    assert!(
        ir.contains("@llvm.usub.sat.i64"),
        "expected @llvm.usub.sat.i64 in IR (i8 inputs zero-extended to i64), got:\n{ir}"
    );
}

#[test]
fn usub_sat_i8_compiles() {
    compile_wat(USUB_SAT_I8_WAT).expect("backend should lower llvm.usub.sat.i64 for i8 inputs");
}

/// WAT pattern for `usub.sat.i16`.
///
/// Same i64-level approach as i8 and i32.
const USUB_SAT_I16_WAT: &str = r#"
    (module
        (memory 1)
        (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
            (local $ae i64)
            (local $be i64)
            (local.set $ae (i64.extend_i32_u (i32.load16_u (local.get $args_ptr))))
            (local.set $be (i64.extend_i32_u (i32.load16_u (i32.add (local.get $args_ptr) (i32.const 2)))))
            (i32.store16 (i32.const 0)
                (i32.wrap_i64
                    (select
                        (i64.sub (local.get $ae) (local.get $be))
                        (i64.const 0)
                        (i64.gt_u (local.get $ae) (local.get $be)))))
            (i64.const 8589934592)))
"#;

#[test]
fn usub_sat_i16_folds_to_intrinsic() {
    let ir = dump_llvm_ir(USUB_SAT_I16_WAT).expect("dump");
    assert!(
        ir.contains("@llvm.usub.sat.i64"),
        "expected @llvm.usub.sat.i64 in IR (i16 inputs zero-extended to i64), got:\n{ir}"
    );
}

#[test]
fn usub_sat_i16_compiles() {
    compile_wat(USUB_SAT_I16_WAT).expect("backend should lower llvm.usub.sat.i64 for i16 inputs");
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
