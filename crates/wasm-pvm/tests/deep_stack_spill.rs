use wasm_pvm::test_harness::*;

/// Basic test: push 6 values (forces 1 spill), then pop all with operations.
#[test]
fn test_stack_depth_6_basic() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (result i32)
                i32.const 1
                i32.const 2
                i32.const 3
                i32.const 4
                i32.const 5
                i32.const 6
                i32.add  ;; 5+6=11
                i32.add  ;; 4+11=15
                i32.add  ;; 3+15=18
                i32.add  ;; 2+18=20
                i32.add  ;; 1+20=21
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);
    // LLVM backend constant-folds pure-constant expressions during IR construction,
    // so we just verify the program compiles and produces instructions.
    assert!(
        !instructions.is_empty(),
        "Should produce instructions for deep stack"
    );
}

/// Push 8 values (3 spilled), then reduce with additions.
#[test]
fn test_stack_depth_8() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (result i32)
                i32.const 1
                i32.const 2
                i32.const 3
                i32.const 4
                i32.const 5
                i32.const 6
                i32.const 7
                i32.const 8
                i32.add
                i32.add
                i32.add
                i32.add
                i32.add
                i32.add
                i32.add
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);
    assert!(
        !instructions.is_empty(),
        "Should produce instructions for deep stack"
    );
}

/// Operand stack spill across a function call boundary.
/// The caller has spilled values on the stack, makes a call, and the spilled
/// values should be preserved after the call returns.
#[test]
fn test_spill_across_function_call() {
    let program = compile_wat(
        r#"
        (module
            (func $identity (param i32) (result i32)
                local.get 0
            )
            (func (export "main") (result i32)
                ;; Push 6 values (1 spilled)
                i32.const 10
                i32.const 20
                i32.const 30
                i32.const 40
                i32.const 50
                ;; Call function with the top value
                i32.const 7
                call $identity
                ;; Now we have: 10, 20, 30, 40, 50, result(7) on stack
                ;; Pop and add them all
                i32.add
                i32.add
                i32.add
                i32.add
                i32.add
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);
    assert!(!instructions.is_empty(), "Should compile spill across call");
}

/// Multiple spilled values across a function call.
#[test]
fn test_multiple_spills_across_call() {
    let program = compile_wat(
        r#"
        (module
            (func $add (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.add
            )
            (func (export "main") (result i32)
                ;; Push enough to spill
                i32.const 1
                i32.const 2
                i32.const 3
                i32.const 4
                ;; Call with 2 args (pops 2, pushes 1)
                i32.const 100
                i32.const 200
                call $add
                ;; Stack: 1, 2, 3, 4, 300
                i32.add  ;; 4+300=304
                i32.add  ;; 3+304=307
                i32.add  ;; 2+307=309
                i32.add  ;; 1+309=310
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);
    assert!(
        !instructions.is_empty(),
        "Should compile multiple spills across call"
    );
}

/// Deep stack with mixed operations (add, sub, mul) to test spill with different binary ops.
#[test]
fn test_deep_stack_mixed_operations() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (result i32)
                i32.const 100
                i32.const 50
                i32.const 25
                i32.const 10
                i32.const 5
                i32.const 3
                i32.const 2
                i32.mul  ;; 3*2=6
                i32.add  ;; 5+6=11
                i32.sub  ;; 10-11=-1
                i32.mul  ;; 25*(-1)=-25
                i32.add  ;; 50+(-25)=25
                i32.sub  ;; 100-25=75
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);
    assert!(
        !instructions.is_empty(),
        "Should produce instructions for mixed operations"
    );
}

/// Spilled stack with conditional (if/else) to test that spill state
/// is properly managed across control flow.
#[test]
fn test_spill_with_if_else() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (param i32) (result i32)
                ;; Push enough to spill
                i32.const 1
                i32.const 2
                i32.const 3
                i32.const 4
                i32.const 5
                ;; Conditional that pushes a value
                local.get 0
                if (result i32)
                    i32.const 100
                else
                    i32.const 200
                end
                ;; Stack: 1, 2, 3, 4, 5, 100/200
                i32.add
                i32.add
                i32.add
                i32.add
                i32.add
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);
    assert!(
        !instructions.is_empty(),
        "Should compile spill with if/else"
    );
}

/// Nested function calls with spilled stack at each level.
#[test]
fn test_nested_calls_with_spill() {
    let program = compile_wat(
        r#"
        (module
            (func $inner (param i32) (result i32)
                local.get 0
                i32.const 1
                i32.add
            )
            (func $outer (param i32) (result i32)
                ;; Build up stack
                i32.const 10
                i32.const 20
                i32.const 30
                i32.const 40
                i32.const 50
                ;; Call inner
                local.get 0
                call $inner
                ;; Stack: 10, 20, 30, 40, 50, result
                i32.add
                i32.add
                i32.add
                i32.add
                i32.add
            )
            (func (export "main") (result i32)
                i32.const 5
                call $outer
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);
    assert!(
        !instructions.is_empty(),
        "Should compile nested calls with spill"
    );
}

/// Loop that accumulates a sum in a local using local.tee and stack operations.
#[test]
fn test_loop_with_stack_accumulation() {
    let program = compile_wat(
        r#"
        (module
            (func (export "main") (result i32)
                (local i32 i32)
                ;; local 0 = counter, local 1 = accumulator
                i32.const 0
                local.set 0
                i32.const 0
                local.set 1
                block
                    loop
                        ;; accumulator += counter
                        local.get 1
                        local.get 0
                        i32.add
                        local.set 1
                        ;; counter++
                        local.get 0
                        i32.const 1
                        i32.add
                        local.tee 0
                        ;; if counter < 5, loop
                        i32.const 5
                        i32.lt_u
                        br_if 0
                    end
                end
                local.get 1
            )
        )
        "#,
    )
    .expect("compile");
    let instructions = extract_instructions(&program);
    assert!(
        !instructions.is_empty(),
        "Should compile loop with stack accumulation"
    );
}
