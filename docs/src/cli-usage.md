# CLI Usage

```bash
# Compile WAT or WASM to JAM
wasm-pvm compile input.wat -o output.jam
wasm-pvm compile input.wasm -o output.jam

# With import resolution
wasm-pvm compile input.wasm -o output.jam \
  --imports imports.txt \
  --adapter adapter.wat

# Disable specific optimizations
wasm-pvm compile input.wasm -o output.jam --no-inline --no-peephole

# Disable all optimizations
wasm-pvm compile input.wasm -o output.jam \
  --no-llvm-passes --no-peephole --no-register-cache \
  --no-icmp-fusion --no-shrink-wrap --no-dead-store-elim \
  --no-const-prop --no-inline --no-cross-block-cache \
  --no-register-alloc --no-fallthrough-jumps
```

## Optimization Flags

All non-trivial optimizations are enabled by default. Each can be individually disabled:

| Flag | What it controls |
|------|------------------|
| `--no-llvm-passes` | LLVM optimization passes (mem2reg, instcombine, etc.) |
| `--no-peephole` | Post-codegen peephole optimizer |
| `--no-register-cache` | Per-block store-load forwarding |
| `--no-icmp-fusion` | Fuse ICmp+Branch into single PVM branch |
| `--no-shrink-wrap` | Only save/restore used callee-saved regs |
| `--no-dead-store-elim` | Remove SP-relative stores never loaded from |
| `--no-const-prop` | Skip redundant LoadImm when register already holds the constant |
| `--no-inline` | LLVM function inlining for small callees |
| `--no-cross-block-cache` | Propagate register cache across single-predecessor block boundaries |
| `--no-register-alloc` | Linear-scan register allocation for loop values |
| `--no-fallthrough-jumps` | Skip redundant Jump when target is next block |

See the [Optimizations](./optimizations.md) chapter for details on each.

## Diagnostic & Triage Flags

These flags affect what the compiler accepts or how it reports failures. They
are *not* optimizations.

| Flag | What it does |
|------|--------------|
| `--trap-floats` | Replace every f32/f64 operator with a runtime trap instead of failing compilation. See [Trap Floats Mode](./trap-floats.md). |

When compilation fails on an unsupported operator, the error message includes
the function index, the function's display name (from the WASM `name` custom
section, falling back to the export name, then `wasm_func_<idx>`), and the
operator's byte offset within the function body. Example:

```text
Error: Compilation failed

Caused by:
    Unsupported WASM feature: F64Load { memarg: ... } (in function #42 'compute_score' at byte offset 0x1a3)
```

This makes it possible to grep into the WASM disassembly (`wasm-tools dump`)
or anan-as source to find the offending site without bisecting the module.
