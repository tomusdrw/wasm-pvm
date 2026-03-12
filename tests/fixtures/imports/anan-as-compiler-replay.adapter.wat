;; Replay adapter for anan-as compiler: forwards ecalli via a buffer protocol.
;;
;; Instead of translating pointers per-ecalli, this adapter uses a simple protocol:
;;   - Outer ecalli 0 ("forward"): sends scratch_pvm_addr + inner ecalli index.
;;     The outer handler writes the response (new registers + memwrites) into
;;     the scratch buffer. The adapter applies memwrites via host_write_memory.
;;   - Outer ecalli 1 ("get r8"): returns the r8 captured by the last forward.
;;
;; Scratch buffer response format (written by outer handler at scratch_pvm_addr):
;;   [8: new_r7 (LE i64)]
;;   [8: new_r8 (LE i64)]
;;   [4: num_memwrites (LE u32)]
;;   [8: new_gas (LE i64), 0 = no change]
;;   For each memwrite:
;;     [4: inner_addr (LE u32)]
;;     [4: data_len (LE u32)]
;;     [data_len: data bytes]
;;
;; Scratch page allocation: grows WASM memory once (on first ecalli) and caches
;; the page address at a fixed sentinel location (WASM addr 0xFFFF0) for reuse.
(module
  ;; Outer ecalli 0: forward inner ecalli.
  ;; host_call_2(ecalli=0, r7=scratch_pvm_addr, r8=inner_ecalli_idx) -> r7
  (import "env" "host_call_2" (func $replay_forward (param i64 i64 i64) (result i64)))

  ;; Outer ecalli 1: get last r8.
  ;; host_call_0(ecalli=1) -> r7
  (import "env" "host_call_0" (func $get_last_r8 (param i64) (result i64)))

  ;; PVM pointer translation
  (import "env" "pvm_ptr" (func $pvm_ptr (param i64) (result i64)))

  ;; Compiler export: write to inner PVM memory (resolved by adapter_merge).
  (import "env" "host_write_memory" (func $host_write_memory (param i32 i32 i32) (result i32)))

  ;; --- abort / console.log: unused in replay mode ---
  (func (export "abort") (param i32 i32 i32 i32)
    unreachable
  )
  (func (export "console.log") (param i32)
    unreachable
  )

  ;; --- host_call_6b: forward inner ecalli via scratch buffer protocol ---
  (func (export "host_call_6b")
    (param $ecalli i64) (param $r7 i64) (param $r8 i64) (param $r9 i64)
    (param $r10 i64) (param $r11 i64) (param $r12 i64)
    (result i64)

    (local $scratch_addr i32)
    (local $scratch_page i32)
    (local $num_writes i32)
    (local $offset i32)
    (local $inner_addr i32)
    (local $data_len i32)

    ;; Allocate scratch page once, reuse on subsequent calls.
    ;; Sentinel at WASM addr 0xFFFF0 stores the scratch base address.
    ;; This address is in the compiler module's linear memory (at least 16 pages = 1MB),
    ;; well beyond the compiler's own data usage. The compiler (anan-as) uses memory
    ;; starting from its heap base (~0x30000+), so 0xFFFF0 is in unused high space
    ;; within the initial 16 pages. This is an adapter-internal convention, not a
    ;; general-purpose pattern — see LEARNINGS.md for caveats.
    (local.set $scratch_addr (i32.load (i32.const 0xFFFF0)))
    (if (i32.eqz (local.get $scratch_addr))
      (then
        (local.set $scratch_page (memory.grow (i32.const 1)))
        (if (i32.eq (local.get $scratch_page) (i32.const -1))
          (then unreachable))
        (local.set $scratch_addr (i32.shl (local.get $scratch_page) (i32.const 16)))
        (i32.store (i32.const 0xFFFF0) (local.get $scratch_addr))
      )
    )

    ;; Forward: outer ecalli 0, r7 = scratch PVM addr, r8 = inner ecalli index
    (drop (call $replay_forward
      (i64.const 0)
      (call $pvm_ptr (i64.extend_i32_u (local.get $scratch_addr)))
      (local.get $ecalli)
    ))

    ;; Process memwrites from the scratch buffer.
    ;; Layout: [8:r7][8:r8][4:num_memwrites][8:new_gas][memwrites...]
    (local.set $num_writes (i32.load (i32.add (local.get $scratch_addr) (i32.const 16))))
    ;; new_gas at offset 20 (8 bytes) — currently ignored by the adapter because
    ;; the inner interpreter manages its own gas counter. To support setgas,
    ;; the compiler module would need to export a gas-setter function.
    ;; TODO(#174): apply setgas when compiler supports it.
    (local.set $offset (i32.add (local.get $scratch_addr) (i32.const 28)))

    (block $done
      (loop $loop
        (br_if $done (i32.eqz (local.get $num_writes)))

        (local.set $inner_addr (i32.load (local.get $offset)))
        (local.set $data_len (i32.load (i32.add (local.get $offset) (i32.const 4))))

        ;; Write data to inner PVM memory
        (drop (call $host_write_memory
          (local.get $inner_addr)
          (i32.add (local.get $offset) (i32.const 8))
          (local.get $data_len)
        ))

        ;; Advance past this entry: 8 header bytes + data_len
        (local.set $offset
          (i32.add (local.get $offset)
            (i32.add (i32.const 8) (local.get $data_len))))
        (local.set $num_writes (i32.sub (local.get $num_writes) (i32.const 1)))
        (br $loop)
      )
    )

    ;; Return new_r7 from scratch buffer offset 0
    (i64.load (local.get $scratch_addr))
  )

  ;; --- host_call_r8: delegate to outer ecalli 1 ---
  (func (export "host_call_r8") (result i64)
    (call $get_last_r8 (i64.const 1))
  )
)
