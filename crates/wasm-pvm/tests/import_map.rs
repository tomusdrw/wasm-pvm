use std::collections::HashMap;
use wasm_pvm::pvm::Opcode;
use wasm_pvm::test_harness::*;
use wasm_pvm::{CompileOptions, ImportAction};

#[test]
fn test_import_map_trap() {
    let wat = r#"
        (module
            (import "env" "abort" (func $abort (param i32 i32 i32 i32)))
            (func (export "main") (param i32 i32) (result i32)
                (call $abort (i32.const 1) (i32.const 2) (i32.const 3) (i32.const 4))
                (i32.const 0)
            )
        )
    "#;

    let mut map = HashMap::new();
    map.insert("abort".to_string(), ImportAction::Trap);

    let program = compile_wat_with_imports(wat, map).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::Trap));
}

#[test]
fn test_import_map_nop() {
    let wat = r#"
        (module
            (import "env" "console.log" (func $log (param i32)))
            (func (export "main") (param i32 i32) (result i32)
                (call $log (i32.const 42))
                (i32.const 0)
            )
        )
    "#;

    let mut map = HashMap::new();
    map.insert("console.log".to_string(), ImportAction::Nop);

    let program = compile_wat_with_imports(wat, map).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Should not have Ecalli for a nop import.
    assert!(!has_opcode(&instructions, Opcode::Ecalli));
}

#[test]
fn test_import_map_unresolved_import_fails() {
    let wat = r#"
        (module
            (import "env" "abort" (func $abort (param i32 i32 i32 i32)))
            (import "env" "console.log" (func $log (param i32)))
            (func (export "main") (param i32 i32) (result i32)
                (i32.const 0)
            )
        )
    "#;

    // Only map "abort", leave "console.log" unmapped.
    let mut map = HashMap::new();
    map.insert("abort".to_string(), ImportAction::Trap);

    let wasm = wat_to_wasm(wat).expect("Failed to parse WAT");
    let result = wasm_pvm::compile_with_options(
        &wasm,
        &CompileOptions {
            import_map: Some(map),
            ..CompileOptions::default()
        },
    );

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(
        err.to_string().contains("console.log"),
        "Error should mention the unresolved import name: {err}"
    );
}

#[test]
fn test_import_map_host_call_not_required() {
    // host_call and pvm_ptr are known intrinsics and don't need to be in the map.
    let wat = r#"
        (module
            (import "env" "host_call" (func $host_call (param i64 i64)))
            (import "env" "pvm_ptr" (func $pvm_ptr (param i64) (result i64)))
            (import "env" "abort" (func $abort (param i32 i32 i32 i32)))
            (memory (export "memory") 1)
            (func (export "main") (param i32 i32) (result i32)
                (i32.const 0)
            )
        )
    "#;

    let mut map = HashMap::new();
    map.insert("abort".to_string(), ImportAction::Trap);

    let program =
        compile_wat_with_imports(wat, map).expect("Should compile - intrinsics don't need mapping");
    let _ = extract_instructions(&program);
}

#[test]
fn test_no_import_map_abort_resolves_by_default() {
    // Without an import map, abort is resolved as trap by default.
    let wat = r#"
        (module
            (import "env" "abort" (func $abort (param i32 i32 i32 i32)))
            (func (export "main") (param i32 i32) (result i32)
                (call $abort (i32.const 1) (i32.const 2) (i32.const 3) (i32.const 4))
                (i32.const 0)
            )
        )
    "#;

    let wasm = wat_to_wasm(wat).expect("Failed to parse WAT");
    let program = wasm_pvm::compile(&wasm).expect("abort should be resolved by default");
    let instructions = extract_instructions(&program);
    assert!(has_opcode(&instructions, Opcode::Trap));
}

#[test]
fn test_no_import_map_unknown_import_fails() {
    // Without an import map, unknown imports should fail.
    let wat = r#"
        (module
            (import "env" "console.log" (func $log (param i32)))
            (func (export "main") (param i32 i32) (result i32)
                (i32.const 0)
            )
        )
    "#;

    let wasm = wat_to_wasm(wat).expect("Failed to parse WAT");
    let result = wasm_pvm::compile(&wasm);
    assert!(
        result.is_err(),
        "Unknown imports should fail without an import map"
    );
    let err = result.err().unwrap();
    assert!(
        err.to_string().contains("console.log"),
        "Error should mention the unresolved import name: {err}"
    );
}
