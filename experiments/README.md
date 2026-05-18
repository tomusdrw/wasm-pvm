# Compiler optimization experiments

Per-flag impact measurement for the WASM→PVM compiler's `OptimizationFlags`.

## What's here

| File | Description |
|---|---|
| `opt_impact.sh` | Driver script — toggles each `--no-X` flag individually and records JAM/code-size deltas vs. the all-on baseline. |
| `results/baseline.csv` | One row per input: source path, baseline JAM size, baseline PVM code size. |
| `results/deltas.csv` | One row per (flag, input): `delta_jam`, `delta_code`, `compile_ok`. Positive = optimization saved that many bytes when on. |
| `results/summary.md` | Aggregate matrix + per-flag totals + provably-dead callout. Read this first. |
| `analysis.md` | Human-written analysis of the most recent run with concrete recommendations. |

## How to run

```bash
# Full run (~75–90 min): all fixtures + as-lan + 14 polkadot runtimes
./experiments/opt_impact.sh

# Phases
./experiments/opt_impact.sh --fixtures   # ~15 min, hand-crafted WAT + AS-built WASM
./experiments/opt_impact.sh --aslan      # ~2 min, aslan-fib / aslan-ecalli / aslan-debug / anan-as-compiler
./experiments/opt_impact.sh --polkadot   # ~60 min, 14 polkadot-fellows v2.2.2 runtimes
```

Each run overwrites `results/`. Commit the output if you want to track drift between commits.

### Prerequisites

- `cargo`, `bun`, `node`, `python3`, `wasm-tools` on `PATH`.
- Polkadot runtimes populated: `cd examples/polkadot && ./compile.sh` (~10 min download + decompress; the same script also compiles them to JAM, which is unnecessary for this experiment but currently coupled).
- anan-as built: `git submodule update --init vendor/anan-as && cd vendor/anan-as && npm install && npm run asbuild:compiler`.

Any input whose source is missing is silently skipped, so partial setups work — they just produce smaller matrices.

## What the deltas mean

`delta_jam` is `(jam_size_with_flag_off) − (jam_size_baseline)`.

- **Positive** → the optimization removed that many bytes when on (it's pulling weight).
- **Zero** → the optimization had no effect on this input (precondition never fired, or was shadowed by another pass).
- **Negative** → the optimization made the output *larger* when on (a regression on that input).

Aggregate over many inputs to see whether a flag is broadly useful, narrowly useful, or net-negative.

## When to re-run

- After any compiler change that touches an optimization pass, its gate, or the LLVM pipeline.
- Before merging a PR that claims to "improve" a pass — verify the claim on the matrix, not just one benchmark.
- After upgrading LLVM or `inkwell` — pass behavior can shift silently.

The summary and analysis files in `results/` are point-in-time snapshots, not living documentation. Always check the timestamp at the top of `analysis.md`.

## Limitations

- **Size-only.** This experiment measures JAM size and PVM code size, not gas usage or runtime time. An optimization that increases size to reduce gas (e.g. unrolling, dispatch tables) shows as a regression here. See [#TODO gas-cross-check issue] for the planned follow-up.
- **One-at-a-time.** Flags are toggled individually, with all others on. Interaction effects between disabled pairs aren't measured.
- **Input set is representative, not exhaustive.** 31 inputs cover hand-crafted WAT, AS-built WASM, real-world as-lan services, and 14 polkadot runtimes — enough to flag dead passes and net-negative behavior, but a new fixture class (e.g. Rust-built WASM with `__multi3`-heavy code) could change conclusions.
