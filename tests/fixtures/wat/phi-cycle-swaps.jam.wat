;; Runtime verification for the slot-based phi-copy parallel-move resolver
;; (issue #219). The loop back-edge contains three independent swap-pairs,
;; so the dependency graph has **three disjoint 2-cycles** — the cycle
;; extractor has to find and emit each one separately rather than collapsing
;; them into a single chain.
;;
;; Initial values: ($a,$b,$c,$d,$e,$f) = (1,2,3,4,5,6)
;; Each iteration:
;;   swap a/b, swap c/d, swap e/f
;; After N iterations, parity decides: N odd → (2,1,4,3,6,5), N even → (1,2,3,4,5,6).
(module
  (memory 1)
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $a i64) (local $b i64) (local $c i64)
    (local $d i64) (local $e i64) (local $f i64)
    (local $i i32) (local $n i32)
    (local $tmp i64)

    (local.set $n (i32.load (local.get $args_ptr)))

    (local.set $a (i64.const 1))
    (local.set $b (i64.const 2))
    (local.set $c (i64.const 3))
    (local.set $d (i64.const 4))
    (local.set $e (i64.const 5))
    (local.set $f (i64.const 6))
    (local.set $i (i32.const 0))

    (block $break
      (loop $L
        (br_if $break (i32.ge_s (local.get $i) (local.get $n)))

        ;; Swap a/b.
        (local.set $tmp (local.get $a))
        (local.set $a (local.get $b))
        (local.set $b (local.get $tmp))

        ;; Swap c/d.
        (local.set $tmp (local.get $c))
        (local.set $c (local.get $d))
        (local.set $d (local.get $tmp))

        ;; Swap e/f.
        (local.set $tmp (local.get $e))
        (local.set $e (local.get $f))
        (local.set $f (local.get $tmp))

        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $L)
      )
    )

    (i64.store        (i32.const 0) (local.get $a))
    (i64.store offset=8  (i32.const 0) (local.get $b))
    (i64.store offset=16 (i32.const 0) (local.get $c))
    (i64.store offset=24 (i32.const 0) (local.get $d))
    (i64.store offset=32 (i32.const 0) (local.get $e))
    (i64.store offset=40 (i32.const 0) (local.get $f))

    (i64.const 206158430208)
  )
)
