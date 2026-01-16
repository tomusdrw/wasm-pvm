(module
  ;; Compute GCD of two hardcoded numbers using Euclidean algorithm
  ;; gcd(48, 18) = 6
  (func (export "gcd") (result i32)
    (local $a i32)
    (local $b i32)
    (local $temp i32)
    
    ;; Hardcoded a = 48, b = 18
    i32.const 48
    local.set $a
    
    i32.const 18
    local.set $b
    
    ;; While b != 0
    (block $break
      (loop $continue
        local.get $b
        i32.const 0
        i32.eq
        br_if $break
        
        ;; temp = b
        local.get $b
        local.set $temp
        
        ;; b = a % b
        local.get $a
        local.get $b
        i32.rem_u
        local.set $b
        
        ;; a = temp
        local.get $temp
        local.set $a
        
        br $continue
      )
    )
    
    ;; Return a
    local.get $a
  )
)
