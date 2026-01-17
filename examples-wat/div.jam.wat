(module
  (memory 1)
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))

  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    ;; Store result of a / b (unsigned) at heap address 0x20100
    (i32.store 
      (i32.const 0x20100)
      (i32.div_u 
        (i32.load (local.get $args_ptr))
        (i32.load (i32.add (local.get $args_ptr) (i32.const 4)))
      )
    )

    ;; Set result pointer and length
    (global.set $result_ptr (i32.const 0x20100))
    (global.set $result_len (i32.const 4))
  )
)
