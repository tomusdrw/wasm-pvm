(module
  (memory 1)

  ;; Test memory.copy with word-sized optimization
  ;; Tests both forward and backward copies >= 8 bytes to exercise word loops.

  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i32 i32)
    (local $test_case i32)
    (local $result i32)

    ;; Load test case from args
    (local.set $test_case
      (i32.load (local.get $args_ptr))
    )

    ;; Initialize memory with pattern: 0x01..0x20 (32 bytes)
    (i32.store8 (i32.const 0x00) (i32.const 0x01))
    (i32.store8 (i32.const 0x01) (i32.const 0x02))
    (i32.store8 (i32.const 0x02) (i32.const 0x03))
    (i32.store8 (i32.const 0x03) (i32.const 0x04))
    (i32.store8 (i32.const 0x04) (i32.const 0x05))
    (i32.store8 (i32.const 0x05) (i32.const 0x06))
    (i32.store8 (i32.const 0x06) (i32.const 0x07))
    (i32.store8 (i32.const 0x07) (i32.const 0x08))
    (i32.store8 (i32.const 0x08) (i32.const 0x09))
    (i32.store8 (i32.const 0x09) (i32.const 0x0A))
    (i32.store8 (i32.const 0x0A) (i32.const 0x0B))
    (i32.store8 (i32.const 0x0B) (i32.const 0x0C))
    (i32.store8 (i32.const 0x0C) (i32.const 0x0D))
    (i32.store8 (i32.const 0x0D) (i32.const 0x0E))
    (i32.store8 (i32.const 0x0E) (i32.const 0x0F))
    (i32.store8 (i32.const 0x0F) (i32.const 0x10))
    (i32.store8 (i32.const 0x10) (i32.const 0x11))
    (i32.store8 (i32.const 0x11) (i32.const 0x12))
    (i32.store8 (i32.const 0x12) (i32.const 0x13))
    (i32.store8 (i32.const 0x13) (i32.const 0x14))
    (i32.store8 (i32.const 0x14) (i32.const 0x15))
    (i32.store8 (i32.const 0x15) (i32.const 0x16))
    (i32.store8 (i32.const 0x16) (i32.const 0x17))
    (i32.store8 (i32.const 0x17) (i32.const 0x18))

    ;; Branch on test case
    (if (i32.eq (local.get $test_case) (i32.const 0))
      (then
        ;; Test 0: Forward non-overlapping copy, 16 bytes (2 full words).
        ;; Copy 16 bytes from src=0 to dst=0x40.
        ;; Expected at dst: [01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F 10]
        ;; Read first 4 bytes at 0x40: 0x04030201
        (memory.copy (i32.const 0x40) (i32.const 0x0) (i32.const 16))
        (local.set $result (i32.load (i32.const 0x40)))
      )
      (else
        (if (i32.eq (local.get $test_case) (i32.const 1))
          (then
            ;; Test 1: Forward non-overlapping copy, 10 bytes (1 word + 2 byte tail).
            ;; Copy 10 bytes from src=0 to dst=0x40.
            ;; Expected at dst: [01 02 03 04 05 06 07 08 09 0A]
            ;; Read last 4 bytes at 0x46: 0x0A090807  (wait... 0x46 = dst+6)
            ;; Actually read 4 at 0x44: [05 06 07 08] = 0x08070605
            (memory.copy (i32.const 0x40) (i32.const 0x0) (i32.const 10))
            (local.set $result (i32.load (i32.const 0x44)))
          )
          (else
            (if (i32.eq (local.get $test_case) (i32.const 2))
              (then
                ;; Test 2: Backward overlapping copy, 10 bytes (dst > src).
                ;; Copy 10 bytes from src=0 to dst=4. Overlap at [4..9].
                ;; Before: [01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E ...]
                ;; After:  [01 02 03 04 01 02 03 04 05 06 07 08 09 0A ...]
                ;; Read at dst=4: [01 02 03 04] = 0x04030201
                (memory.copy (i32.const 0x4) (i32.const 0x0) (i32.const 10))
                (local.set $result (i32.load (i32.const 0x4)))
              )
              (else
                (if (i32.eq (local.get $test_case) (i32.const 3))
                  (then
                    ;; Test 3: Backward overlapping copy, 10 bytes, read later part.
                    ;; Same copy as test 2. Read at dst+6=0xA: [07 08 09 0A] = 0x0A090807
                    (memory.copy (i32.const 0x4) (i32.const 0x0) (i32.const 10))
                    (local.set $result (i32.load (i32.const 0xA)))
                  )
                  (else
                    (if (i32.eq (local.get $test_case) (i32.const 4))
                      (then
                        ;; Test 4: Backward overlapping copy, 16 bytes (2 full words, no tail).
                        ;; Copy 16 bytes from src=0 to dst=4.
                        ;; Before: [01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F 10 11 12 13 14]
                        ;; After:  [01 02 03 04 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F 10]
                        ;; Read at dst=4: [01 02 03 04] = 0x04030201
                        (memory.copy (i32.const 0x4) (i32.const 0x0) (i32.const 16))
                        (local.set $result (i32.load (i32.const 0x4)))
                      )
                      (else
                        (if (i32.eq (local.get $test_case) (i32.const 5))
                          (then
                            ;; Test 5: Backward overlapping copy, 16 bytes, read later part.
                            ;; Same copy as test 4. Read at dst+12=0x10: [0D 0E 0F 10] = 0x100F0E0D
                            (memory.copy (i32.const 0x4) (i32.const 0x0) (i32.const 16))
                            (local.set $result (i32.load (i32.const 0x10)))
                          )
                          (else
                            (if (i32.eq (local.get $test_case) (i32.const 6))
                              (then
                                ;; Test 6: memory.fill with 16 bytes (word-sized fill test).
                                ;; Fill 16 bytes at addr 0x40 with value 0xAB.
                                ;; Expected: [AB AB AB AB] = 0xABABABAB
                                (memory.fill (i32.const 0x40) (i32.const 0xAB) (i32.const 16))
                                (local.set $result (i32.load (i32.const 0x40)))
                              )
                              (else
                                (if (i32.eq (local.get $test_case) (i32.const 7))
                                  (then
                                    ;; Test 7: memory.copy with len=0 (no-op).
                                    ;; Memory at 0x40 should remain 0.
                                    (memory.copy (i32.const 0x40) (i32.const 0x0) (i32.const 0))
                                    (local.set $result (i32.load (i32.const 0x40)))
                                  )
                                  (else
                                    (if (i32.eq (local.get $test_case) (i32.const 8))
                                      (then
                                        ;; Test 8: memory.fill with len=0 (no-op).
                                        ;; Memory at 0x40 should remain 0.
                                        (memory.fill (i32.const 0x40) (i32.const 0xFF) (i32.const 0))
                                        (local.set $result (i32.load (i32.const 0x40)))
                                      )
                                      (else
                                        (if (i32.eq (local.get $test_case) (i32.const 9))
                                          (then
                                            ;; Test 9: memory.copy with len=3 (pure byte tail, no word loop).
                                            ;; Copy 3 bytes from src=0 to dst=0x40.
                                            ;; Expected at 0x40: [01 02 03 00] = 0x00030201
                                            (memory.copy (i32.const 0x40) (i32.const 0x0) (i32.const 3))
                                            (local.set $result (i32.load (i32.const 0x40)))
                                          )
                                          (else
                                            (if (i32.eq (local.get $test_case) (i32.const 10))
                                              (then
                                                ;; Test 10: memory.fill with len=5 (pure byte tail, no word loop).
                                                ;; Fill 5 bytes at 0x40 with 0x42.
                                                ;; Expected at 0x40: [42 42 42 42] = 0x42424242
                                                (memory.fill (i32.const 0x40) (i32.const 0x42) (i32.const 5))
                                                (local.set $result (i32.load (i32.const 0x40)))
                                              )
                                              (else
                                                ;; Test 11: memory.fill with val > 0xFF (masking test).
                                                ;; WASM spec: fill uses val & 0xFF. So 0x1AB -> 0xAB.
                                                ;; Fill 8 bytes at 0x40 with 0x1AB (should use 0xAB).
                                                ;; Expected: [AB AB AB AB] = 0xABABABAB
                                                (memory.fill (i32.const 0x40) (i32.const 0x1AB) (i32.const 8))
                                                (local.set $result (i32.load (i32.const 0x40)))
                                              )
                                            )
                                          )
                                        )
                                      )
                                    )
                                  )
                                )
                              )
                            )
                          )
                        )
                      )
                    )
                  )
                )
              )
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
