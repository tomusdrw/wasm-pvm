(module
  (memory 1)
  ;; Args layout:
  ;;   byte 0     : op selector
  ;;                  0 = i32.bitreverse
  ;;                  1 = i64.bitreverse
  ;;                  2 = i8.bitreverse
  ;;                  3 = i16.bitreverse
  ;;   bytes 4..  : input value, little-endian
  ;; Output: result stored at address 0 with `len` bytes; returned as
  ;; packed `(ptr as i64) | ((len as i64) << 32)`. Lengths: 4 for i32,
  ;; 8 for i64, 1 for i8, 2 for i16.
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $op i32)
    (local $x32 i32)
    (local $x64 i64)

    (local.set $op (i32.load (local.get $args_ptr)))

    (if (i32.eq (local.get $op) (i32.const 0))
      (then
        ;; Canonical i32 bitreverse — LLVM instcombine folds this into
        ;; @llvm.bitreverse.i32, which our backend lowers via 3 mask phases
        ;; + ReverseBytes.
        (local.set $x32 (i32.load (i32.add (local.get $args_ptr) (i32.const 4))))
        (local.set $x32
          (i32.or
            (i32.and (i32.shr_u (local.get $x32) (i32.const 1)) (i32.const 0x55555555))
            (i32.shl (i32.and (local.get $x32) (i32.const 0x55555555)) (i32.const 1))))
        (local.set $x32
          (i32.or
            (i32.and (i32.shr_u (local.get $x32) (i32.const 2)) (i32.const 0x33333333))
            (i32.shl (i32.and (local.get $x32) (i32.const 0x33333333)) (i32.const 2))))
        (local.set $x32
          (i32.or
            (i32.and (i32.shr_u (local.get $x32) (i32.const 4)) (i32.const 0x0F0F0F0F))
            (i32.shl (i32.and (local.get $x32) (i32.const 0x0F0F0F0F)) (i32.const 4))))
        (local.set $x32
          (i32.or
            (i32.or
              (i32.shl (local.get $x32) (i32.const 24))
              (i32.shl (i32.and (local.get $x32) (i32.const 0xFF00)) (i32.const 8)))
            (i32.or
              (i32.and (i32.shr_u (local.get $x32) (i32.const 8)) (i32.const 0xFF00))
              (i32.shr_u (local.get $x32) (i32.const 24)))))
        (i32.store (i32.const 0) (local.get $x32))
        (return (i64.const 17179869184))  ;; ptr=0, len=4
      )
    )

    (if (i32.eq (local.get $op) (i32.const 2))
      (then
        ;; Canonical i8 bitreverse — narrow load/store + low-byte masks
        ;; let LLVM fold this into @llvm.bitreverse.i8 (no byte-swap step).
        (local.set $x32 (i32.load8_u (i32.add (local.get $args_ptr) (i32.const 4))))
        (local.set $x32
          (i32.or
            (i32.and (i32.shr_u (local.get $x32) (i32.const 1)) (i32.const 0x55))
            (i32.shl (i32.and (local.get $x32) (i32.const 0x55)) (i32.const 1))))
        (local.set $x32
          (i32.or
            (i32.and (i32.shr_u (local.get $x32) (i32.const 2)) (i32.const 0x33))
            (i32.shl (i32.and (local.get $x32) (i32.const 0x33)) (i32.const 2))))
        (local.set $x32
          (i32.or
            (i32.and (i32.shr_u (local.get $x32) (i32.const 4)) (i32.const 0x0F))
            (i32.shl (i32.and (local.get $x32) (i32.const 0x0F)) (i32.const 4))))
        (i32.store8 (i32.const 0) (local.get $x32))
        (return (i64.const 4294967296))  ;; ptr=0, len=1
      )
    )

    (if (i32.eq (local.get $op) (i32.const 3))
      (then
        ;; Canonical i16 bitreverse — fold to @llvm.bitreverse.i16.
        (local.set $x32 (i32.load16_u (i32.add (local.get $args_ptr) (i32.const 4))))
        (local.set $x32
          (i32.or
            (i32.and (i32.shr_u (local.get $x32) (i32.const 1)) (i32.const 0x5555))
            (i32.shl (i32.and (local.get $x32) (i32.const 0x5555)) (i32.const 1))))
        (local.set $x32
          (i32.or
            (i32.and (i32.shr_u (local.get $x32) (i32.const 2)) (i32.const 0x3333))
            (i32.shl (i32.and (local.get $x32) (i32.const 0x3333)) (i32.const 2))))
        (local.set $x32
          (i32.or
            (i32.and (i32.shr_u (local.get $x32) (i32.const 4)) (i32.const 0x0F0F))
            (i32.shl (i32.and (local.get $x32) (i32.const 0x0F0F)) (i32.const 4))))
        (local.set $x32
          (i32.or
            (i32.shl (i32.and (local.get $x32) (i32.const 0xFF)) (i32.const 8))
            (i32.shr_u (i32.and (local.get $x32) (i32.const 0xFF00)) (i32.const 8))))
        (i32.store16 (i32.const 0) (local.get $x32))
        (return (i64.const 8589934592))  ;; ptr=0, len=2
      )
    )

    ;; op == 1: i64 bitreverse
    (local.set $x64 (i64.load (i32.add (local.get $args_ptr) (i32.const 4))))
    (local.set $x64
      (i64.or
        (i64.and (i64.shr_u (local.get $x64) (i64.const 1)) (i64.const 0x5555555555555555))
        (i64.shl (i64.and (local.get $x64) (i64.const 0x5555555555555555)) (i64.const 1))))
    (local.set $x64
      (i64.or
        (i64.and (i64.shr_u (local.get $x64) (i64.const 2)) (i64.const 0x3333333333333333))
        (i64.shl (i64.and (local.get $x64) (i64.const 0x3333333333333333)) (i64.const 2))))
    (local.set $x64
      (i64.or
        (i64.and (i64.shr_u (local.get $x64) (i64.const 4)) (i64.const 0x0F0F0F0F0F0F0F0F))
        (i64.shl (i64.and (local.get $x64) (i64.const 0x0F0F0F0F0F0F0F0F)) (i64.const 4))))
    (local.set $x64
      (i64.or
        (i64.or
          (i64.or
            (i64.shl (local.get $x64) (i64.const 56))
            (i64.shl (i64.and (local.get $x64) (i64.const 0xFF00)) (i64.const 40)))
          (i64.or
            (i64.shl (i64.and (local.get $x64) (i64.const 0xFF0000)) (i64.const 24))
            (i64.shl (i64.and (local.get $x64) (i64.const 0xFF000000)) (i64.const 8))))
        (i64.or
          (i64.or
            (i64.and (i64.shr_u (local.get $x64) (i64.const 8)) (i64.const 0xFF000000))
            (i64.and (i64.shr_u (local.get $x64) (i64.const 24)) (i64.const 0xFF0000)))
          (i64.or
            (i64.and (i64.shr_u (local.get $x64) (i64.const 40)) (i64.const 0xFF00))
            (i64.shr_u (local.get $x64) (i64.const 56))))))
    (i64.store (i32.const 0) (local.get $x64))
    (i64.const 34359738368)  ;; ptr=0, len=8
  )
)
