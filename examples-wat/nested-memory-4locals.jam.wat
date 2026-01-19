;; Test memory in nested loops with 4 locals (fits in registers)

(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $y i32)     ;; local 2 -> r11
    (local $x i32)     ;; local 3 -> r12
    
    ;; Note: params use local 0, 1 -> r9, r10
    
    ;; Write checkerboard pattern to 0
    (local.set $y (i32.const 0))
    (block $y_exit
      (loop $y_loop
        (br_if $y_exit (i32.ge_s (local.get $y) (i32.const 4)))
        
        (local.set $x (i32.const 0))
        (block $x_exit
          (loop $x_loop
            (br_if $x_exit (i32.ge_s (local.get $x) (i32.const 4)))
            
            ;; store (y + x) & 1 at 0 + y*4 + x
            (i32.store8 
              (i32.add 
                (i32.const 0)
                (i32.add 
                  (i32.shl (local.get $y) (i32.const 2))
                  (local.get $x)))
              (i32.and 
                (i32.add (local.get $y) (local.get $x)) 
                (i32.const 1)))
            
            (local.set $x (i32.add (local.get $x) (i32.const 1)))
            (br $x_loop)
          )
        )
        
        (local.set $y (i32.add (local.get $y) (i32.const 1)))
        (br $y_loop)
      )
    )
    
    ;; Read back first 4 values
    ;; Expected: 0,1,0,1 (first row of checkerboard)
    (i32.store (i32.const 0x30200)
      (i32.add
        (i32.add
          (i32.load8_u (i32.const 0))
          (i32.shl (i32.load8_u (i32.const 0x30101)) (i32.const 8)))
        (i32.add
          (i32.shl (i32.load8_u (i32.const 0x30102)) (i32.const 16))
          (i32.shl (i32.load8_u (i32.const 0x30103)) (i32.const 24)))))
    
    (global.set $result_ptr (i32.const 0x30200))
    (global.set $result_len (i32.const 4))
  )
)
