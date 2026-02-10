(module
  (memory 1)

  ;; main entry point (PC=0): returns 42
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (i32.store (i32.const 0) (i32.const 42))
    (i32.const 0)  ;; result_ptr
    (i32.const 4)  ;; result_len
  )

  ;; main2 entry point (PC=5): returns 99
  (func (export "main2") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (i32.store (i32.const 0) (i32.const 99))
    (i32.const 0)  ;; result_ptr
    (i32.const 4)  ;; result_len
  )
)
