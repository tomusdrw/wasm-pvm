;; u128 division microbenchmark — measures dynamic gas impact of the
;; `__udivti3` b_hi specialization (fast u64/u64 path).
;;
;; Like u128-mul-bench, this fixture uses a STUB `specialized_div_rem`
;; rather than the real compiler-builtins implementation. The comparison
;; logic is:
;;
;;   With recognition (a_hi=b_hi=0): fast path takes a few PVM instr
;;     and never calls the stub.
;;   Without recognition:            original __udivti3 body runs, calls
;;     the stub, copies the result. ~30 PVM instr per iter.
;;
;; The stub doesn't need to match real division semantics because both
;; configurations use the same stub — the unfair-baseline trap from the
;; mul benchmark doesn't apply here. The fast/slow path *dispatch* is
;; what differs.
;;
;; Entry: main(args_ptr, args_len) -> i64
;;   args = [iter_count: u32 LE]   (recommended: 1000–100000)
;;   returns (acc_ptr=0x200, len=16)

(module
  (memory (export "memory") 1)
  (global $__stack_pointer (mut i32) (i32.const 65536))

  ;; Stub specialized_div_rem — writes a 32-byte result. The exact bytes
  ;; don't matter for gas measurement, only that the call happens.
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

  ;; Canonical compiler-builtins __udivti3 wrapper shape.
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

    ;; acc = 0xCAFEF00DCAFEF00D (arbitrary non-zero starting dividend)
    i64.const 0xCAFEF00DCAFEF00D
    local.set $acc_lo
    i64.const 0
    local.set $acc_hi

    (block $done
      (loop $bench
        local.get $iter
        i32.eqz
        br_if $done

        ;; acc = acc / 3  — divisor is a small constant, a_hi=b_hi=0
        ;; means the fast path fires unconditionally when recognition is on.
        ;; The dividend `acc` and divisor `3` are both u64-sized, fast path
        ;; eligible.
        i32.const 0x100              ;; sret
        local.get $acc_lo
        local.get $acc_hi
        i64.const 3
        i64.const 0
        call $__udivti3

        i32.const 0x100
        i64.load offset=0
        local.set $acc_lo
        i32.const 0x100
        i64.load offset=8
        local.set $acc_hi

        ;; If acc reaches 0, reset to keep loop work non-trivial.
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
