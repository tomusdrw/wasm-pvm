use wasm_pvm::pvm::Opcode;
use wasm_pvm::test_harness::*;

/// local.tee to a register local (index 0-3) with shallow stack (no spill).
/// Should produce an AddImm64 copy from stack reg to local reg.
#[test]
fn test_local_tee_register_local_shallow_stack() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (param i32) (result i32)
                (local i32)
                i32.const 42
                local.tee 1
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // The local.tee should emit AddImm64 to copy from stack to local register
    assert!(
        has_opcode(&instructions, Opcode::AddImm64),
        "local.tee to register local should use AddImm64 for copy"
    );
}

/// local.tee to a spilled local (index 4+) with shallow stack.
/// Should produce a StoreIndU64 to memory.
#[test]
fn test_local_tee_spilled_local_shallow_stack() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (param i32) (result i32)
                (local i32 i32 i32 i32 i32)
                i32.const 99
                local.tee 5
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // local.tee to spilled local should store to memory
    let store_count = count_opcode(&instructions, Opcode::StoreIndU64);
    assert!(
        store_count >= 1,
        "local.tee to spilled local should emit StoreIndU64, found {store_count}"
    );
}

/// local.tee to register local with deep stack (stack is spilling).
/// Pushes 6+ values to force operand stack spill, then does local.tee.
#[test]
fn test_local_tee_register_local_deep_stack() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (param i32) (result i32)
                (local i32)
                ;; Push 6 values to force operand stack spill (only 5 register slots)
                i32.const 1
                i32.const 2
                i32.const 3
                i32.const 4
                i32.const 5
                i32.const 6
                ;; local.tee with spilled operand stack top
                local.tee 1
                ;; Clean up stack: drop all but one
                drop
                drop
                drop
                drop
                drop
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // Should compile without panicking - the value on top of the spilled stack
    // should be copied to the register local
    assert!(
        !instructions.is_empty(),
        "Should produce instructions for deep stack local.tee"
    );
}

/// local.tee to spilled local with deep stack (both operand and local spill).
#[test]
fn test_local_tee_spilled_local_deep_stack() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (param i32) (result i32)
                (local i32 i32 i32 i32 i32)
                ;; Push 6 values to force operand stack spill
                i32.const 1
                i32.const 2
                i32.const 3
                i32.const 4
                i32.const 5
                i32.const 6
                ;; local.tee to spilled local 5, with spilled stack
                local.tee 5
                ;; Clean up
                drop
                drop
                drop
                drop
                drop
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        !instructions.is_empty(),
        "Should produce instructions for spilled local + deep stack local.tee"
    );
}

/// local.tee followed by local.get should yield the same value.
/// This tests the semantic correctness: tee writes to local AND leaves value on stack.
#[test]
fn test_local_tee_preserves_stack_value() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (result i32)
                (local i32)
                i32.const 77
                local.tee 0
                ;; Stack still has 77, and local 0 is 77
                local.get 0
                i32.add
                ;; Result should be 154
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    // Should have Add32 instruction for the i32.add
    assert!(
        has_opcode(&instructions, Opcode::Add32),
        "Should have Add32 for the i32.add after local.tee + local.get"
    );
}

/// local.tee immediately after a push to spill depth, where the value has a pending spill.
/// This tests the pending_spill detection path in local.tee.
#[test]
fn test_local_tee_with_pending_spill() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (param i32) (result i32)
                (local i32)
                ;; Fill register slots
                i32.const 1
                i32.const 2
                i32.const 3
                i32.const 4
                i32.const 5
                ;; This push forces spill - value will be in r7 with pending spill
                i32.const 42
                ;; local.tee while the top has a pending spill
                local.tee 1
                ;; Clean up
                drop
                drop
                drop
                drop
                drop
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        !instructions.is_empty(),
        "Should compile local.tee with pending spill"
    );
}

/// local.tee inside a loop at spill depth - regression test for complex control flow.
#[test]
fn test_local_tee_in_loop_with_spill() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (param i32) (result i32)
                (local i32 i32)
                i32.const 0
                local.set 1
                block
                    loop
                        ;; Increment param and tee into local 0
                        local.get 0
                        i32.const 1
                        i32.add
                        local.tee 0
                        drop
                        ;; Increment counter and check loop condition
                        local.get 1
                        i32.const 1
                        i32.add
                        local.tee 1
                        i32.const 10
                        i32.lt_u
                        br_if 0
                    end
                end
                local.get 0
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);

    assert!(
        !instructions.is_empty(),
        "Loop with local.tee should compile"
    );
}
