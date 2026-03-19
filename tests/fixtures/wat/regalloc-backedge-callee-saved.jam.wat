(module
  (memory 1)

  ;; Helper with 3 params — forces max_call_args=3, so r12 is callee-saved
  ;; beyond call args and eligible for back-edge propagation.
  (func $combine (param $x i32) (param $y i32) (param $z i32) (result i32)
    (i32.add (i32.add (local.get $x) (local.get $y)) (local.get $z))
  )

  ;; Non-leaf loop that carries many values across iterations.
  ;; With max_call_args=3, r9-r11 are used for call args and r12 is the
  ;; only callee-saved register available for allocation beyond the call.
  ;; Phase 11 propagates r12's alloc state across the loop header back-edge.
  ;;
  ;; Computes: accumulate values through loop using call + local arithmetic.
  ;; a, b, c start at 1,2,3. Each iteration:
  ;;   temp = combine(a, b, c) -- call with 3 args
  ;;   a = b + i
  ;;   b = c + i
  ;;   c = temp + i
  ;; Return a + b + c after n iterations.
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $n i32)
    (local $i i32)
    (local $a i32)
    (local $b i32)
    (local $c i32)
    (local $temp i32)

    ;; Read n from args
    (local.set $n (i32.load (local.get $args_ptr)))

    ;; Initialize
    (local.set $i (i32.const 0))
    (local.set $a (i32.const 1))
    (local.set $b (i32.const 2))
    (local.set $c (i32.const 3))

    ;; Loop
    (block $break
      (loop $continue
        (br_if $break
          (i32.ge_u (local.get $i) (local.get $n))
        )

        ;; temp = combine(a, b, c)
        (local.set $temp
          (call $combine
            (local.get $a) (local.get $b) (local.get $c))
        )

        ;; Rotate: a=b+i, b=c+i, c=temp+i
        (local.set $a (i32.add (local.get $b) (local.get $i)))
        (local.set $b (i32.add (local.get $c) (local.get $i)))
        (local.set $c (i32.add (local.get $temp) (local.get $i)))

        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $continue)
      )
    )

    ;; Return a + b + c
    (i32.store
      (i32.const 0)
      (i32.add
        (i32.add (local.get $a) (local.get $b))
        (local.get $c)))

    (i64.const 17179869184)  ;; ptr=0, len=4
  )
)
