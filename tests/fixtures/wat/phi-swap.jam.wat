(module
  (memory 1)

  ;; Phi cycle test: variable swap in a loop.
  ;; Each iteration swaps a and b: (a, b) = (b, a)
  ;; This creates a classic phi cycle at the loop header:
  ;;   a_phi = phi [a_init, entry], [b_prev, loop]
  ;;   b_phi = phi [b_init, entry], [a_prev, loop]
  ;;
  ;; Input: two i32 values (a, b) followed by an i32 iteration count n
  ;; After n swaps: if n is even, result = a; if n is odd, result = b.
  ;; Output: single i32 (value of a after n swaps)
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

        ;; Swap a and b using the stack (creates phi cycle in LLVM IR)
        (local.get $b)        ;; push old b
        (local.get $a)        ;; push old a
        (local.set $b)        ;; b = old a
        (local.set $a)        ;; a = old b

        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $continue)
      )
    )

    ;; Store result (a after n swaps)
    (i32.store (i32.const 0) (local.get $a))
    (i32.const 0)  ;; result_ptr
    (i32.const 4)  ;; result_len
  )
)
