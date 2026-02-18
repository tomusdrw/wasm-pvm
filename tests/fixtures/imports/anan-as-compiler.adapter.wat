;; Adapter for anan-as compiler: maps abort and console.log at WASM level.
;; console.log uses JIP-1 logging host call (ecalli 100).
;;
;; host_call convention: host_call(ecalli_index, r7, r8, r9, r10, r11)
;; JIP-1 register convention:
;;   r7  = log level (3 = helpful/debug)
;;   r8  = target pointer (0 = no target)
;;   r9  = target length  (0 = no target)
;;   r10 = message pointer (PVM address)
;;   r11 = message length  (bytes)
;;
;; AssemblyScript object header: rtSize (byte length) is at ptr - 4.
(module
  (import "env" "host_call" (func $host_call (param i64 i64 i64 i64 i64 i64)))
  (import "env" "pvm_ptr" (func $pvm_ptr (param i64) (result i64)))

  (func (export "abort") (param i32 i32 i32 i32)
    unreachable
  )

  (func (export "console.log") (param i32)
    (call $host_call
      (i64.const 100)                                           ;; ecalli index (JIP-1 log)
      (i64.const 3)                                             ;; r7: log level 3 (helpful)
      (i64.const 0)                                             ;; r8: target pointer (none)
      (i64.const 0)                                             ;; r9: target length (none)
      (call $pvm_ptr (i64.extend_i32_u (local.get 0)))          ;; r10: message PVM pointer
      (i64.extend_i32_u (i32.load offset=0                      ;; r11: message byte length
        (i32.sub (local.get 0) (i32.const 4))))                 ;;   read rtSize at ptr - 4
    )
  )
)
