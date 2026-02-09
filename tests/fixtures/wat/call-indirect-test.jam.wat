(module
  (memory 1)
  ;; Function table with 4 entries
  (table 4 funcref)
  (elem (i32.const 0) $add_ten $add_twenty $multiply_two $set_99)

  ;; Type signature for our table functions: (i32) -> i32
  (type $unary (func (param i32) (result i32)))

  (func $add_ten (param $x i32) (result i32)
    (i32.add (local.get $x) (i32.const 10))
  )

  (func $add_twenty (param $x i32) (result i32)
    (i32.add (local.get $x) (i32.const 20))
  )

  (func $multiply_two (param $x i32) (result i32)
    (i32.mul (local.get $x) (i32.const 2))
  )

  (func $set_99 (param $x i32) (result i32)
    (i32.const 99)
  )

  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (local $step i32)
    (local $val i32)
    (local $func_idx i32)

    (local.set $step (i32.load (local.get $args_ptr)))

    ;; step 0: call_indirect table[0] ($add_ten) with value 5 → expect 15
    (if (i32.eqz (local.get $step))
      (then
        (i32.store (i32.const 0)
          (call_indirect (type $unary) (i32.const 5) (i32.const 0))
        )
        (i32.const 0)  ;; result_ptr
        (i32.const 4)  ;; result_len
        (return)
      )
    )

    ;; step 1: call_indirect table[1] ($add_twenty) with value 5 → expect 25
    (if (i32.eq (local.get $step) (i32.const 1))
      (then
        (i32.store (i32.const 0)
          (call_indirect (type $unary) (i32.const 5) (i32.const 1))
        )
        (i32.const 0)  ;; result_ptr
        (i32.const 4)  ;; result_len
        (return)
      )
    )

    ;; step 2: call_indirect table[2] ($multiply_two) with value 7 → expect 14
    (if (i32.eq (local.get $step) (i32.const 2))
      (then
        (i32.store (i32.const 0)
          (call_indirect (type $unary) (i32.const 7) (i32.const 2))
        )
        (i32.const 0)  ;; result_ptr
        (i32.const 4)  ;; result_len
        (return)
      )
    )

    ;; step 3: call_indirect table[3] ($set_99) with value 0 → expect 99
    (if (i32.eq (local.get $step) (i32.const 3))
      (then
        (i32.store (i32.const 0)
          (call_indirect (type $unary) (i32.const 0) (i32.const 3))
        )
        (i32.const 0)  ;; result_ptr
        (i32.const 4)  ;; result_len
        (return)
      )
    )

    ;; step 4: chain two calls: add_ten(5) then multiply_two(result) → expect 30
    (if (i32.eq (local.get $step) (i32.const 4))
      (then
        (local.set $val
          (call_indirect (type $unary) (i32.const 5) (i32.const 0))
        )
        (i32.store (i32.const 0)
          (call_indirect (type $unary) (local.get $val) (i32.const 2))
        )
        (i32.const 0)  ;; result_ptr
        (i32.const 4)  ;; result_len
        (return)
      )
    )

    ;; step 5: loop calling call_indirect 5 times with add_ten, starting from 0
    ;; 0 → 10 → 20 → 30 → 40 → 50
    (if (i32.eq (local.get $step) (i32.const 5))
      (then
        (local.set $val (i32.const 0))
        (local.set $func_idx (i32.const 0))
        (block $break
          (loop $loop
            (br_if $break (i32.ge_u (local.get $func_idx) (i32.const 5)))
            (local.set $val
              (call_indirect (type $unary) (local.get $val) (i32.const 0))
            )
            (local.set $func_idx (i32.add (local.get $func_idx) (i32.const 1)))
            (br $loop)
          )
        )
        (i32.store (i32.const 0) (local.get $val))
        (i32.const 0)  ;; result_ptr
        (i32.const 4)  ;; result_len
        (return)
      )
    )

    ;; step 6: call with dynamic table index from args[4..8]
    (if (i32.eq (local.get $step) (i32.const 6))
      (then
        (local.set $func_idx
          (i32.load (i32.add (local.get $args_ptr) (i32.const 4)))
        )
        (i32.store (i32.const 0)
          (call_indirect (type $unary) (i32.const 100) (local.get $func_idx))
        )
        (i32.const 0)  ;; result_ptr
        (i32.const 4)  ;; result_len
        (return)
      )
    )

    (i32.const 0)  ;; default result_ptr
    (i32.const 0)  ;; default result_len
  )
)
