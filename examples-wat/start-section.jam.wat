(module
  (memory 1)
  
  (global $g (mut i32) (i32.const 0))
  (func $start
    (global.set $g (i32.const 42))
  )
  (start $start)
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    ;; Return result (pointer to 0x30100)
    (global.set $result_ptr (i32.const 0x30100))
    (global.set $result_len (i32.const 4))
    
    ;; Store global value to result area
    (i32.store (i32.const 0x30100) (global.get $g))
  )
  
  (global $result_ptr (export "result_ptr") (mut i32) (i32.const 0))
  (global $result_len (export "result_len") (mut i32) (i32.const 0))
)
