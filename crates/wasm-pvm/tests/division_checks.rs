//! Regression tests for division overflow and div-by-zero checks.
//!
//! WASM spec requires:
//! - Division/remainder by zero must trap
//! - Signed division of INT_MIN / -1 must trap (overflow)
//! - Signed remainder of INT_MIN % -1 returns 0 (no trap)
//!
//! NOTE: These tests check instruction patterns specific to the legacy backend.
//! The LLVM backend currently relies on PVM hardware behavior for division traps
//! and doesn't emit explicit trap sequences. TODO: Add div-by-zero/overflow checks
//! to the LLVM frontend for full WASM spec compliance (Phase 8).

#![cfg(not(feature = "llvm-backend"))]

use wasm_pvm::Opcode;
use wasm_pvm::test_harness::*;

// =============================================================================
// Division by zero checks - verify Trap instruction is emitted
// =============================================================================

#[test]
fn test_i32_div_u_has_trap_for_div_by_zero() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.div_u
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Should have a Trap instruction for div-by-zero check
    assert!(has_opcode(&instructions, Opcode::Trap));
    assert!(has_opcode(&instructions, Opcode::DivU32));

    // Pattern: BranchNeImm (skip if divisor != 0) followed by Trap
    let pattern = vec![
        InstructionPattern::BranchNeImm {
            reg: Pat::Any,
            value: Pat::Exact(0),
            offset: Pat::Any,
        },
        InstructionPattern::Trap,
    ];
    assert_has_pattern(&instructions, &pattern);
}

#[test]
fn test_i32_div_s_has_trap_for_div_by_zero_and_overflow() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.div_s
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Should have Trap instructions for both div-by-zero and overflow checks
    let trap_count = count_opcode(&instructions, Opcode::Trap);
    assert!(
        trap_count >= 2,
        "i32.div_s should have at least 2 Trap instructions (div-by-zero + overflow), got {trap_count}"
    );

    // Should have the actual division
    assert!(has_opcode(&instructions, Opcode::DivS32));

    // Check for INT_MIN immediate (-2147483648) in the overflow check
    let int_min_pattern = InstructionPattern::BranchNeImm {
        reg: Pat::Any,
        value: Pat::Exact(i32::MIN),
        offset: Pat::Any,
    };
    let has_int_min_check = instructions.iter().any(|i| int_min_pattern.matches(i));
    assert!(
        has_int_min_check,
        "i32.div_s should check for INT_MIN in overflow detection"
    );

    // Check for -1 immediate in the overflow check
    let minus_one_pattern = InstructionPattern::BranchNeImm {
        reg: Pat::Any,
        value: Pat::Exact(-1),
        offset: Pat::Any,
    };
    let has_minus_one_check = instructions.iter().any(|i| minus_one_pattern.matches(i));
    assert!(
        has_minus_one_check,
        "i32.div_s should check for -1 in overflow detection"
    );
}

#[test]
fn test_i32_rem_u_has_div_by_zero_check() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.rem_u
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::Trap));
    assert!(has_opcode(&instructions, Opcode::RemU32));
}

#[test]
fn test_i32_rem_s_has_div_by_zero_but_no_overflow_check() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.rem_s
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Should have div-by-zero check
    assert!(has_opcode(&instructions, Opcode::Trap));
    assert!(has_opcode(&instructions, Opcode::RemS32));

    // Should have exactly 2 Traps: 1 from prologue (stack overflow) + 1 from div-by-zero
    // No overflow check for rem (INT_MIN % -1 = 0, not a trap)
    let trap_count = count_opcode(&instructions, Opcode::Trap);
    assert_eq!(
        trap_count, 2,
        "i32.rem_s should have 2 Traps (prologue + div-by-zero only), got {trap_count}"
    );
}

// =============================================================================
// i64 division checks
// =============================================================================

#[test]
fn test_i64_div_u_has_div_by_zero_check() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.div_u
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::Trap));
    assert!(has_opcode(&instructions, Opcode::DivU64));
}

#[test]
fn test_i64_div_s_has_overflow_check() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.div_s
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Should have Trap instructions for both checks
    let trap_count = count_opcode(&instructions, Opcode::Trap);
    assert!(
        trap_count >= 2,
        "i64.div_s should have at least 2 Trap instructions, got {trap_count}"
    );

    assert!(has_opcode(&instructions, Opcode::DivS64));

    // Should use LoadImm64 to load i64::MIN for the overflow check
    assert!(
        has_opcode(&instructions, Opcode::LoadImm64),
        "i64.div_s overflow check should use LoadImm64 for i64::MIN"
    );

    // Should use Xor to compare dividend with i64::MIN
    assert!(
        has_opcode(&instructions, Opcode::Xor),
        "i64.div_s overflow check should use Xor for comparison"
    );
}

#[test]
fn test_i64_rem_s_has_div_by_zero_only() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.rem_s
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::Trap));
    assert!(has_opcode(&instructions, Opcode::RemS64));

    // Should have exactly 2 Traps: 1 from prologue (stack overflow) + 1 from div-by-zero
    let trap_count = count_opcode(&instructions, Opcode::Trap);
    assert_eq!(
        trap_count, 2,
        "i64.rem_s should have 2 Traps (prologue + div-by-zero only), got {trap_count}"
    );
}

// =============================================================================
// Normal division still works (no false traps)
// =============================================================================

#[test]
fn test_normal_division_compiles_with_check() {
    let wat = r#"
        (module
            (func (export "main") (result i32)
                i32.const 10
                i32.const 3
                i32.div_s
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Should compile without error and have the division instruction
    assert!(has_opcode(&instructions, Opcode::DivS32));
    // Should also have the check
    assert!(has_opcode(&instructions, Opcode::Trap));
}
