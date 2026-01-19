(module
  (memory 1)

  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))

  ;; Test i64 operations with constants
  ;; Input: operation selector (i32)
  ;; 0=div_u, 1=rem_u, 2=shl, 3=shr_u, 4=and, 5=or, 6=xor, 7=ge_u, 8=le_u
  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $op i32)
    (local $result i32)
    ;; Keep only 2 locals (plus 2 params = 4 total) to avoid spilling

    ;; Read operation selector
    (local.set $op (i32.load (local.get $args_ptr)))

    (if (i32.eq (local.get $op) (i32.const 0))
      (then
        ;; i64.div_u: 100 / 7 = 14
        (if (i64.eq (i64.div_u (i64.const 100) (i64.const 7)) (i64.const 14))
          (then (local.set $result (i32.const 14)))
          (else (local.set $result (i32.const 0)))
        )
      )
    )
    (if (i32.eq (local.get $op) (i32.const 1))
      (then
        ;; i64.rem_u: 100 % 7 = 2
        (if (i64.eq (i64.rem_u (i64.const 100) (i64.const 7)) (i64.const 2))
          (then (local.set $result (i32.const 2)))
          (else (local.set $result (i32.const 0)))
        )
      )
    )
    (if (i32.eq (local.get $op) (i32.const 2))
      (then
        ;; i64.shl: 0xFF << 4 = 0xFF0 = 4080
        (if (i64.eq (i64.shl (i64.const 0xFF) (i64.const 4)) (i64.const 4080))
          (then (local.set $result (i32.const 4080)))
          (else (local.set $result (i32.const 0)))
        )
      )
    )
    (if (i32.eq (local.get $op) (i32.const 3))
      (then
        ;; i64.shr_u: 0xFF00 >> 4 = 0xFF0 = 4080
        (if (i64.eq (i64.shr_u (i64.const 0xFF00) (i64.const 4)) (i64.const 4080))
          (then (local.set $result (i32.const 4080)))
          (else (local.set $result (i32.const 0)))
        )
      )
    )
    (if (i32.eq (local.get $op) (i32.const 4))
      (then
        ;; i64.and: 0xF0F0 & 0x0FF0 = 0x00F0 = 240
        (if (i64.eq (i64.and (i64.const 0xF0F0) (i64.const 0x0FF0)) (i64.const 240))
          (then (local.set $result (i32.const 240)))
          (else (local.set $result (i32.const 0)))
        )
      )
    )
    (if (i32.eq (local.get $op) (i32.const 5))
      (then
        ;; i64.or: 0xF0F0 | 0x0FF0 = 0xFFF0 = 65520
        (if (i64.eq (i64.or (i64.const 0xF0F0) (i64.const 0x0FF0)) (i64.const 65520))
          (then (local.set $result (i32.const 65520)))
          (else (local.set $result (i32.const 0)))
        )
      )
    )
    (if (i32.eq (local.get $op) (i32.const 6))
      (then
        ;; i64.xor: 0xF0F0 ^ 0x0FF0 = 0xFF00 = 65280
        (if (i64.eq (i64.xor (i64.const 0xF0F0) (i64.const 0x0FF0)) (i64.const 65280))
          (then (local.set $result (i32.const 65280)))
          (else (local.set $result (i32.const 0)))
        )
      )
    )
    (if (i32.eq (local.get $op) (i32.const 7))
      (then
        ;; i64.ge_u: 100 >= 50 = 1
        (if (i64.ge_u (i64.const 100) (i64.const 50))
          (then (local.set $result (i32.const 1)))
          (else (local.set $result (i32.const 0)))
        )
      )
    )
    (if (i32.eq (local.get $op) (i32.const 8))
      (then
        ;; i64.le_u: 50 <= 100 = 1
        (if (i64.le_u (i64.const 50) (i64.const 100))
          (then (local.set $result (i32.const 1)))
          (else (local.set $result (i32.const 0)))
        )
      )
    )

    ;; Store result
    (i32.store (i32.const 0) (local.get $result))

    ;; Set return value
    (global.set $result_ptr (i32.const 0))
    (global.set $result_len (i32.const 4))
  )
)
