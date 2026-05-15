;; u128 division via __udivti3 — exercises libcall recognition's b_hi
;; specialization fast path AND the slow-path forward.
;;
;; The compiler's `libcall_recognition` pass detects `__udivti3` (and its
;; companion `specialized_div_rem` slow-path callee, found by scanning the
;; original __udivti3 body) and replaces __udivti3 with:
;;
;;   if (a_hi | b_hi) == 0:
;;       q   = DivU64(a_lo, b_lo)
;;       sret = (q, 0)
;;       return
;;   else:
;;       reserve 32-byte WASM stack frame
;;       call specialized_div_rem(frame, a_lo, a_hi, b_lo, b_hi)
;;       copy quotient (16 bytes) to caller sret
;;       restore stack pointer
;;       return
;;
;; This fixture uses a STUB specialized_div_rem that writes predictable
;; sentinel values to its 32-byte sret. That lets us prove the slow path
;; performs the stack-frame dance and quotient copy correctly without
;; depending on a real 128-bit division implementation.
;;
;; Entry: main(args_ptr, args_len) -> i64
;;   args = 32 bytes: [a_lo: u64 LE][a_hi: u64 LE][b_lo: u64 LE][b_hi: u64 LE]
;;   returns (out_ptr=0x100, out_len=16) per unified entry ABI
;;
;; Output (16 bytes at 0x100):
;;   bytes 0..8:  fast path → a_lo / b_lo; slow path → 0xDEADBEEF + 1
;;   bytes 8..16: fast path → 0;            slow path → 0xDEADBEEF + 2

(module
  (memory (export "memory") 1)
  (global $__stack_pointer (mut i32) (i32.const 65536))

  ;; Stub slow-path. Writes sentinel values so tests can verify the slow
  ;; path actually called us (vs the fast path silently doing the math).
  ;; The 32-byte layout matches what real compiler-builtins'
  ;; specialized_div_rem writes: [q_lo, q_hi, r_lo, r_hi].
  ;;
  ;; Sentinel = 0xDEADBEEF + offset/8 + 1. We use a constant base to
  ;; distinguish slow-path results from any fast-path quotient.
  (func $sdr (param $sret i32) (param $a_lo i64) (param $a_hi i64) (param $b_lo i64) (param $b_hi i64)
    local.get $sret
    i64.const 0xDEADBEF0    ;; 0xDEADBEEF + 1
    i64.store offset=0

    local.get $sret
    i64.const 0xDEADBEF1    ;; 0xDEADBEEF + 2
    i64.store offset=8

    local.get $sret
    i64.const 0xDEADBEF2    ;; 0xDEADBEEF + 3
    i64.store offset=16

    local.get $sret
    i64.const 0xDEADBEF3    ;; 0xDEADBEEF + 4
    i64.store offset=24
  )

  ;; Canonical __udivti3 body shape (matches what rustc / compiler-builtins
  ;; emits): reserve 32-byte WASM stack frame, call slow path, copy first
  ;; 16 bytes (quotient) back to caller sret, restore stack pointer.
  ;;
  ;; Libcall recognition replaces this with the fast/slow dispatch.
  (func $__udivti3 (param i32 i64 i64 i64 i64)
    (local i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 5
    global.set $__stack_pointer
    local.get 5
    local.get 1
    local.get 2
    local.get 3
    local.get 4
    call $sdr
    local.get 0
    local.get 5
    i64.load
    i64.store
    local.get 0
    local.get 5
    i64.load offset=8
    i64.store offset=8
    local.get 5
    i32.const 32
    i32.add
    global.set $__stack_pointer
  )

  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    i32.const 0x100              ;; sret
    local.get $args_ptr
    i64.load offset=0            ;; a_lo
    local.get $args_ptr
    i64.load offset=8            ;; a_hi
    local.get $args_ptr
    i64.load offset=16           ;; b_lo
    local.get $args_ptr
    i64.load offset=24           ;; b_hi
    call $__udivti3

    ;; Return (ptr=0x100, len=16). Packed (len << 32) | ptr.
    i64.const 0x1000000100
  )
)
