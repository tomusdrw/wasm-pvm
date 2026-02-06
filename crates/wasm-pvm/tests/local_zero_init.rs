//! Regression test for local variable zero-initialization bug.
//!
//! WebAssembly spec requires all local variables to be zero-initialized.
//! This test ensures non-parameter locals are properly set to 0.

use wasm_pvm::test_harness::*;

/// Test that local variables are zero-initialized even when not explicitly set.
/// This was causing bugs where loop counters would start with garbage values.
#[test]
fn test_wasm_local_zero_init() {
    // This WAT relies on $1 (the loop counter) being zero-initialized.
    // Without proper initialization, the loop would not execute correctly.
    let wat = r#"
        (module
            (memory 1)
            (global $result_ptr (mut i32) (i32.const 0))
            (global $result_len (mut i32) (i32.const 0))
            (export "result_ptr" (global $result_ptr))
            (export "result_len" (global $result_len))

            (func $sum_to_n (param $n i32) (result i32)
                (local $i i32)      ;; NOT explicitly initialized - relies on zero-init
                (local $sum i32)    ;; NOT explicitly initialized - relies on zero-init

                ;; Loop: sum = sum + i, for i in 0..$n
                loop $loop
                    local.get $i
                    local.get $n
                    i32.lt_s
                    if
                        local.get $sum
                        local.get $i
                        i32.add
                        local.set $sum

                        local.get $i
                        i32.const 1
                        i32.add
                        local.set $i

                        br $loop
                    end
                end

                local.get $sum
            )

            (func (export "main") (param $args_ptr i32) (param $args_len i32)
                ;; Calculate sum(0..10) = 0+1+2+3+4+5+6+7+8+9 = 45
                i32.const 0x50100  ;; Result address
                i32.const 10
                call $sum_to_n
                i32.store

                i32.const 0x50100
                global.set $result_ptr
                i32.const 4
                global.set $result_len
            )
        )
    "#;

    let program = compile_wat(wat).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Verify we have LoadImm instructions for zero-initializing locals.
    // The function $sum_to_n has 2 non-parameter locals ($i, $sum) that need initialization.
    let zero_init_count = instructions
        .iter()
        .filter(|instr| matches!(instr, wasm_pvm::pvm::Instruction::LoadImm { value: 0, .. }))
        .count();

    assert!(
        zero_init_count >= 2,
        "Expected at least 2 LoadImm 0 instructions for local zero-init, found {zero_init_count}"
    );
}
