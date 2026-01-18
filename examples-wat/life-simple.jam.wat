;; Simplified Game of Life - 4x4 grid with single blinker
;; Tests the core step logic

(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  ;; Count neighbors at (x,y) in buffer at base, wrapping at SIZE
  (func $count_neighbors (param $base i32) (param $x i32) (param $y i32) (param $size i32) (result i32)
    (local $count i32)
    (local $dx i32)
    (local $dy i32)
    (local $nx i32)
    (local $ny i32)
    
    ;; Check all 8 neighbors
    (local.set $dy (i32.const -1))
    (block $dy_exit
      (loop $dy_loop
        (br_if $dy_exit (i32.gt_s (local.get $dy) (i32.const 1)))
        
        (local.set $dx (i32.const -1))
        (block $dx_exit
          (loop $dx_loop
            (br_if $dx_exit (i32.gt_s (local.get $dx) (i32.const 1)))
            
            ;; Skip self (dx=0, dy=0)
            (if (i32.or (local.get $dx) (local.get $dy))
              (then
                ;; nx = (x + dx + size) % size
                (local.set $nx 
                  (i32.rem_u 
                    (i32.add (i32.add (local.get $x) (local.get $dx)) (local.get $size))
                    (local.get $size)))
                ;; ny = (y + dy + size) % size
                (local.set $ny 
                  (i32.rem_u 
                    (i32.add (i32.add (local.get $y) (local.get $dy)) (local.get $size))
                    (local.get $size)))
                
                ;; count += buffer[ny * size + nx]
                (local.set $count
                  (i32.add (local.get $count)
                    (i32.load8_u 
                      (i32.add (local.get $base)
                        (i32.add (i32.mul (local.get $ny) (local.get $size)) (local.get $nx))))))
              )
            )
            
            (local.set $dx (i32.add (local.get $dx) (i32.const 1)))
            (br $dx_loop)
          )
        )
        
        (local.set $dy (i32.add (local.get $dy) (i32.const 1)))
        (br $dy_loop)
      )
    )
    
    (local.get $count)
  )
  
  ;; Step once: read from src, write to dst
  (func $step (param $src i32) (param $dst i32) (param $size i32)
    (local $x i32)
    (local $y i32)
    (local $neighbors i32)
    (local $cell i32)
    (local $next i32)
    
    (local.set $y (i32.const 0))
    (block $y_exit
      (loop $y_loop
        (br_if $y_exit (i32.ge_s (local.get $y) (local.get $size)))
        
        (local.set $x (i32.const 0))
        (block $x_exit
          (loop $x_loop
            (br_if $x_exit (i32.ge_s (local.get $x) (local.get $size)))
            
            ;; Count neighbors
            (local.set $neighbors
              (call $count_neighbors (local.get $src) (local.get $x) (local.get $y) (local.get $size)))
            
            ;; Get current cell
            (local.set $cell
              (i32.load8_u 
                (i32.add (local.get $src)
                  (i32.add (i32.mul (local.get $y) (local.get $size)) (local.get $x)))))
            
            ;; Apply rules
            (local.set $next (i32.const 0))
            (if (local.get $cell)
              (then
                ;; Alive: survive with 2 or 3 neighbors
                (if (i32.or (i32.eq (local.get $neighbors) (i32.const 2))
                            (i32.eq (local.get $neighbors) (i32.const 3)))
                  (then (local.set $next (i32.const 1))))
              )
              (else
                ;; Dead: born with exactly 3 neighbors
                (if (i32.eq (local.get $neighbors) (i32.const 3))
                  (then (local.set $next (i32.const 1))))
              )
            )
            
            ;; Write result
            (i32.store8
              (i32.add (local.get $dst)
                (i32.add (i32.mul (local.get $y) (local.get $size)) (local.get $x)))
              (local.get $next))
            
            (local.set $x (i32.add (local.get $x) (i32.const 1)))
            (br $x_loop)
          )
        )
        
        (local.set $y (i32.add (local.get $y) (i32.const 1)))
        (br $y_loop)
      )
    )
  )
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $buf_a i32)
    (local $buf_b i32)
    (local $size i32)
    
    (local.set $buf_a (i32.const 0x30100))
    (local.set $buf_b (i32.const 0x30200))
    (local.set $size (i32.const 4))
    
    ;; Initialize a vertical blinker at row 1: (0,1), (1,1), (2,1)
    ;; Grid layout (4x4):
    ;;   0 0 0 0
    ;;   1 1 1 0
    ;;   0 0 0 0
    ;;   0 0 0 0
    (i32.store8 (i32.add (local.get $buf_a) (i32.const 4)) (i32.const 1))  ;; (0,1)
    (i32.store8 (i32.add (local.get $buf_a) (i32.const 5)) (i32.const 1))  ;; (1,1)
    (i32.store8 (i32.add (local.get $buf_a) (i32.const 6)) (i32.const 1))  ;; (2,1)
    
    ;; Step once
    (call $step (local.get $buf_a) (local.get $buf_b) (local.get $size))
    
    ;; After one step, blinker should rotate to vertical:
    ;;   0 1 0 0
    ;;   0 1 0 0
    ;;   0 1 0 0
    ;;   0 0 0 0
    
    ;; Return buf_b contents
    (i32.store (i32.const 0x30300) (local.get $size))
    (i32.store (i32.const 0x30304) (local.get $size))
    
    ;; Copy grid to output
    (i32.store (i32.const 0x30308) 
      (i32.or
        (i32.or
          (i32.load8_u (local.get $buf_b))
          (i32.shl (i32.load8_u (i32.add (local.get $buf_b) (i32.const 1))) (i32.const 8)))
        (i32.or
          (i32.shl (i32.load8_u (i32.add (local.get $buf_b) (i32.const 2))) (i32.const 16))
          (i32.shl (i32.load8_u (i32.add (local.get $buf_b) (i32.const 3))) (i32.const 24)))))
    
    (i32.store (i32.const 0x3030c)
      (i32.or
        (i32.or
          (i32.load8_u (i32.add (local.get $buf_b) (i32.const 4)))
          (i32.shl (i32.load8_u (i32.add (local.get $buf_b) (i32.const 5))) (i32.const 8)))
        (i32.or
          (i32.shl (i32.load8_u (i32.add (local.get $buf_b) (i32.const 6))) (i32.const 16))
          (i32.shl (i32.load8_u (i32.add (local.get $buf_b) (i32.const 7))) (i32.const 24)))))
    
    (i32.store (i32.const 0x30310)
      (i32.or
        (i32.or
          (i32.load8_u (i32.add (local.get $buf_b) (i32.const 8)))
          (i32.shl (i32.load8_u (i32.add (local.get $buf_b) (i32.const 9))) (i32.const 8)))
        (i32.or
          (i32.shl (i32.load8_u (i32.add (local.get $buf_b) (i32.const 10))) (i32.const 16))
          (i32.shl (i32.load8_u (i32.add (local.get $buf_b) (i32.const 11))) (i32.const 24)))))
    
    (i32.store (i32.const 0x30314)
      (i32.or
        (i32.or
          (i32.load8_u (i32.add (local.get $buf_b) (i32.const 12)))
          (i32.shl (i32.load8_u (i32.add (local.get $buf_b) (i32.const 13))) (i32.const 8)))
        (i32.or
          (i32.shl (i32.load8_u (i32.add (local.get $buf_b) (i32.const 14))) (i32.const 16))
          (i32.shl (i32.load8_u (i32.add (local.get $buf_b) (i32.const 15))) (i32.const 24)))))
    
    (global.set $result_ptr (i32.const 0x30300))
    (global.set $result_len (i32.const 24))
  )
)
