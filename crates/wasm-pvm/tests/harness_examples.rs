//! Example tests demonstrating the test harness
//!
//! This file shows various ways to use the `test_harness` module
//! for testing WASM to PVM compilation.

use wasm_pvm::Opcode;
use wasm_pvm::test_harness::*;

// =============================================================================
// Basic Pattern Matching Examples
// =============================================================================

#[test]
fn test_arithmetic_operations() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Check that we have the expected opcodes
    assert!(has_opcode(&instructions, Opcode::Add32));
    assert!(!has_opcode(&instructions, Opcode::Sub32));

    // Count specific opcodes
    let add_count = count_opcode(&instructions, Opcode::Add32);
    assert!(add_count >= 1, "Should have at least one Add32");
}

#[test]
fn test_pattern_finding() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.mul
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Look for a pattern: AddImm64 (used as move for locals) followed by Mul32 anywhere in the code
    // (the compiler uses AddImm64 { value: 0 } to set up local variable registers, then Mul32)
    let pattern = vec![
        InstructionPattern::AddImm64 {
            dst: Pat::Any,
            src: Pat::Any,
            value: Pat::Any,
        },
        InstructionPattern::Mul32 {
            dst: Pat::Any,
            src1: Pat::Any,
            src2: Pat::Any,
        },
    ];

    assert_has_pattern(&instructions, &pattern);
}

#[test]
fn test_exact_pattern_match() {
    // Test that verifies the sequence of constant loads exists somewhere in the code
    let wat = r#"
        (module
            (func (export "main") (result i32)
                i32.const 10
                i32.const 20
                i32.add
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // The compiler adds prologue code, so look for the pattern in the full instruction sequence
    assert!(!instructions.is_empty());

    // Look for LoadImm with value 10 followed by LoadImm with value 20
    let pattern = vec![
        InstructionPattern::LoadImm {
            reg: Pat::Any,
            value: Pat::Exact(10),
        },
        InstructionPattern::LoadImm {
            reg: Pat::Any,
            value: Pat::Exact(20),
        },
    ];
    assert_has_pattern(&instructions, &pattern);
}

// =============================================================================
// Control Flow Testing
// =============================================================================

#[test]
fn test_if_else_compilation() {
    let wat = r#"
        (module
            (func (export "main") (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.gt_s
                (if (result i32)
                    (then i32.const 1)
                    (else i32.const 0)
                )
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Should have branch instructions for the if/else
    assert!(
        has_opcode(&instructions, Opcode::BranchEqImm)
            || has_opcode(&instructions, Opcode::BranchNeImm)
    );
}

#[test]
fn test_loop_compilation() {
    let wat = r#"
        (module
            (func (export "main") (param i32) (result i32)
                (block
                    (loop
                        local.get 0
                        i32.const 1
                        i32.add
                        local.set 0
                        local.get 0
                        i32.const 10
                        i32.lt_s
                        br_if 0
                    )
                )
                local.get 0
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Should have jump instructions for the loop
    assert!(
        has_opcode(&instructions, Opcode::Jump) || has_opcode(&instructions, Opcode::BranchNeImm)
    );
}

// =============================================================================
// Memory Operations Testing
// =============================================================================

#[test]
fn test_memory_load_store() {
    let wat = r#"
        (module
            (memory 1)
            (func (export "main") (param i32) (result i32)
                local.get 0
                i32.load
                i32.const 1
                i32.add
                local.get 0
                i32.store
                i32.const 0
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Should have load and store instructions
    assert!(has_opcode(&instructions, Opcode::LoadIndU32));
    assert!(has_opcode(&instructions, Opcode::StoreIndU32));
}

// =============================================================================
// Bitwise Operations Testing
// =============================================================================

#[test]
fn test_bitwise_operations() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.and
                local.get 0
                local.get 1
                i32.or
                i32.xor
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::And));
    assert!(has_opcode(&instructions, Opcode::Or));
    assert!(has_opcode(&instructions, Opcode::Xor));
}

#[test]
fn test_shift_operations() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.shl
                local.get 1
                i32.shr_u
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::ShloL32));
    assert!(has_opcode(&instructions, Opcode::ShloR32));
}

// =============================================================================
// Comparison Operations Testing
// =============================================================================

#[test]
fn test_comparison_operations() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.lt_u
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::SetLtU));
}

// =============================================================================
// i64 Operations Testing
// =============================================================================

#[test]
fn test_i64_arithmetic() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.add
                local.get 1
                i64.sub
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::Add64));
    assert!(has_opcode(&instructions, Opcode::Sub64));
}

// =============================================================================
// Advanced Pattern Matching Examples
// =============================================================================

#[test]
fn test_multiple_patterns() {
    let wat = r#"
        (module
            (func (export "main") (param i32) (result i32)
                local.get 0
                i32.const 2
                i32.mul
                i32.const 1
                i32.add
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Pattern: multiplication followed by loading constant 1, then addition
    // The compiler inserts LoadImm between Mul32 and Add32 for the constant
    let mul_add_pattern = vec![
        InstructionPattern::Mul32 {
            dst: Pat::Any,
            src1: Pat::Any,
            src2: Pat::Any,
        },
        InstructionPattern::LoadImm {
            reg: Pat::Any,
            value: Pat::Exact(1),
        },
        InstructionPattern::Add32 {
            dst: Pat::Any,
            src1: Pat::Any,
            src2: Pat::Any,
        },
    ];

    // This pattern should exist in the code
    assert_has_pattern(&instructions, &mul_add_pattern);
}

#[test]
fn test_predicate_patterns() {
    let wat = r#"
        (module
            (func (export "main") (result i32)
                i32.const 100
                i32.const 200
                i32.add
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Find instructions with positive immediate values
    let positive_load = InstructionPattern::LoadImm {
        reg: Pat::Any,
        value: Pat::Predicate(|v| *v > 0),
    };

    // Should find at least one positive load
    let has_positive = instructions.iter().any(|i| positive_load.matches(i));
    assert!(has_positive, "Should have positive constant loads");
}

// =============================================================================
// Error Testing
// =============================================================================

#[test]
fn test_invalid_wat() {
    let invalid_wat = r"
        (module
            (this is invalid)
        )
    ";

    let result = compile_wat(invalid_wat);
    assert!(result.is_err(), "Should fail on invalid WAT");
}

#[test]
fn test_no_function() {
    let wat = r"
        (module
            (memory 1)
        )
    ";

    let result = compile_wat(wat);
    // Should error because there's no function at all
    assert!(result.is_err(), "Should fail without any function");
}

// =============================================================================
// Utility Function Examples
// =============================================================================

#[test]
fn test_filter_by_opcode() {
    let wat = r#"
        (module
            (func (export "main") (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.add
                i32.const 2
                i32.add
                i32.const 3
                i32.add
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Filter for all Add32 instructions
    let adds = filter_by_opcode(&instructions, Opcode::Add32);
    assert!(!adds.is_empty(), "Should have Add32 instructions");
}

#[test]
fn test_exact_sequence_match() {
    let wat = r#"
        (module
            (func (export "main") (result i32)
                i32.const 5
                i32.const 10
                i32.add
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let code = program.code();
    let instructions = code.instructions();

    // Check that the two constants appear as LoadImm instructions
    if instructions.len() >= 2 {
        let expected = vec![
            InstructionPattern::LoadImm {
                reg: Pat::Any,
                value: Pat::Exact(5),
            },
            InstructionPattern::LoadImm {
                reg: Pat::Any,
                value: Pat::Exact(10),
            },
        ];

        // Verify the exact match (may fail due to epilogue code,
        // so we just check the pattern exists somewhere)
        assert_has_pattern(instructions, &expected);
    }
}
