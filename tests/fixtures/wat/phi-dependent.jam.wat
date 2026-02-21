(module
  (memory 1)

  ;; Phi cycle test: mutually dependent variables in a loop.
  ;; Each iteration: new_a = a + b, new_b = a * 2
  ;; Both new values depend on old a, creating an interdependency
  ;; that requires correct phi cycle handling.
  ;;
  ;; Input: a (i32), b (i32), n (i32 iteration count)
  ;; Output: value of a after n iterations
  ;;
  ;; Manual trace for a=1, b=2:
  ;;   iter 0: a=1, b=2
  ;;   iter 1: a=1+2=3, b=1*2=2
  ;;   iter 2: a=3+2=5, b=3*2=6
  ;;   iter 3: a=5+6=11, b=5*2=10
  ;;   iter 4: a=11+10=21, b=11*2=22
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (local $a i32)
    (local $b i32)
    (local $n i32)
    (local $i i32)

    ;; Read inputs
    (local.set $a (i32.load (local.get $args_ptr)))
    (local.set $b (i32.load (i32.add (local.get $args_ptr) (i32.const 4))))
    (local.set $n (i32.load (i32.add (local.get $args_ptr) (i32.const 8))))
    (local.set $i (i32.const 0))

    (block $break
      (loop $continue
        (br_if $break (i32.ge_u (local.get $i) (local.get $n)))

        ;; Compute new values from old a and b simultaneously:
        ;; new_a = old_a + old_b
        ;; new_b = old_a * 2
        ;; Push both new values before setting, to ensure old values are read first
        (i32.add (local.get $a) (local.get $b))    ;; push new_a = a + b
        (i32.mul (local.get $a) (i32.const 2))     ;; push new_b = a * 2
        (local.set $b)                               ;; b = new_b
        (local.set $a)                               ;; a = new_a

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
