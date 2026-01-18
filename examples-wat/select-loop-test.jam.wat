;; Test select inside a loop (similar to life.wasm pattern)
;; Output: sum of selected values

(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $i i32)
    (local $sum i32)
    (local $prev i32)
    
    ;; for (i = 0; i < 16; i++)
    (local.set $i (i32.const 0))
    (block $exit
      (loop $loop
        (br_if $exit (i32.ge_s (local.get $i) (i32.const 16)))
        
        ;; prev = (i == 0) ? 15 : (i - 1)
        ;; select(val1, val2, c) returns val1 if c!=0, val2 if c==0
        ;; So: select(15, i-1, i==0)
        ;;   i==0 → c=1 → return 15 ✓
        ;;   i!=0 → c=0 → return i-1 ✓
        (local.set $prev
          (select
            (i32.const 15)                           ;; val1: returned if cond!=0
            (i32.sub (local.get $i) (i32.const 1))   ;; val2: returned if cond==0
            (i32.eqz (local.get $i))                 ;; condition: i==0
          )
        )
        
        ;; sum += prev
        (local.set $sum (i32.add (local.get $sum) (local.get $prev)))
        
        ;; i++
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $loop)
      )
    )
    
    ;; Expected: 15 + 0 + 1 + 2 + ... + 14 = 15 + 105 = 120
    (i32.store (i32.const 0x30100) (local.get $sum))
    (global.set $result_ptr (i32.const 0x30100))
    (global.set $result_len (i32.const 4))
  )
)
