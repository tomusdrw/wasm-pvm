(module
  (memory 1)
  ;; Table for call_indirect
  (table 2 funcref)
  (elem (i32.const 0) $get_delta $get_delta_plus_ten)
  (type $nullary (func (result i32)))

  (func $get_delta (result i32)
    (i32.const 3)
  )

  (func $get_delta_plus_ten (result i32)
    (i32.const 13)
  )

  ;; This function mimics the interpreter's nextSteps pattern:
  ;; - Has 17 locals (like nextSteps)
  ;; - Calls a function via call_indirect
  ;; - Uses the result to update a memory field
  ;; - Loops multiple times
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (local $step i32)
    (local $obj_ptr i32)       ;; pointer to our "struct" in memory
    (local $pc i32)            ;; "PC" field (what we track)
    (local $counter i32)       ;; loop counter
    (local $temp1 i32)
    (local $temp2 i32)
    (local $temp3 i32)
    (local $temp4 i32)
    (local $temp5 i32)
    (local $temp6 i32)
    (local $temp7 i32)
    (local $temp8 i32)
    (local $temp9 i32)
    (local $temp10 i32)
    (local $temp11 i32)
    (local $temp12 i32)
    (local $result i32)

    (local.set $step (i32.load (local.get $args_ptr)))

    ;; Set up a "struct" at memory address 200
    ;; offset 0: some_field = 100
    ;; offset 4: pc = 0
    ;; offset 8: gas = 1000
    (local.set $obj_ptr (i32.const 200))
    (i32.store offset=0 (local.get $obj_ptr) (i32.const 100))
    (i32.store offset=4 (local.get $obj_ptr) (i32.const 0))    ;; pc = 0
    (i32.store offset=8 (local.get $obj_ptr) (i32.const 1000)) ;; gas = 1000

    ;; Initialize temp vars to make sure they don't interfere
    (local.set $temp1 (i32.const 1))
    (local.set $temp2 (i32.const 2))
    (local.set $temp3 (i32.const 3))
    (local.set $temp4 (i32.const 4))
    (local.set $temp5 (i32.const 5))
    (local.set $temp6 (i32.const 6))
    (local.set $temp7 (i32.const 7))
    (local.set $temp8 (i32.const 8))
    (local.set $temp9 (i32.const 9))
    (local.set $temp10 (i32.const 10))
    (local.set $temp11 (i32.const 11))
    (local.set $temp12 (i32.const 12))

    ;; step 0: call get_delta via call_indirect, update pc field
    ;; pc should be 0 + 1 + 3 = 4
    (if (i32.eqz (local.get $step))
      (then
        (local.set $pc (i32.load offset=4 (local.get $obj_ptr)))
        (local.set $result (call_indirect (type $nullary) (i32.const 0)))
        ;; pc = old_pc + 1 + result
        (i32.store offset=4 (local.get $obj_ptr)
          (i32.add (local.get $pc) (i32.add (i32.const 1) (local.get $result)))
        )
        (i32.store (i32.const 0) (i32.load offset=4 (local.get $obj_ptr)))
        (i32.const 0)  ;; result_ptr
        (i32.const 4)  ;; result_len
        (return)
      )
    )

    ;; step 1: loop 5 times, each time call_indirect and advance pc
    ;; pc should be 5 * (1 + 3) = 20
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
            ;; Also decrement gas
            (i32.store offset=8 (local.get $obj_ptr)
              (i32.sub (i32.load offset=8 (local.get $obj_ptr)) (i32.const 1))
            )
            (local.set $counter (i32.add (local.get $counter) (i32.const 1)))
            (br $loop)
          )
        )
        ;; Return pc * 1000 + gas
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

    ;; step 2: verify all temp locals survived the calls
    (if (i32.eq (local.get $step) (i32.const 2))
      (then
        ;; Do a call_indirect
        (drop (call_indirect (type $nullary) (i32.const 0)))
        ;; Check temps are still correct: sum of temps should be 1+2+...+12 = 78
        (i32.store (i32.const 0)
          (i32.add (local.get $temp1)
          (i32.add (local.get $temp2)
          (i32.add (local.get $temp3)
          (i32.add (local.get $temp4)
          (i32.add (local.get $temp5)
          (i32.add (local.get $temp6)
          (i32.add (local.get $temp7)
          (i32.add (local.get $temp8)
          (i32.add (local.get $temp9)
          (i32.add (local.get $temp10)
          (i32.add (local.get $temp11)
                   (local.get $temp12))))))))))))
        )
        (i32.const 0)  ;; result_ptr
        (i32.const 4)  ;; result_len
        (return)
      )
    )

    ;; step 3: use dynamic table index from args
    (if (i32.eq (local.get $step) (i32.const 3))
      (then
        (local.set $counter (i32.const 0))
        (block $break
          (loop $loop
            (br_if $break (i32.ge_u (local.get $counter) (i32.const 3)))
            (local.set $pc (i32.load offset=4 (local.get $obj_ptr)))
            (local.set $result
              (call_indirect (type $nullary)
                (i32.load (i32.add (local.get $args_ptr) (i32.const 4)))
              )
            )
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

    (i32.const 0)  ;; default result_ptr
    (i32.const 0)  ;; default result_len
  )
)
