(module
  (memory 1)

  (global $result_ptr (mut i32) (i32.const 0))
  (global $result_len (mut i32) (i32.const 0))

  (func (export "main") (param $args_ptr i32) (param $args_len i32)
    (local $step i32)
    (local $old_pages i32)
    (local $new_pages i32)

    (local.set $step (i32.load (local.get $args_ptr)))

    ;; step 0: memory.size before grow (expect 1 page)
    (if (i32.eqz (local.get $step))
      (then
        (i32.store (i32.const 0) (memory.size))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; step 1: memory.grow(1), return old pages (expect 1)
    (if (i32.eq (local.get $step) (i32.const 1))
      (then
        (i32.store (i32.const 0) (memory.grow (i32.const 1)))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; step 2: memory.grow(1) then memory.size (expect 2 pages -> grow -> 3)
    (if (i32.eq (local.get $step) (i32.const 2))
      (then
        (local.set $old_pages (memory.grow (i32.const 1)))
        (local.set $new_pages (memory.size))
        ;; Return old_pages * 1000 + new_pages
        (i32.store (i32.const 0)
          (i32.add
            (i32.mul (local.get $old_pages) (i32.const 1000))
            (local.get $new_pages)
          )
        )
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; step 3: grow memory, then store/load at the new page boundary
    (if (i32.eq (local.get $step) (i32.const 3))
      (then
        ;; Grow by 1 page (to have at least 2 pages = 128KB)
        (drop (memory.grow (i32.const 1)))
        ;; Store 42 at offset 65536 (start of 2nd page)
        (i32.store (i32.const 65536) (i32.const 42))
        ;; Read it back
        (i32.store (i32.const 0) (i32.load (i32.const 65536)))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )

    ;; step 4: grow by 2 pages, store at 3rd page boundary, read back
    (if (i32.eq (local.get $step) (i32.const 4))
      (then
        (drop (memory.grow (i32.const 2)))
        ;; Store 99 at offset 131072 (start of 3rd page)
        (i32.store (i32.const 131072) (i32.const 99))
        (i32.store (i32.const 0) (i32.load (i32.const 131072)))
        (global.set $result_ptr (i32.const 0))
        (global.set $result_len (i32.const 4))
        (return)
      )
    )
  )
)
