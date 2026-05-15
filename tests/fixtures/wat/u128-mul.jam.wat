;; u128 multiplication via __multi3 — exercises libcall recognition.
;;
;; The compiler's `libcall_recognition` pass detects the WASM function named
;; `__multi3` and replaces its body with a hand-crafted PVM-friendly version
;; using the `MulUpperUU` opcode (PVM 214). This fixture provides a STUB
;; body (writes zeros to the sret area); a correct test result therefore
;; proves recognition fired AND the synthesized body computes 128-bit
;; multiplication correctly.
;;
;; Entry: main(args_ptr: i32, args_len: i32) -> i64
;;   args = 32 bytes: [a_lo: u64 LE][a_hi: u64 LE][b_lo: u64 LE][b_hi: u64 LE]
;;   args_len must equal 32
;;   returns (out_ptr: i32) | ((out_len: i32) << 32) per unified entry ABI
;;
;; Output (16 bytes at 0x100): low 128 bits of `(a_lo + 2^64·a_hi) * (b_lo + 2^64·b_hi)`,
;;   stored as [result_lo: u64 LE][result_hi: u64 LE].

(module
  (memory (export "memory") 1)

  ;; Compiler-builtins-shaped __multi3: (sret_ptr, a_lo, a_hi, b_lo, b_hi) → void
  ;; with result written as [sret+0] = low_half, [sret+8] = high_half.
  ;;
  ;; STUB BODY: writes zeros. When libcall_recognition is enabled (default),
  ;; this is replaced by the synthesized body before WASM IR translation
  ;; ever happens, so the stub is never executed. The stub still has to
  ;; be valid WASM so the module parses.
  (func $__multi3 (param i32 i64 i64 i64 i64)
    local.get 0
    i64.const 0
    i64.store

    local.get 0
    i32.const 8
    i32.add
    i64.const 0
    i64.store
  )

  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    ;; Call __multi3 with the 4 i64s from args. Result lands at 0x100 (low) / 0x108 (high).
    i32.const 0x100              ;; sret
    local.get $args_ptr
    i64.load offset=0            ;; a_lo
    local.get $args_ptr
    i64.load offset=8            ;; a_hi
    local.get $args_ptr
    i64.load offset=16           ;; b_lo
    local.get $args_ptr
    i64.load offset=24           ;; b_hi
    call $__multi3

    ;; Return (ptr=0x100, len=16). Packed as (len << 32) | ptr.
    ;;   ptr = 0x100, len = 16 → (16 << 32) | 0x100 = 0x1000000100
    i64.const 0x1000000100
  )
)
