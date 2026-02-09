;; Test: br_table dispatch after call_indirect with spilled locals
;; This mimics the EXACT nextSteps pattern:
;; 1. Set spilled local 7 (skipBytes)
;; 2. call_indirect returns an outcome object
;; 3. br_table dispatches on outcome.kind
;; 4. In OK case (kind=0), use spilled local 7 to update PC
;;
;; Expected results:
;; Step 0: kind=0 (OK) → PC update 42+(9+1)=52 → 52
;; Step 1: kind=1 (Jump) → set PC to 999 → 999
;; Step 2: kind=0 (OK) but different skipBytes (4) → 42+(4+1)=47 → 47
;; Step 3: kind=0 (OK) via loop iteration → PC update twice → 52+(9+1)=62 → 62

(module
  (memory 1)

  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))
  (global $outcome_kind (mut i32) (i32.const 0))

  ;; Function table
  (table 2 funcref)
  (elem (i32.const 0) $make_outcome $other_fn)
  (type $exe_type (func (param i32 i32 i32 i32) (result i32)))

  ;; Returns a pointer to an "outcome" object whose kind field is set by global
  ;; Outcome object at address 300: kind(4 bytes) + staticJump(4 bytes) + ...
  (func $make_outcome (param i32) (param i32) (param i32) (param i32) (result i32)
    ;; Write outcome.kind at address 300
    (i32.store (i32.const 300) (global.get $outcome_kind))
    ;; Write outcome.staticJump at address 304 (used for jump case)
    (i32.store offset=4 (i32.const 300) (i32.const 999))
    ;; Return pointer to outcome object
    (i32.const 300)
  )

  (func $other_fn (param i32) (param i32) (param i32) (param i32) (result i32)
    (i32.const 0)
  )

  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    ;; 17 locals total
    (local $step i32)       ;; local 2
    (local $obj_ptr i32)    ;; local 3
    (local $l4 i32)         ;; local 4 - spilled
    (local $l5 i32)         ;; local 5 - spilled
    (local $l6 i32)         ;; local 6 - spilled (pc, read from obj)
    (local $l7 i32)         ;; local 7 - spilled (skipBytes)
    (local $l8 i32)         ;; local 8 - spilled (instruction data)
    (local $l9 i32)         ;; local 9 - spilled (opcode)
    (local $l10 i32)        ;; local 10 - spilled (step counter)
    (local $l11 i32)        ;; local 11 - spilled
    (local $l12 i32)        ;; local 12 - spilled
    (local $outcome i32)    ;; local 13 - spilled (call_indirect result)
    (local $l14 i32)        ;; local 14 - spilled
    (local $l15 i32)        ;; local 15 - spilled
    (local $l16 i32)        ;; local 16 - spilled

    ;; Read step from args
    (local.set $step (i32.load (local.get $args_ptr)))

    ;; Set up
    (local.set $obj_ptr (i32.const 200))
    (local.set $l7 (i32.const 9))    ;; skipBytes = 9
    (local.set $l4 (i32.const 111))
    (local.set $l5 (i32.const 222))
    (local.set $l6 (i32.const 0))
    (local.set $l8 (i32.const 444))
    (local.set $l9 (i32.const 555))

    ;; Store initial PC value: object.field16 = 42
    (i32.store offset=16 (local.get $obj_ptr) (i32.const 42))

    ;; Step 0: OK outcome (kind=0) → PC update with skipBytes=9
    (if (i32.eqz (local.get $step))
      (then
        (global.set $outcome_kind (i32.const 0))  ;; OK

        ;; call_indirect like nextSteps does
        (local.set $outcome
          (call_indirect (type $exe_type)
            (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
            (i32.const 0)  ;; table index
          )
        )

        ;; br_table dispatch on outcome.kind
        ;; Structure: block @outer { block @ok { block @jump { block @panic {
        ;;   br_table @ok @jump @panic @outer
        ;; } panic_code } jump_code } ok_code } fall_through
        (block $done
          (block $ok_target
            (block $jump_target
              (block $panic_target
                (br_table $ok_target $jump_target $panic_target $done
                  (i32.load (local.get $outcome))  ;; outcome.kind
                )
              )
              ;; panic_target: kind=2
              (i32.store (i32.const 0) (i32.const 0xDEAD))
              (br $done)
            )
            ;; jump_target: kind=1
            (i32.store offset=16 (local.get $obj_ptr) (i32.const 999))
            (br $done)
          )
          ;; ok_target: kind=0 - THIS is the PC update path
          ;; EXACT pattern from nextSteps:
          local.get $obj_ptr
          local.get $obj_ptr
          i32.load offset=16
          local.get $l7      ;; spilled local (skipBytes)
          i32.const 1
          i32.add
          i32.add
          i32.store offset=16
        )

        ;; Output
        (i32.store (i32.const 0) (i32.load offset=16 (local.get $obj_ptr)))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; Step 1: Jump outcome (kind=1) → set PC to 999
    (if (i32.eq (local.get $step) (i32.const 1))
      (then
        (global.set $outcome_kind (i32.const 1))  ;; Jump

        (local.set $outcome
          (call_indirect (type $exe_type)
            (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
            (i32.const 0)
          )
        )

        (block $done
          (block $ok_target
            (block $jump_target
              (block $panic_target
                (br_table $ok_target $jump_target $panic_target $done
                  (i32.load (local.get $outcome))
                )
              )
              (i32.store (i32.const 0) (i32.const 0xDEAD))
              (br $done)
            )
            ;; jump_target: set PC to outcome.staticJump
            (i32.store offset=16 (local.get $obj_ptr)
              (i32.load offset=4 (local.get $outcome))  ;; 999
            )
            (br $done)
          )
          ;; ok_target
          local.get $obj_ptr
          local.get $obj_ptr
          i32.load offset=16
          local.get $l7
          i32.const 1
          i32.add
          i32.add
          i32.store offset=16
        )

        (i32.store (i32.const 0) (i32.load offset=16 (local.get $obj_ptr)))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; Step 2: OK with different skipBytes
    (if (i32.eq (local.get $step) (i32.const 2))
      (then
        (local.set $l7 (i32.const 4))  ;; change skipBytes to 4
        (global.set $outcome_kind (i32.const 0))

        (local.set $outcome
          (call_indirect (type $exe_type)
            (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
            (i32.const 0)
          )
        )

        (block $done
          (block $ok_target
            (block $jump_target
              (block $panic_target
                (br_table $ok_target $jump_target $panic_target $done
                  (i32.load (local.get $outcome))
                )
              )
              (i32.store (i32.const 0) (i32.const 0xDEAD))
              (br $done)
            )
            (i32.store offset=16 (local.get $obj_ptr) (i32.const 999))
            (br $done)
          )
          ;; ok_target
          local.get $obj_ptr
          local.get $obj_ptr
          i32.load offset=16
          local.get $l7
          i32.const 1
          i32.add
          i32.add
          i32.store offset=16
        )

        (i32.store (i32.const 0) (i32.load offset=16 (local.get $obj_ptr)))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; Step 3: OK in a loop (2 iterations) → 42+10+10=62
    (if (i32.eq (local.get $step) (i32.const 3))
      (then
        (global.set $outcome_kind (i32.const 0))
        (local.set $l10 (i32.const 0))  ;; counter = 0

        (block $break
          (loop $loop
            (br_if $break (i32.ge_u (local.get $l10) (i32.const 2)))

            (local.set $outcome
              (call_indirect (type $exe_type)
                (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
                (i32.const 0)
              )
            )

            (block $done
              (block $ok_target
                (block $jump_target
                  (block $panic_target
                    (br_table $ok_target $jump_target $panic_target $done
                      (i32.load (local.get $outcome))
                    )
                  )
                  (br $done)
                )
                (br $done)
              )
              ;; ok_target: PC update
              local.get $obj_ptr
              local.get $obj_ptr
              i32.load offset=16
              local.get $l7
              i32.const 1
              i32.add
              i32.add
              i32.store offset=16
            )

            (local.set $l10 (i32.add (local.get $l10) (i32.const 1)))
            (br $loop)
          )
        )

        (i32.store (i32.const 0) (i32.load offset=16 (local.get $obj_ptr)))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; Default
    (i32.store (i32.const 0) (i32.const 0))
    (global.set $result_ptr (i32.const 0))
    (global.set $result_len (i32.const 4))
  )
)
