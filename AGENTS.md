# WASM-PVM Project - AI Agent Knowledge Base

**Project**: WebAssembly to PolkaVM (PVM) bytecode recompiler
**Stack**: Rust (core) + TypeScript (tests) + AssemblyScript (examples)
**Architecture**: `WASM → [inkwell] → LLVM IR → [mem2reg] → [Rust PVM backend] → PVM bytecode`
**Docs**: `ARCHITECTURE.md` (ABI & calling conventions), `LEARNINGS.md` (tech reference), `gp-0.7.2.md` (PVM spec)

---

## Quick Start

```bash
# Build
cargo build --release

# Test (Rust unit tests only)
cargo test                                    # Unit tests (Rust)

# Build integration test artifacts (WASM + JAM files)
cd tests && bun build.ts                      # REQUIRED before running integration tests

# Run integration tests
# IMPORTANT: Always use `bun run test` NOT `bun test` from the tests/ directory
cd tests && bun run test                      # Full test suite (layers 1-5)

# Quick development check (Layer 1 tests only - fastest)
cd tests && bun test layer1/                  # Quick validation

# PVM-in-PVM tests (layers 4-5)
cd tests && bun test layer4/ layer5/ --test-name-pattern "pvm-in-pvm"

# Compile WASM → JAM
cargo run -p wasm-pvm-cli -- compile tests/fixtures/wat/add.jam.wat -o dist/add.jam

# Run JAM
cd tests && bun utils/run-jam.ts ../dist/add.jam --args=0500000007000000
```

### Important Testing Notes

1. **Always do a FULL rebuild before starting work**: Before beginning any task, delete cached WASM files and rebuild everything from scratch. This ensures you are working against the latest source, not stale artifacts:
   ```bash
   rm -f tests/build/wasm/*.wasm           # Delete cached WASM files
   cd tests && bun build.ts                # Full rebuild: AS→WASM, WAT/WASM→JAM, anan-as compiler
   ```
   This is critical because `bun build.ts` skips AS→WASM compilation if the `.wasm` file already exists, so changes to `.ts` fixtures won't be picked up without deleting the cache first.

2. **Use `bun run test` NOT `bun test`**: From the `tests/` directory, use `bun run test` which runs `bun build.ts && bun test`. Running `bun test` directly without building will fail because the JAM files won't exist.

3. **Development workflow**: For quick iteration, run Layer 1 tests only (`bun test layer1/`). These cover core functionality and complete in seconds. Run the full suite (`bun run test`) before committing.

4. **Test organization**:
   - **Layer 1**: Core/smoke tests (~50 tests) - Run these for quick validation
   - **Layer 2**: Feature tests (~100 tests)
   - **Layer 3**: Regression/edge cases (~220 tests)
   - **Layer 4**: PVM-in-PVM smoke tests (3 tests) - Quick pvm-in-pvm sanity check
   - **Layer 5**: Comprehensive PVM-in-PVM tests (all compatible suites) - Runs in CI after regular tests pass
     - Run with: `bun test layer4/ layer5/ --test-name-pattern "pvm-in-pvm"`
     - Some suites skip pvm-in-pvm (`skipPvmInPvm: true`): host-call-log (ecalli 100 unhandled), as-life (timeout), i64-ops (timeout)
   - **CI runs**: Layer 1-3 first (integration job), then Layer 4-5 in separate PVM-in-PVM job (only if integration passes)

### Documentation Update Policy

**After every task or commit**, update all relevant documentation files:
- **`AGENTS.md`** — Update if you added new modules, changed the build process, modified memory layout, or changed conventions
- **`LEARNINGS.md`** — Update with any new technical knowledge discovered (PVM behaviors, WASM edge cases, compiler quirks, external specs like JIP-1)
- **`ARCHITECTURE.md`** — Update if ABI, calling conventions, or SPI format changed
- **Subdirectory `AGENTS.md` files** (`crates/wasm-pvm/src/translate/AGENTS.md`, `crates/wasm-pvm/src/pvm/AGENTS.md`) — Update if the relevant module's internals changed
- **`todo.md`** — Mark completed tasks as `[x]`, add new discovered tasks

This is not optional. Stale documentation causes repeated mistakes and wasted investigation time.

### PR Description Policy

**Every PR description MUST include benchmark results.** Run `./tests/utils/benchmark.sh --base main --current <branch>` and paste the comparison table into the PR body. This ensures reviewers can see the impact on JAM file size, gas usage, and execution time at a glance. PRs without benchmark results should not be merged.

---

## Structure

```
crates/
├── wasm-pvm/              # Core library
│   └── src/
│       ├── llvm_frontend/ # WASM → LLVM IR
│       │   ├── function_builder.rs (~1350 lines - core translator)
│       │   └── mod.rs
│       ├── llvm_backend/  # LLVM IR → PVM bytecode
│       │   ├── mod.rs           # Public API + main lowering dispatch
│       │   ├── emitter.rs       # EmitterConfig + PvmEmitter struct + value management + register cache (~470 lines)
│       │   ├── alu.rs           # Arithmetic, logic, comparisons, conversions (~380 lines)
│       │   ├── memory.rs        # Load/store, memory intrinsics, word-sized bulk ops (~890 lines)
│       │   ├── control_flow.rs  # Branches, phi nodes, switch, return (~290 lines)
│       │   ├── calls.rs         # Direct/indirect calls, import stubs (~190 lines)
│       │   └── intrinsics.rs    # PVM + LLVM intrinsic lowering (~280 lines)
│       ├── translate/     # Compilation orchestration
│       │   ├── mod.rs     (pipeline dispatch + SPI assembly)
│       │   ├── adapter_merge.rs (WAT adapter merge into WASM before compilation)
│       │   ├── wasm_module.rs (WASM section parsing)
│       │   └── memory_layout.rs (PVM memory address constants)
│       ├── pvm/           # PVM instruction definitions
│       │   ├── instruction.rs  # Instruction enum + encoding
│       │   ├── opcode.rs       # Opcode constants
│       │   ├── blob.rs         # Program blob format
│       │   └── peephole.rs     # Post-codegen peephole optimizer
│       ├── spi.rs         # JAM/SPI format encoder
│       └── error.rs       # Error types (thiserror)
└── wasm-pvm-cli/          # CLI binary
    └── src/main.rs
```

---

## Domain Knowledge

### Compiler Pipeline
1. **Adapter merge** (optional): `adapter_merge.rs` merges a WAT adapter module into the main WASM, replacing matching imports with adapter function bodies. Uses `wasm-encoder` to build merged binary.
2. **WASM parsing**: `wasm_module.rs` parses all WASM sections into `WasmModule` struct
3. **LLVM IR generation**: `llvm_frontend/function_builder.rs` translates `wasmparser::Operator` → LLVM IR using inkwell
4. **LLVM optimization passes** (three phases):
   - Phase 1: `mem2reg` (SSA promotion), `instcombine`, `simplifycfg` (pre-inline cleanup)
   - Phase 2: `cgscc(inline)` (function inlining, optional)
   - Phase 3: `instcombine<max-iterations=2>`, `simplifycfg`, `gvn` (redundancy elimination), `simplifycfg`, `dce` (dead code removal)
5. **PVM lowering**: `llvm_backend/` modules read LLVM IR and emit PVM bytecode:
   - `emitter.rs`: `EmitterConfig` (immutable per-function config) + `PvmEmitter` (mutable codegen state) with value slot management and **per-block register cache** (store-load forwarding)
   - `alu.rs`: Arithmetic, logic, comparisons, conversions
   - `memory.rs`: Load/store, memory intrinsics, word-sized bulk memory ops (memory.copy/fill use 64-bit word loops with byte tails)
   - `control_flow.rs`: Branches, phi nodes, switch, return
   - `calls.rs`: Direct/indirect function calls
   - `intrinsics.rs`: PVM and LLVM intrinsic lowering
6. **SPI assembly**: `translate/mod.rs` builds entry header, dispatch tables, ro_data/rw_data

### PVM (Target)
- Register-based (13 × 64-bit registers)
- Flat control flow with jumps/branches
- Gas metering on all instructions
- Memory: addresses < 2^16 panic

### Key Design Decisions
- **PVM-specific intrinsics** for memory ops (`@__pvm_load_i32`, `@__pvm_store_i32`, etc.) — avoids `unsafe` GEP/inttoptr
- **Stack-slot approach**: every SSA value gets a dedicated memory offset from SP (correctness-first, register allocator is future work)
- **Per-block register cache**: `PvmEmitter` tracks which stack slots are live in registers via `slot_cache`/`reg_to_slot`. Eliminates redundant `LoadIndU64` when a value is used shortly after being computed. Cache is cleared at block boundaries and after calls/ecalli. (~50% gas reduction, ~15-40% code size reduction)
- **heap_pages uses initial_pages**: SPI `heap_pages` reflects WASM `initial_pages` (not `max_pages`). Additional memory is allocated on demand via `sbrk`/`memory.grow`. Programs declaring `(memory 0)` get a minimum of 16 WASM pages (1MB).
- **All values as i64**: PVM registers are 64-bit; simplifies translation
- **LLVM backend**: inkwell (LLVM 18 bindings) is a required dependency
- **Callee-save shrink wrapping**: For non-entry functions, only callee-saved registers (r9-r12) that are actually used are saved/restored in prologue/epilogue. A register is "used" if it receives a parameter or the function contains any call instruction. Frame header size is dynamic per-function: `8 (ra) + 8 * num_used_callee_regs`.
- **Configurable optimizations**: All non-trivial optimizations (LLVM passes, peephole, register cache, ICmp+Branch fusion, shrink wrapping) can be disabled via `OptimizationFlags` / CLI `--no-*` flags. All are enabled by default.

---

## Conventions

### Code Style
- `rustfmt` defaults, `clippy` warnings = errors
- `unsafe_code = "deny"` (workspace lint)
- `thiserror` for errors, `tracing` for logging
- Unit tests inline under `#[cfg(test)]`

### Naming
- Types: `PascalCase`, Functions: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Indicate WASM vs PVM context in names

### Project-Specific
- No `lib/` folder in crates — flat src structure
- Integration tests in TypeScript (`tests/`)
- Rust unit tests in `crates/wasm-pvm/tests/`
- Pre-push hook: `.githooks/pre-push` (install with `git config core.hooksPath .githooks`)

---

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| Add WASM operator | `llvm_frontend/function_builder.rs` | Add to operator match |
| Add PVM lowering (arithmetic) | `llvm_backend/alu.rs` | Binary ops, comparisons, conversions |
| Add PVM lowering (memory) | `llvm_backend/memory.rs` | Load/store, memory.size, memory.grow, bulk ops (word-sized) |
| Add PVM lowering (control flow) | `llvm_backend/control_flow.rs` | Branches, phi, switch, return |
| Add PVM lowering (calls) | `llvm_backend/calls.rs` | Direct/indirect calls, import stubs |
| Add PVM lowering (intrinsics) | `llvm_backend/intrinsics.rs` | PVM + LLVM intrinsic lowering |
| Modify emitter core | `llvm_backend/emitter.rs` | EmitterConfig (per-function config) + PvmEmitter (mutable state) |
| Add PVM instruction | `pvm/opcode.rs` + `pvm/instruction.rs` | Add enum + encoding |
| Modify peephole optimizer | `pvm/peephole.rs` | Add patterns, update fixup remapping |
| Fix WASM parsing | `translate/wasm_module.rs` | `WasmModule::parse()` |
| Fix compilation pipeline | `translate/mod.rs` | `compile()` |
| Fix adapter merge | `translate/adapter_merge.rs` | WAT adapter → merged WASM binary |
| Add test case | `tests/layer{1,2,3}/*.test.ts` | Each file calls `defineSuite()` with hex args, little-endian |
| Add/modify import adapter | `tests/fixtures/imports/*.adapter.wat` | WAT adapter files for complex import resolution |
| Add/modify import map | `tests/fixtures/imports/*.imports` | Text-based import maps (simple: trap, nop) |
| Fix test execution | `tests/helpers/run.ts` | `runJam()` |
| Fix test build | `tests/build.ts` + `tests/helpers/compile.ts` | Build orchestrator + compilation helpers |
| Debug execution | `tests/utils/trace-steps.ts` | Shows PC, gas, registers per step |
| Verify JAM file | `tests/utils/verify-jam.ts` | Parse headers, jump table, code |
| Add/modify optimization | `translate/mod.rs` (`OptimizationFlags`) | Add flag + thread through `LoweringContext` → `PvmEmitter` |
| Toggle optimization in CLI | `wasm-pvm-cli/src/main.rs` | Add `--no-*` flag to `Compile` subcommand |

---

## Optimization Flags

All non-trivial optimizations are controlled by `OptimizationFlags` (in `translate/mod.rs`, re-exported from `lib.rs`).
Each flag defaults to `true` (enabled). CLI exposes `--no-*` flags.

| Flag | CLI | What it controls | Where toggled |
|------|-----|------------------|---------------|
| `llvm_passes` | `--no-llvm-passes` | LLVM optimization passes (mem2reg, instcombine, etc.) | `llvm_frontend/function_builder.rs` |
| `peephole` | `--no-peephole` | Post-codegen peephole optimizer | `llvm_backend/mod.rs:lower_function()` |
| `register_cache` | `--no-register-cache` | Per-block store-load forwarding | `llvm_backend/emitter.rs:cache_slot()` |
| `icmp_branch_fusion` | `--no-icmp-fusion` | Fuse ICmp+Branch into single PVM branch | `llvm_backend/alu.rs:lower_icmp()` |
| `shrink_wrap_callee_saves` | `--no-shrink-wrap` | Only save/restore used callee-saved regs | `llvm_backend/emitter.rs:pre_scan_function()` |
| `dead_store_elimination` | `--no-dead-store-elim` | Remove SP-relative stores never loaded from | `llvm_backend/mod.rs:lower_function()` → `peephole.rs` |
| `constant_propagation` | `--no-const-prop` | Skip redundant `LoadImm`/`LoadImm64` when register already holds the constant | `llvm_backend/emitter.rs:emit()` |
| `inlining` | `--no-inline` | LLVM function inlining for small callees (CGSCC inline pass) | `llvm_frontend/function_builder.rs:run_optimization_passes()` |

**Threading path**: `CompileOptions.optimizations` → `LoweringContext.optimizations` → `EmitterConfig` fields (`register_cache_enabled`, `icmp_fusion_enabled`, `shrink_wrap_enabled`, `constant_propagation_enabled`) → `PvmEmitter.config`. LLVM passes and inlining flags are passed directly to `translate_wasm_to_llvm()`.

**Adding a new optimization**: Add a field to `OptimizationFlags`, thread it through `LoweringContext` → `EmitterConfig`, guard the optimization with `e.config.<flag>`, add a `--no-*` CLI flag.

---

## Anti-Patterns (Forbidden)

1. **No `unsafe` code** — Strictly forbidden by workspace lint
2. **No panics in library code** — Use `Result<>` with `Error::Internal`
3. **No floating point** — PVM lacks FP support; reject WASM floats
4. **Don't break register conventions** — Hardcoded in multiple files
5. **NEVER use --no-verify on git push** — Always ensure tests and linters pass

---

## Memory Layout

| Address | Purpose |
|---------|---------|
| `0x10000` | Read-only data (dispatch table, passive segments) |
| `0x30000` | Globals storage (each global = 4 bytes) |
| `0x3FF00` | Parameter overflow area (5th+ args for call_indirect) |
| `0x40000` | Spilled locals (512 bytes per function) |
| `0x50000+` | WASM linear memory base (computed dynamically based on function count) |
| `0xFEFE0000` | Stack segment end (stack grows downward) |
| `0xFEFF0000` | Arguments (`args_ptr`) |
| `0xFFFF0000` | EXIT address (HALT) |

**Dynamic allocation**: User programs should use `heap.alloc()` (AssemblyScript) or `memory.grow` (WASM) for result buffers and scratch memory. Do NOT hardcode addresses like `0x30100`.

---

## Register Allocation

| Register | Usage |
|----------|-------|
| r0 | Return address (jump table index) |
| r1 | Stack pointer |
| r2-r6 | Scratch registers |
| r7 | Return value from calls / SPI args pointer |
| r8 | SPI args length |
| r9-r12 | Local variables (first 4) / callee-saved |

---

## Subdirectory Docs

- **`crates/wasm-pvm/src/translate/AGENTS.md`** — Translation module details
- **`crates/wasm-pvm/src/pvm/AGENTS.md`** — PVM instruction encoding

---

## Contact

Maintainer: @tomusdrw
PVM questions: Gray Paper or PolkaVM repo
