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
  --no-register-alloc --no-dead-function-elim \
  --no-fallthrough-jumps
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
| `--no-dead-function-elim` | Remove unreachable functions from output |
| `--no-fallthrough-jumps` | Skip redundant Jump when target is next block |

See the [Optimizations](./optimizations.md) chapter for details on each.
