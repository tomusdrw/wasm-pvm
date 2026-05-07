# Trap Floats Mode

PVM has no floating-point instructions. By default, the compiler rejects any
f32/f64 operator with a `FloatNotSupported` or `Unsupported(...)` error,
making it impossible to compile any WASM module that touches floats — even if
the float code path is never exercised at runtime.

The `--trap-floats` flag (or `CompileOptions::trap_floats = true` in the
library API) changes this behavior: every f32/f64 operator is replaced with a
runtime PVM `trap` instruction. Compilation completes; if execution ever
reaches one of those operators, the program traps deterministically.

## When to use it

- **Triage**: a real-world WASM module fails on its first float op. Use
  `--trap-floats` to push past the wall and discover what *other* unsupported
  features the module uses (data segments, exotic SIMD ops, etc.). The
  diagnostic upgrade in the same release prints the failing function and op
  offset for any remaining errors, so a single re-compile usually pinpoints
  every blocker.

- **Compiling integer-only entry paths in float-heavy modules**: if the float
  code is dead under your inputs (e.g. error-formatting helpers that you'll
  never trigger), `--trap-floats` makes the rest of the module shippable.

## When *not* to use it

- **Production builds where any float computation is reachable.** The trap is
  silent at compile time and only fires at runtime. If you're not certain the
  float code is dead, you'll ship a JAM that traps on real input.

- **Soft-float emulation.** `--trap-floats` does *not* emulate IEEE 754
  arithmetic. There is currently no plan to add soft-float support; if your
  module needs working floats, PVM is the wrong target.

## How it works

The frontend has a small table mapping each f32/f64 operator to its
`(pop_count, push_count)` stack effect. When `trap_floats` is enabled and a
float operator is encountered:

1. An LLVM `unreachable` instruction is emitted (which the PVM backend lowers
   to a `Trap` instruction).
2. A fresh basic block is created and the IR builder positions there. The
   block has no predecessor edge, so subsequent operators translate into
   provably-dead code that LLVM's `dce` pass removes.
3. The translator pops `pop_count` entries from the operand stack and pushes
   `push_count` zero placeholders, keeping the operand stack shape consistent
   with the WASM validator's view of the rest of the function.

The translator does **not** set its `unreachable` flag. That flag is reserved
for WASM-level dead-code skipping (driven by `unreachable`/`return`/`br`); a
float trap is structurally still "live code" from the WASM operand-stack
perspective — the placeholders flow into subsequent ops normally, even though
LLVM will optimise them away.

This approach handles the tricky corner cases:

- A float op inside one arm of an `if` traps that arm; the merge block's phi
  still receives an incoming edge from the after-trap block (with a
  placeholder zero), keeping the IR valid.
- A function that returns f64 still produces a function-end phi with at least
  one incoming branch (the placeholder zero pushed after the trap).
- Calls between functions with float signatures keep working because the
  i64-uniform calling convention treats every parameter and return value as
  i64 anyway — both caller and callee just pass placeholders that nobody
  reads before the trap fires.

## Float operators covered

All MVP f32/f64 operators (≈60 ops) are covered:

- Constants: `f32.const`, `f64.const`
- Loads / stores: `f{32,64}.{load,store}`
- Unary: `abs`, `neg`, `sqrt`, `ceil`, `floor`, `trunc`, `nearest`
- Binary: `add`, `sub`, `mul`, `div`, `min`, `max`, `copysign`
- Comparisons: `eq`, `ne`, `lt`, `gt`, `le`, `ge` (return i32)
- Conversions: every variant of `i{32,64}.trunc[_sat]_f{32,64}_{s,u}`,
  `f{32,64}.convert_i{32,64}_{s,u}`, `f32.demote_f64`, `f64.promote_f32`,
  `{i,f}{32,64}.reinterpret_{f,i}{32,64}`

SIMD float operators (`f32x4.*`, `f64x2.*`) are *not* in this set; modules
using SIMD will still fail with the SIMD operator's own unsupported error.

## Example

```bash
# Default: compilation fails on the first float op.
$ wasm-pvm compile runtime.wasm -o runtime.jam
Error: Compilation failed

Caused by:
    Unsupported WASM feature: F64Load { memarg: ... } (in function #42 'compute_score' at byte offset 0x1a3)

# With --trap-floats: compiles, traps at runtime if compute_score is called.
$ wasm-pvm compile runtime.wasm -o runtime.jam --trap-floats
wasm-pvm v0.8.0
...
Compiled in 312ms
```
