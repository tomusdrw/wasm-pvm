(module
  (memory 1)

  ;; Helper function that writes to memory and returns a value
  ;; This ensures it's a real call that clobbers caller-saved registers
  (func $helper (param $x i32) (result i32)
    ;; Store to memory to prevent inlining optimization
    (i32.store (i32.const 0x100) (local.get $x))
    ;; Return x * 2
    (i32.mul (local.get $x) (i32.const 2))
  )

  ;; Main: reads step from args, calls helper, then branches on step
  ;; Bug pattern: step is read before call, used after call
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $step i32)
    (local $call_result i32)
    (local $result i32)

    ;; step = args[0] (should be 0, 1, or 2)
    (local.set $step
      (if (result i32) (i32.gt_u (local.get $args_len) (i32.const 0))
        (then (i32.load8_u (local.get $args_ptr)))
        (else (i32.const 0))
      )
    )

    ;; Call helper — this should NOT clobber $step
    (local.set $call_result (call $helper (i32.const 42)))

    ;; Branch on step (read AFTER the call)
    (if (i32.eq (local.get $step) (i32.const 0))
      (then (local.set $result (i32.const 100)))
    )
    (if (i32.eq (local.get $step) (i32.const 1))
      (then (local.set $result (i32.const 200)))
    )
    (if (i32.eq (local.get $step) (i32.const 2))
      (then (local.set $result (i32.const 300)))
    )

    ;; Store result to memory and return pointer
    (i32.store (i32.const 0x200) (local.get $result))
    (i64.or
      (i64.extend_i32_u (i32.const 0x200))
      (i64.shl (i64.const 4) (i64.const 32))
    )
  )
)
