(module
  (memory 1)

  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))

  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $step i32)
    (local $base i32)

    ;; Store values at various offsets from base address 100
    (i32.store offset=0 (i32.const 100) (i32.const 42))
    (i32.store offset=4 (i32.const 100) (i32.const 99))
    (i32.store offset=16 (i32.const 100) (i32.const 77))
    (i32.store offset=20 (i32.const 100) (i32.const 55))

    (local.set $step (i32.load (local.get $args_ptr)))

    ;; step 0: read offset 0 (expect 42)
    (if (i32.eqz (local.get $step))
      (then
        (i32.store (i32.const 0) (i32.load offset=0 (i32.const 100)))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; step 1: read offset 4 (expect 99)
    (if (i32.eq (local.get $step) (i32.const 1))
      (then
        (i32.store (i32.const 0) (i32.load offset=4 (i32.const 100)))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; step 2: read offset 16 (expect 77)
    (if (i32.eq (local.get $step) (i32.const 2))
      (then
        (i32.store (i32.const 0) (i32.load offset=16 (i32.const 100)))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; step 3: read offset 20 (expect 55)
    (if (i32.eq (local.get $step) (i32.const 3))
      (then
        (i32.store (i32.const 0) (i32.load offset=20 (i32.const 100)))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; step 4: read-modify-write at offset 16 (77+10 = 87)
    (if (i32.eq (local.get $step) (i32.const 4))
      (then
        (i32.store offset=16 (i32.const 100)
          (i32.add (i32.load offset=16 (i32.const 100)) (i32.const 10))
        )
        (i32.store (i32.const 0) (i32.load offset=16 (i32.const 100)))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; step 5: dynamic base address from args[4..8], store 123 at offset 16, read back
    (if (i32.eq (local.get $step) (i32.const 5))
      (then
        (local.set $base (i32.load (i32.add (local.get $args_ptr) (i32.const 4))))
        (i32.store offset=16 (local.get $base) (i32.const 123))
        (i32.store (i32.const 0) (i32.load offset=16 (local.get $base)))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )
  )
)
