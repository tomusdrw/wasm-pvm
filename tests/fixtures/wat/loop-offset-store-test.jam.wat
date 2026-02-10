(module
  (memory 1)
  (table 1 funcref)
  (elem (i32.const 0) $get_three)
  (type $nullary (func (result i32)))

  (func $get_three (result i32)
    (i32.const 3)
  )

  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (local $step i32)
    (local $obj_ptr i32)
    (local $pc i32)
    (local $counter i32)
    (local $result i32)

    (local.set $step (i32.load (local.get $args_ptr)))

    ;; Set up struct at address 200
    (local.set $obj_ptr (i32.const 200))
    (i32.store offset=0 (local.get $obj_ptr) (i32.const 100))
    (i32.store offset=4 (local.get $obj_ptr) (i32.const 0))    ;; pc = 0
    (i32.store offset=8 (local.get $obj_ptr) (i32.const 1000)) ;; gas = 1000

    ;; step 0: simple loop, no call_indirect, advance pc by 4 each time
    ;; expected: pc = 20, gas = 995, result = 20995
    (if (i32.eqz (local.get $step))
      (then
        (local.set $counter (i32.const 0))
        (block $break
          (loop $loop
            (br_if $break (i32.ge_u (local.get $counter) (i32.const 5)))
            (local.set $pc (i32.load offset=4 (local.get $obj_ptr)))
            (i32.store offset=4 (local.get $obj_ptr)
              (i32.add (local.get $pc) (i32.const 4))
            )
            (i32.store offset=8 (local.get $obj_ptr)
              (i32.sub (i32.load offset=8 (local.get $obj_ptr)) (i32.const 1))
            )
            (local.set $counter (i32.add (local.get $counter) (i32.const 1)))
            (br $loop)
          )
        )
        (i32.store (i32.const 0)
          (i32.add
            (i32.mul (i32.load offset=4 (local.get $obj_ptr)) (i32.const 1000))
            (i32.load offset=8 (local.get $obj_ptr))
          )
        )
        (i32.const 0)  ;; result_ptr
        (i32.const 4)  ;; result_len
        (return)
      )
    )

    ;; step 1: loop with call_indirect, advance pc by (1 + result)
    ;; expected: pc = 20, gas = 995, result = 20995
    (if (i32.eq (local.get $step) (i32.const 1))
      (then
        (local.set $counter (i32.const 0))
        (block $break
          (loop $loop
            (br_if $break (i32.ge_u (local.get $counter) (i32.const 5)))
            (local.set $pc (i32.load offset=4 (local.get $obj_ptr)))
            (local.set $result (call_indirect (type $nullary) (i32.const 0)))
            (i32.store offset=4 (local.get $obj_ptr)
              (i32.add (local.get $pc) (i32.add (i32.const 1) (local.get $result)))
            )
            (i32.store offset=8 (local.get $obj_ptr)
              (i32.sub (i32.load offset=8 (local.get $obj_ptr)) (i32.const 1))
            )
            (local.set $counter (i32.add (local.get $counter) (i32.const 1)))
            (br $loop)
          )
        )
        (i32.store (i32.const 0)
          (i32.add
            (i32.mul (i32.load offset=4 (local.get $obj_ptr)) (i32.const 1000))
            (i32.load offset=8 (local.get $obj_ptr))
          )
        )
        (i32.const 0)  ;; result_ptr
        (i32.const 4)  ;; result_len
        (return)
      )
    )

    ;; step 2: single call + store with offset, no loop
    ;; expected: pc = 4
    (if (i32.eq (local.get $step) (i32.const 2))
      (then
        (local.set $pc (i32.load offset=4 (local.get $obj_ptr)))
        (local.set $result (call_indirect (type $nullary) (i32.const 0)))
        (i32.store offset=4 (local.get $obj_ptr)
          (i32.add (local.get $pc) (i32.add (i32.const 1) (local.get $result)))
        )
        (i32.store (i32.const 0) (i32.load offset=4 (local.get $obj_ptr)))
        (i32.const 0)  ;; result_ptr
        (i32.const 4)  ;; result_len
        (return)
      )
    )

    ;; step 3: two calls in sequence + store with offset
    ;; expected: pc = 8
    (if (i32.eq (local.get $step) (i32.const 3))
      (then
        (local.set $pc (i32.load offset=4 (local.get $obj_ptr)))
        (local.set $result (call_indirect (type $nullary) (i32.const 0)))
        (i32.store offset=4 (local.get $obj_ptr)
          (i32.add (local.get $pc) (i32.add (i32.const 1) (local.get $result)))
        )
        (local.set $pc (i32.load offset=4 (local.get $obj_ptr)))
        (local.set $result (call_indirect (type $nullary) (i32.const 0)))
        (i32.store offset=4 (local.get $obj_ptr)
          (i32.add (local.get $pc) (i32.add (i32.const 1) (local.get $result)))
        )
        (i32.store (i32.const 0) (i32.load offset=4 (local.get $obj_ptr)))
        (i32.const 0)  ;; result_ptr
        (i32.const 4)  ;; result_len
        (return)
      )
    )

    ;; step 4: just return gas to verify offset=8 store works in loop
    ;; expected: 995
    (if (i32.eq (local.get $step) (i32.const 4))
      (then
        (local.set $counter (i32.const 0))
        (block $break
          (loop $loop
            (br_if $break (i32.ge_u (local.get $counter) (i32.const 5)))
            (i32.store offset=8 (local.get $obj_ptr)
              (i32.sub (i32.load offset=8 (local.get $obj_ptr)) (i32.const 1))
            )
            (local.set $counter (i32.add (local.get $counter) (i32.const 1)))
            (br $loop)
          )
        )
        (i32.store (i32.const 0) (i32.load offset=8 (local.get $obj_ptr)))
        (i32.const 0)  ;; result_ptr
        (i32.const 4)  ;; result_len
        (return)
      )
    )

    ;; step 5: loop advance pc only (no gas), with call_indirect
    ;; expected: pc = 20
    (if (i32.eq (local.get $step) (i32.const 5))
      (then
        (local.set $counter (i32.const 0))
        (block $break
          (loop $loop
            (br_if $break (i32.ge_u (local.get $counter) (i32.const 5)))
            (local.set $pc (i32.load offset=4 (local.get $obj_ptr)))
            (local.set $result (call_indirect (type $nullary) (i32.const 0)))
            (i32.store offset=4 (local.get $obj_ptr)
              (i32.add (local.get $pc) (i32.add (i32.const 1) (local.get $result)))
            )
            (local.set $counter (i32.add (local.get $counter) (i32.const 1)))
            (br $loop)
          )
        )
        (i32.store (i32.const 0) (i32.load offset=4 (local.get $obj_ptr)))
        (i32.const 0)  ;; result_ptr
        (i32.const 4)  ;; result_len
        (return)
      )
    )

    ;; step 6: loop advance pc only, NO call_indirect, just constant 3
    ;; expected: pc = 20
    (if (i32.eq (local.get $step) (i32.const 6))
      (then
        (local.set $counter (i32.const 0))
        (block $break
          (loop $loop
            (br_if $break (i32.ge_u (local.get $counter) (i32.const 5)))
            (local.set $pc (i32.load offset=4 (local.get $obj_ptr)))
            (i32.store offset=4 (local.get $obj_ptr)
              (i32.add (local.get $pc) (i32.const 4))
            )
            (local.set $counter (i32.add (local.get $counter) (i32.const 1)))
            (br $loop)
          )
        )
        (i32.store (i32.const 0) (i32.load offset=4 (local.get $obj_ptr)))
        (i32.const 0)  ;; result_ptr
        (i32.const 4)  ;; result_len
        (return)
      )
    )

    (i32.const 0)  ;; default result_ptr
    (i32.const 0)  ;; default result_len
  )
)
