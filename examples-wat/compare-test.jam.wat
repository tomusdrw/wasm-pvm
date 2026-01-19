;; Test all comparison operators to verify correctness
;; Input: none (hardcoded test values)
;; Output: 8 bytes - each byte is 0 or 1 for each test
;;   byte 0: 3 < 5 should be 1
;;   byte 1: 5 < 3 should be 0
;;   byte 2: 3 > 5 should be 0
;;   byte 3: 5 > 3 should be 1
;;   byte 4: 3 <= 5 should be 1
;;   byte 5: 5 <= 3 should be 0
;;   byte 6: 3 >= 5 should be 0
;;   byte 7: 5 >= 3 should be 1

(module
  (memory 1)
  
  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $result_addr i32)
    
    ;; Result address
    (local.set $result_addr (i32.const 0))
    
    ;; Test 0: 3 < 5 should be 1
    (i32.store8 (local.get $result_addr)
      (i32.lt_s (i32.const 3) (i32.const 5)))
    
    ;; Test 1: 5 < 3 should be 0
    (i32.store8 (i32.add (local.get $result_addr) (i32.const 1))
      (i32.lt_s (i32.const 5) (i32.const 3)))
    
    ;; Test 2: 3 > 5 should be 0
    (i32.store8 (i32.add (local.get $result_addr) (i32.const 2))
      (i32.gt_s (i32.const 3) (i32.const 5)))
    
    ;; Test 3: 5 > 3 should be 1
    (i32.store8 (i32.add (local.get $result_addr) (i32.const 3))
      (i32.gt_s (i32.const 5) (i32.const 3)))
    
    ;; Test 4: 3 <= 5 should be 1
    (i32.store8 (i32.add (local.get $result_addr) (i32.const 4))
      (i32.le_s (i32.const 3) (i32.const 5)))
    
    ;; Test 5: 5 <= 3 should be 0
    (i32.store8 (i32.add (local.get $result_addr) (i32.const 5))
      (i32.le_s (i32.const 5) (i32.const 3)))
    
    ;; Test 6: 3 >= 5 should be 0
    (i32.store8 (i32.add (local.get $result_addr) (i32.const 6))
      (i32.ge_s (i32.const 3) (i32.const 5)))
    
    ;; Test 7: 5 >= 3 should be 1
    (i32.store8 (i32.add (local.get $result_addr) (i32.const 7))
      (i32.ge_s (i32.const 5) (i32.const 3)))
    
    ;; Set result
    (global.set $result_ptr (local.get $result_addr))
    (global.set $result_len (i32.const 8))
  )
)
