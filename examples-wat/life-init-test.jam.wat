;; Test just the initialization part of Game of Life

(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $buf_a i32)
    
    (local.set $buf_a (i32.const 0x30100))
    
    ;; Initialize a horizontal blinker at row 1: (0,1), (1,1), (2,1)
    (i32.store8 (i32.add (local.get $buf_a) (i32.const 4)) (i32.const 1))  ;; (0,1)
    (i32.store8 (i32.add (local.get $buf_a) (i32.const 5)) (i32.const 1))  ;; (1,1)
    (i32.store8 (i32.add (local.get $buf_a) (i32.const 6)) (i32.const 1))  ;; (2,1)
    
    ;; Return buf_a contents (should show 0 0 0 0 | 1 1 1 0 | 0 0 0 0 | 0 0 0 0)
    (i32.store (i32.const 0x30200) (i32.const 4))  ;; width
    (i32.store (i32.const 0x30204) (i32.const 4))  ;; height
    
    ;; Copy all 16 bytes
    (i32.store (i32.const 0x30208) 
      (i32.or
        (i32.or
          (i32.load8_u (local.get $buf_a))
          (i32.shl (i32.load8_u (i32.add (local.get $buf_a) (i32.const 1))) (i32.const 8)))
        (i32.or
          (i32.shl (i32.load8_u (i32.add (local.get $buf_a) (i32.const 2))) (i32.const 16))
          (i32.shl (i32.load8_u (i32.add (local.get $buf_a) (i32.const 3))) (i32.const 24)))))
    
    (i32.store (i32.const 0x3020c)
      (i32.or
        (i32.or
          (i32.load8_u (i32.add (local.get $buf_a) (i32.const 4)))
          (i32.shl (i32.load8_u (i32.add (local.get $buf_a) (i32.const 5))) (i32.const 8)))
        (i32.or
          (i32.shl (i32.load8_u (i32.add (local.get $buf_a) (i32.const 6))) (i32.const 16))
          (i32.shl (i32.load8_u (i32.add (local.get $buf_a) (i32.const 7))) (i32.const 24)))))
    
    (i32.store (i32.const 0x30210)
      (i32.or
        (i32.or
          (i32.load8_u (i32.add (local.get $buf_a) (i32.const 8)))
          (i32.shl (i32.load8_u (i32.add (local.get $buf_a) (i32.const 9))) (i32.const 8)))
        (i32.or
          (i32.shl (i32.load8_u (i32.add (local.get $buf_a) (i32.const 10))) (i32.const 16))
          (i32.shl (i32.load8_u (i32.add (local.get $buf_a) (i32.const 11))) (i32.const 24)))))
    
    (i32.store (i32.const 0x30214)
      (i32.or
        (i32.or
          (i32.load8_u (i32.add (local.get $buf_a) (i32.const 12)))
          (i32.shl (i32.load8_u (i32.add (local.get $buf_a) (i32.const 13))) (i32.const 8)))
        (i32.or
          (i32.shl (i32.load8_u (i32.add (local.get $buf_a) (i32.const 14))) (i32.const 16))
          (i32.shl (i32.load8_u (i32.add (local.get $buf_a) (i32.const 15))) (i32.const 24)))))
    
    (global.set $result_ptr (i32.const 0x30200))
    (global.set $result_len (i32.const 24))
  )
)
