;; STUB — will be replaced in Task 5 with the real blake2b implementation.
;; Writes out_len zero bytes to offset 0 and returns (ptr=0, len=out_len).
;; Exists to let the test harness run end-to-end before the algorithm is written.
(module
  (memory (export "memory") 1)

  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $out_len i32)
    (local $i i32)

    ;; Read out_len = args[0] (as u8)
    (local.set $out_len (i32.load8_u (local.get $args_ptr)))

    ;; Trap if out_len == 0 or > 64
    (if (i32.or (i32.eqz (local.get $out_len))
                (i32.gt_u (local.get $out_len) (i32.const 64)))
      (then (unreachable)))

    ;; Zero the output buffer at offset 0..out_len
    (local.set $i (i32.const 0))
    (block $exit
      (loop $zero_loop
        (br_if $exit (i32.ge_u (local.get $i) (local.get $out_len)))
        (i32.store8 (local.get $i) (i32.const 0))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $zero_loop)))

    ;; Return (ptr=0) | (out_len << 32)
    (i64.or
      (i64.const 0)
      (i64.shl (i64.extend_i32_u (local.get $out_len)) (i64.const 32)))))
