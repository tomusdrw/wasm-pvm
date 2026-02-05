;; Simple memory test - just write and read a byte

(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $addr i32)
    (local $val i32)

    ;; addr = 0 (WASM linear memory address; compiler adds WASM_MEMORY_BASE)
    (local.set $addr (i32.const 0))

    ;; Store 42 at addr (WASM address 0 -> PVM address 0x50000)
    (i32.store8 (local.get $addr) (i32.const 42))

    ;; Read it back
    (local.set $val (i32.load8_u (local.get $addr)))

    ;; Store result at WASM address 0x100 (-> PVM address 0x50100)
    (i32.store (i32.const 0x100) (local.get $val))
    ;; result_ptr is WASM address - epilogue adds WASM_MEMORY_BASE
    (global.set $result_ptr (i32.const 0x100))
    (global.set $result_len (i32.const 4))
  )
)
