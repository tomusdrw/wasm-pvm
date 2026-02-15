(module
  (memory 1)

  ;; Test memory.copy with overlapping regions (memmove semantics)
  ;; Tests that overlapping copies preserve source data correctly.

  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (local $test_case i32)
    (local $result i32)

    ;; Load test case from args
    (local.set $test_case
      (i32.load (local.get $args_ptr))
    )

    ;; Initialize memory with pattern: 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08
    (i32.store8 (i32.const 0x0) (i32.const 0x01))
    (i32.store8 (i32.const 0x1) (i32.const 0x02))
    (i32.store8 (i32.const 0x2) (i32.const 0x03))
    (i32.store8 (i32.const 0x3) (i32.const 0x04))
    (i32.store8 (i32.const 0x4) (i32.const 0x05))
    (i32.store8 (i32.const 0x5) (i32.const 0x06))
    (i32.store8 (i32.const 0x6) (i32.const 0x07))
    (i32.store8 (i32.const 0x7) (i32.const 0x08))

    ;; Branch on test case
    (if (i32.eq (local.get $test_case) (i32.const 0))
      (then
        ;; Test 0: Overlapping copy, dst > src (requires backward copy / memmove)
        ;; Copy 4 bytes from src=2 to dst=4
        ;; src range: [2,5], dst range: [4,7], overlap at [4,5]
        ;; Before: [01 02 03 04 05 06 07 08]
        ;; After:  [01 02 03 04 03 04 05 06]
        ;; Without memmove (forward copy bug): [01 02 03 04 03 04 03 04]
        (memory.copy
          (i32.const 0x4)  ;; dest
          (i32.const 0x2)  ;; src
          (i32.const 4)    ;; len
        )

        ;; Read 4 bytes from addr 4 to verify the overlapping region
        ;; Expected: [03 04 05 06] = 0x06050403 (little-endian)
        (local.set $result
          (i32.load (i32.const 0x4))
        )
      )
      (else
        (if (i32.eq (local.get $test_case) (i32.const 1))
          (then
            ;; Test 1: Overlapping copy, dst < src (forward copy is correct)
            ;; Copy 4 bytes from src=4 to dst=2
            ;; src range: [4,7], dst range: [2,5], overlap at [4,5]
            ;; Before: [01 02 03 04 05 06 07 08]
            ;; After:  [01 02 05 06 07 08 07 08]
            (memory.copy
              (i32.const 0x2)  ;; dest
              (i32.const 0x4)  ;; src
              (i32.const 4)    ;; len
            )

            ;; Read 4 bytes from addr 2
            ;; Expected: [05 06 07 08] = 0x08070605 (little-endian)
            (local.set $result
              (i32.load (i32.const 0x2))
            )
          )
          (else
            ;; Test 2: Non-overlapping copy
            ;; Copy 4 bytes from src=0 to dst=8
            ;; Before: [01 02 03 04 05 06 07 08 00 00 00 00]
            ;; After:  [01 02 03 04 05 06 07 08 01 02 03 04]
            (memory.copy
              (i32.const 0x8)  ;; dest
              (i32.const 0x0)  ;; src
              (i32.const 4)    ;; len
            )

            ;; Read 4 bytes from addr 8
            ;; Expected: [01 02 03 04] = 0x04030201 (little-endian)
            (local.set $result
              (i32.load (i32.const 0x8))
            )
          )
        )
      )
    )

    ;; Return result at wasm-relative address 0x100
    (i32.store (i32.const 0x100) (local.get $result))

    ;; Return (ptr, len) - wasm-relative address
    (i32.const 0x100)
    (i32.const 4)
  )
)
