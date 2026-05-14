# Polkadot Runtimes — WASM → PVM Compilation

Compilation results for the [polkadot-fellows/runtimes v2.2.2](https://github.com/polkadot-fellows/runtimes/releases/tag/v2.2.2) release, produced by `./compile.sh`.

Runtimes ship as Substrate "compact compressed" blobs: an 8-byte magic header (`52 BC 53 76 46 DB 8E 05`) followed by zstd-compressed WASM. The script strips the header, decompresses, verifies the WebAssembly magic (`\0asm`), generates a trap-all import map, then invokes `wasm-pvm compile`.

Compiled with **`--trap-floats`** (f32/f64 ops replaced with runtime traps so compilation can finish past the float wall).

## Results

| Runtime | Compressed | WASM | Imports | Status | JAM | Time | Reason |
|---------|-----------:|-----:|--------:|--------|----:|-----:|--------|
| asset-hub-kusama_runtime-v2002002 | 2.15 MiB | 11.89 MiB | 50 | :white_check_mark: ok | 33.68 MiB | 26s |  |
| asset-hub-polkadot_runtime-v2002002 | 2.09 MiB | 11.32 MiB | 50 | :white_check_mark: ok | 32.15 MiB | 25s |  |
| bridge-hub-kusama_runtime-v2002002 | 1.00 MiB | 4.82 MiB | 42 | :white_check_mark: ok | 14.55 MiB | 9s |  |
| bridge-hub-polkadot_runtime-v2002002 | 1.21 MiB | 5.89 MiB | 43 | :white_check_mark: ok | 17.57 MiB | 11s |  |
| bulletin-polkadot_runtime-v2002002 | 899.18 KiB | 4.21 MiB | 42 | :white_check_mark: ok | 12.93 MiB | 9s |  |
| collectives-polkadot_runtime-v2002002 | 1.08 MiB | 5.56 MiB | 42 | :white_check_mark: ok | 16.57 MiB | 11s |  |
| coretime-kusama_runtime-v2002002 | 977.73 KiB | 4.64 MiB | 42 | :white_check_mark: ok | 14.03 MiB | 9s |  |
| coretime-polkadot_runtime-v2002002 | 976.53 KiB | 4.64 MiB | 42 | :white_check_mark: ok | 14.03 MiB | 10s |  |
| encointer-kusama_runtime-v2002002 | 1.16 MiB | 5.75 MiB | 42 | :white_check_mark: ok | 16.99 MiB | 11s |  |
| glutton-kusama_runtime-v2002002 | 459.42 KiB | 2.04 MiB | 41 | :white_check_mark: ok | 6.86 MiB | 5s |  |
| kusama_runtime-v2002002 | 1.65 MiB | 8.43 MiB | 47 | :white_check_mark: ok | 24.10 MiB | 17s |  |
| people-kusama_runtime-v2002002 | 973.89 KiB | 4.63 MiB | 42 | :white_check_mark: ok | 14.03 MiB | 10s |  |
| people-polkadot_runtime-v2002002 | 1.03 MiB | 5.14 MiB | 42 | :white_check_mark: ok | 15.45 MiB | 10s |  |
| polkadot_runtime-v2002002 | 1.58 MiB | 7.94 MiB | 47 | :white_check_mark: ok | 22.81 MiB | 17s |  |

## Float operations per runtime

The compiler rejects `f32`/`f64` instructions (PVM has no floating-point support). This table lists the float-op kinds present in each module.

| Runtime | Float ops |
|---------|-----------|
| asset-hub-kusama_runtime-v2002002 | f32.add,f32.convert_i,f32.gt,f32.load,f32.mul,f64.abs,f64.const,f64.convert_i,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.promote_f,f64.store |
| asset-hub-polkadot_runtime-v2002002 | f32.add,f32.convert_i,f32.gt,f32.load,f32.mul,f64.abs,f64.const,f64.convert_i,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.promote_f,f64.store |
| bridge-hub-kusama_runtime-v2002002 | f64.abs,f64.const,f64.convert_i,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store |
| bridge-hub-polkadot_runtime-v2002002 | f64.abs,f64.const,f64.convert_i,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store |
| bulletin-polkadot_runtime-v2002002 | f64.abs,f64.const,f64.convert_i,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store |
| collectives-polkadot_runtime-v2002002 | f64.abs,f64.const,f64.convert_i,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store |
| coretime-kusama_runtime-v2002002 | f64.abs,f64.const,f64.convert_i,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store |
| coretime-polkadot_runtime-v2002002 | f64.abs,f64.const,f64.convert_i,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store |
| encointer-kusama_runtime-v2002002 | f64.abs,f64.const,f64.convert_i,f64.div,f64.eq,f64.load,f64.lt,f64.mul,f64.ne,f64.neg,f64.store |
| glutton-kusama_runtime-v2002002 | f64.abs,f64.const,f64.convert_i,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store |
| kusama_runtime-v2002002 | f64.abs,f64.const,f64.convert_i,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store |
| people-kusama_runtime-v2002002 | f64.abs,f64.const,f64.convert_i,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store |
| people-polkadot_runtime-v2002002 | f64.abs,f64.const,f64.convert_i,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store |
| polkadot_runtime-v2002002 | f64.abs,f64.const,f64.convert_i,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store |

## How to reproduce

```bash
cd examples/polkadot && ./compile.sh
```

Set `RELEASE_TAG=vX.Y.Z` to target a different release, `COMPILE_TIMEOUT=600` to relax the per-runtime time budget, or `TRAP_FLOATS=0` to disable the `--trap-floats` flag (so the first f32/f64 op surfaces as a hard error instead of becoming a runtime trap).

Compressed downloads land in `runtimes/`, decompressed modules in `wasm/`, generated trap-all import maps in `imports/`, JAM outputs in `jam/`, and full compile logs in `logs/`. All four directories are gitignored.
