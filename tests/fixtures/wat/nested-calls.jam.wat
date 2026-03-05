(module
  (memory 1)
  
  
  (func $add_one (param $x i32) (result i32)
    (i32.add (local.get $x) (i32.const 1))
  )
  
  (func $add_two (param $x i32) (result i32)
    (i32.add (call $add_one (local.get $x)) (i32.const 1))
  )
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $result i32)
    
    (local.set $result
      (call $add_two (i32.load (local.get $args_ptr)))
    )
    
    (i32.store (i32.const 0) (local.get $result))
    (i64.const 17179869184)  ;; ptr=0, len=4
  )
)
