# Testing

The project has a comprehensive multi-layer test suite covering unit tests, integration tests, differential tests, and PVM-in-PVM execution tests.

## Quick Reference

```bash
# Rust unit tests
cargo test

# Lint
cargo clippy -- -D warnings

# Full integration tests (builds artifacts first)
cd tests && bun run test

# Quick validation (Layer 1 only — requires build first)
cd tests && bun build.ts && bun test layer1/

# PVM-in-PVM tests (requires build first)
cd tests && bun build.ts && bun test layer4/ layer5/ --test-name-pattern "pvm-in-pvm"

# Differential tests (PVM vs native WASM)
cd tests && bun run test:differential
```

**Important**: Always use `bun run test` (not `bun test`) from the `tests/` directory — it runs `bun build.ts` first to compile fixtures.

## Test Layers

| Layer | Tests | Purpose | Speed |
|-------|-------|---------|-------|
| Layer 1 | ~50 | Core/smoke tests | Fast — use for development |
| Layer 2 | ~100 | Feature tests | Medium |
| Layer 3 | ~220 | Regression/edge cases | Medium |
| Layer 4 | 3 | PVM-in-PVM smoke tests | Slow (~85s each) |
| Layer 5 | ~270 | Comprehensive PVM-in-PVM | Slow |
| Differential | ~142 | PVM vs native WASM comparison | Medium |

## Test Organization

- **Integration tests**: `tests/layer{1,2,3}/*.test.ts` — each file calls `defineSuite()` with hex args (little-endian)
- **Rust integration tests**: `crates/wasm-pvm/tests/` — operator coverage, emitter units, stack spill, property tests (true unit tests live inline under `#[cfg(test)]` in source files)
- **Differential tests**: `tests/differential/differential.test.ts` — verifies PVM output matches Bun's WebAssembly engine
- **PVM-in-PVM tests**: Layers 4-5 — the anan-as PVM interpreter compiled to PVM, running test programs inside

## CI Structure

CI runs in stages:
1. **Rust**: lint, clippy, unit tests, release build
2. **Integration**: layers 1-3
3. **Differential**: PVM vs native WASM
4. **PVM-in-PVM**: layers 4-5 (only if integration passes)

## Fixtures

Test programs live in `tests/fixtures/`:
- `wat/` — hand-written WAT programs
- `assembly/` — AssemblyScript programs
- `imports/` — import maps (`.imports`) and adapter files (`.adapter.wat`)

## Build Process

`tests/build.ts` orchestrates three phases:
1. Compile AssemblyScript `.ts` → `.wasm` (skipped if `.wasm` exists)
2. Compile `.wat`/`.wasm` → `.jam` files
3. Compile anan-as compiler.wasm → compiler.jam (for PVM-in-PVM)

**Important**: Delete cached WASM files before working on fixtures:
```bash
rm -f tests/build/wasm/*.wasm
cd tests && bun build.ts
```

## Benchmarks

Run `./tests/utils/benchmark.sh` for performance data. For branch comparisons:
```bash
./tests/utils/benchmark.sh --base main --current <branch>
```

Every PR must include benchmark results in its description.
