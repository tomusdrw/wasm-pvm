(module
  ;; Compute factorial of a hardcoded number
  ;; factorial(5) = 120
  (func (export "factorial") (result i32)
    (local $n i32)
    (local $result i32)
    (local $i i32)
    
    ;; Hardcoded n = 5
    i32.const 5
    local.set $n
    
    ;; Initialize result = 1
    i32.const 1
    local.set $result
    
    ;; Initialize counter i = 1
    i32.const 1
    local.set $i
    
    ;; Loop while i <= n
    (block $break
      (loop $continue
        ;; Check if i > n
        local.get $i
        local.get $n
        i32.gt_u
        br_if $break
        
        ;; result *= i
        local.get $result
        local.get $i
        i32.mul
        local.set $result
        
        ;; i++
        local.get $i
        i32.const 1
        i32.add
        local.set $i
        
        br $continue
      )
    )
    
    ;; Return result
    local.get $result
  )
)
