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
