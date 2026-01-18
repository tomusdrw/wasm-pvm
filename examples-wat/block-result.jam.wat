(module
  (memory 1)

  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))

  ;; Test block result values
  ;; Input: operation selector
  ;; 0 = simple block result
  ;; 1 = block with br and result
  ;; 2 = if with result
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $op i32)
    (local $result i32)

    (local.set $op (i32.load (local.get $args_ptr)))

    (if (i32.eq (local.get $op) (i32.const 0))
      (then
        ;; Test 1: Simple block with result = 42
        (local.set $result
          (block (result i32)
            (i32.const 42)
          )
        )
      )
    )

    (if (i32.eq (local.get $op) (i32.const 1))
      (then
        ;; Test 2: Block with br and result = 100
        (local.set $result
          (block (result i32)
            (i32.const 100)
            (br 0)
            (i32.const 999)  ;; Should not reach here
          )
        )
      )
    )

    (if (i32.eq (local.get $op) (i32.const 2))
      (then
        ;; Test 3: If with result
        (local.set $result
          (if (result i32) (i32.const 1)
            (then (i32.const 200))
            (else (i32.const 300))
          )
        )
      )
    )

    (if (i32.eq (local.get $op) (i32.const 3))
      (then
        ;; Test 4: If-else with result (else branch)
        (local.set $result
          (if (result i32) (i32.const 0)
            (then (i32.const 200))
            (else (i32.const 300))
          )
        )
      )
    )

    ;; Store result
    (i32.store (i32.const 0x30100) (local.get $result))

    ;; Set return value
    (global.set $result_ptr (i32.const 0x30100))
    (global.set $result_len (i32.const 4))
  )
)
