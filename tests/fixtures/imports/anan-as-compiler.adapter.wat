;; Adapter for anan-as compiler: handles abort, console.log, and ecalli forwarding.
;;
;; The compiler calls host_call_6b(ecalli, r7..r12) for each inner ecalli.
;; This adapter handles:
;;   - ecalli 100 (JIP-1 log): translates inner memory pointers via host_read_memory
;;   - all other ecalli: trap (not yet supported)
;;
;; host_read_memory(addr, len) -> packed i64 (lower 32 = wasm_ptr, upper 32 = len)
;; is an export of the compiler module, resolved by adapter_merge.
(module
  ;; Outer PVM intrinsics
  (import "env" "host_call_5" (func $outer_host_call_5 (param i64 i64 i64 i64 i64 i64) (result i64)))
  (import "env" "pvm_ptr" (func $pvm_ptr (param i64) (result i64)))

  ;; Compiler export resolved by adapter_merge (adapter import matched to main export).
  (import "env" "host_read_memory" (func $host_read_memory (param i32 i32) (result i64)))

  ;; --- abort: AS runtime abort handler ---
  (func (export "abort") (param i32 i32 i32 i32)
    unreachable
  )

  ;; --- console.log: AS runtime logging (compiler's own logs) ---
  ;; Uses JIP-1 ecalli 100 with level=3 (helpful).
  ;; AS strings are at ptr with rtSize (byte length) at ptr-4.
  (func (export "console.log") (param i32)
    (drop (call $outer_host_call_5
      (i64.const 100)                                           ;; ecalli 100
      (i64.const 3)                                             ;; r7: log level 3
      (i64.const 0)                                             ;; r8: target ptr (none)
      (i64.const 0)                                             ;; r9: target len (none)
      (call $pvm_ptr (i64.extend_i32_u (local.get 0)))          ;; r10: message PVM ptr
      (i64.extend_i32_u (i32.load offset=0                      ;; r11: message byte len
        (i32.sub (local.get 0) (i32.const 4))))                 ;;   rtSize at ptr-4
    ))
  )

  ;; --- host_call_6b: inner program ecalli forwarding ---
  ;; Called by the compiler when the inner program executes ecalli.
  ;; Dispatches based on ecalli index.
  (func (export "host_call_6b")
    (param $ecalli i64) (param $r7 i64) (param $r8 i64) (param $r9 i64)
    (param $r10 i64) (param $r11 i64) (param $r12 i64)
    (result i64)

    (local $target_packed i64)
    (local $msg_packed i64)

    ;; Check if ecalli == 100 (JIP-1 log)
    (if (i64.eq (local.get $ecalli) (i64.const 100))
      (then
        ;; ecalli 100: JIP-1 log
        ;; r7 = level, r8 = target_ptr, r9 = target_len, r10 = msg_ptr, r11 = msg_len

        ;; Copy target string from inner memory (r8=addr, r9=len)
        (local.set $target_packed
          (call $host_read_memory
            (i32.wrap_i64 (local.get $r8))
            (i32.wrap_i64 (local.get $r9))
          )
        )

        ;; Copy message string from inner memory (r10=addr, r11=len)
        (local.set $msg_packed
          (call $host_read_memory
            (i32.wrap_i64 (local.get $r10))
            (i32.wrap_i64 (local.get $r11))
          )
        )

        ;; Issue outer ecalli 100 with translated pointers.
        ;; host_read_memory returns packed i64: lower 32 = wasm ptr.
        (return
          (call $outer_host_call_5
            (i64.const 100)                                               ;; ecalli 100
            (local.get $r7)                                               ;; r7: level
            (call $pvm_ptr (i64.and (local.get $target_packed)            ;; r8: target PVM ptr
              (i64.const 0xffffffff)))
            (local.get $r9)                                               ;; r9: target len
            (call $pvm_ptr (i64.and (local.get $msg_packed)               ;; r10: msg PVM ptr
              (i64.const 0xffffffff)))
            (local.get $r11)                                              ;; r11: msg len
          )
        )
      )
    )

    ;; Default: unsupported ecalli — trap.
    ;; TODO: implement dynamic ecalli forwarding for non-100 ecalli indices.
    unreachable
  )

  ;; --- host_call_r8: return captured r8 from last host call ---
  ;; For ecalli 100, r8 is not meaningful. Return 0.
  ;; TODO: implement proper r8 forwarding when dynamic ecalli is supported.
  (func (export "host_call_r8") (result i64)
    (i64.const 0)
  )
)
