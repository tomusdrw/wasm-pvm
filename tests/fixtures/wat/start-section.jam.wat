(module
  (memory 1)

  (global $g (mut i32) (i32.const 0))
  (func $start
    (global.set $g (i32.const 42))
  )
  (start $start)

  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    ;; Store global value to result area (at WASM address 0x100)
    ;; The i32.store address also gets WASM_MEMORY_BASE added by the compiler
    (i32.store (i32.const 0x100) (global.get $g))

    ;; Return result pointer and length
    ;; result_ptr is a WASM address - the epilogue adds WASM_MEMORY_BASE
    ;; So WASM address 0x100 becomes PVM address 0x50100
    (i32.const 0x100)  ;; result_ptr
    (i32.const 4)  ;; result_len
  )
)
