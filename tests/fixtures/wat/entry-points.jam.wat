(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  ;; main entry point (PC=0): returns 42
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (i32.store (i32.const 0) (i32.const 42))
    (global.set $result_ptr (i32.const 0))
    (global.set $result_len (i32.const 4))
  )
  
  ;; main2 entry point (PC=5): returns 99
  (func (export "main2") (param $args_ptr i32) (param $args_len i32)
    (i32.store (i32.const 0) (i32.const 99))
    (global.set $result_ptr (i32.const 0))
    (global.set $result_len (i32.const 4))
  )
)
