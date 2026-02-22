//! Unit tests for PvmEmitter: slot allocation, label management, fixup resolution,
//! and constant emission — the core "stack machine" logic (issue #32).

use wasm_pvm::test_harness::*;
use wasm_pvm::{CompileOptions, Instruction, Opcode, OptimizationFlags};

// ── Slot Allocation ──

/// Slot offsets start after the frame header and increment by 8.
#[test]
fn test_slot_allocation_offsets() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // The add instruction should load two values from stack slots (params),
    // compute, and store the result back. We expect LoadIndU64 instructions
    // that load from SP+offset where offsets are multiples of 8.
    assert_has_pattern(
        &instructions,
        &[
            // Load param 0 from slot
            InstructionPattern::LoadIndU64 {
                dst: Pat::Any,
                base: Pat::Exact(1), // SP
                offset: Pat::Any,
            },
        ],
    );

    // Verify slot offsets are multiples of 8 (each value gets an 8-byte slot).
    let sp_offsets: Vec<i32> = instructions
        .iter()
        .filter_map(|i| match i {
            Instruction::LoadIndU64 {
                base: 1, offset, ..
            }
            | Instruction::StoreIndU64 {
                base: 1, offset, ..
            } => Some(*offset),
            _ => None,
        })
        .filter(|o| *o >= 0) // Exclude negative offsets (spill area)
        .collect();
    for offset in &sp_offsets {
        assert_eq!(offset % 8, 0, "Slot offset {offset} is not a multiple of 8");
    }
}

/// Each SSA value gets its own slot — verify multiple slots are allocated.
#[test]
fn test_multiple_slot_allocation() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (param i32 i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
                local.get 2
                i32.add
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // Should have multiple StoreIndU64 instructions to store results to slots.
    let store_count = count_opcode(&instructions, Opcode::StoreIndU64);
    // At minimum: store result of first add + store result of second add + prologue saves
    assert!(
        store_count >= 2,
        "Expected at least 2 StoreIndU64 for intermediate results, got {store_count}"
    );
}

// ── Constant Loading ──

/// Small constants (fits in i32) should use LoadImm, not LoadImm64.
#[test]
fn test_small_constant_uses_load_imm() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (result i32)
                i32.const 42
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // LLVM may constant-fold, but the value 42 should appear as a LoadImm somewhere.
    let load_imm_count = count_opcode(&instructions, Opcode::LoadImm);
    assert!(
        load_imm_count > 0,
        "Expected LoadImm for small constant, got none"
    );

    // Should NOT need LoadImm64 for a small constant.
    // (LoadImm64 may appear for other reasons like frame size, so we check the value.)
    let has_42 = instructions
        .iter()
        .any(|i| matches!(i, Instruction::LoadImm { value: 42, .. }));
    assert!(has_42, "Expected LoadImm with value 42");
}

/// Large i64 constants that don't fit in i32 should use LoadImm64.
#[test]
fn test_large_constant_uses_load_imm64() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (result i64)
                i64.const 0x1_0000_0000
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    let load_imm64_count = count_opcode(&instructions, Opcode::LoadImm64);
    assert!(
        load_imm64_count > 0,
        "Expected LoadImm64 for large constant, got none"
    );
}

/// Negative i32 constants should use sign-extended LoadImm (compact encoding).
#[test]
fn test_negative_constant_uses_load_imm() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (result i32)
                i32.const -1
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // LLVM constant-folds `i32.const -1` at IR level. Depending on how LLVM
    // represents it, the backend emits either:
    //   - LoadImm { value: -1 } (sign-extended i32)
    //   - LoadImm64 { value: 0xFFFF_FFFF } (i32 -1 stored as u32 in u64)
    //   - LoadImm64 { value: u64::MAX } (i64 -1, all bits set)
    let has_neg1 = instructions.iter().any(|i| match i {
        Instruction::LoadImm { value, .. } => *value == -1,
        Instruction::LoadImm64 { value, .. } => *value == 0xFFFF_FFFF || *value == u64::MAX,
        _ => false,
    });
    assert!(has_neg1, "Expected LoadImm(-1) or LoadImm64(0xFFFFFFFF)");
}

// ── Label & Fixup Resolution ──

/// Branch instructions should have non-zero offsets after fixup resolution.
#[test]
fn test_branch_fixup_resolution() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (param i32) (result i32)
                local.get 0
                if (result i32)
                    i32.const 1
                else
                    i32.const 2
                end
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // Should have at least one branch with a non-zero offset (resolved fixup).
    let has_resolved_branch = instructions.iter().any(|i| match i {
        Instruction::BranchEqImm { offset, .. }
        | Instruction::BranchNeImm { offset, .. }
        | Instruction::Jump { offset } => *offset != 0,
        _ => false,
    });
    assert!(
        has_resolved_branch,
        "Expected at least one branch with resolved (non-zero) offset"
    );
}

/// Loop generates a backward branch (negative offset).
#[test]
fn test_loop_backward_branch() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (param i32) (result i32)
                (local $i i32)
                (local.set $i (i32.const 0))
                (block
                    (loop
                        local.get $i
                        i32.const 1
                        i32.add
                        local.tee $i
                        i32.const 10
                        i32.lt_u
                        br_if 0
                    )
                )
                local.get $i
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // Loop's br_if should produce a backward branch (negative offset).
    // The branch type depends on how LLVM lowers the comparison.
    let has_backward = instructions.iter().any(|i| match i {
        Instruction::BranchNeImm { offset, .. }
        | Instruction::BranchEqImm { offset, .. }
        | Instruction::BranchGeSImm { offset, .. }
        | Instruction::Jump { offset } => *offset < 0,
        Instruction::BranchGeU { offset, .. } | Instruction::BranchLtU { offset, .. } => {
            *offset < 0
        }
        _ => false,
    });
    assert!(
        has_backward,
        "Expected at least one backward branch (negative offset) for loop"
    );
}

// ── Stack Slot Load/Store Patterns ──

/// Verify that parameters are loaded from stack slots via SP-relative loads.
#[test]
fn test_param_loaded_from_sp_slot() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.sub
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // Both params should be loaded from SP-relative slots.
    // Pattern: LoadIndU64 { dst: TEMP1/TEMP2, base: SP(1), offset: slot }
    assert_has_pattern(
        &instructions,
        &[
            InstructionPattern::LoadIndU64 {
                dst: Pat::Any,
                base: Pat::Exact(1), // SP register
                offset: Pat::Any,
            },
            InstructionPattern::LoadIndU64 {
                dst: Pat::Any,
                base: Pat::Exact(1), // SP register
                offset: Pat::Any,
            },
            InstructionPattern::Sub32 {
                dst: Pat::Any,
                src1: Pat::Any,
                src2: Pat::Any,
            },
        ],
    );
}

/// Results flow through registers; dead stores to slots are eliminated by DSE.
#[test]
fn test_result_stored_to_sp_slot() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
                i32.const 1
                i32.add
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // Both adds produce results; DSE removes any dead intermediate slot stores.
    assert_has_pattern(
        &instructions,
        &[InstructionPattern::Add32 {
            dst: Pat::Any,
            src1: Pat::Any,
            src2: Pat::Any,
        }],
    );
    assert_has_pattern(
        &instructions,
        &[InstructionPattern::AddImm32 {
            dst: Pat::Any,
            src: Pat::Any,
            value: Pat::Exact(1),
        }],
    );
}

// ── Frame Prologue/Epilogue ──

/// Every function should have a prologue that adjusts SP.
#[test]
fn test_function_prologue_adjusts_sp() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (result i32)
                i32.const 0
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // Prologue should subtract frame size from SP (AddImm64 with negative value on SP).
    // The stack grows downwards, so the value must be negative.
    assert_has_pattern(
        &instructions,
        &[InstructionPattern::AddImm64 {
            dst: Pat::Exact(1),                // SP
            src: Pat::Exact(1),                // SP
            value: Pat::Predicate(|v| *v < 0), // negative frame size
        }],
    );
}

/// Function call saves/restores return address to stack (for non-leaf non-main functions).
/// Requires inlining disabled to preserve call structure.
#[test]
fn test_call_saves_return_address() {
    use wasm_pvm::test_harness::compile_wat_with_options;
    use wasm_pvm::{CompileOptions, OptimizationFlags};

    let program = compile_wat_with_options(
        r#"
        (module
            (func $leaf (result i32)
                i32.const 42
            )
            (func $caller (result i32)
                call $leaf
            )
            (func (export "main") (result i32)
                call $caller
            )
        )
        "#,
        &CompileOptions {
            optimizations: OptimizationFlags {
                inlining: false,
                ..OptimizationFlags::default()
            },
            ..CompileOptions::default()
        },
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // The caller ($caller) calls $leaf, so it must save r0.
    assert_has_pattern(
        &instructions,
        &[InstructionPattern::StoreIndU64 {
            base: Pat::Exact(1),   // SP
            src: Pat::Exact(0),    // r0 (return address)
            offset: Pat::Exact(0), // first slot in frame header
        }],
    );
}

// ── Multi-value / Deeper Stack ──

/// Verify that chained operations properly load intermediate results from slots.
#[test]
fn test_chained_operations_use_slots() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (param i32 i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
                local.get 2
                i32.mul
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // The intermediate add result should be stored, then loaded for the mul.
    // Pattern: Add32 → StoreIndU64 → ... → LoadIndU64 → LoadIndU64 → Mul32
    let add_count = count_opcode(&instructions, Opcode::Add32);
    let mul_count = count_opcode(&instructions, Opcode::Mul32);
    assert!(add_count >= 1, "Expected at least 1 Add32");
    assert!(mul_count >= 1, "Expected at least 1 Mul32");

    // The intermediate result of add must be stored and then reloaded for mul.
    let store_count = count_opcode(&instructions, Opcode::StoreIndU64);
    assert!(
        store_count >= 2,
        "Expected at least 2 StoreIndU64 (add result + mul result), got {store_count}"
    );
}

/// Select (ternary) operation should produce correct slot pattern.
#[test]
fn test_select_uses_slots() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (param i32 i32 i32) (result i32)
                local.get 0
                local.get 1
                local.get 2
                select
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // Select should load all 3 operands from slots and branch.
    let load_count = count_opcode(&instructions, Opcode::LoadIndU64);
    assert!(
        load_count >= 3,
        "Expected at least 3 LoadIndU64 for select operands, got {load_count}"
    );
}

/// Verify that spilling across function calls preserves values.
/// After a call, previously computed values must still be loadable from their slots.
#[test]
fn test_spill_preservation_across_call() {
    let program = compile_wat(
        r#"
        (module
            (func $identity (param i32) (result i32)
                local.get 0
            )
            (func (export "main") (param i32 i32) (result i32)
                ;; Compute a value before the call
                local.get 0
                local.get 1
                i32.add
                ;; Call function (clobbers temp registers)
                local.get 0
                call $identity
                ;; Use the pre-call result (must be loaded from slot, not register)
                i32.add
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // After the call, the pre-call add result must be loaded back from its slot.
    // We should see: Add32 → StoreIndU64 → ... → Jump (call) → ... → LoadIndU64 → Add32
    let add_count = count_opcode(&instructions, Opcode::Add32);
    assert!(
        add_count >= 2,
        "Expected at least 2 Add32 (before and after call), got {add_count}"
    );

    // Verify spill round-trip: the first Add32 result is stored to an SP-relative slot,
    // and later reloaded from the same offset after the call.
    let stored_offsets: Vec<i32> = instructions
        .iter()
        .filter_map(|i| match i {
            Instruction::StoreIndU64 {
                base: 1, offset, ..
            } if *offset >= 40 => Some(*offset), // >= FRAME_HEADER_SIZE
            _ => None,
        })
        .collect();
    let loaded_offsets: Vec<i32> = instructions
        .iter()
        .filter_map(|i| match i {
            Instruction::LoadIndU64 {
                base: 1, offset, ..
            } if *offset >= 40 => Some(*offset),
            _ => None,
        })
        .collect();
    // At least one stored slot offset should also appear in loaded offsets (round-trip).
    let has_round_trip = stored_offsets.iter().any(|s| loaded_offsets.contains(s));
    assert!(
        has_round_trip,
        "Expected at least one SP slot to be stored and later reloaded (spill round-trip)"
    );
}

// ── Phi Node / Control Flow Merge ──

/// If/else with a result value should produce phi-like slot stores from both branches.
#[test]
fn test_if_else_result_phi_slots() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (param i32) (result i32)
                local.get 0
                if (result i32)
                    i32.const 10
                else
                    i32.const 20
                end
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // Both branch values should be stored to slots (phi nodes in the merge block).
    let store_count = count_opcode(&instructions, Opcode::StoreIndU32)
        + count_opcode(&instructions, Opcode::StoreIndU64)
        + count_opcode(&instructions, Opcode::StoreImmIndU32)
        + count_opcode(&instructions, Opcode::StoreImmIndU64);
    assert!(
        store_count >= 2,
        "Expected at least 2 StoreIndU64 for phi values, got {store_count}"
    );
}

// ── Callee-Save Shrink Wrapping ──

/// Shrink wrapping should produce smaller code for leaf functions.
/// A leaf function with 0 params and no calls needs no callee-saved register saves.
#[test]
fn test_shrink_wrap_reduces_code_size() {
    use wasm_pvm::test_harness::compile_wat_with_options;
    use wasm_pvm::{CompileOptions, OptimizationFlags};

    let wat = r#"
        (module
            (func $leaf (result i32)
                i32.const 42
            )
            (func (export "main") (result i32)
                call $leaf
            )
        )
    "#;

    let with_shrink = compile_wat(wat).expect("compile with shrink wrap");
    let without_shrink = compile_wat_with_options(
        wat,
        &CompileOptions {
            optimizations: OptimizationFlags {
                shrink_wrap_callee_saves: false,
                ..OptimizationFlags::default()
            },
            ..CompileOptions::default()
        },
    )
    .expect("compile without shrink wrap");

    let size_with = with_shrink.code().instructions().len();
    let size_without = without_shrink.code().instructions().len();

    // Shrink wrapping should produce fewer instructions for the leaf function
    // by removing 4 saves + 4 restores = 8 instructions.
    assert!(
        size_with < size_without,
        "Shrink wrapping should produce smaller code: {size_with} >= {size_without}"
    );
}

/// With --no-shrink-wrap disabled, a leaf function with 0 params should still
/// save all 4 callee-saved registers (r9-r12).
#[test]
fn test_no_shrink_wrap_saves_all() {
    use wasm_pvm::test_harness::compile_wat_with_options;
    use wasm_pvm::{CompileOptions, OptimizationFlags};

    let wat = r#"
        (module
            (func $leaf (result i32)
                i32.const 42
            )
            (func (export "main") (result i32)
                call $leaf
            )
        )
    "#;

    let program = compile_wat_with_options(
        wat,
        &CompileOptions {
            optimizations: OptimizationFlags {
                shrink_wrap_callee_saves: false,
                ..OptimizationFlags::default()
            },
            ..CompileOptions::default()
        },
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // $leaf should save all 4 callee-saved regs (r9=9, r10=10, r11=11, r12=12).
    // Look for the 4 stores to SP-relative offsets 8, 16, 24, 32.
    for (i, offset) in [8, 16, 24, 32].iter().enumerate() {
        let reg = 9 + i as u8;
        let has_save = instructions.iter().any(|instr| {
            matches!(
                instr,
                Instruction::StoreIndU64 { base: 1, src, offset: o } if *src == reg && *o == *offset
            )
        });
        assert!(
            has_save,
            "Expected callee-save store for r{reg} at offset {offset}"
        );
    }
}

/// Shrink wrapping: leaf function with 0 params should NOT save any callee-saved
/// registers (no StoreIndU64 at offsets 8/16/24/32 with r9-r12 as source).
#[test]
fn test_shrink_wrap_leaf_0_params_no_saves() {
    let program = compile_wat(
        r#"
        (module
            (func $leaf (result i32)
                i32.const 42
            )
            (func (export "main") (result i32)
                call $leaf
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // $leaf saves ra at offset 0, but should NOT save any callee-saved registers.
    // Entry function (main) never saves callee-saved registers.
    // So there should be no StoreIndU64 at offsets 8-32 targeting r9-r12.
    for (i, offset) in [8i32, 16, 24, 32].iter().enumerate() {
        let reg = 9 + i as u8;
        let has_save = instructions.iter().any(|instr| {
            matches!(
                instr,
                Instruction::StoreIndU64 { base: 1, src, offset: o } if *src == reg && *o == *offset
            )
        });
        assert!(
            !has_save,
            "Unexpected callee-save store for r{reg} at offset {offset} — shrink wrapping should skip it"
        );
    }
}

/// Shrink wrapping: function that calls another should save all callee-saved regs.
/// Uses `--no-inline` to prevent LLVM from inlining the callee.
#[test]
fn test_shrink_wrap_calling_function_saves_all() {
    let program = compile_wat_with_options(
        r#"
        (module
            (func $leaf (result i32)
                i32.const 42
            )
            (func $caller (result i32)
                call $leaf
            )
            (func (export "main") (result i32)
                call $caller
            )
        )
        "#,
        &CompileOptions {
            optimizations: OptimizationFlags {
                inlining: false,
                ..OptimizationFlags::default()
            },
            ..CompileOptions::default()
        },
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // $caller calls $leaf → must save all 4 callee-saved regs.
    // Check all 4 stores exist (at consecutive offsets 8, 16, 24, 32).
    for (i, offset) in [8, 16, 24, 32].iter().enumerate() {
        let reg = 9 + i as u8;
        let has_save = instructions.iter().any(|instr| {
            matches!(
                instr,
                Instruction::StoreIndU64 { base: 1, src, offset: o } if *src == reg && *o == *offset
            )
        });
        assert!(
            has_save,
            "Expected callee-save store for r{reg} at offset {offset} in calling function"
        );
    }
}

/// Inlining: calling a small leaf function should produce different (inlined) code.
#[test]
fn test_inlining_changes_codegen() {
    let wat = r#"
        (module
            (func $add_ten (param i32) (result i32)
                local.get 0
                i32.const 10
                i32.add
            )
            (func (export "main") (param i32) (result i32)
                local.get 0
                call $add_ten
            )
        )
    "#;

    let with_inline = compile_wat_with_options(
        wat,
        &CompileOptions {
            optimizations: OptimizationFlags {
                inlining: true,
                ..OptimizationFlags::default()
            },
            ..CompileOptions::default()
        },
    )
    .expect("compile with inlining");

    let without_inline = compile_wat_with_options(
        wat,
        &CompileOptions {
            optimizations: OptimizationFlags {
                inlining: false,
                ..OptimizationFlags::default()
            },
            ..CompileOptions::default()
        },
    )
    .expect("compile without inlining");

    // Inlining should produce different encoded output (the call is eliminated
    // from the entry function and replaced with the inlined body).
    let inlined_bytes = with_inline.encode();
    let noinline_bytes = without_inline.encode();
    assert_ne!(
        inlined_bytes, noinline_bytes,
        "Inlining should change the generated code"
    );

    // Without inlining, we expect LoadImmJump for the call (combined return addr load + jump).
    // With inlining, this overhead is eliminated.
    let noinline_instrs = extract_instructions(&without_inline);
    let noinline_call_count = noinline_instrs
        .iter()
        .filter(|i| matches!(i, Instruction::LoadImmJump { .. }))
        .count();

    let inlined_instrs = extract_instructions(&with_inline);
    let inlined_call_count = inlined_instrs
        .iter()
        .filter(|i| matches!(i, Instruction::LoadImmJump { .. }))
        .count();

    // Without inlining needs LoadImmJump for direct call; with inlining doesn't.
    assert!(
        inlined_call_count < noinline_call_count,
        "Inlining should reduce LoadImmJump count (call overhead): inlined={inlined_call_count}, no-inline={noinline_call_count}"
    );
}
/// Leaf function optimization: leaf function (no calls) should NOT save return address (r0).
#[test]
fn test_leaf_function_optimization_skips_ra_save() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (result i32)
                i32.const 42
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // Should NOT save RA (r0) to stack (offset 0).
    // StoreIndU64 { base: 1, src: 0, offset: 0 } should NOT exist.
    let has_ra_save = instructions.iter().any(|i| {
        matches!(
            i,
            Instruction::StoreIndU64 {
                base: 1,
                src: 0,
                offset: 0
            }
        )
    });
    assert!(
        !has_ra_save,
        "Leaf function should NOT save return address (r0)"
    );
}

/// Leaf function optimization: a non-main leaf function should NOT save RA,
/// even when that module also has a non-leaf function.
/// Inlining is disabled so helper functions are compiled as standalone functions.
/// Verifies that adding a non-leaf caller does not cause the leaf function to save RA;
/// by comparing RA save count between a module with two non-leaf callers vs.
/// one non-leaf caller + one leaf callee.
#[test]
fn test_non_main_leaf_function_skips_ra_save() {
    use wasm_pvm::test_harness::compile_wat_with_options;
    use wasm_pvm::{CompileOptions, OptimizationFlags};

    let opts = CompileOptions {
        optimizations: OptimizationFlags {
            inlining: false,
            ..OptimizationFlags::default()
        },
        ..CompileOptions::default()
    };

    // Module A: $helper is a leaf (no calls). $caller calls $helper. $main calls $caller.
    // Expected: $helper saves 0 RA, $caller saves 1 RA.
    let program_leaf = compile_wat_with_options(
        r#"
        (module
            (func $helper (result i32)
                i32.const 42
            )
            (func $caller (result i32)
                call $helper
            )
            (func (export "main") (result i32)
                call $caller
            )
        )
        "#,
        &opts,
    )
    .expect("compile leaf module");

    // Module B: $helper2 also calls something, making it non-leaf.
    // $caller calls $helper2. $main calls $caller.
    // Expected: $helper2 saves 1 RA, $caller saves 1 RA — total 2.
    let program_nonleaf = compile_wat_with_options(
        r#"
        (module
            (func $inner (result i32)
                i32.const 1
            )
            (func $helper2 (result i32)
                call $inner
            )
            (func $caller (result i32)
                call $helper2
            )
            (func (export "main") (result i32)
                call $caller
            )
        )
        "#,
        &opts,
    )
    .expect("compile non-leaf module");

    let count_ra_saves = |instrs: &[wasm_pvm::pvm::Instruction]| {
        instrs
            .iter()
            .filter(|i| {
                matches!(
                    i,
                    wasm_pvm::pvm::Instruction::StoreIndU64 {
                        base: 1,
                        src: 0,
                        offset: 0
                    }
                )
            })
            .count()
    };

    let leaf_ra_saves = count_ra_saves(&extract_instructions(&program_leaf));
    let nonleaf_ra_saves = count_ra_saves(&extract_instructions(&program_nonleaf));

    assert!(
        leaf_ra_saves < nonleaf_ra_saves,
        "Leaf $helper should save fewer RA than non-leaf $helper2: leaf={leaf_ra_saves}, nonleaf={nonleaf_ra_saves}"
    );
}

/// Non-leaf function (has calls) MUST save return address (r0).
/// Requires inlining disabled to preserve call structure.
#[test]
fn test_non_leaf_function_saves_ra() {
    use wasm_pvm::test_harness::compile_wat_with_options;
    use wasm_pvm::{CompileOptions, OptimizationFlags};

    let program = compile_wat_with_options(
        r#"
        (module
            (func $leaf (result i32)
                i32.const 42
            )
            (func $caller (result i32)
                call $leaf
            )
            (func (export "main") (result i32)
                call $caller
            )
        )
        "#,
        &CompileOptions {
            optimizations: OptimizationFlags {
                inlining: false,
                ..OptimizationFlags::default()
            },
            ..CompileOptions::default()
        },
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // We expect at least one RA save (from $caller).
    // StoreIndU64 { base: 1, src: 0, offset: 0 }
    let ra_save_count = instructions
        .iter()
        .filter(|i| {
            matches!(
                i,
                Instruction::StoreIndU64 {
                    base: 1,
                    src: 0,
                    offset: 0
                }
            )
        })
        .count();

    assert!(
        ra_save_count >= 1,
        "Non-leaf function ($caller) MUST save return address (r0)"
    );
}

/// Test that direct calls use the compact LoadImmJump instruction instead of
/// separate LoadImm64 + Jump, saving 1 instruction (1 gas) per call site.
#[test]
fn test_direct_calls_use_load_imm_jump() {
    use wasm_pvm::test_harness::compile_wat_with_options;
    use wasm_pvm::{CompileOptions, OptimizationFlags};

    let wat = r#"
        (module
            (func $helper (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.add
            )
            (func (export "main") (param i32) (result i32)
                local.get 0
                call $helper
                call $helper
            )
        )
    "#;

    let program = compile_wat_with_options(
        wat,
        &CompileOptions {
            optimizations: OptimizationFlags {
                inlining: false,
                ..OptimizationFlags::default()
            },
            ..CompileOptions::default()
        },
    )
    .expect("should compile");

    let instrs = extract_instructions(&program);

    // Direct calls should use LoadImmJump (not separate LoadImm64 + Jump).
    let load_imm_jump_count = instrs
        .iter()
        .filter(|i| matches!(i, Instruction::LoadImmJump { reg: 0, .. }))
        .count();
    assert!(
        load_imm_jump_count >= 2,
        "Expected at least 2 LoadImmJump instructions for 2 calls, got {load_imm_jump_count}"
    );

    // No LoadImm64 should be used for return addresses (reg 0).
    let load_imm64_r0_count = instrs
        .iter()
        .filter(|i| matches!(i, Instruction::LoadImm64 { reg: 0, .. }))
        .count();
    assert_eq!(
        load_imm64_r0_count, 0,
        "LoadImm64 with reg 0 should not be used for direct calls (LoadImmJump replaces it)"
    );

    // Verify jump table addresses are sequential: (0+1)*2=2, (1+1)*2=4, (2+1)*2=6, etc.
    let jump_addrs: Vec<i32> = instrs
        .iter()
        .filter_map(|i| match i {
            Instruction::LoadImmJump { reg: 0, value, .. } => Some(*value),
            _ => None,
        })
        .collect();
    for (i, addr) in jump_addrs.iter().enumerate() {
        let expected = ((i + 1) * 2) as i32;
        assert_eq!(
            *addr, expected,
            "Jump table address mismatch at call {i}: expected {expected}, got {addr}"
        );
    }
}
