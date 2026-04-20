;; STUB — will be replaced in Task 3 with the real SHA-512 implementation.
;; Writes 64 zero bytes to offset 0 and returns (ptr=0, len=64).
;; Exists to let the test harness run end-to-end before the algorithm is written.
(module
  (memory (export "memory") 1)

  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $i i32)

    ;; Zero the 64-byte output buffer at offset 0..64.
    (local.set $i (i32.const 0))
    (block $exit
      (loop $zero_loop
        (br_if $exit (i32.ge_u (local.get $i) (i32.const 64)))
        (i32.store8 (local.get $i) (i32.const 0))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $zero_loop)))

    ;; Return (ptr=0) | (64 << 32) = 0x00000040_00000000.
    (i64.const 274877906944)))
