;; Test memory operations inside nested loops (similar to life.wasm pattern)
;; Reads from one buffer, writes to another

(module
  (memory 1)
  
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (local $y i32)
    (local $x i32)
    (local $src i32)
    (local $dst i32)
    (local $sum i32)
    
    (local.set $src (i32.const 0))  ;; source buffer
    (local.set $dst (i32.const 0x30200))  ;; dest buffer
    
    ;; Initialize source buffer with pattern: cell[y*16+x] = (y + x) & 1
    (local.set $y (i32.const 0))
    (block $init_y_exit
      (loop $init_y
        (br_if $init_y_exit (i32.ge_s (local.get $y) (i32.const 16)))
        (local.set $x (i32.const 0))
        (block $init_x_exit
          (loop $init_x
            (br_if $init_x_exit (i32.ge_s (local.get $x) (i32.const 16)))
            
            ;; store (y + x) & 1
            (i32.store8 
              (i32.add (local.get $src) 
                       (i32.add (i32.shl (local.get $y) (i32.const 4)) (local.get $x)))
              (i32.and (i32.add (local.get $y) (local.get $x)) (i32.const 1))
            )
            
            (local.set $x (i32.add (local.get $x) (i32.const 1)))
            (br $init_x)
          )
        )
        (local.set $y (i32.add (local.get $y) (i32.const 1)))
        (br $init_y)
      )
    )
    
    ;; Copy from src to dst and count 1s
    (local.set $y (i32.const 0))
    (local.set $sum (i32.const 0))
    (block $copy_y_exit
      (loop $copy_y
        (br_if $copy_y_exit (i32.ge_s (local.get $y) (i32.const 16)))
        (local.set $x (i32.const 0))
        (block $copy_x_exit
          (loop $copy_x
            (br_if $copy_x_exit (i32.ge_s (local.get $x) (i32.const 16)))
            
            ;; Read from src, write to dst
            (i32.store8 
              (i32.add (local.get $dst) 
                       (i32.add (i32.shl (local.get $y) (i32.const 4)) (local.get $x)))
              (i32.load8_u
                (i32.add (local.get $src) 
                         (i32.add (i32.shl (local.get $y) (i32.const 4)) (local.get $x))))
            )
            
            ;; Count 1s
            (local.set $sum 
              (i32.add (local.get $sum)
                       (i32.load8_u
                         (i32.add (local.get $src) 
                                  (i32.add (i32.shl (local.get $y) (i32.const 4)) (local.get $x))))))
            
            (local.set $x (i32.add (local.get $x) (i32.const 1)))
            (br $copy_x)
          )
        )
        (local.set $y (i32.add (local.get $y) (i32.const 1)))
        (br $copy_y)
      )
    )
    
    ;; Expected: 128 ones (checkerboard pattern has 128 1s in 16x16)
    (i32.store (i32.const 0x30300) (local.get $sum))
    (i32.const 0x30300)  ;; result_ptr
    (i32.const 4)  ;; result_len
  )
)
