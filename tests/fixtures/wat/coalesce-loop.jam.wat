(module
  (memory 1)
  ;; Simple accumulator loop: sum(1..n) where n is the first arg.
  ;; Creates 2 phi nodes at the loop header (i, sum), each with a back-edge
  ;; incoming value. Target for loop phi early interval expiration.
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $i i32)
    (local $sum i32)
    ;; Load n from args
    (local.set $i (i32.load (local.get $args_ptr)))
    (local.set $sum (i32.const 0))
    (block $break
      (loop $loop
        ;; sum += i
        (local.set $sum (i32.add (local.get $sum) (local.get $i)))
        ;; i -= 1
        (local.set $i (i32.sub (local.get $i) (i32.const 1)))
        ;; continue if i > 0
        (br_if $loop (i32.gt_s (local.get $i) (i32.const 0)))
      )
    )
    ;; Return sum (packed i64: ptr=0, len=4)
    (i32.store (i32.const 0) (local.get $sum))
    (i64.const 17179869184)
  )
)
