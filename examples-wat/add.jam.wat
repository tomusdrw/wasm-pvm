(module
  (memory 1)
  
  ;; Globals stored at 0x20000 + idx*4 by compiler
  ;; User heap should start at 0x30100 to avoid collision
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    ;; Write sum of two u32 args to heap (0x30100)
    (i32.store
      (i32.const 0x30100)
      (i32.add
        (i32.load (local.get $args_ptr))
        (i32.load (i32.add (local.get $args_ptr) (i32.const 4)))
      )
    )
    
    (global.set $result_ptr (i32.const 0x30100))
    (global.set $result_len (i32.const 4))
  )
)
