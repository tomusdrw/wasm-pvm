;; Test the loop-if pattern used by AssemblyScript (like life.wasm)

(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $y i32)
    (local $x i32)
    (local $count i32)
    
    ;; Pattern: loop { if (cond) { body; br 1 } }
    ;; This is how AssemblyScript compiles for loops
    
    (local.set $y (i32.const 0))
    (loop $outer                          ;; @1
      (local.get $y)
      (i32.const 4)
      (i32.lt_s)
      (if                                 ;; @2
        (then
          (local.set $x (i32.const 0))
          (loop $inner                    ;; @3
            (local.get $x)
            (i32.const 4)
            (i32.lt_s)
            (if                           ;; @4
              (then
                ;; count++
                (local.set $count (i32.add (local.get $count) (i32.const 1)))
                
                ;; x++
                (local.set $x (i32.add (local.get $x) (i32.const 1)))
                (br 1)                    ;; -> @3 (inner loop)
              )
            )
          )
          
          ;; y++
          (local.set $y (i32.add (local.get $y) (i32.const 1)))
          (br 1)                          ;; -> @1 (outer loop)
        )
      )
    )
    
    ;; Expected: 16 iterations
    (i32.store (i32.const 0x30100) (local.get $count))
    (global.set $result_ptr (i32.const 0x30100))
    (global.set $result_len (i32.const 4))
  )
)
