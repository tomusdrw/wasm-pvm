(module
  (memory 1)

  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))

  ;; Test function with many locals (triggers spilling)
  ;; Computes: a + b + c + d + e + f where each is set from input + index
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $a i32)  ;; local 2 -> r11
    (local $b i32)  ;; local 3 -> r12
    (local $c i32)  ;; local 4 -> spilled
    (local $d i32)  ;; local 5 -> spilled
    (local $e i32)  ;; local 6 -> spilled
    (local $f i32)  ;; local 7 -> spilled
    (local $result i32)

    ;; Read base value from input
    (local.set $a (i32.add (i32.load (local.get $args_ptr)) (i32.const 1)))
    (local.set $b (i32.add (i32.load (local.get $args_ptr)) (i32.const 2)))
    (local.set $c (i32.add (i32.load (local.get $args_ptr)) (i32.const 3)))
    (local.set $d (i32.add (i32.load (local.get $args_ptr)) (i32.const 4)))
    (local.set $e (i32.add (i32.load (local.get $args_ptr)) (i32.const 5)))
    (local.set $f (i32.add (i32.load (local.get $args_ptr)) (i32.const 6)))

    ;; Sum all locals: if input is 10, result = 11+12+13+14+15+16 = 81
    (local.set $result 
      (i32.add
        (i32.add
          (i32.add (local.get $a) (local.get $b))
          (i32.add (local.get $c) (local.get $d))
        )
        (i32.add (local.get $e) (local.get $f))
      )
    )

    ;; Store result
    (i32.store (i32.const 0x30100) (local.get $result))

    ;; Set return value
    (global.set $result_ptr (i32.const 0x30100))
    (global.set $result_len (i32.const 4))
  )
)
