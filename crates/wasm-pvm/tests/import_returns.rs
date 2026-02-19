//! Regression tests for import function handling.
//!
//! When an imported function has a return value, the compiler must push
//! a dummy value (0) to maintain stack balance.

use std::collections::HashMap;
use wasm_pvm::ImportAction;
use wasm_pvm::Opcode;
use wasm_pvm::test_harness::*;

#[test]
fn test_import_with_return_pushes_dummy_value() {
    let wat = r#"
        (module
            (import "env" "get_value" (func $get_value (result i32)))
            (func (export "main") (result i32)
                call $get_value
                i32.const 1
                i32.add
            )
        )
    "#;

    let mut map = HashMap::new();
    map.insert("get_value".to_string(), ImportAction::Nop);

    let program = compile_wat_with_imports(wat, map).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Should have an Add32 instruction (proving we got past the import call)
    assert!(has_opcode(&instructions, Opcode::Add32));

    // The import stub should push a LoadImm 0 for the return value
    let load_zero = InstructionPattern::LoadImm {
        reg: Pat::Any,
        value: Pat::Exact(0),
    };
    let has_load_zero = instructions.iter().any(|i| load_zero.matches(i));
    assert!(
        has_load_zero,
        "Import with return value should push dummy value (0)"
    );
}

#[test]
fn test_import_without_return_no_extra_push() {
    let wat = r#"
        (module
            (import "env" "log" (func $log (param i32)))
            (func (export "main") (param i32) (result i32)
                local.get 0
                call $log
                i32.const 42
            )
        )
    "#;

    let mut map = HashMap::new();
    map.insert("log".to_string(), ImportAction::Nop);

    // Should compile without error (no stack imbalance)
    let _program = compile_wat_with_imports(wat, map).expect("Failed to compile");
}

#[test]
fn test_abort_import_emits_trap() {
    let wat = r#"
        (module
            (import "env" "abort" (func $abort (param i32 i32 i32 i32)))
            (func (export "main") (result i32)
                i32.const 0
                i32.const 0
                i32.const 0
                i32.const 0
                call $abort
                i32.const 0
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // abort import should emit Trap
    assert!(has_opcode(&instructions, Opcode::Trap));
}

#[test]
fn test_import_with_args_and_return() {
    let wat = r#"
        (module
            (import "env" "compute" (func $compute (param i32 i32) (result i32)))
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                call $compute
            )
        )
    "#;

    let mut map = HashMap::new();
    map.insert("compute".to_string(), ImportAction::Nop);

    // Should compile without error (stack: push 2 args, pop 2 for import, push 1 return)
    let _program = compile_wat_with_imports(wat, map).expect("Failed to compile");
}
