;; Simple nested loop test
;; Output: Number of loop iterations (should be 16*16 = 256)

(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $y i32)
    (local $x i32)
    (local $count i32)
    
    ;; for (y = 0; y < 16; y++)
    (local.set $y (i32.const 0))
    (block $outer_exit
      (loop $outer
        ;; if y >= 16, exit
        (br_if $outer_exit (i32.ge_s (local.get $y) (i32.const 16)))
        
        ;; for (x = 0; x < 16; x++)
        (local.set $x (i32.const 0))
        (block $inner_exit
          (loop $inner
            ;; if x >= 16, exit
            (br_if $inner_exit (i32.ge_s (local.get $x) (i32.const 16)))
            
            ;; count++
            (local.set $count (i32.add (local.get $count) (i32.const 1)))
            
            ;; x++
            (local.set $x (i32.add (local.get $x) (i32.const 1)))
            (br $inner)
          )
        )
        
        ;; y++
        (local.set $y (i32.add (local.get $y) (i32.const 1)))
        (br $outer)
      )
    )
    
    ;; Store result
    (i32.store (i32.const 0) (local.get $count))
    (global.set $result_ptr (i32.const 0))
    (global.set $result_len (i32.const 4))
  )
)
