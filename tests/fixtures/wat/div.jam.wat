(module
  (memory 1)

  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    ;; Store result of a / b (unsigned) at heap address 0
    (i32.store 
      (i32.const 0)
      (i32.div_u 
        (i32.load (local.get $args_ptr))
        (i32.load (i32.add (local.get $args_ptr) (i32.const 4)))
      )
    )

    (i32.const 0)  ;; result_ptr
    (i32.const 4)  ;; result_len
  )
)
