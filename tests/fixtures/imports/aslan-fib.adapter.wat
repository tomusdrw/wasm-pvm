(module
  (import "env" "host_call_5" (func $host_call_5 (param i64 i64 i64 i64 i64 i64) (result i64)))
  (import "env" "pvm_ptr" (func $pvm_ptr (param i64) (result i64)))

  (func (export "abort") (param i32 i32 i32 i32)
    unreachable
  )

  (func (export "console.log") (param i32)
    (drop (call $host_call_5
      (i64.const 100)                                           ;; ecalli index (JIP-1 log)
      (i64.const 3)                                             ;; r7: log level 3 (helpful)
      (i64.const 0)                                             ;; r8: target pointer (none)
      (i64.const 0)                                             ;; r9: target length (none)
      (call $pvm_ptr (i64.extend_i32_u (local.get 0)))          ;; r10: message PVM pointer
      (i64.extend_i32_u (i32.load offset=0                      ;; r11: message byte length
        (i32.sub (local.get 0) (i32.const 4))))                 ;;   read rtSize at ptr - 4
    ))
  )

  ;; imports.log(level, target_ptr, target_len, msg_ptr, msg_len) -> i32
  ;; Maps to ecalli 100 (JIP-1 log)
  (func (export "log") (param $level i32) (param $target_ptr i32) (param $target_len i32) (param $msg_ptr i32) (param $msg_len i32) (result i32)
    (drop (call $host_call_5
      (i64.const 100)                                                     ;; ecalli index (JIP-1 log)
      (i64.extend_i32_u (local.get $level))                               ;; r7: log level
      (call $pvm_ptr (i64.extend_i32_u (local.get $target_ptr)))          ;; r8: target PVM pointer
      (i64.extend_i32_u (local.get $target_len))                          ;; r9: target length
      (call $pvm_ptr (i64.extend_i32_u (local.get $msg_ptr)))             ;; r10: message PVM pointer
      (i64.extend_i32_u (local.get $msg_len))                             ;; r11: message length
    ))
    (i32.const 0)  ;; return 0 (success)
  )
)
