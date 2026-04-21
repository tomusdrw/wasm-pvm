;; SHA-512 (FIPS 180-4), fixed 64-byte output.
;;
;; Entry: main(args_ptr: i32, args_len: i32) -> i64
;;   args = [input: bytes]  (input may be 0..=65536 bytes; no prefix)
;;   returns (out_ptr: i32) | ((64: i32) << 32)
;;
;; WASM memory layout (all offsets WASM-relative):
;;   0x000..0x03F  output hash buffer (64 bytes)
;;   0x040..0x07F  h[8] state (mutable, 8 x i64)
;;   0x080..0x0BF  initial H[8] constants (data segment, 8 x i64 LE)
;;   0x0C0..0x33F  K[80] round constants (data segment, 80 x i64 LE)
;;   0x340..0x5BF  W[80] message schedule (mutable, 80 x i64)
;;   0x5C0..0x63F  final-block padding buffer (128 bytes)
;;   0x640..0x67F  working state a..hh during rounds (8 x i64)
;;   0x1000..0x11000  input buffer (up to 64 KB; args are copied here once
;;                    at entry, all stream/tail reads go through this region)

(module
  ;; 2 pages (128 KB): enough for the internal buffers (first ~1.7 KB) plus
  ;; input data up to 64 KB placed by the native-WASM runner at offset 0x1000.
  (memory (export "memory") 2)

  ;; Initial hash values H[0..7] at 0x80 (64 bytes, 8 x i64 LE).
  (data (i32.const 0x080)
    "\08\c9\bc\f3\67\e6\09\6a"  ;; H[0] = 0x6a09e667f3bcc908
    "\3b\a7\ca\84\85\ae\67\bb"  ;; H[1] = 0xbb67ae8584caa73b
    "\2b\f8\94\fe\72\f3\6e\3c"  ;; H[2] = 0x3c6ef372fe94f82b
    "\f1\36\1d\5f\3a\f5\4f\a5"  ;; H[3] = 0xa54ff53a5f1d36f1
    "\d1\82\e6\ad\7f\52\0e\51"  ;; H[4] = 0x510e527fade682d1
    "\1f\6c\3e\2b\8c\68\05\9b"  ;; H[5] = 0x9b05688c2b3e6c1f
    "\6b\bd\41\fb\ab\d9\83\1f"  ;; H[6] = 0x1f83d9abfb41bd6b
    "\79\21\7e\13\19\cd\e0\5b") ;; H[7] = 0x5be0cd19137e2179

  ;; Round constants K[0..79] at 0xC0 (640 bytes, 80 x i64 LE).
  (data (i32.const 0x0c0)
    "\22\ae\28\d7\98\2f\8a\42"  ;; K[0]  = 0x428a2f98d728ae22
    "\cd\65\ef\23\91\44\37\71"  ;; K[1]  = 0x7137449123ef65cd
    "\2f\3b\4d\ec\cf\fb\c0\b5"  ;; K[2]  = 0xb5c0fbcfec4d3b2f
    "\bc\db\89\81\a5\db\b5\e9"  ;; K[3]  = 0xe9b5dba58189dbbc
    "\38\b5\48\f3\5b\c2\56\39"  ;; K[4]  = 0x3956c25bf348b538
    "\19\d0\05\b6\f1\11\f1\59"  ;; K[5]  = 0x59f111f1b605d019
    "\9b\4f\19\af\a4\82\3f\92"  ;; K[6]  = 0x923f82a4af194f9b
    "\18\81\6d\da\d5\5e\1c\ab"  ;; K[7]  = 0xab1c5ed5da6d8118
    "\42\02\03\a3\98\aa\07\d8"  ;; K[8]  = 0xd807aa98a3030242
    "\be\6f\70\45\01\5b\83\12"  ;; K[9]  = 0x12835b0145706fbe
    "\8c\b2\e4\4e\be\85\31\24"  ;; K[10] = 0x243185be4ee4b28c
    "\e2\b4\ff\d5\c3\7d\0c\55"  ;; K[11] = 0x550c7dc3d5ffb4e2
    "\6f\89\7b\f2\74\5d\be\72"  ;; K[12] = 0x72be5d74f27b896f
    "\b1\96\16\3b\fe\b1\de\80"  ;; K[13] = 0x80deb1fe3b1696b1
    "\35\12\c7\25\a7\06\dc\9b"  ;; K[14] = 0x9bdc06a725c71235
    "\94\26\69\cf\74\f1\9b\c1"  ;; K[15] = 0xc19bf174cf692694
    "\d2\4a\f1\9e\c1\69\9b\e4"  ;; K[16] = 0xe49b69c19ef14ad2
    "\e3\25\4f\38\86\47\be\ef"  ;; K[17] = 0xefbe4786384f25e3
    "\b5\d5\8c\8b\c6\9d\c1\0f"  ;; K[18] = 0x0fc19dc68b8cd5b5
    "\65\9c\ac\77\cc\a1\0c\24"  ;; K[19] = 0x240ca1cc77ac9c65
    "\75\02\2b\59\6f\2c\e9\2d"  ;; K[20] = 0x2de92c6f592b0275
    "\83\e4\a6\6e\aa\84\74\4a"  ;; K[21] = 0x4a7484aa6ea6e483
    "\d4\fb\41\bd\dc\a9\b0\5c"  ;; K[22] = 0x5cb0a9dcbd41fbd4
    "\b5\53\11\83\da\88\f9\76"  ;; K[23] = 0x76f988da831153b5
    "\ab\df\66\ee\52\51\3e\98"  ;; K[24] = 0x983e5152ee66dfab
    "\10\32\b4\2d\6d\c6\31\a8"  ;; K[25] = 0xa831c66d2db43210
    "\3f\21\fb\98\c8\27\03\b0"  ;; K[26] = 0xb00327c898fb213f
    "\e4\0e\ef\be\c7\7f\59\bf"  ;; K[27] = 0xbf597fc7beef0ee4
    "\c2\8f\a8\3d\f3\0b\e0\c6"  ;; K[28] = 0xc6e00bf33da88fc2
    "\25\a7\0a\93\47\91\a7\d5"  ;; K[29] = 0xd5a79147930aa725
    "\6f\82\03\e0\51\63\ca\06"  ;; K[30] = 0x06ca6351e003826f
    "\70\6e\0e\0a\67\29\29\14"  ;; K[31] = 0x142929670a0e6e70
    "\fc\2f\d2\46\85\0a\b7\27"  ;; K[32] = 0x27b70a8546d22ffc
    "\26\c9\26\5c\38\21\1b\2e"  ;; K[33] = 0x2e1b21385c26c926
    "\ed\2a\c4\5a\fc\6d\2c\4d"  ;; K[34] = 0x4d2c6dfc5ac42aed
    "\df\b3\95\9d\13\0d\38\53"  ;; K[35] = 0x53380d139d95b3df
    "\de\63\af\8b\54\73\0a\65"  ;; K[36] = 0x650a73548baf63de
    "\a8\b2\77\3c\bb\0a\6a\76"  ;; K[37] = 0x766a0abb3c77b2a8
    "\e6\ae\ed\47\2e\c9\c2\81"  ;; K[38] = 0x81c2c92e47edaee6
    "\3b\35\82\14\85\2c\72\92"  ;; K[39] = 0x92722c851482353b
    "\64\03\f1\4c\a1\e8\bf\a2"  ;; K[40] = 0xa2bfe8a14cf10364
    "\01\30\42\bc\4b\66\1a\a8"  ;; K[41] = 0xa81a664bbc423001
    "\91\97\f8\d0\70\8b\4b\c2"  ;; K[42] = 0xc24b8b70d0f89791
    "\30\be\54\06\a3\51\6c\c7"  ;; K[43] = 0xc76c51a30654be30
    "\18\52\ef\d6\19\e8\92\d1"  ;; K[44] = 0xd192e819d6ef5218
    "\10\a9\65\55\24\06\99\d6"  ;; K[45] = 0xd69906245565a910
    "\2a\20\71\57\85\35\0e\f4"  ;; K[46] = 0xf40e35855771202a
    "\b8\d1\bb\32\70\a0\6a\10"  ;; K[47] = 0x106aa07032bbd1b8
    "\c8\d0\d2\b8\16\c1\a4\19"  ;; K[48] = 0x19a4c116b8d2d0c8
    "\53\ab\41\51\08\6c\37\1e"  ;; K[49] = 0x1e376c085141ab53
    "\99\eb\8e\df\4c\77\48\27"  ;; K[50] = 0x2748774cdf8eeb99
    "\a8\48\9b\e1\b5\bc\b0\34"  ;; K[51] = 0x34b0bcb5e19b48a8
    "\63\5a\c9\c5\b3\0c\1c\39"  ;; K[52] = 0x391c0cb3c5c95a63
    "\cb\8a\41\e3\4a\aa\d8\4e"  ;; K[53] = 0x4ed8aa4ae3418acb
    "\73\e3\63\77\4f\ca\9c\5b"  ;; K[54] = 0x5b9cca4f7763e373
    "\a3\b8\b2\d6\f3\6f\2e\68"  ;; K[55] = 0x682e6ff3d6b2b8a3
    "\fc\b2\ef\5d\ee\82\8f\74"  ;; K[56] = 0x748f82ee5defb2fc
    "\60\2f\17\43\6f\63\a5\78"  ;; K[57] = 0x78a5636f43172f60
    "\72\ab\f0\a1\14\78\c8\84"  ;; K[58] = 0x84c87814a1f0ab72
    "\ec\39\64\1a\08\02\c7\8c"  ;; K[59] = 0x8cc702081a6439ec
    "\28\1e\63\23\fa\ff\be\90"  ;; K[60] = 0x90befffa23631e28
    "\e9\bd\82\de\eb\6c\50\a4"  ;; K[61] = 0xa4506cebde82bde9
    "\15\79\c6\b2\f7\a3\f9\be"  ;; K[62] = 0xbef9a3f7b2c67915
    "\2b\53\72\e3\f2\78\71\c6"  ;; K[63] = 0xc67178f2e372532b
    "\9c\61\26\ea\ce\3e\27\ca"  ;; K[64] = 0xca273eceea26619c
    "\07\c2\c0\21\c7\b8\86\d1"  ;; K[65] = 0xd186b8c721c0c207
    "\1e\eb\e0\cd\d6\7d\da\ea"  ;; K[66] = 0xeada7dd6cde0eb1e
    "\78\d1\6e\ee\7f\4f\7d\f5"  ;; K[67] = 0xf57d4f7fee6ed178
    "\ba\6f\17\72\aa\67\f0\06"  ;; K[68] = 0x06f067aa72176fba
    "\a6\98\c8\a2\c5\7d\63\0a"  ;; K[69] = 0x0a637dc5a2c898a6
    "\ae\0d\f9\be\04\98\3f\11"  ;; K[70] = 0x113f9804bef90dae
    "\1b\47\1c\13\35\0b\71\1b"  ;; K[71] = 0x1b710b35131c471b
    "\84\7d\04\23\f5\77\db\28"  ;; K[72] = 0x28db77f523047d84
    "\93\24\c7\40\7b\ab\ca\32"  ;; K[73] = 0x32caab7b40c72493
    "\bc\be\c9\15\0a\be\9e\3c"  ;; K[74] = 0x3c9ebe0a15c9bebc
    "\4c\0d\10\9c\c4\67\1d\43"  ;; K[75] = 0x431d67c49c100d4c
    "\b6\42\3e\cb\be\d4\c5\4c"  ;; K[76] = 0x4cc5d4becb3e42b6
    "\2a\7e\65\fc\9c\29\7f\59"  ;; K[77] = 0x597f299cfc657e2a
    "\ec\fa\d6\3a\ab\6f\cb\5f"  ;; K[78] = 0x5fcb6fab3ad6faec
    "\17\58\47\4a\8c\19\44\6c") ;; K[79] = 0x6c44198c4a475817

  ;; --- Helper: byte-swap an i64 (LE ↔ BE) ---
  ;; WAT has no native bswap. Implemented via shifts + masks.
  (func $bswap64 (param $x i64) (result i64)
    (i64.or
      (i64.or
        (i64.or
          (i64.shl (local.get $x) (i64.const 56))
          (i64.shl (i64.and (local.get $x) (i64.const 0x000000000000FF00))
                   (i64.const 40)))
        (i64.or
          (i64.shl (i64.and (local.get $x) (i64.const 0x0000000000FF0000))
                   (i64.const 24))
          (i64.shl (i64.and (local.get $x) (i64.const 0x00000000FF000000))
                   (i64.const  8))))
      (i64.or
        (i64.or
          (i64.and (i64.shr_u (local.get $x) (i64.const  8))
                   (i64.const 0x00000000FF000000))
          (i64.and (i64.shr_u (local.get $x) (i64.const 24))
                   (i64.const 0x0000000000FF0000)))
        (i64.or
          (i64.and (i64.shr_u (local.get $x) (i64.const 40))
                   (i64.const 0x000000000000FF00))
          (i64.shr_u (local.get $x) (i64.const 56))))))

  ;; --- compress(block_ptr: i32) ---
  ;;
  ;; Consumes one 128-byte block at $block_ptr, mutates h[].
  (func $compress (param $block_ptr i32)
    (local $a i64) (local $b i64) (local $c i64) (local $d i64)
    (local $e i64) (local $f i64) (local $g i64) (local $hh i64)
    (local $t1 i64) (local $t2 i64)
    (local $i i32) (local $w_ptr i32)

    ;; --- Build W[0..15] from the block: load LE then bswap to BE ---
    (local.set $i (i32.const 0))
    (block $w_load_exit
      (loop $w_load
        (br_if $w_load_exit (i32.ge_u (local.get $i) (i32.const 128)))
        (i64.store
          (i32.add (i32.const 0x340) (local.get $i))
          (call $bswap64
            (i64.load (i32.add (local.get $block_ptr) (local.get $i)))))
        (local.set $i (i32.add (local.get $i) (i32.const 8)))
        (br $w_load)))

    ;; --- Extend W[16..79] using SSIG0/SSIG1 ---
    ;; W[t] = SSIG1(W[t-2]) + W[t-7] + SSIG0(W[t-15]) + W[t-16]
    ;; SSIG0(x) = rotr(x,1) ^ rotr(x,8) ^ (x >> 7)
    ;; SSIG1(x) = rotr(x,19) ^ rotr(x,61) ^ (x >> 6)
    ;; We index W[] in memory: byte offset = t*8, from 0x340.
    (local.set $i (i32.const 16))
    (block $w_ext_exit
      (loop $w_ext
        (br_if $w_ext_exit (i32.ge_u (local.get $i) (i32.const 80)))
        ;; w_ptr = 0x340 + i*8
        (local.set $w_ptr
          (i32.add (i32.const 0x340)
            (i32.shl (local.get $i) (i32.const 3))))
        (i64.store (local.get $w_ptr)
          (i64.add
            (i64.add
              ;; SSIG1(W[i-2]) — W[i-2] is at w_ptr - 16
              (i64.xor
                (i64.xor
                  (i64.rotr (i64.load (i32.sub (local.get $w_ptr) (i32.const 16))) (i64.const 19))
                  (i64.rotr (i64.load (i32.sub (local.get $w_ptr) (i32.const 16))) (i64.const 61)))
                (i64.shr_u (i64.load (i32.sub (local.get $w_ptr) (i32.const 16))) (i64.const 6)))
              ;; W[i-7] — at w_ptr - 56
              (i64.load (i32.sub (local.get $w_ptr) (i32.const 56))))
            (i64.add
              ;; SSIG0(W[i-15]) — W[i-15] is at w_ptr - 120
              (i64.xor
                (i64.xor
                  (i64.rotr (i64.load (i32.sub (local.get $w_ptr) (i32.const 120))) (i64.const 1))
                  (i64.rotr (i64.load (i32.sub (local.get $w_ptr) (i32.const 120))) (i64.const 8)))
                (i64.shr_u (i64.load (i32.sub (local.get $w_ptr) (i32.const 120))) (i64.const 7)))
              ;; W[i-16] — at w_ptr - 128
              (i64.load (i32.sub (local.get $w_ptr) (i32.const 128))))))
        (local.set $i (i32.add (local.get $i) (i32.const 1)))
        (br $w_ext)))

    ;; --- Initialize working state at 0x640 from h[] at 0x040 ---
    ;; We keep working state in memory (not WASM locals) to avoid excessive
    ;; phi pressure at the round-loop header — the PVM backend caps loop-
    ;; carried values at ~5 due to temp-register limits.
    (memory.copy (i32.const 0x640) (i32.const 0x040) (i32.const 64))

    ;; --- 80 rounds ---
    ;; T1 = hh + BSIG1(e) + Ch(e,f,g) + K[t] + W[t]
    ;; T2 = BSIG0(a) + Maj(a,b,c)
    ;; BSIG0(x) = rotr(x,28) ^ rotr(x,34) ^ rotr(x,39)
    ;; BSIG1(x) = rotr(x,14) ^ rotr(x,18) ^ rotr(x,41)
    ;; Ch(x,y,z)  = (x & y) ^ (~x & z)
    ;; Maj(x,y,z) = (x & y) ^ (x & z) ^ (y & z)
    ;;
    ;; Working state layout at 0x640: a,b,c,d,e,f,g,hh (each 8 bytes).
    ;; K[t] at 0x0C0 + i, W[t] at 0x340 + i (both 8-byte indexed).
    (local.set $i (i32.const 0))
    (block $rounds_exit
      (loop $rounds
        (br_if $rounds_exit (i32.ge_u (local.get $i) (i32.const 640))) ;; 80*8

        ;; Load e early (used for BSIG1 + Ch).
        (local.set $e (i64.load offset=32 (i32.const 0x640)))
        ;; Load a (for BSIG0 + Maj).
        (local.set $a (i64.load offset=0 (i32.const 0x640)))

        ;; T1 = hh + BSIG1(e) + Ch(e,f,g) + K[t] + W[t]
        (local.set $t1
          (i64.add
            (i64.add
              (i64.add (i64.load offset=56 (i32.const 0x640)) ;; hh
                ;; BSIG1(e)
                (i64.xor
                  (i64.xor
                    (i64.rotr (local.get $e) (i64.const 14))
                    (i64.rotr (local.get $e) (i64.const 18)))
                  (i64.rotr (local.get $e) (i64.const 41))))
              ;; Ch(e,f,g) = (e & f) ^ (~e & g)
              (i64.xor
                (i64.and (local.get $e) (i64.load offset=40 (i32.const 0x640))) ;; f
                (i64.and (i64.xor (local.get $e) (i64.const -1))
                         (i64.load offset=48 (i32.const 0x640))))) ;; g
            (i64.add
              (i64.load (i32.add (i32.const 0x0c0) (local.get $i))) ;; K[t]
              (i64.load (i32.add (i32.const 0x340) (local.get $i)))))) ;; W[t]

        ;; T2 = BSIG0(a) + Maj(a,b,c)
        (local.set $t2
          (i64.add
            ;; BSIG0(a)
            (i64.xor
              (i64.xor
                (i64.rotr (local.get $a) (i64.const 28))
                (i64.rotr (local.get $a) (i64.const 34)))
              (i64.rotr (local.get $a) (i64.const 39)))
            ;; Maj(a,b,c) = (a & b) ^ (a & c) ^ (b & c)
            (i64.xor
              (i64.xor
                (i64.and (local.get $a) (i64.load offset=8  (i32.const 0x640))) ;; b
                (i64.and (local.get $a) (i64.load offset=16 (i32.const 0x640)))) ;; c
              (i64.and (i64.load offset=8  (i32.const 0x640))
                       (i64.load offset=16 (i32.const 0x640))))))

        ;; Shift state: hh = g; g = f; f = e; e = d + T1; d = c; c = b; b = a; a = T1 + T2.
        ;; Write from highest-offset down so we read old values before overwriting.
        (i64.store offset=56 (i32.const 0x640) (i64.load offset=48 (i32.const 0x640))) ;; hh = g
        (i64.store offset=48 (i32.const 0x640) (i64.load offset=40 (i32.const 0x640))) ;; g  = f
        (i64.store offset=40 (i32.const 0x640) (local.get $e))                          ;; f  = old e
        (i64.store offset=32 (i32.const 0x640) (i64.add (i64.load offset=24 (i32.const 0x640)) (local.get $t1))) ;; e = d + t1
        (i64.store offset=24 (i32.const 0x640) (i64.load offset=16 (i32.const 0x640))) ;; d  = c
        (i64.store offset=16 (i32.const 0x640) (i64.load offset=8  (i32.const 0x640))) ;; c  = b
        (i64.store offset=8  (i32.const 0x640) (local.get $a))                          ;; b  = old a
        (i64.store offset=0  (i32.const 0x640) (i64.add (local.get $t1) (local.get $t2))) ;; a = t1+t2

        (local.set $i (i32.add (local.get $i) (i32.const 8)))
        (br $rounds)))

    ;; --- Add the compressed chunk (working state) back into h[] ---
    (i64.store offset=0  (i32.const 0x040) (i64.add (i64.load offset=0  (i32.const 0x040)) (i64.load offset=0  (i32.const 0x640))))
    (i64.store offset=8  (i32.const 0x040) (i64.add (i64.load offset=8  (i32.const 0x040)) (i64.load offset=8  (i32.const 0x640))))
    (i64.store offset=16 (i32.const 0x040) (i64.add (i64.load offset=16 (i32.const 0x040)) (i64.load offset=16 (i32.const 0x640))))
    (i64.store offset=24 (i32.const 0x040) (i64.add (i64.load offset=24 (i32.const 0x040)) (i64.load offset=24 (i32.const 0x640))))
    (i64.store offset=32 (i32.const 0x040) (i64.add (i64.load offset=32 (i32.const 0x040)) (i64.load offset=32 (i32.const 0x640))))
    (i64.store offset=40 (i32.const 0x040) (i64.add (i64.load offset=40 (i32.const 0x040)) (i64.load offset=40 (i32.const 0x640))))
    (i64.store offset=48 (i32.const 0x040) (i64.add (i64.load offset=48 (i32.const 0x040)) (i64.load offset=48 (i32.const 0x640))))
    (i64.store offset=56 (i32.const 0x040) (i64.add (i64.load offset=56 (i32.const 0x040)) (i64.load offset=56 (i32.const 0x640)))))

  ;; --- main ---
  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $data_ptr i32)
    (local $remaining i32)
    (local $bit_len_lo i64)
    (local $tail_len i32)

    ;; h[0..7] = H[0..7] (one 64-byte copy from the initial-H data segment).
    (memory.copy (i32.const 0x040) (i32.const 0x080) (i32.const 64))

    ;; Copy the whole input into WASM memory at 0x1000 in one shot, then do
    ;; all stream/tail reads from WASM memory. This keeps the hot compress
    ;; loop's reads within the pre-allocated 2-page WASM region and avoids
    ;; any interleaved reads from the PVM args region (0xFEFF0000) during
    ;; computation. Under native WASM, args are already at 0x1000, so this
    ;; is effectively a no-op self-copy; under PVM, it pulls args into
    ;; WASM memory once up-front.
    (memory.copy (i32.const 0x1000) (local.get $args_ptr) (local.get $args_len))
    (local.set $data_ptr  (i32.const 0x1000))
    (local.set $remaining (local.get $args_len))

    ;; Total bit length (for the final padding). SHA-512 uses a 128-bit big-
    ;; endian bit-count; our cap (64 KB) keeps the value in the low 64 bits.
    ;; Cast args_len -> i64 and << 3.
    (local.set $bit_len_lo
      (i64.shl (i64.extend_i32_u (local.get $args_len)) (i64.const 3)))

    ;; --- Stream full 128-byte blocks while remaining >= 128 ---
    (block $stream_exit
      (loop $stream
        (br_if $stream_exit (i32.lt_u (local.get $remaining) (i32.const 128)))
        (call $compress (local.get $data_ptr))
        (local.set $data_ptr  (i32.add (local.get $data_ptr)  (i32.const 128)))
        (local.set $remaining (i32.sub (local.get $remaining) (i32.const 128)))
        (br $stream)))

    ;; --- Final-block padding ---
    ;; tail_len = remaining (0..127)
    (local.set $tail_len (local.get $remaining))

    ;; Zero the padding buffer (128 bytes).
    (memory.fill (i32.const 0x5c0) (i32.const 0) (i32.const 128))

    ;; Copy tail_len bytes of input into the buffer.
    (if (i32.gt_u (local.get $tail_len) (i32.const 0))
      (then
        (memory.copy (i32.const 0x5c0) (local.get $data_ptr) (local.get $tail_len))))

    ;; Write the 0x80 terminator byte at [tail_len].
    (i32.store8
      (i32.add (i32.const 0x5c0) (local.get $tail_len))
      (i32.const 0x80))

    (if (i32.le_u (local.get $tail_len) (i32.const 111))
      (then
        ;; --- Single-block padding ---
        ;; Write the 128-bit BE bit length into bytes [112..127].
        ;; Upper 8 bytes are zero (memory.fill already zeroed them); write
        ;; the lower 8 as big-endian (bswap).
        (i64.store offset=120 (i32.const 0x5c0)
          (call $bswap64 (local.get $bit_len_lo)))
        (call $compress (i32.const 0x5c0)))
      (else
        ;; --- Two-block padding ---
        ;; First block: tail + 0x80 + zeros to end-of-block. (Already
        ;; assembled in 0x5C0..0x63F — terminator at tail_len, rest zero.)
        (call $compress (i32.const 0x5c0))
        ;; Second block: 112 zeros + 16-byte BE bit length.
        (memory.fill (i32.const 0x5c0) (i32.const 0) (i32.const 128))
        (i64.store offset=120 (i32.const 0x5c0)
          (call $bswap64 (local.get $bit_len_lo)))
        (call $compress (i32.const 0x5c0))))

    ;; --- Output: h[] as 8 × BE i64 into the output buffer at offset 0. ---
    (i64.store offset=0  (i32.const 0) (call $bswap64 (i64.load offset=0  (i32.const 0x040))))
    (i64.store offset=8  (i32.const 0) (call $bswap64 (i64.load offset=8  (i32.const 0x040))))
    (i64.store offset=16 (i32.const 0) (call $bswap64 (i64.load offset=16 (i32.const 0x040))))
    (i64.store offset=24 (i32.const 0) (call $bswap64 (i64.load offset=24 (i32.const 0x040))))
    (i64.store offset=32 (i32.const 0) (call $bswap64 (i64.load offset=32 (i32.const 0x040))))
    (i64.store offset=40 (i32.const 0) (call $bswap64 (i64.load offset=40 (i32.const 0x040))))
    (i64.store offset=48 (i32.const 0) (call $bswap64 (i64.load offset=48 (i32.const 0x040))))
    (i64.store offset=56 (i32.const 0) (call $bswap64 (i64.load offset=56 (i32.const 0x040))))

    ;; Return (0 | (64 << 32)) = 0x0000_0040_0000_0000 = 274877906944
    (i64.const 274877906944)))
