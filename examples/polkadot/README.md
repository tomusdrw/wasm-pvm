# Polkadot Runtime Triage

This directory exists for batch-compiling Polkadot fellowship runtime WASM
binaries through the `wasm-pvm` compiler. The runtimes themselves are not
checked in — drop your `*.wasm` files alongside `compile.sh` before running.

## Usage

```bash
# Default: compilation fails on the first unsupported operator. The new
# diagnostic prints the offending function and byte offset.
./compile.sh

# Trap-floats mode: convert every f32/f64 operator to a runtime trap so
# compilation can finish. Useful for finding what *other* unsupported
# features each runtime uses past the float wall.
TRAP_FLOATS=1 ./compile.sh

# Pass arbitrary extra flags to wasm-pvm.
COMPILE_ARGS="--no-inline --max-memory 32" ./compile.sh
```

Compiled `.jam` files and per-runtime logs land in `examples/polkadot/dist/`.

## Results

This table is intentionally blank in the upstream repo. Re-run the script
locally with whichever runtime set you care about and paste the output here:

| Runtime | Default | `--trap-floats` | First unsupported feature |
|---------|---------|-----------------|---------------------------|
| _example_ | FAIL @ `compute_score` 0x1a3 (`F64Load`) | OK | n/a |
