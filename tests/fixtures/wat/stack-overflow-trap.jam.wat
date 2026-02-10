;; Test: Stack overflow detection should trigger TRAP (PANIC)
;; This uses a smaller stack limit by having more operand stack depth
(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  ;; Recursive function with some operand stack values to increase frame size
  (func $deep_recurse (param $n i32) (result i32)
    (if (result i32) (i32.eqz (local.get $n))
      (then (i32.const 1))
      (else
        ;; Push some values on operand stack before call to increase frame size
        (i32.add
          (i32.const 1)
          (i32.add
            (i32.const 2)
            (call $deep_recurse (i32.sub (local.get $n) (i32.const 1)))
          )
        )
      )
    )
  )
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $result i32)
    
    ;; Read the recursion depth from args
    (local.set $result
      (call $deep_recurse (i32.load (local.get $args_ptr)))
    )
    
    (i32.store (i32.const 0) (local.get $result))
    (global.set $result_ptr (i32.const 0))
    (global.set $result_len (i32.const 4))
  )
)
