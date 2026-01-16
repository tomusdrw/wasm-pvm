(module
  ;; Compute Fibonacci number (iterative approach)
  ;; fib(10) = 55
  (func (export "fib") (result i32)
    (local $n i32)
    (local $a i32)
    (local $b i32)
    (local $temp i32)
    (local $i i32)
    
    ;; Hardcoded n = 10
    i32.const 10
    local.set $n
    
    ;; Initialize a = 0, b = 1
    i32.const 0
    local.set $a
    
    i32.const 1
    local.set $b
    
    ;; Initialize counter i = 0
    i32.const 0
    local.set $i
    
    ;; Loop while i < n
    (block $break
      (loop $continue
        ;; Check if i >= n
        local.get $i
        local.get $n
        i32.ge_u
        br_if $break
        
        ;; temp = a + b
        local.get $a
        local.get $b
        i32.add
        local.set $temp
        
        ;; a = b
        local.get $b
        local.set $a
        
        ;; b = temp
        local.get $temp
        local.set $b
        
        ;; i++
        local.get $i
        i32.const 1
        i32.add
        local.set $i
        
        br $continue
      )
    )
    
    ;; Return a
    local.get $a
  )
)
