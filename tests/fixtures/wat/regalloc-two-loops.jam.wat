(module
  (memory 1)

  ;; Targeted register-allocation stress fixture:
  ;; - two sequential loops,
  ;; - each loop carries multiple long-lived values,
  ;; - values from loop #1 are dead before loop #2 starts.
  ;; This is intended to exercise non-overlapping live ranges.
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (local $n i32)

    ;; Loop 1 state
    (local $i i32)
    (local $a i32)
    (local $b i32)
    (local $c i32)
    (local $d i32)

    ;; Loop 2 state
    (local $j i32)
    (local $e i32)
    (local $f i32)
    (local $g i32)
    (local $h i32)

    ;; n = first u32 argument
    (local.set $n (i32.load (local.get $args_ptr)))

    ;; Loop 1 initialization
    (local.set $i (i32.const 0))
    (local.set $a (i32.const 1))
    (local.set $b (i32.const 3))
    (local.set $c (i32.const 5))
    (local.set $d (i32.const 7))

    (block $loop1_break
      (loop $loop1
        (br_if $loop1_break (i32.ge_u (local.get $i) (local.get $n)))

        (local.set $a
          (i32.add
            (i32.add (local.get $a) (local.get $b))
            (local.get $i)))
        (local.set $b
          (i32.add
            (i32.add (local.get $b) (local.get $c))
            (local.get $i)))
        (local.set $c
          (i32.add
            (i32.add (local.get $c) (local.get $d))
            (local.get $i)))
        (local.set $d
          (i32.add
            (i32.add (local.get $d) (local.get $a))
            (local.get $i)))

        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $loop1)
      )
    )

    ;; Loop 2 initialization (depends on loop 1 result to keep both live regions meaningful)
    (local.set $j (i32.const 0))
    (local.set $e (i32.add (local.get $a) (i32.const 11)))
    (local.set $f (i32.add (local.get $b) (i32.const 13)))
    (local.set $g (i32.add (local.get $c) (i32.const 17)))
    (local.set $h (i32.add (local.get $d) (i32.const 19)))

    (block $loop2_break
      (loop $loop2
        (br_if $loop2_break (i32.ge_u (local.get $j) (local.get $n)))

        (local.set $e
          (i32.add
            (i32.add (local.get $e) (local.get $f))
            (local.get $j)))
        (local.set $f
          (i32.add
            (i32.add (local.get $f) (local.get $g))
            (local.get $j)))
        (local.set $g
          (i32.add
            (i32.add (local.get $g) (local.get $h))
            (local.get $j)))
        (local.set $h
          (i32.add
            (i32.add (local.get $h) (local.get $e))
            (local.get $j)))

        (local.set $j (i32.add (local.get $j) (i32.const 1)))
        (br $loop2)
      )
    )

    ;; Return low 32 bits of combined state at memory[0]
    (i32.store
      (i32.const 0)
      (i32.add
        (i32.add (local.get $e) (local.get $f))
        (i32.add (local.get $g) (local.get $h))))

    (i32.const 0)
    (i32.const 4)
  )
)
