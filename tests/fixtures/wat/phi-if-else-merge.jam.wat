;; Runtime verification for the slot-based phi-copy parallel-move resolver
;; (issue #219). The if/else produces 8 phi nodes at the merge — past the
;; 5-temp-register threshold. The "then" arm uses runtime-derived values
;; (so LLVM cannot fold the merge); the "else" arm uses small constants.
;; This exercises the constant-copy path of the resolver alongside the
;; slot-based parallel move.
;;
;; Input: 4-byte i32 selector. selector == 0 → "else" arm (constants).
;;        Otherwise → "then" arm: variable_k = (selector + k).
;;
;; Output: 8 i64s starting at args_ptr.
(module
  (memory 1)
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $a i64) (local $b i64) (local $c i64) (local $d i64)
    (local $e i64) (local $f i64) (local $g i64) (local $h i64)
    (local $sel i32)

    (local.set $sel (i32.load (local.get $args_ptr)))

    (if (i32.ne (local.get $sel) (i32.const 0))
      (then
        ;; Runtime-derived values so LLVM doesn't constant-fold the merge.
        (local.set $a (i64.extend_i32_u (i32.add (local.get $sel) (i32.const 1))))
        (local.set $b (i64.extend_i32_u (i32.add (local.get $sel) (i32.const 2))))
        (local.set $c (i64.extend_i32_u (i32.add (local.get $sel) (i32.const 3))))
        (local.set $d (i64.extend_i32_u (i32.add (local.get $sel) (i32.const 4))))
        (local.set $e (i64.extend_i32_u (i32.add (local.get $sel) (i32.const 5))))
        (local.set $f (i64.extend_i32_u (i32.add (local.get $sel) (i32.const 6))))
        (local.set $g (i64.extend_i32_u (i32.add (local.get $sel) (i32.const 7))))
        (local.set $h (i64.extend_i32_u (i32.add (local.get $sel) (i32.const 8))))
      )
      (else
        (local.set $a (i64.const 101))
        (local.set $b (i64.const 102))
        (local.set $c (i64.const 103))
        (local.set $d (i64.const 104))
        (local.set $e (i64.const 105))
        (local.set $f (i64.const 106))
        (local.set $g (i64.const 107))
        (local.set $h (i64.const 108))
      )
    )

    (i64.store        (i32.const 0) (local.get $a))
    (i64.store offset=8  (i32.const 0) (local.get $b))
    (i64.store offset=16 (i32.const 0) (local.get $c))
    (i64.store offset=24 (i32.const 0) (local.get $d))
    (i64.store offset=32 (i32.const 0) (local.get $e))
    (i64.store offset=40 (i32.const 0) (local.get $f))
    (i64.store offset=48 (i32.const 0) (local.get $g))
    (i64.store offset=56 (i32.const 0) (local.get $h))

    ;; Return packed (ptr=0, len=64).
    (i64.const 274877906944)
  )
)
