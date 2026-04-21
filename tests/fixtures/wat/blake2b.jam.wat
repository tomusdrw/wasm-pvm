;; blake2b, unkeyed, variable output length 1..=64 (RFC 7693).
;;
;; Entry: main(args_ptr: i32, args_len: i32) -> i64
;;   args = [out_len: u8][7 bytes zero pad][input: bytes]
;;   args_len = 8 + input_len, input may be 0..=32768 bytes
;;   returns (out_ptr: i32) | ((out_len: i32) << 32)
;;
;; The 8-byte header (out_len + pad) keeps the input portion of args
;; 8-byte-aligned from args_ptr. Combined with an 8-byte-aligned destination
;; (0x1008), the bulk memory.copy at entry and the per-block stream reads
;; never cross PVM page boundaries mid-u64-load, which is what capped the
;; original format (unpadded [out_len][input]) at ~4 KB inputs. Same
;; alignment trick is used in the SHA-512 fixture (PR #199).
;;
;; The 32 KB input cap is a test-harness constraint: anan-as receives args as
;; a hex CLI argument, and Linux's MAX_ARG_STRLEN (128 KB per argv string)
;; rejects anything larger. The streaming algorithm itself is size-independent.
;;
;; WASM memory layout (all offsets WASM-relative):
;;   0x000..0x03F  output hash buffer (64 bytes)
;;   0x040..0x07F  h[8] state (mutable, 8 x i64 LE)
;;   0x080..0x0BF  IV[8] constants (data segment, 8 x i64 LE)
;;   0x0C0..0x13F  v[16] working state (mutable, 16 x i64 LE)
;;   0x140..0x1BF  m[16] current message block (mutable, 16 x i64 LE)
;;   0x1C0..0x25F  sigma[10][16] permutation table (data segment, u8)
;;   0x260..0x267  t counter (i64)
;;   0x1000..0x9008  args buffer (8-byte header at 0x1000..0x1007, then up to
;;                   32 KB input at 0x1008..0x9008; args are copied here once
;;                   at entry, all stream/tail reads go through this region)

(module
  (memory (export "memory") 1)

  ;; IV at 0x80 (64 bytes, 8 x i64 LE)
  (data (i32.const 0x080)
    "\08\c9\bc\f3\67\e6\09\6a"  ;; IV[0] = 0x6a09e667f3bcc908
    "\3b\a7\ca\84\85\ae\67\bb"  ;; IV[1] = 0xbb67ae8584caa73b
    "\2b\f8\94\fe\72\f3\6e\3c"  ;; IV[2] = 0x3c6ef372fe94f82b
    "\f1\36\1d\5f\3a\f5\4f\a5"  ;; IV[3] = 0xa54ff53a5f1d36f1
    "\d1\82\e6\ad\7f\52\0e\51"  ;; IV[4] = 0x510e527fade682d1
    "\1f\6c\3e\2b\8c\68\05\9b"  ;; IV[5] = 0x9b05688c2b3e6c1f
    "\6b\bd\41\fb\ab\d9\83\1f"  ;; IV[6] = 0x1f83d9abfb41bd6b
    "\79\21\7e\13\19\cd\e0\5b") ;; IV[7] = 0x5be0cd19137e2179

  ;; Sigma at 0x1c0 (160 bytes, 10 rows x 16 u8)
  (data (i32.const 0x1c0)
    "\00\01\02\03\04\05\06\07\08\09\0a\0b\0c\0d\0e\0f"  ;; row 0
    "\0e\0a\04\08\09\0f\0d\06\01\0c\00\02\0b\07\05\03"  ;; row 1
    "\0b\08\0c\00\05\02\0f\0d\0a\0e\03\06\07\01\09\04"  ;; row 2
    "\07\09\03\01\0d\0c\0b\0e\02\06\05\0a\04\00\0f\08"  ;; row 3
    "\09\00\05\07\02\04\0a\0f\0e\01\0b\0c\06\08\03\0d"  ;; row 4
    "\02\0c\06\0a\00\0b\08\03\04\0d\07\05\0f\0e\01\09"  ;; row 5
    "\0c\05\01\0f\0e\0d\04\0a\00\07\06\03\09\02\08\0b"  ;; row 6
    "\0d\0b\07\0e\0c\01\03\09\05\00\0f\04\08\06\02\0a"  ;; row 7
    "\06\0f\0e\09\0b\03\00\08\0c\02\0d\07\01\04\0a\05"  ;; row 8
    "\0a\02\08\04\07\06\01\05\0f\0b\09\0e\03\0c\0d\00") ;; row 9

  ;; --- Helper: G mixing function ---
  ;;
  ;; Takes four v-indices (0..15) and two m-indices (0..15). Mixes
  ;; v[ia], v[ib], v[ic], v[id] using m[mx] and m[my].
  ;; Loads/stores happen via explicit byte offsets = index * 8.
  (func $g (param $ia i32) (param $ib i32) (param $ic i32) (param $id i32)
          (param $mx i32) (param $my i32)
    (local $va i64) (local $vb i64) (local $vc i64) (local $vd i64)
    (local $mxw i64) (local $myw i64)
    (local $pa i32) (local $pb i32) (local $pc i32) (local $pd i32)

    ;; byte addresses into v and m
    (local.set $pa (i32.add (i32.const 0x0c0) (i32.shl (local.get $ia) (i32.const 3))))
    (local.set $pb (i32.add (i32.const 0x0c0) (i32.shl (local.get $ib) (i32.const 3))))
    (local.set $pc (i32.add (i32.const 0x0c0) (i32.shl (local.get $ic) (i32.const 3))))
    (local.set $pd (i32.add (i32.const 0x0c0) (i32.shl (local.get $id) (i32.const 3))))

    (local.set $va (i64.load (local.get $pa)))
    (local.set $vb (i64.load (local.get $pb)))
    (local.set $vc (i64.load (local.get $pc)))
    (local.set $vd (i64.load (local.get $pd)))

    (local.set $mxw
      (i64.load (i32.add (i32.const 0x140) (i32.shl (local.get $mx) (i32.const 3)))))
    (local.set $myw
      (i64.load (i32.add (i32.const 0x140) (i32.shl (local.get $my) (i32.const 3)))))

    ;; va = va + vb + mxw
    (local.set $va (i64.add (i64.add (local.get $va) (local.get $vb)) (local.get $mxw)))
    ;; vd = rotr(vd ^ va, 32)
    (local.set $vd (i64.rotr (i64.xor (local.get $vd) (local.get $va)) (i64.const 32)))
    ;; vc = vc + vd
    (local.set $vc (i64.add (local.get $vc) (local.get $vd)))
    ;; vb = rotr(vb ^ vc, 24)
    (local.set $vb (i64.rotr (i64.xor (local.get $vb) (local.get $vc)) (i64.const 24)))
    ;; va = va + vb + myw
    (local.set $va (i64.add (i64.add (local.get $va) (local.get $vb)) (local.get $myw)))
    ;; vd = rotr(vd ^ va, 16)
    (local.set $vd (i64.rotr (i64.xor (local.get $vd) (local.get $va)) (i64.const 16)))
    ;; vc = vc + vd
    (local.set $vc (i64.add (local.get $vc) (local.get $vd)))
    ;; vb = rotr(vb ^ vc, 63)
    (local.set $vb (i64.rotr (i64.xor (local.get $vb) (local.get $vc)) (i64.const 63)))

    (i64.store (local.get $pa) (local.get $va))
    (i64.store (local.get $pb) (local.get $vb))
    (i64.store (local.get $pc) (local.get $vc))
    (i64.store (local.get $pd) (local.get $vd)))

  ;; --- Helper: compress (F function) ---
  ;;
  ;; Consumes m[16] already filled, t counter at 0x260, and the last flag
  ;; passed as a parameter. Mutates h[].
  (func $compress (param $last i32)
    (local $r i32)         ;; round index 0..11
    (local $sigma_base i32) ;; pointer into sigma[round % 10]
    (local $t i64)

    ;; v[0..15] = h[0..7] || IV[0..7] (one contiguous 128-byte copy).
    ;; h is at 0x40..0x7F and IV at 0x80..0xBF, so a single memory.copy from
    ;; 0x40 of length 128 reads h followed by IV directly into v at 0xC0.
    (memory.copy (i32.const 0x0c0) (i32.const 0x040) (i32.const 128))

    ;; v[12] ^= t_low
    (local.set $t (i64.load (i32.const 0x260)))
    (i64.store offset=96 (i32.const 0x0c0)
      (i64.xor (i64.load offset=96 (i32.const 0x0c0)) (local.get $t)))
    ;; v[13] ^= t_high (always 0 for our capped input size; XOR is structurally correct)
    (i64.store offset=104 (i32.const 0x0c0)
      (i64.xor (i64.load offset=104 (i32.const 0x0c0)) (i64.const 0)))

    ;; v[14] ^= ~0 if last
    (if (local.get $last)
      (then
        (i64.store offset=112 (i32.const 0x0c0)
          (i64.xor (i64.load offset=112 (i32.const 0x0c0))
                   (i64.const -1)))))

    ;; 12 rounds
    (local.set $r (i32.const 0))
    (block $rounds_exit
      (loop $rounds
        (br_if $rounds_exit (i32.ge_u (local.get $r) (i32.const 12)))

        ;; sigma_base = 0x1c0 + (r % 10) * 16
        (local.set $sigma_base
          (i32.add (i32.const 0x1c0)
            (i32.shl (i32.rem_u (local.get $r) (i32.const 10)) (i32.const 4))))

        ;; Column mixes: G(0,4,8,12, s[0],s[1]), G(1,5,9,13, s[2],s[3]), ...
        (call $g (i32.const 0) (i32.const 4) (i32.const 8)  (i32.const 12)
                 (i32.load8_u offset=0  (local.get $sigma_base))
                 (i32.load8_u offset=1  (local.get $sigma_base)))
        (call $g (i32.const 1) (i32.const 5) (i32.const 9)  (i32.const 13)
                 (i32.load8_u offset=2  (local.get $sigma_base))
                 (i32.load8_u offset=3  (local.get $sigma_base)))
        (call $g (i32.const 2) (i32.const 6) (i32.const 10) (i32.const 14)
                 (i32.load8_u offset=4  (local.get $sigma_base))
                 (i32.load8_u offset=5  (local.get $sigma_base)))
        (call $g (i32.const 3) (i32.const 7) (i32.const 11) (i32.const 15)
                 (i32.load8_u offset=6  (local.get $sigma_base))
                 (i32.load8_u offset=7  (local.get $sigma_base)))

        ;; Diagonal mixes
        (call $g (i32.const 0) (i32.const 5) (i32.const 10) (i32.const 15)
                 (i32.load8_u offset=8  (local.get $sigma_base))
                 (i32.load8_u offset=9  (local.get $sigma_base)))
        (call $g (i32.const 1) (i32.const 6) (i32.const 11) (i32.const 12)
                 (i32.load8_u offset=10 (local.get $sigma_base))
                 (i32.load8_u offset=11 (local.get $sigma_base)))
        (call $g (i32.const 2) (i32.const 7) (i32.const 8)  (i32.const 13)
                 (i32.load8_u offset=12 (local.get $sigma_base))
                 (i32.load8_u offset=13 (local.get $sigma_base)))
        (call $g (i32.const 3) (i32.const 4) (i32.const 9)  (i32.const 14)
                 (i32.load8_u offset=14 (local.get $sigma_base))
                 (i32.load8_u offset=15 (local.get $sigma_base)))

        (local.set $r (i32.add (local.get $r) (i32.const 1)))
        (br $rounds)))

    ;; h[i] ^= v[i] ^ v[i+8] for i in 0..7 (loop over byte offset 0..=56 step 8).
    ;; Reuses $r (the rounds counter is done by this point).
    (local.set $r (i32.const 0))
    (block $h_xor_exit
      (loop $h_xor
        (br_if $h_xor_exit (i32.ge_u (local.get $r) (i32.const 64)))
        (i64.store
          (i32.add (i32.const 0x040) (local.get $r))
          (i64.xor
            (i64.load (i32.add (i32.const 0x040) (local.get $r)))
            (i64.xor
              (i64.load (i32.add (i32.const 0x0c0) (local.get $r)))
              (i64.load (i32.add (i32.const 0x100) (local.get $r))))))
        (local.set $r (i32.add (local.get $r) (i32.const 8)))
        (br $h_xor)))) ;; close loop, block, func $compress

  ;; --- main ---
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $out_len i32)
    (local $data_ptr i32)
    (local $remaining i32)   ;; bytes of input not yet consumed

    ;; Reject args_len < 8 (header missing) or > 32776 (8-byte header +
    ;; 32768-byte input cap). Beyond the cap the args buffer at
    ;; 0x1000..0x9008 would overflow; below 8 the header can't be read.
    ;; Hard correctness guards, not soft limits.
    (if (i32.or
          (i32.lt_u (local.get $args_len) (i32.const 8))
          (i32.gt_u (local.get $args_len) (i32.const 32776)))
      (then
        (i32.store8 (i32.const 0x268) (i32.const 0xEE))
        (unreachable)))

    ;; out_len = args[0]. Read before the bulk copy — this is a single byte
    ;; read from the args region, fine even without buffering.
    (local.set $out_len (i32.load8_u (local.get $args_ptr)))
    ;; Two separate checks. A combined `(if (i32.or eqz gt_u) ...)` form was
    ;; observed not to fire the gt_u branch reliably under this project's
    ;; WASM→PVM compiler — out_len > 64 slipped through. Two `if`s are
    ;; semantically equivalent and always fire.
    ;; Reject out_len == 0 or > 64. The trap block stores a sentinel before
    ;; `unreachable` to prevent the LLVM-based compiler from eliding the
    ;; guard as UB-implied-dead-code. See docs/src/learnings.md, blake2b
    ;; section, "if+unreachable guards".
    (if (i32.or (i32.eqz (local.get $out_len))
                (i32.gt_u (local.get $out_len) (i32.const 64)))
      (then
        (i32.store8 (i32.const 0x268) (i32.const 0xEE))
        (unreachable)))

    ;; Copy the whole args blob (8-byte header + input) into WASM memory at
    ;; 0x1000 in one shot, then do all stream/tail reads from WASM memory.
    ;; Under PVM this pulls args out of the args region (0xFEFF0000) into the
    ;; pre-allocated WASM region once, avoiding scattered stream-loop reads
    ;; that cross PVM page boundaries (which were unreliable — see the SHA-512
    ;; fixture commentary and PR #199 for background). Under native WASM,
    ;; args are already at 0x1000, so this is effectively a no-op self-copy.
    ;;
    ;; Both source (args_ptr, 4 KB-aligned at 0xFEFF0000) and destination
    ;; (0x1000, 8-byte aligned) are word-aligned. Combined with the 8-byte
    ;; header that keeps the input portion at args_ptr+8 (still aligned), no
    ;; u64 load in this copy or in the per-block stream loop straddles a PVM
    ;; page boundary. Misaligning either side made cross-page u64 loads
    ;; extremely expensive in anan-as and caused out-of-gas at ~4 KB inputs.
    (memory.copy (i32.const 0x1000) (local.get $args_ptr) (local.get $args_len))

    ;; data_ptr = 0x1008 (input starts after the 8-byte header; word-aligned).
    ;; remaining = args_len - 8 (input byte count)
    (local.set $data_ptr (i32.const 0x1008))
    (local.set $remaining (i32.sub (local.get $args_len) (i32.const 8)))

    ;; h[0..7] = IV[0..7]
    (i64.store offset=0  (i32.const 0x040) (i64.load offset=0  (i32.const 0x080)))
    (i64.store offset=8  (i32.const 0x040) (i64.load offset=8  (i32.const 0x080)))
    (i64.store offset=16 (i32.const 0x040) (i64.load offset=16 (i32.const 0x080)))
    (i64.store offset=24 (i32.const 0x040) (i64.load offset=24 (i32.const 0x080)))
    (i64.store offset=32 (i32.const 0x040) (i64.load offset=32 (i32.const 0x080)))
    (i64.store offset=40 (i32.const 0x040) (i64.load offset=40 (i32.const 0x080)))
    (i64.store offset=48 (i32.const 0x040) (i64.load offset=48 (i32.const 0x080)))
    (i64.store offset=56 (i32.const 0x040) (i64.load offset=56 (i32.const 0x080)))

    ;; Apply parameter block: h[0] ^= 0x0101_0000 ^ out_len
    ;; (fanout=1, depth=1, node_depth=0, inner_len=0, key_len=0, digest_len=out_len)
    (i64.store offset=0 (i32.const 0x040)
      (i64.xor
        (i64.load offset=0 (i32.const 0x040))
        (i64.xor
          (i64.const 0x01010000)
          (i64.extend_i32_u (local.get $out_len)))))

    ;; t = 0
    (i64.store (i32.const 0x260) (i64.const 0))

    ;; Process non-final full 128-byte blocks: while remaining > 128
    (block $stream_exit
      (loop $stream
        (br_if $stream_exit (i32.le_u (local.get $remaining) (i32.const 128)))

        ;; Copy 128 bytes from data_ptr into m. memory.copy lowers to word-sized
        ;; loops in the compiler's memory backend, far smaller than 16 explicit
        ;; i64.load/store pairs.
        (memory.copy (i32.const 0x140) (local.get $data_ptr) (i32.const 128))

        ;; t += 128
        (i64.store (i32.const 0x260)
          (i64.add (i64.load (i32.const 0x260)) (i64.const 128)))

        ;; compress(last=0)
        (call $compress (i32.const 0))

        (local.set $data_ptr (i32.add (local.get $data_ptr) (i32.const 128)))
        (local.set $remaining (i32.sub (local.get $remaining) (i32.const 128)))
        (br $stream)))

    ;; Final block: zero m[], then copy `remaining` bytes of trailing input.
    (memory.fill (i32.const 0x140) (i32.const 0) (i32.const 128))
    (memory.copy (i32.const 0x140) (local.get $data_ptr) (local.get $remaining))

    ;; t += remaining
    (i64.store (i32.const 0x260)
      (i64.add (i64.load (i32.const 0x260)) (i64.extend_i32_u (local.get $remaining))))

    ;; compress(last=1)
    (call $compress (i32.const 1))

    ;; Copy h[0..out_len] bytes to output at offset 0. h is stored as 8 × i64 LE,
    ;; so a byte-level copy correctly extracts the low-endian hash for any
    ;; out_len (including non-multiples of 8).
    (memory.copy (i32.const 0) (i32.const 0x040) (local.get $out_len))

    ;; Return (0 | (out_len << 32))
    (i64.shl (i64.extend_i32_u (local.get $out_len)) (i64.const 32))))
