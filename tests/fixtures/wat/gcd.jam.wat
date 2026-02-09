(module
  (memory 1)
  
  ;; Globals stored at 0x20000 + idx*4 by compiler
  
  ;; Compute GCD of two u32 arguments using Euclidean algorithm
  ;; Input: two u32 values (little-endian) at args_ptr
  ;; Output: one u32 at 0
  ;;
  ;; Register budget: 2 params + 2 locals = 4 (r9-r12)
  ;; Reuse $args_ptr as $a, $args_len as $b after loading args
  (func (export "main") (param $a i32) (param $b i32) (result i32 i32)
    (local $temp i32)
    (local $rem i32)
    
    ;; Load a = arg[0], b = arg[1]
    ;; $a (param 0) currently holds args_ptr
    ;; $b (param 1) currently holds args_len (unused)
    (local.set $b (i32.load (i32.add (local.get $a) (i32.const 4))))
    (local.set $a (i32.load (local.get $a)))
    
    ;; While b != 0
    (block $break
      (loop $continue
        ;; if b == 0, break
        (local.get $b)
        i32.const 0
        i32.eq
        br_if $break
        
        ;; temp = b
        (local.set $temp (local.get $b))
        
        ;; rem = a % b (must compute before overwriting b)
        (local.set $rem 
          (i32.rem_u (local.get $a) (local.get $b)))
        
        ;; b = rem
        (local.set $b (local.get $rem))
        
        ;; a = temp
        (local.set $a (local.get $temp))
        
        br $continue
      )
    )
    
    ;; Store result (a) at 0
    (i32.store (i32.const 0) (local.get $a))
    
    (i32.const 0)  ;; result_ptr
    (i32.const 4)  ;; result_len
  )
)
