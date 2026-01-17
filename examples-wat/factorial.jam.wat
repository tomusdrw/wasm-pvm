(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  ;; factorial(n) using only 2 extra locals (reuse params after reading args)
  ;; params: $args_ptr (idx 0) -> reused as $i
  ;;         $args_len (idx 1) -> reused as $n
  ;; locals: $result (idx 2)
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $result i32)
    
    ;; Read n from args into $args_len (reusing param as $n)
    (local.set $args_len (i32.load (local.get $args_ptr)))
    
    ;; Initialize result = 1
    (local.set $result (i32.const 1))
    
    ;; Initialize counter $args_ptr = 1 (reusing as $i)
    (local.set $args_ptr (i32.const 1))
    
    ;; Loop while i <= n
    (block $break
      (loop $continue
        ;; Check if i > n, break if true
        (br_if $break
          (i32.gt_u (local.get $args_ptr) (local.get $args_len))
        )
        
        ;; result *= i
        (local.set $result
          (i32.mul (local.get $result) (local.get $args_ptr))
        )
        
        ;; i++
        (local.set $args_ptr
          (i32.add (local.get $args_ptr) (i32.const 1))
        )
        
        (br $continue)
      )
    )
    
    ;; Write result to heap
    (i32.store (i32.const 0x20100) (local.get $result))
    
    (global.set $result_ptr (i32.const 0x20100))
    (global.set $result_len (i32.const 4))
  )
)
