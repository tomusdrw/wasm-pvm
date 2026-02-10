;; Test memory in nested loops with 11 locals (7 spilled) - similar to life.wasm

(module
  (memory 1)
  
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    ;; params: local 0, 1 -> r9, r10
    (local $y i32)       ;; local 2 -> r11
    (local $x i32)       ;; local 3 -> r12
    (local $sum i32)     ;; local 4 -> spilled
    (local $base i32)    ;; local 5 -> spilled
    (local $tmp1 i32)    ;; local 6 -> spilled
    (local $tmp2 i32)    ;; local 7 -> spilled
    (local $tmp3 i32)    ;; local 8 -> spilled
    (local $tmp4 i32)    ;; local 9 -> spilled
    (local $tmp5 i32)    ;; local 10 -> spilled
    
    (local.set $base (i32.const 0))
    
    ;; Write checkerboard pattern using tmp variables
    (local.set $y (i32.const 0))
    (block $y_exit
      (loop $y_loop
        (br_if $y_exit (i32.ge_s (local.get $y) (i32.const 4)))
        
        ;; tmp1 = y * 4
        (local.set $tmp1 (i32.shl (local.get $y) (i32.const 2)))
        
        (local.set $x (i32.const 0))
        (block $x_exit
          (loop $x_loop
            (br_if $x_exit (i32.ge_s (local.get $x) (i32.const 4)))
            
            ;; tmp2 = base + tmp1 + x
            (local.set $tmp2 
              (i32.add 
                (local.get $base)
                (i32.add (local.get $tmp1) (local.get $x))))
            
            ;; tmp3 = (y + x) & 1
            (local.set $tmp3
              (i32.and 
                (i32.add (local.get $y) (local.get $x)) 
                (i32.const 1)))
            
            ;; store tmp3 at tmp2
            (i32.store8 (local.get $tmp2) (local.get $tmp3))
            
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
        
        (local.set $tmp4 (i32.shl (local.get $y) (i32.const 2)))
        
        (local.set $x (i32.const 0))
        (block $read_x_exit
          (loop $read_x_loop
            (br_if $read_x_exit (i32.ge_s (local.get $x) (i32.const 4)))
            
            ;; tmp5 = load(base + tmp4 + x)
            (local.set $tmp5
              (i32.load8_u 
                (i32.add 
                  (local.get $base)
                  (i32.add (local.get $tmp4) (local.get $x)))))
            
            (local.set $sum (i32.add (local.get $sum) (local.get $tmp5)))
            
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
    (i32.const 0x30200)  ;; result_ptr
    (i32.const 4)  ;; result_len
  )
)
