use wasm_pvm::pvm::Opcode;
use wasm_pvm::test_harness::*;

#[test]
fn test_host_call_emits_ecalli_100() {
    let wat = r#"
        (module
            (import "env" "host_call" (func $host_call (param i64 i64 i64 i64 i64 i64)))
            (func (export "main") (param i32 i32) (result i32)
                (call $host_call
                    (i64.const 100)
                    (i64.const 3)
                    (i64.const 0)
                    (i64.const 8)
                    (i64.const 0)
                    (i64.const 15))
                (i32.const 42)
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::Ecalli));
    assert_has_pattern(
        &instructions,
        &[InstructionPattern::Ecalli {
            index: Pat::Exact(100),
        }],
    );
}

#[test]
fn test_host_call_with_return_value() {
    let wat = r#"
        (module
            (import "env" "host_call" (func $host_call (param i64 i64) (result i64)))
            (func (export "main") (param i32 i32) (result i32)
                (i32.wrap_i64 (call $host_call (i64.const 5) (i64.const 42)))
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::Ecalli));
    assert_has_pattern(
        &instructions,
        &[InstructionPattern::Ecalli {
            index: Pat::Exact(5),
        }],
    );
}

#[test]
fn test_unknown_import_fails_compilation() {
    let wat = r#"
        (module
            (import "env" "unknown_func" (func $unknown (param i32)))
            (func (export "main") (param i32 i32) (result i32)
                (call $unknown (i32.const 1))
                (i32.const 0)
            )
        )
    "#;

    let wasm = wat_to_wasm(wat).expect("Failed to parse WAT");
    let result = wasm_pvm::compile(&wasm);
    let Err(err) = result else {
        panic!("Unknown imports should fail compilation")
    };
    assert!(
        matches!(err, wasm_pvm::Error::UnresolvedImport(_)),
        "Expected UnresolvedImport error, got: {err}"
    );
    assert!(
        err.to_string().contains("unknown_func"),
        "Error should mention the import name: {err}"
    );
}

#[test]
fn test_host_call_ecalli_not_terminating() {
    let wat = r#"
        (module
            (import "env" "host_call" (func $host_call (param i64 i64)))
            (func (export "main") (param i32 i32) (result i32)
                (call $host_call (i64.const 100) (i64.const 0))
                (i32.const 99)
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Find the Ecalli instruction and verify code continues after it.
    let ecalli_pos = instructions
        .iter()
        .position(|i| matches!(i, wasm_pvm::pvm::Instruction::Ecalli { .. }))
        .expect("Ecalli not found");

    assert!(
        ecalli_pos < instructions.len() - 1,
        "Ecalli should not be the last instruction"
    );
}

#[test]
fn test_pvm_ptr_emits_add_imm() {
    let wat = r#"
        (module
            (import "env" "pvm_ptr" (func $pvm_ptr (param i64) (result i64)))
            (memory (export "memory") 1)
            (func (export "main") (param i32 i32) (result i32)
                (i32.wrap_i64 (call $pvm_ptr (i64.const 0)))
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // `pvm_ptr` should emit AddImm64 (adding wasm_memory_base).
    assert!(has_opcode(&instructions, Opcode::AddImm64));
    assert!(!has_opcode(&instructions, Opcode::Ecalli));
}

#[test]
fn test_pvm_ptr_with_host_call() {
    let wat = r#"
        (module
            (import "env" "pvm_ptr" (func $pvm_ptr (param i64) (result i64)))
            (import "env" "host_call" (func $host_call (param i64 i64 i64 i64 i64 i64)))
            (memory (export "memory") 1)
            (data (i32.const 0) "test")
            (data (i32.const 4) "Hello!")
            (func (export "main") (param i32 i32) (result i32)
                (call $host_call
                    (i64.const 100)
                    (i64.const 3)
                    (call $pvm_ptr (i64.const 0))
                    (i64.const 4)
                    (call $pvm_ptr (i64.const 4))
                    (i64.const 6))
                (i32.const 0)
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Should have both AddImm64 (from pvm_ptr) and Ecalli (from host_call).
    assert!(has_opcode(&instructions, Opcode::AddImm64));
    assert!(has_opcode(&instructions, Opcode::Ecalli));
    assert_has_pattern(
        &instructions,
        &[InstructionPattern::Ecalli {
            index: Pat::Exact(100),
        }],
    );
}
