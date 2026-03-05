(module
  (memory 1)
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $op i32)
    (local $val i32)
    (local $result i32)

    (local.set $op (i32.load (local.get $args_ptr)))
    (local.set $val (i32.load (i32.add (local.get $args_ptr) (i32.const 4))))

    (if (i32.eq (local.get $op) (i32.const 0))
      (then
        (local.set $result (i32.clz (local.get $val)))
      )
    )

    (if (i32.eq (local.get $op) (i32.const 1))
      (then
        (local.set $result (i32.ctz (local.get $val)))
      )
    )

    (if (i32.eq (local.get $op) (i32.const 2))
      (then
        (local.set $result (i32.popcnt (local.get $val)))
      )
    )

    (i32.store (i32.const 0) (local.get $result))

    (i64.const 17179869184)  ;; ptr=0, len=4
  )
)
