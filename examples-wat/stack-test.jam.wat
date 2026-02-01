(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  ;; Test stack operations: compute x*2 + 10 where x is 10 or 20 based on arg
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $test_case i32)
    (local $x i32)
    (local $result i32)
    
    ;; Read test case from args
    (local.set $test_case (i32.load (local.get $args_ptr)))
    
    ;; x = 10 for test 0, x = 20 for test 1
    (if (i32.eq (local.get $test_case) (i32.const 0))
      (then (local.set $x (i32.const 10)))
      (else (local.set $x (i32.const 20)))
    )
    
    ;; Compute x*2 + 10 using stack operations
    (local.set $result
      (i32.add
        (i32.mul (local.get $x) (i32.const 2))
        (i32.const 10)
      )
    )
    
    (i32.store (i32.const 0) (local.get $result))
    (global.set $result_ptr (i32.const 0))
    (global.set $result_len (i32.const 4))
  )
)
