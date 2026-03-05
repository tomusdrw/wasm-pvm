(module
  (memory 1)

  ;; main entry point (PC=0): returns 42
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (i32.store (i32.const 0) (i32.const 42))
    (i64.const 17179869184)  ;; ptr=0, len=4
  )

  ;; main2 entry point (PC=5): returns 99
  (func (export "main2") (param $args_ptr i32) (param $args_len i32) (result i64)
    (i32.store (i32.const 0) (i32.const 99))
    (i64.const 17179869184)  ;; ptr=0, len=4
  )
)
