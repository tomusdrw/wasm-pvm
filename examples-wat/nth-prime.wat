(module
  ;; Find the nth prime number
  ;; Finds the 5th prime (which is 11)
  (func $isPrime (param $num i32) (result i32)
    (local $i i32)
    
    ;; Check if num <= 1
    local.get $num
    i32.const 1
    i32.le_s
    (if (result i32)
      (then
        i32.const 0
        return
      )
      (else
        nop
      )
    )
    
    ;; Check if num == 2
    local.get $num
    i32.const 2
    i32.eq
    (if (result i32)
      (then
        i32.const 1
        return
      )
      (else
        nop
      )
    )
    
    ;; Check if even
    local.get $num
    i32.const 2
    i32.rem_u
    i32.const 0
    i32.eq
    (if (result i32)
      (then
        i32.const 0
        return
      )
      (else
        nop
      )
    )
    
    ;; Check odd divisors
    i32.const 3
    local.set $i
    
    (block $break
      (loop $continue
        local.get $i
        local.get $i
        i32.mul
        local.get $num
        i32.gt_u
        br_if $break
        
        local.get $num
        local.get $i
        i32.rem_u
        i32.const 0
        i32.eq
        (if (result i32)
          (then
            i32.const 0
            return
          )
          (else
            nop
          )
        )
        
        local.get $i
        i32.const 2
        i32.add
        local.set $i
        
        br $continue
      )
    )
    
    i32.const 1
  )
  
  (func (export "nthPrime") (result i32)
    (local $n i32)
    (local $count i32)
    (local $candidate i32)
    
    ;; Hardcoded n = 5 (find 5th prime)
    i32.const 5
    local.set $n
    
    ;; Initialize count = 0
    i32.const 0
    local.set $count
    
    ;; Start checking from 2
    i32.const 2
    local.set $candidate
    
    (block $break
      (loop $continue
        ;; Check if count >= n
        local.get $count
        local.get $n
        i32.ge_u
        br_if $break
        
        ;; Check if candidate is prime
        local.get $candidate
        call $isPrime
        (if
          (then
            ;; Increment count
            local.get $count
            i32.const 1
            i32.add
            local.set $count
          )
        )
        
        ;; Move to next candidate
        local.get $candidate
        i32.const 1
        i32.add
        local.set $candidate
        
        br $continue
      )
    )
    
    ;; Return the last candidate checked (the nth prime)
    local.get $candidate
    i32.const 1
    i32.sub
  )
)
