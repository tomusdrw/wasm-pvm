;; Runtime verification for the slot-based phi-copy parallel-move resolver
;; (issue #219). The loop back-edge produces a 6-element permutation cycle
;; on the phi nodes — past the 5-temp-register fast-path threshold, so the
;; slot-based fallback handles cycle resolution.
;;
;; Initial values: $a..$f = 1..6
;; Each iteration rotates left: a' = b, b' = c, c' = d, d' = e, e' = f, f' = a
;; After N iterations, value at position k is ((initial_k + N) mod 6).
;; The output is 6 little-endian i64s starting at args_ptr.
(module
  (memory 1)
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $a i64) (local $b i64) (local $c i64)
    (local $d i64) (local $e i64) (local $f i64)
    (local $i i32) (local $n i32)
    (local $tmp_a i64) (local $tmp_b i64) (local $tmp_c i64)
    (local $tmp_d i64) (local $tmp_e i64) (local $tmp_f i64)

    ;; Read iteration count from args (4-byte i32).
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

        ;; Snapshot before rotation so each new value reads the original.
        (local.set $tmp_a (local.get $b))
        (local.set $tmp_b (local.get $c))
        (local.set $tmp_c (local.get $d))
        (local.set $tmp_d (local.get $e))
        (local.set $tmp_e (local.get $f))
        (local.set $tmp_f (local.get $a))

        (local.set $a (local.get $tmp_a))
        (local.set $b (local.get $tmp_b))
        (local.set $c (local.get $tmp_c))
        (local.set $d (local.get $tmp_d))
        (local.set $e (local.get $tmp_e))
        (local.set $f (local.get $tmp_f))

        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $L)
      )
    )

    ;; Write final state to WASM offset 0 (the result region).
    (i64.store        (i32.const 0)  (local.get $a))
    (i64.store offset=8  (i32.const 0) (local.get $b))
    (i64.store offset=16 (i32.const 0) (local.get $c))
    (i64.store offset=24 (i32.const 0) (local.get $d))
    (i64.store offset=32 (i32.const 0) (local.get $e))
    (i64.store offset=40 (i32.const 0) (local.get $f))

    ;; Return packed (ptr=0, len=48).
    (i64.const 206158430208)
  )
)
