(module
  (memory 1)
  
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    ;; Write sum of two u32 args to WASM memory address 0
    ;; (Compiler translates this to PVM address 0x50000)
    (i32.store
      (i32.const 0)
      (i32.add
        (i32.load (local.get $args_ptr))
        (i32.load (i32.add (local.get $args_ptr) (i32.const 4)))
      )
    )
    
    (i64.const 17179869184)  ;; ptr=0, len=4
  )
)
