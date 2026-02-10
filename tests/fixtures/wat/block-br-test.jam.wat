(module
  (memory 1)

  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))

  ;; Test block with br and br_if
  ;; arg 0: br_if with condition false (skip branch) -> returns 10
  ;; arg 1: br_if with condition true (take branch) -> returns 20  
  ;; arg 2: nested blocks with br_if -> returns 30
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $test_case i32)
    (local $result i32)

    ;; Read test case from args
    (local.set $test_case (i32.load (local.get $args_ptr)))
    
    ;; Test 0: br_if with false condition - continue after br_if, return 10
    (if (i32.eq (local.get $test_case) (i32.const 0))
      (then
        (local.set $result
          (block (result i32)
            (i32.const 10)  ;; Result value
            (br_if 0 (i32.const 0))  ;; Don't branch (condition is false)
            (drop)          ;; Drop the 10
            (i32.const 10)  ;; Return 10
          )
        )
      )
    )
    
    ;; Test 1: br_if with true condition - take branch, return 20
    (if (i32.eq (local.get $test_case) (i32.const 1))
      (then
        (local.set $result
          (block (result i32)
            (i32.const 20)  ;; This is the result when we branch
            (br_if 0 (i32.const 1))  ;; Branch (condition is true)
            (drop)          ;; Drop the 20 if we fall through
            (i32.const 999) ;; This won't execute
          )
        )
      )
    )
    
    ;; Test 2: nested blocks with br_if - return 30
    (if (i32.eq (local.get $test_case) (i32.const 2))
      (then
        (local.set $result
          (block (result i32)  ;; Outer block
            (i32.const 30)    ;; Final result
            (block (result i32)  ;; Inner block
              (i32.const 40)
              (br_if 0 (i32.const 1))  ;; Branch out of inner block with value 40
              (drop)          ;; Drop the 40 if we fall through
              (i32.const 50)  ;; Won't execute
            )
            (drop)  ;; Drop the 40 from inner block
          )
        )
      )
    )

    ;; Store result
    (i32.store (i32.const 0) (local.get $result))
    (global.set $result_ptr (i32.const 0))
    (global.set $result_len (i32.const 4))
  )
)
