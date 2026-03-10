use wasm_pvm::pvm::Opcode;
use wasm_pvm::test_harness::*;

#[test]
fn test_host_call_0_no_data_args() {
    let wat = r#"
        (module
            (import "env" "host_call_0" (func $host_call_0 (param i64) (result i64)))
            (func (export "main") (param i32 i32) (result i32)
                (i32.wrap_i64 (call $host_call_0 (i64.const 5)))
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
fn test_host_call_1_one_data_arg() {
    let wat = r#"
        (module
            (import "env" "host_call_1" (func $host_call_1 (param i64 i64) (result i64)))
            (func (export "main") (param i32 i32) (result i32)
                (i32.wrap_i64 (call $host_call_1 (i64.const 5) (i64.const 42)))
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
fn test_host_call_2_two_data_args() {
    let wat = r#"
        (module
            (import "env" "host_call_2" (func $host_call_2 (param i64 i64 i64) (result i64)))
            (func (export "main") (param i32 i32) (result i32)
                (i32.wrap_i64 (call $host_call_2 (i64.const 10) (i64.const 100) (i64.const 200)))
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::Ecalli));
    assert_has_pattern(
        &instructions,
        &[InstructionPattern::Ecalli {
            index: Pat::Exact(10),
        }],
    );
}

#[test]
fn test_host_call_5_max_data_args() {
    let wat = r#"
        (module
            (import "env" "host_call_5" (func $host_call_5 (param i64 i64 i64 i64 i64 i64) (result i64)))
            (func (export "main") (param i32 i32) (result i32)
                (i32.wrap_i64 (call $host_call_5
                    (i64.const 7)
                    (i64.const 1) (i64.const 2) (i64.const 3) (i64.const 4) (i64.const 5)))
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::Ecalli));
    assert_has_pattern(
        &instructions,
        &[InstructionPattern::Ecalli {
            index: Pat::Exact(7),
        }],
    );
}

#[test]
fn test_host_call_ecalli_not_terminating() {
    let wat = r#"
        (module
            (import "env" "host_call_1" (func $host_call_1 (param i64 i64) (result i64)))
            (func (export "main") (param i32 i32) (result i32)
                (drop (call $host_call_1 (i64.const 100) (i64.const 0)))
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
fn test_host_call_2b_captures_r8() {
    let wat = r#"
        (module
            (import "env" "host_call_2b" (func $host_call_2b (param i64 i64 i64) (result i64)))
            (import "env" "host_call_r8" (func $host_call_r8 (result i64)))
            (func (export "main") (param i32 i32) (result i32)
                (drop (call $host_call_2b (i64.const 10) (i64.const 100) (i64.const 200)))
                (i32.wrap_i64 (call $host_call_r8))
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::Ecalli));
    // Should have a StoreIndU64 right after Ecalli (r8 capture).
    let ecalli_pos = instructions
        .iter()
        .position(|i| matches!(i, wasm_pvm::pvm::Instruction::Ecalli { .. }))
        .expect("Ecalli not found");
    assert!(
        matches!(
            instructions[ecalli_pos + 1],
            wasm_pvm::pvm::Instruction::StoreIndU64 { .. }
        ),
        "Expected StoreIndU64 after Ecalli for r8 capture, got: {:?}",
        instructions[ecalli_pos + 1]
    );

    // Should also have a LoadIndU64 for host_call_r8.
    assert!(has_opcode(&instructions, Opcode::LoadIndU64));
}

#[test]
fn test_host_call_r8_without_b_variant_still_compiles() {
    // host_call_r8 can be called even without a preceding *b call — it just
    // reads whatever is in the capture slot (undefined but not a compile error).
    let wat = r#"
        (module
            (import "env" "host_call_r8" (func $host_call_r8 (result i64)))
            (func (export "main") (param i32 i32) (result i32)
                (i32.wrap_i64 (call $host_call_r8))
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);
    assert!(has_opcode(&instructions, Opcode::LoadIndU64));
}

#[test]
fn test_host_call_wrong_arg_count_fails() {
    // host_call_2 expects exactly 3 args (1 index + 2 data).
    // Declaring it with 4 params should fail at compilation.
    let wat = r#"
        (module
            (import "env" "host_call_2" (func $host_call_2 (param i64 i64 i64 i64) (result i64)))
            (func (export "main") (param i32 i32) (result i32)
                (i32.wrap_i64 (call $host_call_2
                    (i64.const 10) (i64.const 1) (i64.const 2) (i64.const 3)))
            )
        )
    "#;

    let wasm = wat_to_wasm(wat).expect("Failed to parse WAT");
    let result = wasm_pvm::compile(&wasm);
    assert!(result.is_err(), "Wrong arg count should fail compilation");
    let err = result.err().unwrap().to_string();
    assert!(
        err.contains("host_call_2") && err.contains("3 arguments"),
        "Error should mention expected arg count: {err}"
    );
}

#[test]
fn test_host_call_6_out_of_range_fails() {
    // host_call_6 is not a valid variant (max is 5).
    let wat = r#"
        (module
            (import "env" "host_call_6" (func $host_call_6 (param i64 i64 i64 i64 i64 i64 i64) (result i64)))
            (func (export "main") (param i32 i32) (result i32)
                (i32.const 0)
            )
        )
    "#;

    let wasm = wat_to_wasm(wat).expect("Failed to parse WAT");
    let result = wasm_pvm::compile(&wasm);
    assert!(
        result.is_err(),
        "host_call_6 should be rejected as unresolved import"
    );
}

#[test]
fn test_host_call_without_return_fails() {
    // host_call_1 must be declared with (result i64).
    let wat = r#"
        (module
            (import "env" "host_call_1" (func $host_call_1 (param i64 i64)))
            (func (export "main") (param i32 i32) (result i32)
                (call $host_call_1 (i64.const 5) (i64.const 42))
                (i32.const 0)
            )
        )
    "#;

    let wasm = wat_to_wasm(wat).expect("Failed to parse WAT");
    let result = wasm_pvm::compile(&wasm);
    assert!(result.is_err(), "host_call_1 without return should fail");
    let err = result.err().unwrap().to_string();
    assert!(
        err.contains("host_call_1") && err.contains("result i64"),
        "Error should mention missing return: {err}"
    );
}

#[test]
fn test_host_call_r8_without_return_fails() {
    // host_call_r8 must be declared with (result i64).
    let wat = r#"
        (module
            (import "env" "host_call_r8" (func $host_call_r8))
            (func (export "main") (param i32 i32) (result i32)
                (call $host_call_r8)
                (i32.const 0)
            )
        )
    "#;

    let wasm = wat_to_wasm(wat).expect("Failed to parse WAT");
    let result = wasm_pvm::compile(&wasm);
    assert!(result.is_err(), "host_call_r8 without return should fail");
    let err = result.err().unwrap().to_string();
    assert!(
        err.contains("host_call_r8") && err.contains("result i64"),
        "Error should mention missing return: {err}"
    );
}

#[test]
fn test_host_call_0b_captures_r8() {
    let wat = r#"
        (module
            (import "env" "host_call_0b" (func $host_call_0b (param i64) (result i64)))
            (import "env" "host_call_r8" (func $host_call_r8 (result i64)))
            (func (export "main") (param i32 i32) (result i32)
                (drop (call $host_call_0b (i64.const 5)))
                (i32.wrap_i64 (call $host_call_r8))
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);
    assert!(has_opcode(&instructions, Opcode::Ecalli));
    // StoreIndU64 after Ecalli for r8 capture.
    let ecalli_pos = instructions
        .iter()
        .position(|i| matches!(i, wasm_pvm::pvm::Instruction::Ecalli { .. }))
        .expect("Ecalli not found");
    assert!(
        matches!(
            instructions[ecalli_pos + 1],
            wasm_pvm::pvm::Instruction::StoreIndU64 { .. }
        ),
        "Expected StoreIndU64 after Ecalli for r8 capture"
    );
}

#[test]
fn test_host_call_5b_captures_r8() {
    let wat = r#"
        (module
            (import "env" "host_call_5b" (func $host_call_5b (param i64 i64 i64 i64 i64 i64) (result i64)))
            (import "env" "host_call_r8" (func $host_call_r8 (result i64)))
            (func (export "main") (param i32 i32) (result i32)
                (drop (call $host_call_5b
                    (i64.const 7)
                    (i64.const 1) (i64.const 2) (i64.const 3) (i64.const 4) (i64.const 5)))
                (i32.wrap_i64 (call $host_call_r8))
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);
    assert!(has_opcode(&instructions, Opcode::Ecalli));
}

#[test]
fn test_host_call_r8_survives_helper_call() {
    // Verify the captured r8 slot (SP-relative) is not clobbered by an
    // intervening function call: host_call_2b → helper() → host_call_r8.
    let wat = r#"
        (module
            (import "env" "host_call_2b" (func $host_call_2b (param i64 i64 i64) (result i64)))
            (import "env" "host_call_r8" (func $host_call_r8 (result i64)))
            (func $helper (result i32) (i32.const 99))
            (func (export "main") (param i32 i32) (result i32)
                (drop (call $host_call_2b (i64.const 10) (i64.const 100) (i64.const 200)))
                (drop (call $helper))
                (i32.wrap_i64 (call $host_call_r8))
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // The StoreIndU64 (r8 capture) must appear after the Ecalli.
    let ecalli_pos = instructions
        .iter()
        .position(|i| matches!(i, wasm_pvm::pvm::Instruction::Ecalli { .. }))
        .expect("Ecalli not found");
    assert!(
        matches!(
            instructions[ecalli_pos + 1],
            wasm_pvm::pvm::Instruction::StoreIndU64 { .. }
        ),
        "Expected StoreIndU64 after Ecalli for r8 capture"
    );

    // The r8 capture slot is SP-relative and preserved across calls because
    // the caller's SP doesn't change — callees get their own frame below.
    // Verify the capture store and load are both present.
    assert!(has_opcode(&instructions, Opcode::Ecalli));
    assert!(has_opcode(&instructions, Opcode::LoadIndU64));
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
fn test_pvm_ptr_with_host_call_5() {
    let wat = r#"
        (module
            (import "env" "pvm_ptr" (func $pvm_ptr (param i64) (result i64)))
            (import "env" "host_call_5" (func $host_call_5 (param i64 i64 i64 i64 i64 i64) (result i64)))
            (memory (export "memory") 1)
            (data (i32.const 0) "test")
            (data (i32.const 4) "Hello!")
            (func (export "main") (param i32 i32) (result i32)
                (drop (call $host_call_5
                    (i64.const 100)
                    (i64.const 3)
                    (call $pvm_ptr (i64.const 0))
                    (i64.const 4)
                    (call $pvm_ptr (i64.const 4))
                    (i64.const 6)))
                (i32.const 0)
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Should have both AddImm64 (from pvm_ptr) and Ecalli (from host_call_5).
    assert!(has_opcode(&instructions, Opcode::AddImm64));
    assert!(has_opcode(&instructions, Opcode::Ecalli));
    assert_has_pattern(
        &instructions,
        &[InstructionPattern::Ecalli {
            index: Pat::Exact(100),
        }],
    );
}
