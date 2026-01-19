;; Test memory with computed addresses

(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $base i32)
    (local $offset i32)
    (local $val i32)
    
    ;; base = 0
    (local.set $base (i32.const 0))
    ;; offset = 5
    (local.set $offset (i32.const 5))
    
    ;; Store 99 at base + offset
    (i32.store8 
      (i32.add (local.get $base) (local.get $offset))
      (i32.const 99))
    
    ;; Read it back
    (local.set $val 
      (i32.load8_u 
        (i32.add (local.get $base) (local.get $offset))))
    
    ;; Store result
    (i32.store (i32.const 0x30200) (local.get $val))
    (global.set $result_ptr (i32.const 0x30200))
    (global.set $result_len (i32.const 4))
  )
)
