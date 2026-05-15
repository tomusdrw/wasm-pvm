;; u128 division microbenchmark — slow path variant.
;;
;; Sister fixture to `u128-div-bench.jam.wat`. Same structure, but the
;; iter loop sets `b_hi = 1` so the b_hi specialization's check fails
;; and every iteration falls through to the slow path. Used to verify
;; the slow path doesn't regress under recognition (worst case: the
;; dispatch overhead adds ~4 PVM instr per call but doesn't change the
;; underlying compiler-builtins work).
;;
;; Same stub `specialized_div_rem` as the fast variant — the gas
;; comparison is between (with-recognition slow path) vs (no-recognition
;; original body); the stub's work is fixed across both runs.
;;
;; Entry: main(args_ptr, args_len) -> i64
;;   args = [iter_count: u32 LE]
;;   returns (acc_ptr=0x200, len=16)

(module
  (memory (export "memory") 1)
  (global $__stack_pointer (mut i32) (i32.const 65536))

  (func $sdr (param $sret i32) (param $a_lo i64) (param $a_hi i64) (param $b_lo i64) (param $b_hi i64)
    local.get $sret
    local.get $a_lo
    i64.store offset=0
    local.get $sret
    local.get $a_lo
    i64.store offset=8
    local.get $sret
    local.get $a_lo
    i64.store offset=16
    local.get $sret
    local.get $a_lo
    i64.store offset=24
  )

  (func $__udivti3 (param i32 i64 i64 i64 i64)
    (local i32)
    global.get $__stack_pointer
    i32.const 32
    i32.sub
    local.tee 5
    global.set $__stack_pointer
    local.get 5
    local.get 1
    local.get 2
    local.get 3
    local.get 4
    call $sdr
    local.get 0
    local.get 5
    i64.load
    i64.store
    local.get 0
    local.get 5
    i64.load offset=8
    i64.store offset=8
    local.get 5
    i32.const 32
    i32.add
    global.set $__stack_pointer
  )

  (func (export "main") (param $args_ptr i32) (param $args_len i32) (result i64)
    (local $iter i32)
    (local $acc_lo i64)
    (local $acc_hi i64)

    local.get $args_ptr
    i32.load offset=0
    local.set $iter

    i64.const 0xCAFEF00DCAFEF00D
    local.set $acc_lo
    i64.const 0
    local.set $acc_hi

    (block $done
      (loop $bench
        local.get $iter
        i32.eqz
        br_if $done

        ;; b_hi = 1  ─ forces the slow path on every iter.
        i32.const 0x100
        local.get $acc_lo
        local.get $acc_hi
        i64.const 3
        i64.const 1
        call $__udivti3

        i32.const 0x100
        i64.load offset=0
        local.set $acc_lo
        i32.const 0x100
        i64.load offset=8
        local.set $acc_hi

        local.get $acc_lo
        i64.eqz
        if
          i64.const 0xCAFEF00DCAFEF00D
          local.set $acc_lo
        end

        local.get $iter
        i32.const 1
        i32.sub
        local.set $iter
        br $bench
      )
    )

    i32.const 0x200
    local.get $acc_lo
    i64.store offset=0
    i32.const 0x200
    local.get $acc_hi
    i64.store offset=8

    i64.const 0x1000000200
  )
)
