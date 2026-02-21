(module
  (memory 1)

  ;; Phi test: multi-predecessor merge with different values from different branches.
  ;; Depending on a selector, assigns different values to two variables,
  ;; then runs them through a swap loop to exercise the phi cycle.
  ;;
  ;; Input: selector (i32), swap_count (i32)
  ;; If selector == 0: a=10, b=20
  ;; If selector == 1: a=100, b=200
  ;; Otherwise:        a=1000, b=2000
  ;; Then swap a,b swap_count times.
  ;; Output: value of a after swaps
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (local $sel i32)
    (local $swaps i32)
    (local $a i32)
    (local $b i32)
    (local $i i32)

    ;; Read inputs
    (local.set $sel (i32.load (local.get $args_ptr)))
    (local.set $swaps (i32.load (i32.add (local.get $args_ptr) (i32.const 4))))

    ;; Branch based on selector - each branch sets different (a, b) values
    ;; This creates a merge point phi with 3 predecessors
    (if (i32.eq (local.get $sel) (i32.const 0))
      (then
        (local.set $a (i32.const 10))
        (local.set $b (i32.const 20))
      )
      (else
        (if (i32.eq (local.get $sel) (i32.const 1))
          (then
            (local.set $a (i32.const 100))
            (local.set $b (i32.const 200))
          )
          (else
            (local.set $a (i32.const 1000))
            (local.set $b (i32.const 2000))
          )
        )
      )
    )

    ;; Now swap a,b in a loop (phi cycle)
    (local.set $i (i32.const 0))
    (block $break
      (loop $continue
        (br_if $break (i32.ge_u (local.get $i) (local.get $swaps)))

        (local.get $b)
        (local.get $a)
        (local.set $b)
        (local.set $a)

        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $continue)
      )
    )

    ;; Store result
    (i32.store (i32.const 0) (local.get $a))
    (i32.const 0)  ;; result_ptr
    (i32.const 4)  ;; result_len
  )
)
