;; Test fixture for `--trap-floats` mode.
;;
;; Default compile: must FAIL (`f64.add` is unsupported).
;; --trap-floats compile: must SUCCEED. At runtime, the function takes a
;; one-byte input. When the input is zero the function returns normally
;; (no float op executed); when non-zero it falls into the float branch and
;; traps deterministically.
;;
;; This fixture lives outside `tests/fixtures/wat/` (which the build
;; orchestrator auto-compiles with default flags) so we can drive its
;; compilation manually from `layer1/trap-floats.test.ts`.
(module
  (memory (export "memory") 1)
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    ;; Read first byte of args. Zero → safe path; non-zero → float trap path.
    (local $flag i32)
    local.get $args_ptr
    i32.load8_u
    local.set $flag

    local.get $flag
    i32.eqz
    if (result i64)
      ;; Safe path: return ptr=0, len=4 (no float op encountered).
      i64.const 17179869184
    else
      ;; Trap path: any float op fires the trap before this branch returns.
      f64.const 1.0
      f64.const 2.0
      f64.add
      drop
      i64.const 17179869184
    end
  )
)
