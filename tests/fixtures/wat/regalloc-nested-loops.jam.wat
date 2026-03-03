(module
  (memory 1)

  ;; Nested loops with multiple loop-carried variables.
  ;; Exercises cross-block register state at loop headers with 2 predecessors.
  ;;
  ;; Computes: sum = 0; for i in 0..n: for j in 0..n: sum += (i * n + j)
  ;; For n=4: sum = 0+1+2+3 + 4+5+6+7 + 8+9+10+11 + 12+13+14+15 = 120
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (local $n i32)
    (local $i i32)
    (local $j i32)
    (local $sum i32)
    (local $temp i32)

    ;; Read n from args
    (local.set $n (i32.load (local.get $args_ptr)))

    ;; i = 0, sum = 0
    (local.set $i (i32.const 0))
    (local.set $sum (i32.const 0))

    ;; Outer loop
    (block $outer_break
      (loop $outer_continue
        ;; if i >= n, break
        (br_if $outer_break
          (i32.ge_u (local.get $i) (local.get $n))
        )

        ;; j = 0
        (local.set $j (i32.const 0))

        ;; Inner loop
        (block $inner_break
          (loop $inner_continue
            ;; if j >= n, break
            (br_if $inner_break
              (i32.ge_u (local.get $j) (local.get $n))
            )

            ;; temp = i * n + j
            (local.set $temp
              (i32.add
                (i32.mul (local.get $i) (local.get $n))
                (local.get $j)
              )
            )

            ;; sum += temp
            (local.set $sum
              (i32.add (local.get $sum) (local.get $temp))
            )

            ;; j++
            (local.set $j
              (i32.add (local.get $j) (i32.const 1))
            )

            (br $inner_continue)
          )
        )

        ;; i++
        (local.set $i
          (i32.add (local.get $i) (i32.const 1))
        )

        (br $outer_continue)
      )
    )

    ;; Write result
    (i32.store (i32.const 0) (local.get $sum))

    (i32.const 0)  ;; result_ptr
    (i32.const 4)  ;; result_len
  )
)
