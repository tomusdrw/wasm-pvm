# Polkadot Runtimes — WASM → PVM Compilation

Compilation results for the [polkadot-fellows/runtimes v2.2.2](https://github.com/polkadot-fellows/runtimes/releases/tag/v2.2.2) release, produced by `./compile.sh`.

Runtimes ship as Substrate "compact compressed" blobs: an 8-byte magic header (`52 BC 53 76 46 DB 8E 05`) followed by zstd-compressed WASM. The script strips the header, decompresses, verifies the WebAssembly magic (`\0asm`), generates a trap-all import map, then invokes `wasm-pvm compile`.

Compiled with **`--trap-floats`** (f32/f64 ops replaced with runtime traps so compilation can finish past the float wall).

## Results

| Runtime | Compressed | WASM | Imports | Status | JAM | Time | Reason |
|---------|-----------:|-----:|--------:|--------|----:|-----:|--------|
| asset-hub-kusama_runtime-v2002002 | 2.15 MiB | 11.89 MiB | 50 | :white_check_mark: ok | 33.68 MiB | 28s |  |
| asset-hub-polkadot_runtime-v2002002 | 2.09 MiB | 11.32 MiB | 50 | :white_check_mark: ok | 32.15 MiB | 25s |  |
| bridge-hub-kusama_runtime-v2002002 | 1.00 MiB | 4.82 MiB | 42 | :white_check_mark: ok | 14.55 MiB | 10s |  |
| bridge-hub-polkadot_runtime-v2002002 | 1.21 MiB | 5.89 MiB | 43 | :white_check_mark: ok | 17.57 MiB | 11s |  |
| bulletin-polkadot_runtime-v2002002 | 899.18 KiB | 4.21 MiB | 42 | :white_check_mark: ok | 12.93 MiB | 9s |  |
| collectives-polkadot_runtime-v2002002 | 1.08 MiB | 5.56 MiB | 42 | :white_check_mark: ok | 16.57 MiB | 11s |  |
| coretime-kusama_runtime-v2002002 | 977.73 KiB | 4.64 MiB | 42 | :white_check_mark: ok | 14.03 MiB | 9s |  |
| coretime-polkadot_runtime-v2002002 | 976.53 KiB | 4.64 MiB | 42 | :white_check_mark: ok | 14.03 MiB | 9s |  |
| encointer-kusama_runtime-v2002002 | 1.16 MiB | 5.75 MiB | 42 | :white_check_mark: ok | 16.99 MiB | 12s |  |
| glutton-kusama_runtime-v2002002 | 459.42 KiB | 2.04 MiB | 41 | :white_check_mark: ok | 6.86 MiB | 4s |  |
| kusama_runtime-v2002002 | 1.65 MiB | 8.43 MiB | 47 | :white_check_mark: ok | 24.10 MiB | 18s |  |
| people-kusama_runtime-v2002002 | 973.89 KiB | 4.63 MiB | 42 | :white_check_mark: ok | 14.03 MiB | 9s |  |
| people-polkadot_runtime-v2002002 | 1.03 MiB | 5.14 MiB | 42 | :white_check_mark: ok | 15.45 MiB | 10s |  |
| polkadot_runtime-v2002002 | 1.58 MiB | 7.94 MiB | 47 | :white_check_mark: ok | 22.81 MiB | 17s |  |

## Float operations per runtime

The compiler rejects `f32`/`f64` instructions (PVM has no floating-point support). This table lists the float-op kinds present in each module.

| Runtime | Float ops |
|---------|-----------|
| asset-hub-kusama_runtime-v2002002 | f32.add,f32.convert_i32_u,f32.gt,f32.load,f32.mul,f64.abs,f64.const,f64.convert_i64_u,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.promote_f32,f64.store,i64.reinterpret_f64 |
| asset-hub-polkadot_runtime-v2002002 | f32.add,f32.convert_i32_u,f32.gt,f32.load,f32.mul,f64.abs,f64.const,f64.convert_i64_u,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.promote_f32,f64.store,i64.reinterpret_f64 |
| bridge-hub-kusama_runtime-v2002002 | f64.abs,f64.const,f64.convert_i64_u,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store,i64.reinterpret_f64 |
| bridge-hub-polkadot_runtime-v2002002 | f64.abs,f64.const,f64.convert_i64_u,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store,i64.reinterpret_f64 |
| bulletin-polkadot_runtime-v2002002 | f64.abs,f64.const,f64.convert_i64_u,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store,i64.reinterpret_f64 |
| collectives-polkadot_runtime-v2002002 | f64.abs,f64.const,f64.convert_i64_u,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store,i64.reinterpret_f64 |
| coretime-kusama_runtime-v2002002 | f64.abs,f64.const,f64.convert_i64_u,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store,i64.reinterpret_f64 |
| coretime-polkadot_runtime-v2002002 | f64.abs,f64.const,f64.convert_i64_u,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store,i64.reinterpret_f64 |
| encointer-kusama_runtime-v2002002 | f64.abs,f64.const,f64.convert_i32_u,f64.convert_i64_u,f64.div,f64.eq,f64.load,f64.lt,f64.mul,f64.ne,f64.neg,f64.store,i64.reinterpret_f64 |
| glutton-kusama_runtime-v2002002 | f64.abs,f64.const,f64.convert_i64_u,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store,i64.reinterpret_f64 |
| kusama_runtime-v2002002 | f64.abs,f64.const,f64.convert_i64_u,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store,i64.reinterpret_f64 |
| people-kusama_runtime-v2002002 | f64.abs,f64.const,f64.convert_i64_u,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store,i64.reinterpret_f64 |
| people-polkadot_runtime-v2002002 | f64.abs,f64.const,f64.convert_i64_u,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store,i64.reinterpret_f64 |
| polkadot_runtime-v2002002 | f64.abs,f64.const,f64.convert_i64_u,f64.div,f64.eq,f64.load,f64.mul,f64.ne,f64.neg,f64.store,i64.reinterpret_f64 |

## How to reproduce

```bash
cd examples/polkadot && ./compile.sh
```

Set `COMPILE_TIMEOUT=600` to relax the per-runtime time budget, or `TRAP_FLOATS=0` to disable the `--trap-floats` flag (so the first f32/f64 op surfaces as a hard error instead of becoming a runtime trap). The pipeline is currently pinned to release v2.2.2; overriding `RELEASE_TAG` is rejected until the asset list is derived from the tag.

Compressed downloads land in `runtimes/`, decompressed modules in `wasm/`, generated trap-all import maps in `imports/`, JAM outputs in `jam/`, and full compile logs in `logs/`. All five directories are gitignored.
