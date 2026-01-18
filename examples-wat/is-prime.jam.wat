(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  ;; Check if a number is prime
  ;; Register budget: 2 params + 2 locals = 4 (r9-r12)
  (func (export "main") (param $n i32) (param $i i32)
    (local $result i32)
    (local $temp i32)
    
    ;; Load n from args
    (local.set $n (i32.load (local.get $n)))
    
    ;; Default result = 1 (prime)
    (local.set $result (i32.const 1))
    
    ;; If n <= 1 -> not prime
    (block $done_le1
      (br_if $done_le1
        (i32.eqz (i32.le_s (local.get $n) (i32.const 1))))
      (local.set $result (i32.const 0))
    )
    
    ;; If n > 2 and n is even -> not prime  
    (block $done_even
      (br_if $done_even
        (i32.le_u (local.get $n) (i32.const 2)))
      (br_if $done_even
        (i32.eqz (i32.eq (i32.rem_u (local.get $n) (i32.const 2)) (i32.const 0))))
      (local.set $result (i32.const 0))
    )
    
    ;; Check divisibility from 3 to sqrt(n), step by 2
    (local.set $i (i32.const 3))
    
    (block $break
      (loop $continue
        ;; If i * i > n, we're done
        (local.set $temp (i32.mul (local.get $i) (local.get $i)))
        (br_if $break (i32.gt_u (local.get $temp) (local.get $n)))
        
        ;; If n % i == 0, not prime and break
        (block $not_divisible
          (br_if $not_divisible
            (i32.eqz (i32.eq (i32.rem_u (local.get $n) (local.get $i)) (i32.const 0))))
          (local.set $result (i32.const 0))
          (br $break)
        )
        
        ;; i += 2
        (local.set $i (i32.add (local.get $i) (i32.const 2)))
        
        br $continue
      )
    )
    
    ;; Store result at 0x30100
    (i32.store (i32.const 0x30100) (local.get $result))
    
    (global.set $result_ptr (i32.const 0x30100))
    (global.set $result_len (i32.const 4))
  )
)
