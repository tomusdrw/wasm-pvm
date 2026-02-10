(module
  (memory 1)
  
  
  (func $factorial (param $n i32) (result i32)
    (if (result i32) (i32.le_u (local.get $n) (i32.const 1))
      (then (i32.const 1))
      (else
        (i32.mul
          (local.get $n)
          (call $factorial (i32.sub (local.get $n) (i32.const 1)))
        )
      )
    )
  )
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (local $result i32)
    
    (local.set $result
      (call $factorial (i32.load (local.get $args_ptr)))
    )
    
    (i32.store (i32.const 0) (local.get $result))
    (i32.const 0)  ;; result_ptr
    (i32.const 4)  ;; result_len
  )
)
