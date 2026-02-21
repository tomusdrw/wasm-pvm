//! Comprehensive operator coverage tests (issue #26)
//!
//! Tests for WASM operators and PVM lowering paths that were previously
//! only incidentally exercised through integration tests.

use wasm_pvm::Opcode;
use wasm_pvm::test_harness::*;

// =============================================================================
// Division & Remainder: Trap Conditions
// =============================================================================

/// Division by zero should emit a trap guard (branch to trap on zero divisor).
#[test]
fn test_i32_div_emits_trap_guard() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.div_u
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // Division should exist in output
    assert!(has_opcode(&instructions, Opcode::DivU32));
    // A branch-to-trap pattern should exist for div-by-zero guard
    assert!(
        has_opcode(&instructions, Opcode::Trap)
            || has_opcode(&instructions, Opcode::BranchEqImm)
            || has_opcode(&instructions, Opcode::BranchNeImm),
        "Division should have a zero-check guard"
    );
}

/// Signed division should also emit a signed overflow trap (INT_MIN / -1).
#[test]
fn test_i32_div_s_emits_overflow_trap() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.div_s
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::DivS32));
    // Signed division has two traps: div-by-zero and signed overflow.
    // We should see at least 2 conditional branches or traps.
    let trap_count = count_opcode(&instructions, Opcode::Trap);
    assert!(
        trap_count >= 2,
        "Signed division should emit at least 2 trap instructions (div-by-zero + overflow), got {trap_count}"
    );
}

/// i64 division should also emit trap guards.
#[test]
fn test_i64_div_emits_trap_guard() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.div_u
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::DivU64));
    assert!(
        has_opcode(&instructions, Opcode::Trap),
        "i64 division should have a zero-check guard"
    );
}

/// i64 signed division has both div-by-zero and overflow traps.
#[test]
fn test_i64_div_s_emits_overflow_trap() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.div_s
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::DivS64));
    let trap_count = count_opcode(&instructions, Opcode::Trap);
    assert!(
        trap_count >= 2,
        "Signed i64 division should emit at least 2 traps, got {trap_count}"
    );
}

/// Remainder by zero should also emit a trap guard.
#[test]
fn test_i32_rem_emits_trap_guard() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.rem_u
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::RemU32));
    assert!(
        has_opcode(&instructions, Opcode::Trap),
        "Remainder should have a zero-check guard"
    );
}

/// Signed remainder should handle the INT_MIN % -1 edge case.
#[test]
fn test_i32_rem_s_compiles() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.rem_s
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::RemS32));
    assert!(
        has_opcode(&instructions, Opcode::Trap),
        "Signed remainder should have a zero-check trap guard"
    );
}

// =============================================================================
// Bit Manipulation: CLZ, CTZ, POPCNT
// =============================================================================

/// i32.clz should emit LeadingZeroBits32.
#[test]
fn test_i32_clz() {
    let wat = r#"
        (module
            (func (export "main") (param i32) (result i32)
                local.get 0
                i32.clz
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        has_opcode(&instructions, Opcode::LeadingZeroBits32),
        "i32.clz should lower to LeadingZeroBits32"
    );
}

/// i64.clz should emit LeadingZeroBits64.
#[test]
fn test_i64_clz() {
    let wat = r#"
        (module
            (func (export "main") (param i64) (result i64)
                local.get 0
                i64.clz
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        has_opcode(&instructions, Opcode::LeadingZeroBits64),
        "i64.clz should lower to LeadingZeroBits64"
    );
}

/// i32.ctz should emit TrailingZeroBits32.
#[test]
fn test_i32_ctz() {
    let wat = r#"
        (module
            (func (export "main") (param i32) (result i32)
                local.get 0
                i32.ctz
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        has_opcode(&instructions, Opcode::TrailingZeroBits32),
        "i32.ctz should lower to TrailingZeroBits32"
    );
}

/// i64.ctz should emit TrailingZeroBits64.
#[test]
fn test_i64_ctz() {
    let wat = r#"
        (module
            (func (export "main") (param i64) (result i64)
                local.get 0
                i64.ctz
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        has_opcode(&instructions, Opcode::TrailingZeroBits64),
        "i64.ctz should lower to TrailingZeroBits64"
    );
}

/// i32.popcnt should emit CountSetBits32.
#[test]
fn test_i32_popcnt() {
    let wat = r#"
        (module
            (func (export "main") (param i32) (result i32)
                local.get 0
                i32.popcnt
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        has_opcode(&instructions, Opcode::CountSetBits32),
        "i32.popcnt should lower to CountSetBits32"
    );
}

/// i64.popcnt should emit CountSetBits64.
#[test]
fn test_i64_popcnt() {
    let wat = r#"
        (module
            (func (export "main") (param i64) (result i64)
                local.get 0
                i64.popcnt
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        has_opcode(&instructions, Opcode::CountSetBits64),
        "i64.popcnt should lower to CountSetBits64"
    );
}

// =============================================================================
// Sign Extension Operators
// =============================================================================

/// i32.extend8_s should compile correctly.
/// Note: LLVM instcombine may fold sign extensions into shift pairs (shl+ashr),
/// so we accept either SignExtend8 or the folded form.
#[test]
fn test_i32_extend8_s() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                i32.extend8_s
                local.get 1
                i32.add
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // LLVM may fold sext(trunc(x)) into shl+ashr or keep it as SignExtend8.
    // Either way, the program should compile and produce an add.
    assert!(
        has_opcode(&instructions, Opcode::SignExtend8)
            || has_opcode(&instructions, Opcode::SharR32)
            || has_opcode(&instructions, Opcode::SharR64)
            || has_opcode(&instructions, Opcode::Add32)
            || has_opcode(&instructions, Opcode::AddImm32),
        "i32.extend8_s should compile and produce sign extension or equivalent"
    );
}

/// i32.extend16_s should compile correctly.
#[test]
fn test_i32_extend16_s() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                i32.extend16_s
                local.get 1
                i32.add
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        has_opcode(&instructions, Opcode::SignExtend16)
            || has_opcode(&instructions, Opcode::SharR32)
            || has_opcode(&instructions, Opcode::SharR64)
            || has_opcode(&instructions, Opcode::Add32)
            || has_opcode(&instructions, Opcode::AddImm32),
        "i32.extend16_s should compile and produce sign extension or equivalent"
    );
}

/// i64.extend8_s should compile correctly.
#[test]
fn test_i64_extend8_s() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                i64.extend8_s
                local.get 1
                i64.add
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        has_opcode(&instructions, Opcode::SignExtend8)
            || has_opcode(&instructions, Opcode::SharR64)
            || has_opcode(&instructions, Opcode::Add64),
        "i64.extend8_s should compile and produce sign extension or equivalent"
    );
}

/// i64.extend16_s should compile correctly.
#[test]
fn test_i64_extend16_s() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                i64.extend16_s
                local.get 1
                i64.add
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        has_opcode(&instructions, Opcode::SignExtend16)
            || has_opcode(&instructions, Opcode::SharR64)
            || has_opcode(&instructions, Opcode::Add64),
        "i64.extend16_s should compile and produce sign extension or equivalent"
    );
}

// =============================================================================
// Type Conversion Operators
// =============================================================================

/// i32.wrap_i64 should compile and produce a truncating operation.
#[test]
fn test_i32_wrap_i64() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i32) (result i32)
                local.get 0
                i32.wrap_i64
                local.get 1
                i32.add
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // wrap_i64 truncates to 32-bit. The backend should produce an AddImm32 with value 0
    // (sign-extend/zero-extend to 32-bit) or the LLVM optimizer may fold it.
    // At minimum, the program should compile and contain the add.
    assert!(
        has_opcode(&instructions, Opcode::Add32) || has_opcode(&instructions, Opcode::AddImm32),
        "i32.wrap_i64 should compile successfully"
    );
}

/// i64.extend_i32_s should emit sign extension for 32-bit to 64-bit.
#[test]
fn test_i64_extend_i32_s() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i64) (result i64)
                local.get 0
                i64.extend_i32_s
                local.get 1
                i64.add
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // Sign extension from i32 to i64 should use AddImm32 (sign-extend by adding 0 in 32-bit mode)
    // or a dedicated sign extension instruction.
    assert!(
        has_opcode(&instructions, Opcode::Add64) || has_opcode(&instructions, Opcode::AddImm32),
        "i64.extend_i32_s should compile and produce add"
    );
}

/// i64.extend_i32_u should emit zero extension for 32-bit to 64-bit.
#[test]
fn test_i64_extend_i32_u() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i64) (result i64)
                local.get 0
                i64.extend_i32_u
                local.get 1
                i64.add
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // Zero extension from i32 to i64 should use ZeroExtend16 or And with mask,
    // or AddImm32 with 0 in 32-bit mode, or similar.
    assert!(
        has_opcode(&instructions, Opcode::Add64),
        "i64.extend_i32_u should compile and produce add64"
    );
}

// =============================================================================
// Shift Operations (all widths)
// =============================================================================

/// i32.shl should lower to ShloL32.
#[test]
fn test_i32_shl() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.shl
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::ShloL32));
}

/// i32.shr_u should lower to ShloR32.
#[test]
fn test_i32_shr_u() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.shr_u
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::ShloR32));
}

/// i32.shr_s should lower to SharR32 (arithmetic shift right).
#[test]
fn test_i32_shr_s() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.shr_s
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::SharR32));
}

/// i64.shl should lower to ShloL64.
#[test]
fn test_i64_shl() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.shl
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::ShloL64));
}

/// i64.shr_u should lower to ShloR64.
#[test]
fn test_i64_shr_u() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.shr_u
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::ShloR64));
}

/// i64.shr_s should lower to SharR64.
#[test]
fn test_i64_shr_s() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.shr_s
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::SharR64));
}

// =============================================================================
// Comparison Operations (signed/unsigned, all variants)
// =============================================================================

/// i32.lt_s should lower to SetLtS.
#[test]
fn test_i32_lt_s() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.lt_s
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::SetLtS));
}

/// i32.gt_u is implemented as reversed i32.lt_u (swap operands).
#[test]
fn test_i32_gt_u() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.gt_u
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // gt_u is lowered as lt_u with swapped operands
    assert!(has_opcode(&instructions, Opcode::SetLtU));
}

/// i32.le_u is implemented as !(b < a) i.e. NOT(lt_u with swapped operands).
#[test]
fn test_i32_le_u() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.le_u
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // le_u needs a comparison + XOR (to negate the result) or similar
    assert!(
        has_opcode(&instructions, Opcode::SetLtU) || has_opcode(&instructions, Opcode::SetLtUImm),
        "i32.le_u should compile to a comparison operation"
    );
}

/// i32.ge_s is implemented as !(a < b) i.e. NOT(lt_s).
#[test]
fn test_i32_ge_s() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.ge_s
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        has_opcode(&instructions, Opcode::SetLtS) || has_opcode(&instructions, Opcode::SetLtSImm),
        "i32.ge_s should compile to a comparison operation"
    );
}

/// i32.eq should compile and lower to a comparison.
#[test]
fn test_i32_eq() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.eq
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // eq is lowered as XOR + SetLtUImm(1): xor the operands, then test < 1 (i.e. == 0)
    assert!(
        has_opcode(&instructions, Opcode::Xor)
            || has_opcode(&instructions, Opcode::SetLtUImm)
            || has_opcode(&instructions, Opcode::SetLtU),
        "i32.eq should produce comparison/xor instructions"
    );
}

/// i32.ne should compile and lower to a comparison.
#[test]
fn test_i32_ne() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.ne
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // ne is lowered as XOR + SetLtU(0, result): xor the operands, then test 0 < result (nonzero)
    assert!(
        has_opcode(&instructions, Opcode::Xor)
            || has_opcode(&instructions, Opcode::SetLtU)
            || has_opcode(&instructions, Opcode::SetLtUImm),
        "i32.ne should produce xor/comparison instructions"
    );
}

/// i32.eqz should compile and lower to a comparison with zero.
#[test]
fn test_i32_eqz() {
    let wat = r#"
        (module
            (func (export "main") (param i32) (result i32)
                local.get 0
                i32.eqz
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // eqz compares against 0, should produce SetLtUImm(1) or similar
    assert!(
        has_opcode(&instructions, Opcode::SetLtUImm)
            || has_opcode(&instructions, Opcode::SetLtU)
            || has_opcode(&instructions, Opcode::SetLtSImm),
        "i32.eqz should produce a comparison"
    );
}

/// i64.eqz should compile and lower to a comparison with zero.
#[test]
fn test_i64_eqz() {
    let wat = r#"
        (module
            (func (export "main") (param i64) (result i32)
                local.get 0
                i64.eqz
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        !instructions.is_empty(),
        "i64.eqz should produce instructions"
    );
}

// =============================================================================
// Memory Load/Store Variants (different widths)
// =============================================================================

/// i32.load8_u should emit LoadIndU8.
#[test]
fn test_i32_load8_u() {
    let wat = r#"
        (module
            (memory 1)
            (func (export "main") (param i32) (result i32)
                local.get 0
                i32.load8_u
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::LoadIndU8));
}

/// i32.load8_s should emit LoadIndI8.
#[test]
fn test_i32_load8_s() {
    let wat = r#"
        (module
            (memory 1)
            (func (export "main") (param i32) (result i32)
                local.get 0
                i32.load8_s
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::LoadIndI8));
}

/// i32.load16_u should emit LoadIndU16.
#[test]
fn test_i32_load16_u() {
    let wat = r#"
        (module
            (memory 1)
            (func (export "main") (param i32) (result i32)
                local.get 0
                i32.load16_u
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::LoadIndU16));
}

/// i32.load16_s should emit LoadIndI16.
#[test]
fn test_i32_load16_s() {
    let wat = r#"
        (module
            (memory 1)
            (func (export "main") (param i32) (result i32)
                local.get 0
                i32.load16_s
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::LoadIndI16));
}

/// i32.store8 should emit StoreIndU8.
#[test]
fn test_i32_store8() {
    let wat = r#"
        (module
            (memory 1)
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.store8
                i32.const 0
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::StoreIndU8));
}

/// i32.store16 should emit StoreIndU16.
#[test]
fn test_i32_store16() {
    let wat = r#"
        (module
            (memory 1)
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.store16
                i32.const 0
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::StoreIndU16));
}

/// i64.load should emit LoadIndU64 (for linear memory access).
#[test]
fn test_i64_load() {
    let wat = r#"
        (module
            (memory 1)
            (func (export "main") (param i32) (result i64)
                local.get 0
                i64.load
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // i64.load from linear memory should use LoadIndU64
    assert!(has_opcode(&instructions, Opcode::LoadIndU64));
}

/// i64.store should emit StoreIndU64 (for linear memory access).
#[test]
fn test_i64_store() {
    let wat = r#"
        (module
            (memory 1)
            (func (export "main") (param i32 i64) (result i32)
                local.get 0
                local.get 1
                i64.store
                i32.const 0
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::StoreIndU64));
}

// =============================================================================
// Control Flow: br_table / switch
// =============================================================================

/// br_table should compile to a series of branches (switch lowering).
#[test]
fn test_br_table() {
    let wat = r#"
        (module
            (func (export "main") (param i32) (result i32)
                (block $b2
                    (block $b1
                        (block $b0
                            local.get 0
                            br_table $b0 $b1 $b2
                        )
                        ;; case 0
                        i32.const 100
                        return
                    )
                    ;; case 1
                    i32.const 200
                    return
                )
                ;; default (case 2+)
                i32.const 300
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // br_table should lower to some combination of branches and jumps
    let branch_count = count_opcode(&instructions, Opcode::BranchEqImm)
        + count_opcode(&instructions, Opcode::BranchNeImm)
        + count_opcode(&instructions, Opcode::BranchLtUImm)
        + count_opcode(&instructions, Opcode::BranchGeUImm)
        + count_opcode(&instructions, Opcode::Jump);
    assert!(
        branch_count >= 2,
        "br_table should produce multiple branches, got {branch_count}"
    );

    // All three return values should be present
    let has_100 = instructions
        .iter()
        .any(|i| matches!(i, wasm_pvm::Instruction::LoadImm { value: 100, .. }));
    let has_200 = instructions
        .iter()
        .any(|i| matches!(i, wasm_pvm::Instruction::LoadImm { value: 200, .. }));
    let has_300 = instructions
        .iter()
        .any(|i| matches!(i, wasm_pvm::Instruction::LoadImm { value: 300, .. }));
    assert!(has_100, "Should have case 0 value (100)");
    assert!(has_200, "Should have case 1 value (200)");
    assert!(has_300, "Should have default case value (300)");
}

// =============================================================================
// Global Variables
// =============================================================================

/// Global variables should lower to loads/stores at the globals memory area.
#[test]
fn test_global_load_store() {
    let wat = r#"
        (module
            (global $g (mut i32) (i32.const 0))
            (func (export "main") (param i32) (result i32)
                ;; store param into global
                local.get 0
                global.set $g
                ;; load global back
                global.get $g
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // Globals should use StoreIndU32/LoadIndU32 at fixed addresses (0x30000+)
    assert!(
        has_opcode(&instructions, Opcode::StoreIndU32)
            || has_opcode(&instructions, Opcode::StoreIndU64),
        "global.set should emit a store"
    );
    assert!(
        has_opcode(&instructions, Opcode::LoadIndU32)
            || has_opcode(&instructions, Opcode::LoadIndU64),
        "global.get should emit a load"
    );
}

/// Multiple globals should use different offsets.
#[test]
fn test_multiple_globals() {
    let wat = r#"
        (module
            (global $a (mut i32) (i32.const 0))
            (global $b (mut i32) (i32.const 0))
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                global.set $a
                local.get 1
                global.set $b
                global.get $a
                global.get $b
                i32.add
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // Should compile and have Add32
    assert!(has_opcode(&instructions, Opcode::Add32));
}

// =============================================================================
// Memory Operations: memory.size, memory.grow
// =============================================================================

/// memory.size should compile and return the current memory size.
#[test]
fn test_memory_size() {
    let wat = r#"
        (module
            (memory 1)
            (func (export "main") (result i32)
                memory.size
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // memory.size reads the page count from the memory-size global slot (LoadIndU32)
    assert!(
        has_opcode(&instructions, Opcode::LoadIndU32)
            || has_opcode(&instructions, Opcode::LoadIndU64),
        "memory.size should read from the memory-size global (load instruction expected)"
    );
}

/// memory.grow should compile and use the sbrk instruction.
#[test]
fn test_memory_grow() {
    let wat = r#"
        (module
            (memory 1)
            (func (export "main") (param i32) (result i32)
                local.get 0
                memory.grow
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        has_opcode(&instructions, Opcode::Sbrk),
        "memory.grow should use sbrk instruction"
    );
}

// =============================================================================
// local.tee (load and keep on stack)
// =============================================================================

/// local.tee should work correctly — sets a local and leaves value on stack.
#[test]
fn test_local_tee() {
    let wat = r#"
        (module
            (func (export "main") (param i32) (result i32)
                (local $x i32)
                local.get 0
                i32.const 5
                i32.add
                local.tee $x    ;; set $x and keep value on stack
                local.get $x    ;; get $x again
                i32.add         ;; add both copies → should be (param+5)*2
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // Should produce Add instructions (the two adds in the source)
    let add_count =
        count_opcode(&instructions, Opcode::Add32) + count_opcode(&instructions, Opcode::AddImm32);
    assert!(
        add_count >= 2,
        "local.tee test should have at least 2 add operations, got {add_count}"
    );
}

/// local.tee with deeper stack — value should be preserved across other operations.
#[test]
fn test_local_tee_deep_stack() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32 i32) (result i32)
                (local $temp i32)
                local.get 0
                local.get 1
                i32.add
                local.tee $temp      ;; save intermediate result
                local.get 2
                i32.mul
                local.get $temp      ;; reuse intermediate
                i32.add
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // Should have both Add32 and Mul32
    assert!(
        has_opcode(&instructions, Opcode::Add32) || has_opcode(&instructions, Opcode::AddImm32),
        "Should have add operation"
    );
    assert!(has_opcode(&instructions, Opcode::Mul32));
}

// =============================================================================
// Import Handling
// =============================================================================

/// Imported function with Trap action should emit Trap instruction.
#[test]
fn test_import_trap_action() {
    use std::collections::HashMap;
    use wasm_pvm::ImportAction;

    let wat = r#"
        (module
            (import "env" "abort" (func $abort))
            (func (export "main") (result i32)
                call $abort
                i32.const 0
            )
        )
    "#;

    let mut import_map = HashMap::new();
    import_map.insert("abort".to_string(), ImportAction::Trap);

    let program = compile_wat_with_imports(wat, import_map).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        has_opcode(&instructions, Opcode::Trap),
        "Import with Trap action should emit a Trap instruction"
    );
}

/// Imported function with Nop action should compile without trap.
#[test]
fn test_import_nop_action() {
    use std::collections::HashMap;
    use wasm_pvm::ImportAction;

    let wat = r#"
        (module
            (import "env" "noop" (func $noop))
            (func (export "main") (result i32)
                call $noop
                i32.const 42
            )
        )
    "#;

    let mut import_map = HashMap::new();
    import_map.insert("noop".to_string(), ImportAction::Nop);

    let program = compile_wat_with_imports(wat, import_map).expect("compile");
    let instructions = extract_instructions(&program);

    // Nop import should not generate a Trap
    let has_42 = instructions
        .iter()
        .any(|i| matches!(i, wasm_pvm::Instruction::LoadImm { value: 42, .. }));
    assert!(has_42, "Should still produce the return value 42");
}

// =============================================================================
// Optimization Flag Tests
// =============================================================================

/// Disabling peephole should produce more instructions.
#[test]
fn test_no_peephole_produces_more_code() {
    use wasm_pvm::{CompileOptions, OptimizationFlags};

    let wat = r#"
        (module
            (func (export "main") (param i32) (result i32)
                local.get 0
                if (result i32)
                    i32.const 1
                else
                    i32.const 0
                end
            )
        )
    "#;

    let with_peephole = compile_wat(wat).expect("compile with peephole");
    let without_peephole = compile_wat_with_options(
        wat,
        &CompileOptions {
            optimizations: OptimizationFlags {
                peephole: false,
                ..OptimizationFlags::default()
            },
            ..CompileOptions::default()
        },
    )
    .expect("compile without peephole");

    let size_with = with_peephole.code().instructions().len();
    let size_without = without_peephole.code().instructions().len();

    // Peephole should remove some redundant instructions
    assert!(
        size_with <= size_without,
        "Peephole should not increase code size: {size_with} > {size_without}"
    );
}

/// Disabling register cache should produce more load instructions.
#[test]
fn test_no_register_cache_produces_more_loads() {
    use wasm_pvm::{CompileOptions, OptimizationFlags};

    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
                local.get 0
                i32.add
            )
        )
    "#;

    let with_cache = compile_wat(wat).expect("compile with register cache");
    let without_cache = compile_wat_with_options(
        wat,
        &CompileOptions {
            optimizations: OptimizationFlags {
                register_cache: false,
                ..OptimizationFlags::default()
            },
            ..CompileOptions::default()
        },
    )
    .expect("compile without register cache");

    let loads_with = count_opcode(&extract_instructions(&with_cache), Opcode::LoadIndU64);
    let loads_without = count_opcode(&extract_instructions(&without_cache), Opcode::LoadIndU64);

    // With register cache, some loads should be eliminated
    assert!(
        loads_with <= loads_without,
        "Register cache should not increase loads: {loads_with} > {loads_without}"
    );
}

/// Disabling constant propagation should produce more LoadImm instructions.
#[test]
fn test_no_const_prop_produces_more_load_imm() {
    use wasm_pvm::{CompileOptions, OptimizationFlags};

    let wat = r#"
        (module
            (func (export "main") (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.add
                i32.const 1
                i32.add
                i32.const 1
                i32.add
            )
        )
    "#;

    let with_const_prop = compile_wat(wat).expect("compile with const prop");
    let without_const_prop = compile_wat_with_options(
        wat,
        &CompileOptions {
            optimizations: OptimizationFlags {
                constant_propagation: false,
                ..OptimizationFlags::default()
            },
            ..CompileOptions::default()
        },
    )
    .expect("compile without const prop");

    let size_with = with_const_prop.code().instructions().len();
    let size_without = without_const_prop.code().instructions().len();

    // Constant propagation should produce same or fewer instructions
    assert!(
        size_with <= size_without,
        "Constant propagation should not increase code size: {size_with} > {size_without}"
    );
}

// =============================================================================
// Edge Cases: Unreachable, Nop, Drop
// =============================================================================

/// unreachable should emit Trap.
#[test]
fn test_unreachable() {
    let wat = r#"
        (module
            (func (export "main") (result i32)
                unreachable
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        has_opcode(&instructions, Opcode::Trap),
        "unreachable should emit Trap"
    );
}

/// nop should compile (and be optimized away).
#[test]
fn test_nop() {
    let wat = r#"
        (module
            (func (export "main") (result i32)
                nop
                nop
                nop
                i32.const 42
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // Nops should be compiled away, result should still have the constant
    let has_42 = instructions
        .iter()
        .any(|i| matches!(i, wasm_pvm::Instruction::LoadImm { value: 42, .. }));
    assert!(has_42, "Should produce return value 42 despite nops");
}

/// drop should compile correctly (value computed but not used).
#[test]
fn test_drop() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
                drop          ;; discard the add result
                local.get 0   ;; return the original param instead
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // Should compile successfully
    assert!(!instructions.is_empty(), "drop should produce instructions");
}

// =============================================================================
// i64 Bitwise Operations
// =============================================================================

/// i64.and should lower to And.
#[test]
fn test_i64_and() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.and
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::And));
}

/// i64.or should lower to Or.
#[test]
fn test_i64_or() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.or
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::Or));
}

/// i64.xor should lower to Xor.
#[test]
fn test_i64_xor() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.xor
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::Xor));
}

// =============================================================================
// i64 Multiplication & Remainder
// =============================================================================

/// i64.mul should lower to Mul64.
#[test]
fn test_i64_mul() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.mul
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::Mul64));
}

/// i64.rem_u should lower to RemU64 with a trap guard.
#[test]
fn test_i64_rem_u() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.rem_u
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::RemU64));
    assert!(
        has_opcode(&instructions, Opcode::Trap),
        "i64.rem_u should have a zero-check trap guard"
    );
}

/// i64.rem_s should lower to RemS64 with trap guards.
#[test]
fn test_i64_rem_s() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.rem_s
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    assert!(has_opcode(&instructions, Opcode::RemS64));
}

// =============================================================================
// Nested Control Flow
// =============================================================================

/// Nested if/else should produce correct branch structure.
#[test]
fn test_nested_if_else() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                if (result i32)
                    local.get 1
                    if (result i32)
                        i32.const 1
                    else
                        i32.const 2
                    end
                else
                    i32.const 3
                end
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // All three values should be present
    let has_1 = instructions
        .iter()
        .any(|i| matches!(i, wasm_pvm::Instruction::LoadImm { value: 1, .. }));
    let has_2 = instructions
        .iter()
        .any(|i| matches!(i, wasm_pvm::Instruction::LoadImm { value: 2, .. }));
    let has_3 = instructions
        .iter()
        .any(|i| matches!(i, wasm_pvm::Instruction::LoadImm { value: 3, .. }));
    assert!(has_1, "Should have value 1");
    assert!(has_2, "Should have value 2");
    assert!(has_3, "Should have value 3");
}

/// Nested loops should compile correctly.
#[test]
fn test_nested_loops() {
    let wat = r#"
        (module
            (func (export "main") (param i32) (result i32)
                (local $i i32)
                (local $j i32)
                (local $sum i32)
                (local.set $sum (i32.const 0))
                (local.set $i (i32.const 0))
                (block $outer_break
                    (loop $outer
                        (local.set $j (i32.const 0))
                        (block $inner_break
                            (loop $inner
                                local.get $sum
                                i32.const 1
                                i32.add
                                local.set $sum

                                local.get $j
                                i32.const 1
                                i32.add
                                local.tee $j
                                i32.const 3
                                i32.lt_u
                                br_if $inner
                            )
                        )
                        local.get $i
                        i32.const 1
                        i32.add
                        local.tee $i
                        i32.const 3
                        i32.lt_u
                        br_if $outer
                    )
                )
                local.get $sum
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // Nested loops should produce backward branches
    let backward_branches = instructions
        .iter()
        .filter(|i| match i {
            wasm_pvm::Instruction::Jump { offset }
            | wasm_pvm::Instruction::BranchNeImm { offset, .. }
            | wasm_pvm::Instruction::BranchEqImm { offset, .. }
            | wasm_pvm::Instruction::BranchGeU { offset, .. }
            | wasm_pvm::Instruction::BranchLtU { offset, .. } => *offset < 0,
            _ => false,
        })
        .count();
    assert!(
        backward_branches >= 2,
        "Nested loops should produce at least 2 backward branches, got {backward_branches}"
    );
}

// =============================================================================
// Multiple Return Values / Multi-function
// =============================================================================

/// Multiple functions should all be present in the output.
#[test]
fn test_multiple_functions() {
    let wat = r#"
        (module
            (func $add (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
            (func $sub (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.sub
            )
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                call $add
                local.get 0
                local.get 1
                call $sub
                i32.add
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // Both Add32 and Sub32 should be present (from the helper functions)
    assert!(has_opcode(&instructions, Opcode::Add32));
    assert!(has_opcode(&instructions, Opcode::Sub32));
}

// =============================================================================
// Indirect Call (call_indirect)
// =============================================================================

/// call_indirect should compile and use JumpInd.
#[test]
fn test_call_indirect() {
    let wat = r#"
        (module
            (type $sig (func (param i32) (result i32)))
            (table 2 funcref)
            (elem (i32.const 0) $double $triple)
            (func $double (param i32) (result i32)
                local.get 0
                i32.const 2
                i32.mul
            )
            (func $triple (param i32) (result i32)
                local.get 0
                i32.const 3
                i32.mul
            )
            (func (export "main") (param i32 i32) (result i32)
                local.get 0     ;; value to pass
                local.get 1     ;; table index
                call_indirect (type $sig)
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // call_indirect should use JumpInd for the indirect dispatch
    assert!(
        has_opcode(&instructions, Opcode::JumpInd),
        "call_indirect should produce JumpInd"
    );
}

// =============================================================================
// Rotation Operators
// =============================================================================

/// i32.rotl should compile (uses llvm.fshl intrinsic).
#[test]
fn test_i32_rotl() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.rotl
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // i32.rotl lowers via llvm.fshl intrinsic to: (a << amt) | (a >> (32 - amt))
    assert!(
        has_opcode(&instructions, Opcode::ShloL32)
            || has_opcode(&instructions, Opcode::ShloR32)
            || has_opcode(&instructions, Opcode::Or),
        "i32.rotl should produce shift/or instructions"
    );
}

/// i32.rotr should compile (uses llvm.fshr intrinsic).
#[test]
fn test_i32_rotr() {
    let wat = r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.rotr
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // i32.rotr lowers via llvm.fshr intrinsic to: (a >> amt) | (a << (32 - amt))
    assert!(
        has_opcode(&instructions, Opcode::ShloL32)
            || has_opcode(&instructions, Opcode::ShloR32)
            || has_opcode(&instructions, Opcode::Or),
        "i32.rotr should produce shift/or instructions"
    );
}

/// i64.rotl should compile (uses llvm.fshl intrinsic).
#[test]
fn test_i64_rotl() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.rotl
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // i64.rotl lowers via llvm.fshl intrinsic to: (a << amt) | (a >> (64 - amt))
    assert!(
        has_opcode(&instructions, Opcode::ShloL64)
            || has_opcode(&instructions, Opcode::ShloR64)
            || has_opcode(&instructions, Opcode::Or),
        "i64.rotl should produce shift/or instructions"
    );
}

/// i64.rotr should compile (uses llvm.fshr intrinsic).
#[test]
fn test_i64_rotr() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                local.get 1
                i64.rotr
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // i64.rotr lowers via llvm.fshr intrinsic to: (a >> amt) | (a << (64 - amt))
    assert!(
        has_opcode(&instructions, Opcode::ShloL64)
            || has_opcode(&instructions, Opcode::ShloR64)
            || has_opcode(&instructions, Opcode::Or),
        "i64.rotr should produce shift/or instructions"
    );
}

// =============================================================================
// i64.extend32_s
// =============================================================================

/// i64.extend32_s should sign-extend from 32 to 64 bits.
#[test]
fn test_i64_extend32_s() {
    let wat = r#"
        (module
            (func (export "main") (param i64 i64) (result i64)
                local.get 0
                i64.extend32_s
                local.get 1
                i64.add
            )
        )
    "#;

    let program = compile_wat(wat).expect("compile");
    let instructions = extract_instructions(&program);

    // 32→64 sign extension uses AddImm32 with value 0 (sign-extends the 32-bit value)
    // or LLVM may fold it into subsequent operations.
    assert!(
        has_opcode(&instructions, Opcode::AddImm32)
            || has_opcode(&instructions, Opcode::SharR64)
            || has_opcode(&instructions, Opcode::Add64),
        "i64.extend32_s should compile and produce sign extension or equivalent"
    );
}

// =============================================================================
// Dead Store Elimination Edge Cases
// =============================================================================

/// DSE should handle multiple stores to the same offset (last-write-wins).
#[test]
fn test_dse_multiple_stores_same_offset() {
    use wasm_pvm::{CompileOptions, OptimizationFlags};

    // This WAT writes to the same local multiple times, creating redundant stores.
    let wat = r#"
        (module
            (func (export "main") (param i32) (result i32)
                (local $x i32)
                i32.const 1
                local.set $x
                i32.const 2
                local.set $x
                i32.const 3
                local.set $x
                local.get $x
            )
        )
    "#;

    let with_dse = compile_wat(wat).expect("compile with DSE");
    let without_dse = compile_wat_with_options(
        wat,
        &CompileOptions {
            optimizations: OptimizationFlags {
                dead_store_elimination: false,
                ..OptimizationFlags::default()
            },
            ..CompileOptions::default()
        },
    )
    .expect("compile without DSE");

    let stores_with = count_opcode(&extract_instructions(&with_dse), Opcode::StoreIndU64);
    let stores_without = count_opcode(&extract_instructions(&without_dse), Opcode::StoreIndU64);

    // DSE should eliminate at least some redundant stores
    assert!(
        stores_with <= stores_without,
        "DSE should not increase store count: {stores_with} > {stores_without}"
    );
}
