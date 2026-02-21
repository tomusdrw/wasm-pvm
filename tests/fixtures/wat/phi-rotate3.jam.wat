(module
  (memory 1)

  ;; Phi cycle test: three-variable rotation in a loop.
  ;; Each iteration rotates: (a, b, c) = (b, c, a)
  ;; This creates a 3-way phi cycle at the loop header:
  ;;   a_phi = phi [a_init, entry], [b_prev, loop]
  ;;   b_phi = phi [b_init, entry], [c_prev, loop]
  ;;   c_phi = phi [c_init, entry], [a_prev, loop]
  ;;
  ;; Input: four i32 values: a, b, c, n (iteration count)
  ;; After n rotations: if n%3==0 -> (a,b,c), n%3==1 -> (b,c,a), n%3==2 -> (c,a,b)
  ;; Output: single i32 (value of a after n rotations)
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (local $a i32)
    (local $b i32)
    (local $c i32)
    (local $n i32)
    (local $i i32)

    ;; Read inputs
    (local.set $a (i32.load (local.get $args_ptr)))
    (local.set $b (i32.load (i32.add (local.get $args_ptr) (i32.const 4))))
    (local.set $c (i32.load (i32.add (local.get $args_ptr) (i32.const 8))))
    (local.set $n (i32.load (i32.add (local.get $args_ptr) (i32.const 12))))
    (local.set $i (i32.const 0))

    (block $break
      (loop $continue
        (br_if $break (i32.ge_u (local.get $i) (local.get $n)))

        ;; Rotate: (a, b, c) = (b, c, a)
        ;; Push all three values on stack, then set in reverse order
        (local.get $b)        ;; push old b (will become new a)
        (local.get $c)        ;; push old c (will become new b)
        (local.get $a)        ;; push old a (will become new c)
        (local.set $c)        ;; c = old a
        (local.set $b)        ;; b = old c
        (local.set $a)        ;; a = old b

        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $continue)
      )
    )

    ;; Store result (a after n rotations)
    (i32.store (i32.const 0) (local.get $a))
    (i32.const 0)  ;; result_ptr
    (i32.const 4)  ;; result_len
  )
)
