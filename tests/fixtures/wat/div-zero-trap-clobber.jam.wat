(module
  (memory 1)

  ;; Regression test for issue #256.
  ;;
  ;; In a do-while loop the carried `value` phi is used TWICE per
  ;; iteration: once for `value % 10` (sum-of-digits) and again, after
  ;; that rem_u's div-by-zero trap-bypass label, for `value / 10`. The
  ;; second read goes through `load_operand`. Before the fix, the
  ;; trap-bypass `define_label` cleared `alloc_reg_slot`, so the
  ;; second read fell back to `LoadIndU64` from the phi's stack slot;
  ;; with lazy-spill on, that slot's back-edge update was elided and
  ;; the loop read stale data.
  ;;
  ;; Mirrors the AssemblyScript `utoa_dec_simple` shape that triggered
  ;; the aslan-ecalli all-opts OOM. The do-while body forms a single
  ;; LLVM block (phi at top, conditional back-branch at bottom), so
  ;; LLVM keeps the phi value in a register across the trap-bypasses
  ;; instead of re-spilling at a loop-header block boundary. Extra
  ;; live phis (`ext_a..d`) raise register pressure to push the value
  ;; phi into a callee-saved register and exercise lazy-spill.
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $value i32)
    (local $sum i32)
    (local $ext_a i32)
    (local $ext_b i32)
    (local $ext_c i32)
    (local $ext_d i32)

    (local.set $value (i32.load (local.get $args_ptr)))
    (local.set $sum (i32.const 0))
    (local.set $ext_a (i32.load offset=4 (local.get $args_ptr)))
    (local.set $ext_b (i32.load offset=8 (local.get $args_ptr)))
    (local.set $ext_c (i32.load offset=12 (local.get $args_ptr)))
    (local.set $ext_d (i32.load offset=16 (local.get $args_ptr)))

    ;; If the input is < 10 we still want one digit; pad sum so the
    ;; caller's expected output is the digit sum either way. The
    ;; interesting case is the do-while below.
    (if (i32.ge_u (local.get $value) (i32.const 10))
      (then
        (loop $continue
          ;; Rotate extras through arithmetic on every iteration and
          ;; park them to memory so they stay live across the body but
          ;; do not contaminate the result.
          (local.set $ext_a (i32.add (local.get $ext_a) (local.get $value)))
          (local.set $ext_b (i32.xor (local.get $ext_b) (local.get $ext_a)))
          (local.set $ext_c (i32.add (local.get $ext_c) (local.get $ext_b)))
          (local.set $ext_d (i32.xor (local.get $ext_d) (local.get $ext_c)))
          (i32.store (i32.const 64) (local.get $ext_a))
          (i32.store (i32.const 68) (local.get $ext_b))
          (i32.store (i32.const 72) (local.get $ext_c))
          (i32.store (i32.const 76) (local.get $ext_d))

          ;; First use of $value: rem_u — trap-bypass #1.
          (local.set $sum
            (i32.add (local.get $sum)
              (i32.rem_u (local.get $value) (i32.const 10))))

          ;; Second use of $value: div_u — trap-bypass #2. With the
          ;; pre-fix bug this read falls back to slot[$value], which
          ;; was never written on the back-edge.
          (local.set $value (i32.div_u (local.get $value) (i32.const 10)))

          ;; do-while: keep looping while value >= 10.
          (br_if $continue (i32.ge_u (local.get $value) (i32.const 10)))
        )
      )
    )

    (i32.store (i32.const 0) (local.get $sum))
    (i64.const 17179869184)  ;; ptr=0, len=4
  )
)
