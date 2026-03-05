(module
  (memory 1)

  ;; Helper: multiply two numbers (forces non-leaf in caller)
  (func $multiply (param $a i32) (param $b i32) (result i32)
    (i32.mul (local.get $a) (local.get $b))
  )

  ;; Loop that calls a function each iteration.
  ;; Tests register allocation state invalidation after calls.
  ;;
  ;; Computes: sum = 0; for i in 1..=n: sum += multiply(i, i)
  ;; For n=5: sum = 1 + 4 + 9 + 16 + 25 = 55
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $n i32)
    (local $i i32)
    (local $sum i32)
    (local $product i32)

    ;; Read n from args
    (local.set $n (i32.load (local.get $args_ptr)))

    ;; i = 1, sum = 0
    (local.set $i (i32.const 1))
    (local.set $sum (i32.const 0))

    ;; Loop
    (block $break
      (loop $continue
        ;; if i > n, break
        (br_if $break
          (i32.gt_u (local.get $i) (local.get $n))
        )

        ;; product = multiply(i, i) -- this call clobbers allocated regs
        (local.set $product
          (call $multiply (local.get $i) (local.get $i))
        )

        ;; sum += product
        (local.set $sum
          (i32.add (local.get $sum) (local.get $product))
        )

        ;; i++
        (local.set $i
          (i32.add (local.get $i) (i32.const 1))
        )

        (br $continue)
      )
    )

    ;; Write result
    (i32.store (i32.const 0) (local.get $sum))

    (i64.const 17179869184)  ;; ptr=0, len=4
  )
)
