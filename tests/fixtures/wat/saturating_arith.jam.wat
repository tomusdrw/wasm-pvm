(module
  (memory 1)
  ;; Args layout:
  ;;   bytes  0..3  : op selector (u32 LE)
  ;;     op = intrinsic_idx * 4 + width_idx
  ;;       intrinsic_idx: 0=uadd 1=usub 2=sadd 3=ssub
  ;;       width_idx:     0=i8   1=i16   2=i32   3=i64
  ;;     ⇒ ops 0..15
  ;;   bytes  4..11 : operand a (low N bytes used)
  ;;   bytes 12..19 : operand b
  ;; Result: written at address 0; len = width-bytes (1/2/4/8).
  ;; Returns packed (ptr=0) | (len << 32).
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $op i32)
    (local $a32 i32)
    (local $b32 i32)
    (local $s32 i32)
    (local $a64 i64)
    (local $b64 i64)
    (local $s64 i64)

    (local.set $op (i32.load (local.get $args_ptr)))

    ;; -------------------------------------------------------------------
    ;; op 0: uadd.sat.i8
    ;; -------------------------------------------------------------------
    (if (i32.eq (local.get $op) (i32.const 0)) (then
      (local.set $a32 (i32.load8_u (i32.add (local.get $args_ptr) (i32.const 4))))
      (local.set $b32 (i32.load8_u (i32.add (local.get $args_ptr) (i32.const 12))))
      (local.set $s32 (i32.and (i32.add (local.get $a32) (local.get $b32)) (i32.const 0xFF)))
      (i32.store8 (i32.const 0)
        (select
          (i32.const 0xFF)
          (local.get $s32)
          (i32.lt_u (local.get $s32) (local.get $a32))))
      (return (i64.const 4294967296))))

    ;; op 1: uadd.sat.i16
    (if (i32.eq (local.get $op) (i32.const 1)) (then
      (local.set $a32 (i32.load16_u (i32.add (local.get $args_ptr) (i32.const 4))))
      (local.set $b32 (i32.load16_u (i32.add (local.get $args_ptr) (i32.const 12))))
      (local.set $s32 (i32.and (i32.add (local.get $a32) (local.get $b32)) (i32.const 0xFFFF)))
      (i32.store16 (i32.const 0)
        (select
          (i32.const 0xFFFF)
          (local.get $s32)
          (i32.lt_u (local.get $s32) (local.get $a32))))
      (return (i64.const 8589934592))))

    ;; op 2: uadd.sat.i32
    (if (i32.eq (local.get $op) (i32.const 2)) (then
      (local.set $a32 (i32.load (i32.add (local.get $args_ptr) (i32.const 4))))
      (local.set $b32 (i32.load (i32.add (local.get $args_ptr) (i32.const 12))))
      (local.set $s32 (i32.add (local.get $a32) (local.get $b32)))
      (i32.store (i32.const 0)
        (select
          (i32.const -1)
          (local.get $s32)
          (i32.lt_u (local.get $s32) (local.get $a32))))
      (return (i64.const 17179869184))))

    ;; op 3: uadd.sat.i64
    (if (i32.eq (local.get $op) (i32.const 3)) (then
      (local.set $a64 (i64.load (i32.add (local.get $args_ptr) (i32.const 4))))
      (local.set $b64 (i64.load (i32.add (local.get $args_ptr) (i32.const 12))))
      (local.set $s64 (i64.add (local.get $a64) (local.get $b64)))
      (i64.store (i32.const 0)
        (select
          (i64.const -1)
          (local.get $s64)
          (i64.lt_u (local.get $s64) (local.get $a64))))
      (return (i64.const 34359738368))))

    ;; -------------------------------------------------------------------
    ;; op 4: usub.sat.i8
    ;; -------------------------------------------------------------------
    (if (i32.eq (local.get $op) (i32.const 4)) (then
      (local.set $a32 (i32.load8_u (i32.add (local.get $args_ptr) (i32.const 4))))
      (local.set $b32 (i32.load8_u (i32.add (local.get $args_ptr) (i32.const 12))))
      (i32.store8 (i32.const 0)
        (select
          (i32.sub (local.get $a32) (local.get $b32))
          (i32.const 0)
          (i32.gt_u (local.get $a32) (local.get $b32))))
      (return (i64.const 4294967296))))

    ;; op 5: usub.sat.i16
    (if (i32.eq (local.get $op) (i32.const 5)) (then
      (local.set $a32 (i32.load16_u (i32.add (local.get $args_ptr) (i32.const 4))))
      (local.set $b32 (i32.load16_u (i32.add (local.get $args_ptr) (i32.const 12))))
      (i32.store16 (i32.const 0)
        (select
          (i32.sub (local.get $a32) (local.get $b32))
          (i32.const 0)
          (i32.gt_u (local.get $a32) (local.get $b32))))
      (return (i64.const 8589934592))))

    ;; op 6: usub.sat.i32
    (if (i32.eq (local.get $op) (i32.const 6)) (then
      (local.set $a32 (i32.load (i32.add (local.get $args_ptr) (i32.const 4))))
      (local.set $b32 (i32.load (i32.add (local.get $args_ptr) (i32.const 12))))
      (i32.store (i32.const 0)
        (select
          (i32.sub (local.get $a32) (local.get $b32))
          (i32.const 0)
          (i32.gt_u (local.get $a32) (local.get $b32))))
      (return (i64.const 17179869184))))

    ;; op 7: usub.sat.i64
    (if (i32.eq (local.get $op) (i32.const 7)) (then
      (local.set $a64 (i64.load (i32.add (local.get $args_ptr) (i32.const 4))))
      (local.set $b64 (i64.load (i32.add (local.get $args_ptr) (i32.const 12))))
      (i64.store (i32.const 0)
        (select
          (i64.sub (local.get $a64) (local.get $b64))
          (i64.const 0)
          (i64.gt_u (local.get $a64) (local.get $b64))))
      (return (i64.const 34359738368))))

    ;; -------------------------------------------------------------------
    ;; op 8: sadd.sat.i8
    ;; clamp(s, -128, 127) via two selects:
    ;;   inner = (s > -128) ? s : -128    ; max(s, -128)
    ;;   outer = (s < 127)  ? inner : 127 ; min(inner, 127)
    ;; -------------------------------------------------------------------
    (if (i32.eq (local.get $op) (i32.const 8)) (then
      (local.set $a32 (i32.load8_s (i32.add (local.get $args_ptr) (i32.const 4))))
      (local.set $b32 (i32.load8_s (i32.add (local.get $args_ptr) (i32.const 12))))
      (local.set $s32 (i32.add (local.get $a32) (local.get $b32)))
      (i32.store8 (i32.const 0)
        (select
          (select
            (local.get $s32)
            (i32.const -128)
            (i32.gt_s (local.get $s32) (i32.const -128)))
          (i32.const 127)
          (i32.lt_s (local.get $s32) (i32.const 127))))
      (return (i64.const 4294967296))))

    ;; op 9: sadd.sat.i16
    (if (i32.eq (local.get $op) (i32.const 9)) (then
      (local.set $a32 (i32.load16_s (i32.add (local.get $args_ptr) (i32.const 4))))
      (local.set $b32 (i32.load16_s (i32.add (local.get $args_ptr) (i32.const 12))))
      (local.set $s32 (i32.add (local.get $a32) (local.get $b32)))
      (i32.store16 (i32.const 0)
        (select
          (select
            (local.get $s32)
            (i32.const -32768)
            (i32.gt_s (local.get $s32) (i32.const -32768)))
          (i32.const 32767)
          (i32.lt_s (local.get $s32) (i32.const 32767))))
      (return (i64.const 8589934592))))

    ;; op 10: sadd.sat.i32
    (if (i32.eq (local.get $op) (i32.const 10)) (then
      (local.set $a32 (i32.load (i32.add (local.get $args_ptr) (i32.const 4))))
      (local.set $b32 (i32.load (i32.add (local.get $args_ptr) (i32.const 12))))
      (local.set $s64 (i64.add
        (i64.extend_i32_s (local.get $a32))
        (i64.extend_i32_s (local.get $b32))))
      (i32.store (i32.const 0)
        (i32.wrap_i64
          (select
            (select
              (local.get $s64)
              (i64.const -2147483648)
              (i64.gt_s (local.get $s64) (i64.const -2147483648)))
            (i64.const 0x7FFFFFFF)
            (i64.lt_s (local.get $s64) (i64.const 0x7FFFFFFF)))))
      (return (i64.const 17179869184))))

    ;; op 11: sadd.sat.i64
    (if (i32.eq (local.get $op) (i32.const 11)) (then
      (local.set $a64 (i64.load (i32.add (local.get $args_ptr) (i32.const 4))))
      (local.set $b64 (i64.load (i32.add (local.get $args_ptr) (i32.const 12))))
      (local.set $s64 (i64.add (local.get $a64) (local.get $b64)))
      (i64.store (i32.const 0)
        (if (result i64)
          (i64.lt_s
            (i64.and
              (i64.xor (local.get $a64) (local.get $s64))
              (i64.xor (local.get $b64) (local.get $s64)))
            (i64.const 0))
          (then
            (select
              (i64.const -9223372036854775808)
              (i64.const 0x7FFFFFFFFFFFFFFF)
              (i64.lt_s (local.get $a64) (i64.const 0))))
          (else (local.get $s64))))
      (return (i64.const 34359738368))))

    ;; -------------------------------------------------------------------
    ;; op 12: ssub.sat.i8 — same clamp shape as sadd.sat (see ops 8-10)
    ;; -------------------------------------------------------------------
    (if (i32.eq (local.get $op) (i32.const 12)) (then
      (local.set $a32 (i32.load8_s (i32.add (local.get $args_ptr) (i32.const 4))))
      (local.set $b32 (i32.load8_s (i32.add (local.get $args_ptr) (i32.const 12))))
      (local.set $s32 (i32.sub (local.get $a32) (local.get $b32)))
      (i32.store8 (i32.const 0)
        (select
          (select
            (local.get $s32)
            (i32.const -128)
            (i32.gt_s (local.get $s32) (i32.const -128)))
          (i32.const 127)
          (i32.lt_s (local.get $s32) (i32.const 127))))
      (return (i64.const 4294967296))))

    ;; op 13: ssub.sat.i16
    (if (i32.eq (local.get $op) (i32.const 13)) (then
      (local.set $a32 (i32.load16_s (i32.add (local.get $args_ptr) (i32.const 4))))
      (local.set $b32 (i32.load16_s (i32.add (local.get $args_ptr) (i32.const 12))))
      (local.set $s32 (i32.sub (local.get $a32) (local.get $b32)))
      (i32.store16 (i32.const 0)
        (select
          (select
            (local.get $s32)
            (i32.const -32768)
            (i32.gt_s (local.get $s32) (i32.const -32768)))
          (i32.const 32767)
          (i32.lt_s (local.get $s32) (i32.const 32767))))
      (return (i64.const 8589934592))))

    ;; op 14: ssub.sat.i32
    (if (i32.eq (local.get $op) (i32.const 14)) (then
      (local.set $a32 (i32.load (i32.add (local.get $args_ptr) (i32.const 4))))
      (local.set $b32 (i32.load (i32.add (local.get $args_ptr) (i32.const 12))))
      (local.set $s64 (i64.sub
        (i64.extend_i32_s (local.get $a32))
        (i64.extend_i32_s (local.get $b32))))
      (i32.store (i32.const 0)
        (i32.wrap_i64
          (select
            (select
              (local.get $s64)
              (i64.const -2147483648)
              (i64.gt_s (local.get $s64) (i64.const -2147483648)))
            (i64.const 0x7FFFFFFF)
            (i64.lt_s (local.get $s64) (i64.const 0x7FFFFFFF)))))
      (return (i64.const 17179869184))))

    ;; op 15: ssub.sat.i64
    (local.set $a64 (i64.load (i32.add (local.get $args_ptr) (i32.const 4))))
    (local.set $b64 (i64.load (i32.add (local.get $args_ptr) (i32.const 12))))
    (local.set $s64 (i64.sub (local.get $a64) (local.get $b64)))
    (i64.store (i32.const 0)
      (if (result i64)
        (i64.lt_s
          (i64.and
            (i64.xor (local.get $a64) (local.get $b64))
            (i64.xor (local.get $a64) (local.get $s64)))
          (i64.const 0))
        (then
          (select
            (i64.const -9223372036854775808)
            (i64.const 0x7FFFFFFFFFFFFFFF)
            (i64.lt_s (local.get $a64) (i64.const 0))))
        (else (local.get $s64))))
    (i64.const 34359738368)
  )
)
