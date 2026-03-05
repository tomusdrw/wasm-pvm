(module
  (memory 1)

  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    ;; Store result of a / b (unsigned) at heap address 0
    (i32.store 
      (i32.const 0)
      (i32.div_u 
        (i32.load (local.get $args_ptr))
        (i32.load (i32.add (local.get $args_ptr) (i32.const 4)))
      )
    )

    (i64.const 17179869184)  ;; ptr=0, len=4
  )
)
