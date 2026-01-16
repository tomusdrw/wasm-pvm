(module
  ;; Check if a hardcoded number is prime
  ;; Returns 1 if prime, 0 if not prime
  ;; Checks if 17 is prime (it is)
  (func (export "isPrime") (result i32)
    (local $n i32)
    (local $i i32)
    
    ;; Hardcoded n = 17
    i32.const 17
    local.set $n
    
    ;; Check if n <= 1
    local.get $n
    i32.const 1
    i32.le_s
    (if (result i32)
      (then
        i32.const 0  ;; Not prime
        return
      )
      (else
        nop
      )
    )
    
    ;; Check if n == 2
    local.get $n
    i32.const 2
    i32.eq
    (if (result i32)
      (then
        i32.const 1  ;; Prime
        return
      )
      (else
        nop
      )
    )
    
    ;; Check if n is even
    local.get $n
    i32.const 2
    i32.rem_u
    i32.const 0
    i32.eq
    (if (result i32)
      (then
        i32.const 0  ;; Not prime
        return
      )
      (else
        nop
      )
    )
    
    ;; Check divisibility from 3 to sqrt(n)
    i32.const 3
    local.set $i
    
    (block $break
      (loop $continue
        ;; Check if i * i > n
        local.get $i
        local.get $i
        i32.mul
        local.get $n
        i32.gt_u
        br_if $break
        
        ;; Check if n % i == 0
        local.get $n
        local.get $i
        i32.rem_u
        i32.const 0
        i32.eq
        (if (result i32)
          (then
            i32.const 0  ;; Not prime
            return
          )
          (else
            nop
          )
        )
        
        ;; i += 2 (check only odd numbers)
        local.get $i
        i32.const 2
        i32.add
        local.set $i
        
        br $continue
      )
    )
    
    ;; If we get here, n is prime
    i32.const 1
  )
)
