;; Test all comparison operators to verify correctness
;; Input: test case index (0-3 for the four main tests)
;; Output: 4 bytes - the comparison result (0 or 1)

(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $test_case i32)
    (local $result i32)
    
    ;; Read test case from args
    (local.set $test_case (i32.load (local.get $args_ptr)))
    
    ;; Run the appropriate test based on test_case
    (block
      ;; Test 0: 3 < 5 should be 1
      (if (i32.eq (local.get $test_case) (i32.const 0))
        (then
          (local.set $result (i32.lt_s (i32.const 3) (i32.const 5)))
          (br 1)
        )
      )
      
      ;; Test 1: 5 < 3 should be 0
      (if (i32.eq (local.get $test_case) (i32.const 1))
        (then
          (local.set $result (i32.lt_s (i32.const 5) (i32.const 3)))
          (br 1)
        )
      )
      
      ;; Test 2: 10 > 5 should be 1
      (if (i32.eq (local.get $test_case) (i32.const 2))
        (then
          (local.set $result (i32.gt_s (i32.const 10) (i32.const 5)))
          (br 1)
        )
      )
      
      ;; Test 3: 5 > 10 should be 0
      (if (i32.eq (local.get $test_case) (i32.const 3))
        (then
          (local.set $result (i32.gt_s (i32.const 5) (i32.const 10)))
          (br 1)
        )
      )
      
      ;; Default: return 0
      (local.set $result (i32.const 0))
    )
    
    ;; Store result
    (i32.store (i32.const 0) (local.get $result))
    
    ;; Set result metadata
    (global.set $result_ptr (i32.const 0))
    (global.set $result_len (i32.const 4))
  )
)
