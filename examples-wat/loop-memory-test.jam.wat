;; Test memory in a loop

(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $base i32)
    (local $i i32)
    (local $sum i32)
    
    (local.set $base (i32.const 0x30100))
    
    ;; Write 0..15 to buffer
    (local.set $i (i32.const 0))
    (block $write_exit
      (loop $write_loop
        (br_if $write_exit (i32.ge_s (local.get $i) (i32.const 16)))
        
        (i32.store8 
          (i32.add (local.get $base) (local.get $i))
          (local.get $i))
        
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $write_loop)
      )
    )
    
    ;; Read back and sum
    (local.set $i (i32.const 0))
    (local.set $sum (i32.const 0))
    (block $read_exit
      (loop $read_loop
        (br_if $read_exit (i32.ge_s (local.get $i) (i32.const 16)))
        
        (local.set $sum 
          (i32.add (local.get $sum)
                   (i32.load8_u (i32.add (local.get $base) (local.get $i)))))
        
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $read_loop)
      )
    )
    
    ;; Expected: 0+1+2+...+15 = 120
    (i32.store (i32.const 0x30200) (local.get $sum))
    (global.set $result_ptr (i32.const 0x30200))
    (global.set $result_len (i32.const 4))
  )
)
