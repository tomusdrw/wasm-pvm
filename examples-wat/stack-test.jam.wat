(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  (func $helper (param $x i32) (result i32)
    (i32.add (local.get $x) (i32.const 1))
  )
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $result i32)
    
    (local.set $result
      (call $helper (i32.const 10))
    )
    
    (i32.store (i32.const 0x30100) (local.get $result))
    (global.set $result_ptr (i32.const 0x30100))
    (global.set $result_len (i32.const 4))
  )
)
