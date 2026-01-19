;; Test memory in nested loops with 6 locals (2 spilled)

(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $y i32)       ;; local 2 -> r11
    (local $x i32)       ;; local 3 -> r12
    (local $sum i32)     ;; local 4 -> spilled
    (local $base i32)    ;; local 5 -> spilled
    
    (local.set $base (i32.const 0))
    
    ;; Write checkerboard pattern
    (local.set $y (i32.const 0))
    (block $y_exit
      (loop $y_loop
        (br_if $y_exit (i32.ge_s (local.get $y) (i32.const 4)))
        
        (local.set $x (i32.const 0))
        (block $x_exit
          (loop $x_loop
            (br_if $x_exit (i32.ge_s (local.get $x) (i32.const 4)))
            
            ;; store (y + x) & 1 at base + y*4 + x
            (i32.store8 
              (i32.add 
                (local.get $base)
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
    
    ;; Read back and sum all values
    (local.set $y (i32.const 0))
    (local.set $sum (i32.const 0))
    (block $read_y_exit
      (loop $read_y_loop
        (br_if $read_y_exit (i32.ge_s (local.get $y) (i32.const 4)))
        
        (local.set $x (i32.const 0))
        (block $read_x_exit
          (loop $read_x_loop
            (br_if $read_x_exit (i32.ge_s (local.get $x) (i32.const 4)))
            
            (local.set $sum
              (i32.add 
                (local.get $sum)
                (i32.load8_u 
                  (i32.add 
                    (local.get $base)
                    (i32.add 
                      (i32.shl (local.get $y) (i32.const 2))
                      (local.get $x))))))
            
            (local.set $x (i32.add (local.get $x) (i32.const 1)))
            (br $read_x_loop)
          )
        )
        
        (local.set $y (i32.add (local.get $y) (i32.const 1)))
        (br $read_y_loop)
      )
    )
    
    ;; Expected: 8 ones in 4x4 checkerboard
    (i32.store (i32.const 0x30200) (local.get $sum))
    (global.set $result_ptr (i32.const 0x30200))
    (global.set $result_len (i32.const 4))
  )
)
