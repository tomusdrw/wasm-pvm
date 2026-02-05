;; Test memory with computed addresses
;; arg 0: offset test - store 42 at base + 5, read it back
;; arg 1: scale test - store 84 at base * 2 + 4, read it back

(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $test_case i32)
    (local $base i32)
    (local $addr i32)
    (local $val i32)
    
    ;; Read test case from args
    (local.set $test_case (i32.load (local.get $args_ptr)))
    
    ;; Test 0: computed address with offset
    ;; base = 0x100, offset = 5, store 42 at base + offset
    (if (i32.eq (local.get $test_case) (i32.const 0))
      (then
        (local.set $base (i32.const 0x100))
        ;; Store 42 at base + 5
        (i32.store8 
          (i32.add (local.get $base) (i32.const 5))
          (i32.const 42))
        ;; Read it back
        (local.set $val 
          (i32.load8_u 
            (i32.add (local.get $base) (i32.const 5))))
      )
    )
    
    ;; Test 1: computed address with scale
    ;; base = 40, scale = 2, offset = 4
    ;; addr = base * 2 + 4 = 84, store 84 at that address
    (if (i32.eq (local.get $test_case) (i32.const 1))
      (then
        (local.set $base (i32.const 40))
        ;; Calculate address: base * 2 + 4
        (local.set $addr
          (i32.add
            (i32.mul (local.get $base) (i32.const 2))
            (i32.const 4)))
        ;; Store 84 at computed address
        (i32.store8 (local.get $addr) (i32.const 84))
        ;; Read it back
        (local.set $val (i32.load8_u (local.get $addr)))
      )
    )
    
    ;; Store result at WASM address 0x200 (-> PVM address 0x50200)
    (i32.store (i32.const 0x200) (local.get $val))
    ;; result_ptr is WASM address - epilogue adds WASM_MEMORY_BASE
    (global.set $result_ptr (i32.const 0x200))
    (global.set $result_len (i32.const 4))
  )
)
