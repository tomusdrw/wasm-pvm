(module
  (memory 1)
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (local $op i32)
    (local $val i32)
    (local $n i32)
    (local $result i32)

    (local.set $op (i32.load (local.get $args_ptr)))
    (local.set $val (i32.load (i32.add (local.get $args_ptr) (i32.const 4))))
    (local.set $n (i32.load (i32.add (local.get $args_ptr) (i32.const 8))))

    (if (i32.eq (local.get $op) (i32.const 0))
      (then
        (local.set $result (i32.rotl (local.get $val) (local.get $n)))
      )
    )

    (if (i32.eq (local.get $op) (i32.const 1))
      (then
        (local.set $result (i32.rotr (local.get $val) (local.get $n)))
      )
    )

    (i32.store (i32.const 0) (local.get $result))

    (i32.const 0)  ;; result_ptr
    (i32.const 4)  ;; result_len
  )
)
