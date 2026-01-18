(module
  (memory 1)

  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))

  ;; Simple test: block with br and result
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $result i32)

    ;; Block with br carrying a result value
    (local.set $result
      (block (result i32)
        (i32.const 123)
        (br 0)
        (i32.const 999)
      )
    )

    ;; Store result
    (i32.store (i32.const 0x20100) (local.get $result))

    (global.set $result_ptr (i32.const 0x20100))
    (global.set $result_len (i32.const 4))
  )
)
