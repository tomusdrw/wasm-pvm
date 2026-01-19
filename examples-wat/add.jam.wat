(module
  (memory 1)
  
  ;; Globals for return value pointers (WASM addresses, translated to PVM by compiler)
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    ;; Write sum of two u32 args to WASM memory address 0
    ;; (Compiler translates this to PVM address 0x50000)
    (i32.store
      (i32.const 0)
      (i32.add
        (i32.load (local.get $args_ptr))
        (i32.load (i32.add (local.get $args_ptr) (i32.const 4)))
      )
    )
    
    ;; Set result pointer to WASM address 0 (compiler adds WASM_MEMORY_BASE)
    (global.set $result_ptr (i32.const 0))
    (global.set $result_len (i32.const 4))
  )
)
