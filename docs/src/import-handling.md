# Import Handling

WASM modules that import external functions need those imports resolved before compilation. Two mechanisms are available, and they can be combined.

## Import Map (`--imports`)

A text file mapping import names to simple actions:

```text
# my-imports.txt
abort = trap        # emit unreachable (panic)
console.log = nop   # do nothing, return zero
```

## Adapter WAT (`--adapter`)

A WAT module whose exported functions replace matching WASM imports, enabling arbitrary logic for import resolution (pointer conversion, memory reads, host calls). Adapters are function-only overlays — tables, memories, globals, and data sections from the adapter are not merged:

```wat
(module
  (import "env" "host_call_5" (func $host_call_5 (param i64 i64 i64 i64 i64 i64) (result i64)))
  (import "env" "pvm_ptr" (func $pvm_ptr (param i64) (result i64)))

  (func (export "console.log") (param i32)
    (drop (call $host_call_5
      (i64.const 100)                                    ;; ecalli index
      (i64.const 3)                                      ;; log level
      (i64.const 0) (i64.const 0)                        ;; target ptr/len
      (call $pvm_ptr (i64.extend_i32_u (local.get 0)))   ;; message ptr
      (i64.extend_i32_u (i32.load offset=0
        (i32.sub (local.get 0) (i32.const 4)))))))       ;; message len
)
```

When both `--imports` and `--adapter` are provided, the adapter runs first, then the import map handles remaining unresolved imports. All imports must be resolved or compilation fails.

## Host Call Imports

A family of typed `host_call_N` imports (N=0..6) map to PVM `ecalli` instructions, where N is the number of data registers (r7..r7+N-1) to set. See the [ABI & Calling Conventions](./architecture.md) chapter for the full reference table and examples.

Variants with a `b` suffix (e.g. `host_call_2b`) also capture r8 to a stack slot, retrievable via `host_call_r8() -> i64`.

The `pvm_ptr(wasm_addr) -> pvm_addr` import converts a WASM-space address to a PVM-space address.
