(module
  (memory 1)

  ;; Result globals

  ;; Helper function: double a number
  (func $double (param $x i32) (result i32)
    (i32.add (local.get $x) (local.get $x))
  )

  ;; Main entry point: reads one u32, doubles it via call
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (local $result i32)

    ;; result = double(input)
    (local.set $result
      (call $double
        (i32.load (local.get $args_ptr))
      )
    )

    ;; Store result
    (i32.store (i32.const 0) (local.get $result))

    ;; Set output pointer and length
    (i32.const 0)  ;; result_ptr
    (i32.const 4)  ;; result_len
  )
)
