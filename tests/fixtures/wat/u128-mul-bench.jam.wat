;; u128 multiplication microbenchmark — measures dynamic gas impact of the
;; `__multi3` body replacement.
;;
;; This fixture contains a *real* compiler-builtins-shaped `__multi3` body
;; (Knuth-style i64 partial products + carry tracking) so that comparing
;; with/without `libcall_recognition` is meaningful: the only difference
;; between runs is whether this body executes (recognition off) or whether
;; the synthesized 8-instruction `Mul64 + MulUpperUU + ...` body executes
;; (recognition on).
;;
;; A stub body would short-circuit to writing zeros — fast, but unfair as a
;; baseline because the "no-recognition" run wouldn't measure the actual
;; compiler-builtins work the optimization is trying to displace.
;;
;; Entry: main(args_ptr, args_len) -> i64
;;   args = [iter_count: u32 LE]   (recommended: 1000–100000)
;;   returns (final_acc_ptr=0x200, len=16) per unified entry ABI
;;
;; Workload: starting from acc=1, iterate N times: acc = (acc * 0x100000001).
;; Each iter triggers `__multi3` (modular u64×u64 product, taking low half).
;; Result is deterministic from N — verifies correctness via the unit test
;; rather than the benchmark, but the final value being non-trivial
;; confirms the loop ran.

(module
  (memory (export "memory") 1)

  ;; Canonical compiler-builtins __multi3 body shape (Knuth i64 partial
  ;; products + carry detection). Mirrors what rustc-emitted WASM for
  ;; wasm32-unknown-unknown contains, modulo register-allocation flavor.
  ;;
  ;; sret_ptr → 16-byte struct return [low: u64][high: u64]
  ;; Computes the low 128 bits of `(a_lo + 2^64 · a_hi) * (b_lo + 2^64 · b_hi)`.
  (func $__multi3 (param $sret i32) (param $a_lo i64) (param $a_hi i64) (param $b_lo i64) (param $b_hi i64)
    (local $a32lo i64) (local $a32hi i64) (local $b32lo i64) (local $b32hi i64)
    (local $ll i64) (local $hl i64) (local $lh i64) (local $hh i64)
    (local $mid i64) (local $low i64) (local $high i64)

    ;; Split a_lo, b_lo into 32-bit halves.
    local.get $a_lo
    i64.const 0xFFFFFFFF
    i64.and
    local.set $a32lo
    local.get $a_lo
    i64.const 32
    i64.shr_u
    local.set $a32hi
    local.get $b_lo
    i64.const 0xFFFFFFFF
    i64.and
    local.set $b32lo
    local.get $b_lo
    i64.const 32
    i64.shr_u
    local.set $b32hi

    ;; Four 32×32 partial products (each fits in 64 bits).
    local.get $a32lo
    local.get $b32lo
    i64.mul
    local.set $ll
    local.get $a32hi
    local.get $b32lo
    i64.mul
    local.set $hl
    local.get $a32lo
    local.get $b32hi
    i64.mul
    local.set $lh
    local.get $a32hi
    local.get $b32hi
    i64.mul
    local.set $hh

    ;; mid = hl + lh; low = ll + (mid << 32); track carry from low addition.
    local.get $hl
    local.get $lh
    i64.add
    local.set $mid

    local.get $mid
    i64.const 32
    i64.shl
    local.get $ll
    i64.add
    local.set $low

    ;; Store low half.
    local.get $sret
    local.get $low
    i64.store offset=0

    ;; high = hh + (mid >> 32)
    ;;        + (carry from mid = hl + lh overflow ? 1<<32 : 0)
    ;;        + (carry from low = ll + (mid<<32) ? 1 : 0)
    ;;        + a_hi * b_lo + b_hi * a_lo
    local.get $mid
    i64.const 32
    i64.shr_u
    local.get $hh
    i64.add
    ;; carry from mid: if mid < hl (since unsigned wrap), add 1 << 32
    local.get $mid
    local.get $hl
    i64.lt_u
    i64.extend_i32_u
    i64.const 32
    i64.shl
    i64.add
    ;; carry from low: if low < ll, add 1
    local.get $low
    local.get $ll
    i64.lt_u
    i64.extend_i32_u
    i64.add
    ;; i128 sign-correction: a_hi * b_lo + a_lo * b_hi (mod 2^64)
    local.get $a_hi
    local.get $b_lo
    i64.mul
    i64.add
    local.get $a_lo
    local.get $b_hi
    i64.mul
    i64.add
    local.set $high

    local.get $sret
    local.get $high
    i64.store offset=8
  )

  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $iter i32)
    (local $acc_lo i64)
    (local $acc_hi i64)

    ;; iter_count = *(args_ptr as *u32)
    local.get $args_ptr
    i32.load offset=0
    local.set $iter

    ;; acc = 1
    i64.const 1
    local.set $acc_lo
    i64.const 0
    local.set $acc_hi

    (block $done
      (loop $bench
        local.get $iter
        i32.eqz
        br_if $done

        ;; acc = acc * 0x100000001  (a chosen multiplier that exercises
        ;; both halves: low bit + bit 32 set, so mid term ≠ 0)
        i32.const 0x100              ;; sret buffer
        local.get $acc_lo
        local.get $acc_hi
        i64.const 0x100000001
        i64.const 0
        call $__multi3

        i32.const 0x100
        i64.load offset=0
        local.set $acc_lo
        i32.const 0x100
        i64.load offset=8
        local.set $acc_hi

        local.get $iter
        i32.const 1
        i32.sub
        local.set $iter
        br $bench
      )
    )

    ;; Write final acc at 0x200 (16 bytes).
    i32.const 0x200
    local.get $acc_lo
    i64.store offset=0
    i32.const 0x200
    local.get $acc_hi
    i64.store offset=8

    ;; Return (ptr=0x200, len=16).
    i64.const 0x1000000200
  )
)
