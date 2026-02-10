;; Test: Stack overflow detection
;; This function recurses deeply to trigger stack overflow
(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  ;; Recursive function that decrements n until 0
  ;; Each call uses ~48 bytes of stack (40 bytes header + some for operand stack)
  ;; With 64KB stack (default), we can handle ~1300 recursion levels safely
  ;; With 2000 recursion levels, it should overflow
  (func $deep_recurse (param $n i32) (result i32)
    (if (result i32) (i32.eqz (local.get $n))
      (then (i32.const 1))
      (else
        (call $deep_recurse (i32.sub (local.get $n) (i32.const 1)))
      )
    )
  )
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $result i32)
    
    ;; Read the recursion depth from args
    (local.set $result
      (call $deep_recurse (i32.load (local.get $args_ptr)))
    )
    
    (i32.store (i32.const 0) (local.get $result))
    (global.set $result_ptr (i32.const 0))
    (global.set $result_len (i32.const 4))
  )
)
