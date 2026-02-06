//! Test for nested if-result blocks with ternary
//!
//! This test reproduces a bug where nested if-result blocks with a ternary
//! that takes the THEN branch (memory load) causes incorrect behavior.

use wasm_pvm::test_harness::*;

/// Minimal WAT that reproduces the nested if-result bug
const NESTED_IF_RESULT_WAT: &str = r#"
(module
    (memory 1)
    (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32)
        (local $step i32)
        (local $arr_base i32)

        ;; Initialize: step from args, arr_base = some address
        local.get $args_len
        i32.const 0
        i32.gt_s
        if (result i32)
            local.get $args_ptr
            i32.load8_u
        else
            i32.const 0
        end
        local.set $step

        ;; arr_base = 0x100 (simulate array at this address)
        i32.const 256
        local.set $arr_base

        ;; Initialize array: arr[0]=0, arr[1]=1
        local.get $arr_base
        i32.const 0
        i32.store8
        local.get $arr_base
        i32.const 1
        i32.store8 offset=1

        ;; Push result address (we'll use this for the store)
        i32.const 0x200

        ;; OUTER IF: if step != 0
        local.get $step
        if (result i32)
            ;; INNER IF: if step == 1
            local.get $step
            i32.const 1
            i32.eq
            if (result i32)
                ;; TERNARY: args_len > 1 ? load(args_ptr+1) : 5
                local.get $args_len
                i32.const 1
                i32.gt_s
                if (result i32)
                    local.get $args_ptr
                    i32.load8_u offset=1
                else
                    i32.const 5
                end
                drop  ;; Drop ternary result

                ;; Load arr[1] - should be 1
                local.get $arr_base
                i32.load8_u offset=1
            else
                i32.const 99
            end
        else
            ;; step == 0: just load arr[1]
            local.get $arr_base
            i32.load8_u offset=1
        end

        ;; Store result
        i32.store

        ;; Return the stored value for verification
        i32.const 0x200
        i32.load
    )
)
"#;

#[test]
fn test_nested_if_result_instructions() {
    let program = compile_wat(NESTED_IF_RESULT_WAT).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    // Print all instructions for debugging
    println!("Generated {} instructions:", instructions.len());
    for (i, instr) in instructions.iter().enumerate() {
        println!("{i:4}: {instr:?}");
    }

    // The test should compile without panicking
    assert!(!instructions.is_empty());
}

/// Test matching the exact nested-repro pattern
const EXACT_NESTED_REPRO_WAT: &str = r#"
(module
    (memory 1)

    ;; Simulate createArray - function with a loop
    (func $createArray (result i32)
        (local $base i32)
        (local $i i32)

        ;; Allocate at fixed address 0x200
        i32.const 0x200
        local.set $base

        ;; Initialize: arr[i] = i for i in 0..10
        i32.const 0
        local.set $i
        (loop $loop
            ;; store arr[i] = i
            local.get $base
            local.get $i
            i32.add
            local.get $i
            i32.store8

            ;; i++
            local.get $i
            i32.const 1
            i32.add
            local.set $i

            ;; continue if i < 10
            local.get $i
            i32.const 10
            i32.lt_s
            br_if $loop
        )

        local.get $base
    )

    (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32)
        (local $step i32)
        (local $arr i32)

        ;; Get step from args (ternary: args_len > 0 ? load(args_ptr) : 0)
        local.get $args_len
        i32.const 0
        i32.gt_s
        if (result i32)
            local.get $args_ptr
            i32.load8_u
        else
            i32.const 0
        end
        local.set $step

        ;; Create array (call function with loop)
        call $createArray
        local.set $arr

        ;; Create second array (dropped - matches nested-repro)
        call $createArray
        drop

        ;; Push result address onto stack
        i32.const 0x100

        ;; OUTER IF: if step != 0
        local.get $step
        if (result i32)
            ;; INNER IF: if step == 1
            local.get $step
            i32.const 1
            i32.eq
            if (result i32)
                ;; TERNARY: args_len > 1 ? load(args_ptr+1) : 5
                local.get $args_len
                i32.const 1
                i32.gt_s
                if (result i32)
                    local.get $args_ptr
                    i32.load8_u offset=1
                else
                    i32.const 5
                end
                drop  ;; DROP the ternary result

                ;; Load arr[1] - should be 1
                local.get $arr
                i32.load8_u offset=1
            else
                i32.const 99
            end
        else
            ;; step == 0: just load arr[1]
            local.get $arr
            i32.load8_u offset=1
        end

        ;; Store result
        i32.store

        ;; Return stored value
        i32.const 0x100
        i32.load
    )
)
"#;

#[test]
fn test_exact_nested_repro() {
    let program = compile_wat(EXACT_NESTED_REPRO_WAT).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    println!("\n=== Exact Nested Repro ===");
    println!("Generated {} instructions:", instructions.len());
    for (i, instr) in instructions.iter().enumerate() {
        println!("{i:4}: {instr:?}");
    }

    assert!(!instructions.is_empty());
}

/// Test with a function call inside nested if-result
const NESTED_WITH_CALL_WAT: &str = r#"
(module
    (memory 1)

    ;; Simple function that returns its argument + 1
    (func $add_one (param $x i32) (result i32)
        local.get $x
        i32.const 1
        i32.add
    )

    (func (export "main") (param $cond i32) (result i32)
        (local $result i32)

        ;; Store 0 at address 0
        i32.const 0
        i32.const 0
        i32.store

        ;; Push result accumulator address
        i32.const 0x100

        ;; Outer if (result i32)
        local.get $cond
        if (result i32)
            ;; Inner if (result i32) - always true
            i32.const 1
            if (result i32)
                ;; Ternary - take THEN branch if cond > 1
                local.get $cond
                i32.const 1
                i32.gt_s
                if (result i32)
                    i32.const 99  ;; THEN: push some value
                else
                    i32.const 5   ;; ELSE: push different value
                end
                drop  ;; Drop ternary result

                ;; Call a function (this might trigger the bug)
                i32.const 0
                call $add_one  ;; Should return 1
            else
                i32.const 77
            end
        else
            ;; No function call in else
            i32.const 42
        end

        ;; Store to result address
        i32.store

        ;; Return stored value
        i32.const 0x100
        i32.load
    )
)
"#;

#[test]
fn test_nested_with_call() {
    let program = compile_wat(NESTED_WITH_CALL_WAT).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    println!("\n=== Nested If-Result with Function Call ===");
    println!("Generated {} instructions:", instructions.len());
    for (i, instr) in instructions.iter().enumerate() {
        println!("{i:4}: {instr:?}");
    }

    assert!(!instructions.is_empty());
}

/// A simpler test to verify the pattern
const SIMPLE_NESTED_WAT: &str = r#"
(module
    (memory 1)
    (func (export "main") (param $cond i32) (result i32)
        ;; Store 1 at address 0
        i32.const 0
        i32.const 1
        i32.store8

        ;; Push result accumulator address
        i32.const 0x100

        ;; Outer if (result i32)
        local.get $cond
        if (result i32)
            ;; Inner if (result i32) - always true
            i32.const 1
            if (result i32)
                ;; Ternary - take THEN branch if cond > 1
                local.get $cond
                i32.const 1
                i32.gt_s
                if (result i32)
                    i32.const 99  ;; THEN: push some value
                else
                    i32.const 5   ;; ELSE: push different value
                end
                drop  ;; Drop ternary result

                ;; Load from address 0 - should be 1
                i32.const 0
                i32.load8_u
            else
                i32.const 77
            end
        else
            ;; Just load from address 0
            i32.const 0
            i32.load8_u
        end

        ;; Store to result address
        i32.store

        ;; Return stored value
        i32.const 0x100
        i32.load
    )
)
"#;

#[test]
fn test_simple_nested_if_result() {
    let program = compile_wat(SIMPLE_NESTED_WAT).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    println!("\n=== Simple Nested If-Result ===");
    println!("Generated {} instructions:", instructions.len());
    for (i, instr) in instructions.iter().enumerate() {
        println!("{i:4}: {instr:?}");
    }

    assert!(!instructions.is_empty());
}

/// Test that matches the EXACT `AssemblyScript` __get pattern:
/// - A function with internal control flow (if-unreachable bounds check)
/// - Called after a ternary with memory load in THEN branch + drop
const AS_GET_PATTERN_WAT: &str = r#"
(module
    (memory 1)

    ;; Simulate AssemblyScript __get function with bounds check
    ;; Takes array pointer, returns arr[1]
    (func $get_at_1 (param $arr i32) (result i32)
        ;; Bounds check: if arr.length <= 1, trap
        local.get $arr
        i32.load offset=12      ;; load arr.length from offset 12
        i32.const 1
        i32.le_u
        if
            unreachable
        end
        ;; Load dataStart, then load byte at offset 1
        local.get $arr
        i32.load offset=4       ;; load arr.dataStart from offset 4
        i32.load8_u offset=1    ;; load byte at dataStart + 1
    )

    ;; Create array at fixed address with structure:
    ;; offset 0: unused
    ;; offset 4: dataStart pointer
    ;; offset 8: byteLength
    ;; offset 12: length
    (func $createArray (result i32)
        (local $arr i32)
        (local $data i32)
        (local $i i32)

        ;; Array structure at 0x100
        i32.const 0x100
        local.set $arr

        ;; Data at 0x200
        i32.const 0x200
        local.set $data

        ;; arr[4] = dataStart = 0x200
        local.get $arr
        local.get $data
        i32.store offset=4

        ;; arr[8] = byteLength = 10
        local.get $arr
        i32.const 10
        i32.store offset=8

        ;; arr[12] = length = 10
        local.get $arr
        i32.const 10
        i32.store offset=12

        ;; Fill data: data[i] = i for i in 0..10
        i32.const 0
        local.set $i
        (loop $loop
            local.get $data
            local.get $i
            i32.add
            local.get $i
            i32.store8

            local.get $i
            i32.const 1
            i32.add
            local.set $i

            local.get $i
            i32.const 10
            i32.lt_s
            br_if $loop
        )

        local.get $arr
    )

    ;; Globals for result
    (global $result_ptr (mut i32) (i32.const 0))
    (global $result_len (mut i32) (i32.const 0))
    (export "result_ptr" (global $result_ptr))
    (export "result_len" (global $result_len))

    (func (export "main") (param $args_ptr i32) (param $args_len i32)
        (local $step i32)
        (local $arr i32)

        ;; Get step from args (ternary)
        local.get $args_len
        i32.const 0
        i32.gt_s
        if (result i32)
            local.get $args_ptr
            i32.load8_u
        else
            i32.const 0
        end
        local.set $step

        ;; Create array (with loop inside)
        call $createArray
        local.set $arr

        ;; Create second array and drop (matches AS pattern)
        call $createArray
        drop

        ;; Push result address
        i32.const 0x300

        ;; OUTER IF: if step != 0
        local.get $step
        if (result i32)
            ;; INNER IF: if step == 1
            local.get $step
            i32.const 1
            i32.eq
            if (result i32)
                ;; TERNARY: args_len > 1 ? load(args_ptr+1) : 5
                local.get $args_len
                i32.const 1
                i32.gt_s
                if (result i32)
                    local.get $args_ptr
                    i32.load8_u offset=1
                else
                    i32.const 5
                end
                drop  ;; DROP the ternary result

                ;; Call get_at_1 - SHOULD RETURN 1
                local.get $arr
                call $get_at_1
            else
                i32.const 99
            end
        else
            local.get $arr
            call $get_at_1
        end

        ;; Store result
        i32.store

        ;; Set result globals
        i32.const 0x300
        global.set $result_ptr
        i32.const 4
        global.set $result_len
    )
)
"#;

#[test]
fn test_as_get_pattern() {
    let program = compile_wat(AS_GET_PATTERN_WAT).expect("Failed to compile");
    let instructions = extract_instructions(&program);

    println!("\n=== AssemblyScript __get Pattern ===");
    println!("Generated {} instructions:", instructions.len());
    for (i, instr) in instructions.iter().enumerate() {
        println!("{i:4}: {instr:?}");
    }

    assert!(!instructions.is_empty());
}
