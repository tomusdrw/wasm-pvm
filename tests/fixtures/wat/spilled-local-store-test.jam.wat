;; Test: spilled locals + call_indirect + i32.store offset
;; This mimics the exact pattern from nextSteps in anan-as:
;; 1. Set spilled local (skipBytes)
;; 2. call_indirect to execute instruction
;; 3. Use spilled local in PC update (i32.store offset=16)
;;
;; Expected results per step:
;; Step 0: PC update without call_indirect → 42 + (9+1) = 52
;; Step 1: Read spilled local 7 after call_indirect → 9
;; Step 2: PC update after call_indirect → 52 + (9+1) = 62
;; Step 3: call_indirect return value → 7
;; Step 4: Combined: store_initial + call_indirect + read_skipBytes + pc_update → 52 + (9+1) = 62

(module
  (memory 1)

  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))

  ;; Function table for call_indirect
  (table 2 funcref)
  (elem (i32.const 0) $return_seven $return_three)
  (type $exe_type (func (param i32 i32 i32 i32) (result i32)))
  (type $simple_type (func (result i32)))

  ;; Simple functions that return constants (simulates instruction execution)
  (func $return_seven (param i32) (param i32) (param i32) (param i32) (result i32)
    (i32.const 7)
  )
  (func $return_three (result i32)
    (i32.const 3)
  )

  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    ;; 2 params + 15 additional locals = 17 total (same as nextSteps)
    (local $step i32)       ;; local 2
    (local $obj_ptr i32)    ;; local 3
    (local $l4 i32)         ;; local 4 - spilled
    (local $l5 i32)         ;; local 5 - spilled
    (local $l6 i32)         ;; local 6 - spilled
    (local $l7 i32)         ;; local 7 - spilled (skipBytes equivalent)
    (local $l8 i32)         ;; local 8 - spilled
    (local $l9 i32)         ;; local 9 - spilled
    (local $l10 i32)        ;; local 10 - spilled
    (local $l11 i32)        ;; local 11 - spilled
    (local $l12 i32)        ;; local 12 - spilled
    (local $l13 i32)        ;; local 13 - spilled
    (local $l14 i32)        ;; local 14 - spilled
    (local $l15 i32)        ;; local 15 - spilled
    (local $l16 i32)        ;; local 16 - spilled

    ;; Read step from args
    (local.set $step (i32.load (local.get $args_ptr)))

    ;; Set up: use address 200 as our "object" base pointer
    (local.set $obj_ptr (i32.const 200))

    ;; Set spilled locals
    (local.set $l7 (i32.const 9))   ;; skipBytes
    (local.set $l4 (i32.const 111))
    (local.set $l5 (i32.const 222))
    (local.set $l6 (i32.const 333))
    (local.set $l8 (i32.const 444))

    ;; Store initial value: object.field16 = 42
    (i32.store offset=16 (local.get $obj_ptr) (i32.const 42))

    ;; Step 0: PC update WITHOUT call_indirect → should be 52
    (if (i32.eqz (local.get $step))
      (then
        ;; PC update pattern
        local.get $obj_ptr
        local.get $obj_ptr
        i32.load offset=16
        local.get $l7
        i32.const 1
        i32.add
        i32.add
        i32.store offset=16
        ;; Output
        (i32.store (i32.const 0) (i32.load offset=16 (local.get $obj_ptr)))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; Step 1: Read spilled local 7 AFTER call_indirect → should be 9
    (if (i32.eq (local.get $step) (i32.const 1))
      (then
        ;; Do a call_indirect (discard result)
        (drop (call_indirect (type $exe_type) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)))
        ;; Read spilled local
        (i32.store (i32.const 0) (local.get $l7))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; Step 2: call_indirect THEN PC update → should be 62 (42 + 10 from step 0 pattern + 10)
    ;; Actually step 0 already ran the first update, but each step is independent
    ;; So: initial 42, first PC update to 52, then call_indirect, then second PC update to 62
    (if (i32.eq (local.get $step) (i32.const 2))
      (then
        ;; First PC update (no call)
        local.get $obj_ptr
        local.get $obj_ptr
        i32.load offset=16
        local.get $l7
        i32.const 1
        i32.add
        i32.add
        i32.store offset=16
        ;; call_indirect
        (drop (call_indirect (type $exe_type) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)))
        ;; Second PC update (after call)
        local.get $obj_ptr
        local.get $obj_ptr
        i32.load offset=16
        local.get $l7
        i32.const 1
        i32.add
        i32.add
        i32.store offset=16
        ;; Output
        (i32.store (i32.const 0) (i32.load offset=16 (local.get $obj_ptr)))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; Step 3: call_indirect return value → should be 7
    (if (i32.eq (local.get $step) (i32.const 3))
      (then
        (i32.store (i32.const 0) (call_indirect (type $exe_type) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; Step 4: The FULL pattern: set skipBytes, call_indirect, THEN PC update
    ;; This is the exact order from nextSteps: compute skipBytes → call → use skipBytes
    (if (i32.eq (local.get $step) (i32.const 4))
      (then
        ;; skipBytes is already set to 9 (local 7)
        ;; call_indirect (instruction execution)
        (drop (call_indirect (type $exe_type) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)))
        ;; NOW do the PC update using skipBytes (local 7) - this is what might fail
        local.get $obj_ptr
        local.get $obj_ptr
        i32.load offset=16
        local.get $l7
        i32.const 1
        i32.add
        i32.add
        i32.store offset=16
        ;; Output
        (i32.store (i32.const 0) (i32.load offset=16 (local.get $obj_ptr)))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; Default: return 0
    (i32.store (i32.const 0) (i32.const 0))
    (global.set $result_ptr (i32.const 0))
    (global.set $result_len (i32.const 4))
  )
)
