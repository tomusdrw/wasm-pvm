(module
  (memory 1)
  
  ;; Test memory.copy with overlapping regions
  ;; This tests both forward and backward copy scenarios
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (local $test_case i32)
    (local $result i32)
    
    ;; Load test case from args (0 = forward overlap, 1 = backward overlap)
    (local.set $test_case
      (i32.load (local.get $args_ptr))
    )
    
    ;; Initialize memory with pattern: 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08
    (i32.store8 (i32.const 0x50000) (i32.const 0x01))
    (i32.store8 (i32.const 0x50001) (i32.const 0x02))
    (i32.store8 (i32.const 0x50002) (i32.const 0x03))
    (i32.store8 (i32.const 0x50003) (i32.const 0x04))
    (i32.store8 (i32.const 0x50004) (i32.const 0x05))
    (i32.store8 (i32.const 0x50005) (i32.const 0x06))
    (i32.store8 (i32.const 0x50006) (i32.const 0x07))
    (i32.store8 (i32.const 0x50007) (i32.const 0x08))
    
    ;; Branch on test case
    (if (i32.eq (local.get $test_case) (i32.const 0))
      (then
        ;; Test 0: Forward overlap
        ;; Copy 4 bytes from 0x50000 to 0x50002
        ;; Before: [01 02 03 04 05 06 07 08]
        ;; After:  [01 02 01 02 05 06 07 08]
        (memory.copy
          (i32.const 0x50002)  ;; dest
          (i32.const 0x50000)  ;; src
          (i32.const 4)        ;; len
        )
        
        ;; Verify result: load 4 bytes from 0x50000 and check
        ;; Expected: 0x02010201 (little-endian: 01 02 01 02)
        (local.set $result
          (i32.load (i32.const 0x50000))
        )
      )
      (else
        ;; Test 1: Backward overlap
        ;; Copy 4 bytes from 0x50004 to 0x50002
        ;; Before: [01 02 03 04 05 06 07 08]
        ;; After:  [01 02 05 06 07 06 07 08]
        (memory.copy
          (i32.const 0x50002)  ;; dest
          (i32.const 0x50004)  ;; src
          (i32.const 4)        ;; len
        )
        
        ;; Verify result: load 4 bytes from 0x50002 and check
        ;; Expected: 0x07060605 (little-endian: 05 06 07 06)
        (local.set $result
          (i32.load (i32.const 0x50002))
        )
      )
    )
    
    ;; Return result in heap at 0x30100
    (i32.store (i32.const 0x30100) (local.get $result))
    
    ;; Return (ptr, len)
    (i32.const 0x30100)
    (i32.const 4)
  )
)
