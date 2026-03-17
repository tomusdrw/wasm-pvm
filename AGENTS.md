# WASM-PVM Project - AI Agent Knowledge Base

**Project**: WebAssembly to PolkaVM (PVM) bytecode recompiler
**Stack**: Rust (core) + TypeScript (tests) + AssemblyScript (examples)
**Architecture**: `WASM → [inkwell] → LLVM IR → [mem2reg] → [Rust PVM backend] → PVM bytecode`
**Docs**: `docs/src/` (mdbook — run `mdbook serve docs` to browse), `gp-0.7.2.md` (PVM spec)

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
     - Some suites skip pvm-in-pvm (`skipPvmInPvm: true`): as-life (timeout), i64-ops (timeout)
   - **Differential**: PVM vs native WASM (~142 tests) - Verifies PVM output matches Bun's WebAssembly engine
     - Run with: `cd tests && bun run test:differential`
     - Auto-skips modules with function imports (AssemblyScript (AS) modules, host-call-log)
     - Auto-skips tests with custom `pc` (PVM-specific entry points)
   - **CI runs**: Layer 1-3 first (integration job), then Layer 4-5 in separate PVM-in-PVM job (only if integration passes)

### Documentation Update Policy

**After every task or commit**, update all relevant documentation files:
- **`AGENTS.md`** — Update if you added new modules, changed the build process, modified memory layout, or changed conventions
- **`docs/src/learnings.md`** — Update with any new technical knowledge discovered (PVM behaviors, WASM edge cases, compiler quirks, external specs like JIP-1)
- **`docs/src/architecture.md`** — Update if ABI, calling conventions, or SPI format changed
- **`docs/src/internals/translation.md`** — Update if translation module internals changed
- **`docs/src/internals/pvm-instructions.md`** — Update if PVM instruction encoding changed
- **`todo.md`** — Mark completed tasks as `[x]`, add new discovered tasks
- **`README.md` benchmark tables** — If JAM sizes or gas usage changed (e.g. code generation, memory layout, or optimization changes), re-run `./tests/utils/benchmark.sh` and update the two benchmark tables in `README.md` (Optimizations Impact + PVM-in-PVM) with the latest numbers

All documentation lives in `docs/src/` (mdbook). Preview locally with `mdbook serve docs`.

This is not optional. Stale documentation causes repeated mistakes and wasted investigation time.

### PR Description Policy

**Every PR description MUST include benchmark results.** Run `./tests/utils/benchmark.sh --base main --current <branch>` and paste the comparison table into the PR body. The script produces both direct execution and PVM-in-PVM benchmark comparisons (JAM file size, gas usage, and execution time). PRs without benchmark results should not be merged.

### Regalloc Debugging

- Enable allocator logs with `RUST_LOG=wasm_pvm::regalloc=debug`.
- `regalloc::run()` prints candidate/assignment stats (`total_values`, `total_intervals`, `has_loops`, `allocatable_regs`, `allocated_values`, `skipped_reason`).
- `lower_function()` prints usage counters (`alloc_load_hits`, `alloc_load_reloads`, `alloc_load_moves`, `alloc_store_hits`, `alloc_store_moves`) plus `emitted_instructions`.
- Quick triage:
  - `allocatable_regs=0` or `skipped_reason` usually means no allocation will happen.
  - Non-zero `allocated_values` with near-zero load/store hits usually indicates move/reload overhead dominates that function.

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
│       │   ├── alu.rs           # Arithmetic, logic, comparisons, conversions, fused bitwise (AndInv/OrInv/Xnor), CmovIz (~810 lines)
│       │   ├── memory.rs        # Load/store, memory intrinsics, word-sized bulk ops (~920 lines)
│       │   ├── control_flow.rs  # Branches, phi nodes, switch, return (~290 lines)
│       │   ├── calls.rs         # Direct/indirect calls, import stubs (~190 lines)
│       │   ├── intrinsics.rs    # PVM + LLVM intrinsic lowering (~440 lines)
│       │   └── regalloc.rs      # Linear-scan register allocator (all functions, spill-weight eviction) (~690 lines)
│       ├── translate/     # Compilation orchestration (feature = "compiler")
│       │   ├── mod.rs     (pipeline dispatch + SPI assembly)
│       │   ├── adapter_merge.rs (WAT adapter merge into WASM before compilation)
│       │   └── wasm_module.rs (WASM section parsing)
│       ├── pvm/           # PVM instruction definitions (always available)
│       │   ├── instruction.rs  # Instruction enum + encode/decode helpers
│       │   ├── opcode.rs       # Opcode constants
│       │   ├── blob.rs         # Program blob format
│       │   └── peephole.rs     # Post-codegen peephole optimizer (feature = "compiler")
│       ├── memory_layout.rs  # PVM memory address constants (always available)
│       ├── spi.rs         # JAM/SPI format encoder (always available)
│       └── error.rs       # Error types (thiserror)
└── wasm-pvm-cli/          # CLI binary
    └── src/main.rs
```

---

## Crate Features

The `wasm-pvm` crate uses feature flags to allow lightweight downstream usage without the full compiler toolchain (inkwell/LLVM).

| Feature | Default | What it enables |
|---------|---------|-----------------|
| `compiler` | Yes | Full WASM-to-PVM compiler (`llvm_frontend`, `llvm_backend`, `translate` modules, `inkwell`/`wasmparser`/`wasm-encoder` deps) |
| `test-harness` | Yes | Test utilities (implies `compiler`) |

**Without `compiler`** (i.e., `default-features = false`), only PVM types are available: `Instruction`, `Opcode`, `ProgramBlob`, `SpiProgram`, `abi::*`, `memory_layout::*`, and `Error` (without the `WasmParse` variant). This configuration compiles to `wasm32-unknown-unknown`.

```toml
# Full compiler (default)
wasm-pvm = "0.5.2"

# PVM types only (no LLVM dependency, WASM-compatible)
wasm-pvm = { version = "0.5.2", default-features = false }
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
- **Unified entry ABI**: All entry functions must use `main(args_ptr: i32, args_len: i32) -> i64`. The i64 return packs a WASM pointer (lower 32 bits) and result length (upper 32 bits). The PVM epilogue unpacks to `r7 = ptr + wasm_memory_base`, `r8 = r7 + len`. WAT files use `(i64.const 17179869184)` for the common ptr=0, len=4 case. AS files use a `writeResult` helper returning `(ptr as i64) | ((len as i64) << 32)`.
- **PVM-specific intrinsics** for memory ops (`@__pvm_load_i32`, `@__pvm_store_i32`, etc.) — avoids `unsafe` GEP/inttoptr
- **Stack-slot approach**: every SSA value gets a dedicated memory offset from SP (correctness-first). A **linear-scan register allocator** (`regalloc.rs`) assigns values to physical registers using spill-weight eviction (`use_count × 10^loop_depth`). Allocatable registers: r5-r8 in all functions (not just leaf), r9-r12 (callee-saved) beyond the parameter count in all functions. In non-leaf functions, outgoing call argument registers are reserved, and the existing call lowering (`spill_allocated_regs` + `clear_reg_cache` + lazy reload) handles spill/reload around calls automatically. After calls, only registers actually clobbered by that call's argument count are invalidated (per-call-site arity-aware invalidation). r5/r6 allocation requires per-function LLVM IR scan proving no bulk memory ops or funnel shifts.
- **Per-block register cache**: `PvmEmitter` tracks which stack slots are live in registers via `slot_cache`/`reg_to_slot`. Eliminates redundant `LoadIndU64` when a value is used shortly after being computed. Cache is cleared at block boundaries and after calls/ecalli. (~50% gas reduction, ~15-40% code size reduction)
- **Cross-block register cache**: When a block has exactly one predecessor and no phi nodes, the predecessor's cache snapshot is propagated instead of clearing. The snapshot is taken before the terminator instruction with TEMP1/TEMP2 invalidated (since terminators load operands into those registers). Predecessor map is computed in `pre_scan_function` by scanning terminator successors.
- **Indirect-call fusion**: `call_indirect` emits `LoadImmJumpInd` (opcode 180) to combine return-address setup and `JumpInd` in one instruction; return jump-table refs are still pre-assigned at emission time for size-stable fixups.
- **Trimmed RW data**: `build_rw_data()` trims trailing zero bytes before SPI encoding. Heap pages are zero-initialized, so omitted high-address zero tails are semantically equivalent and reduce blob size.
- **heap_pages computed after rw_data**: SPI `heap_pages` is calculated in `compile_via_llvm()` **after** `build_rw_data()`, using the actual trimmed `rw_data.len()`. The formula: `total_pages - rw_pages`, where `total_pages` covers from `0x30000` to the end of initial WASM memory (using `initial_pages`, min 16). Additional memory is allocated on demand via `sbrk`/`memory.grow`.
- **All values as i64**: PVM registers are 64-bit; simplifies translation
- **LLVM backend**: inkwell (LLVM 18 bindings) is a required dependency
- **Leaf function detection**: `is_real_call()` in `emitter.rs` distinguishes real function calls (`wasm_func_*`, `__pvm_call_indirect`) from PVM intrinsics (`__pvm_load_*`, `__pvm_store_*`, etc.) and LLVM intrinsics (`llvm.*`). PVM intrinsics are lowered inline with temp registers only and don't use the calling convention, so functions containing only these are classified as leaf (no callee-save prologue/epilogue needed). This was a major optimization — previously ALL functions with memory access were non-leaf.
- **Cross-block alloc_reg_slot propagation**: In leaf functions (without lazy spill), `alloc_reg_slot` is preserved across all block boundaries (allocated registers are never clobbered). In non-leaf functions, predecessor exit snapshots are intersected at multi-predecessor blocks — only entries where ALL processed predecessors agree are kept. Back-edges use conservative clearing. With lazy spill enabled, leaf functions use `define_label` (clear all) at block boundaries to avoid stale alloc_reg_slot from unrelated blocks.
- **Lazy spill**: When enabled, `store_to_slot()` for register-allocated values writes only to the register and marks it dirty, skipping the `StoreIndU64`. The value is flushed to the stack when: (1) the register is clobbered by another instruction (`invalidate_reg` auto-spill), (2) before function calls/ecalli, (3) before epilogue (return), (4) before terminators at block boundaries, or (5) after prologue parameter stores. DSE protects allocated slot offsets from elimination. With register-aware phi resolution (Phase 5), phi copies use direct register moves when possible, and target blocks restore `alloc_reg_slot` for phi destinations after `define_label`.
- **Store-side coalescing**: `result_reg()` / `result_reg_or()` helpers in `emitter.rs` return the allocated register for an instruction's result (or a fallback temp). ALU, memory load, and intrinsic lowering paths compute directly into the allocated register instead of TEMP_RESULT, eliminating the subsequent `MoveReg` in `store_to_slot`. Not applied to `lower_select` (loading into alloc_reg corrupts state for subsequent operand loads), `emit_pvm_memory_grow` (TEMP_RESULT used across control flow), or `lower_abs` (TEMP_RESULT used across control flow). `result_reg_or()` is used by zext/sext/trunc to specify TEMP1 as fallback instead of TEMP_RESULT, preserving register cache behavior in non-allocated paths.
- **Callee-save shrink wrapping**: For non-entry functions, only callee-saved registers (r9-r12) that are actually used are saved/restored in prologue/epilogue. A register is "used" if it receives a parameter, the function contains any real call instruction, or register allocation assigns values to it. Frame header size is dynamic per-function: `8 (ra) + 8 * num_used_callee_regs`.
- **Configurable optimizations**: All non-trivial optimizations (LLVM passes, peephole, register cache, ICmp+Branch fusion, shrink wrapping) can be disabled via `OptimizationFlags` / CLI `--no-*` flags. All are enabled by default.
- **Peephole immediate chain fusion**: `LoadImm + AddImm` and chained `AddImm` sequences are fused into single instructions. Self-moves (`MoveReg r, r`) are eliminated. This reduces code size for address calculations and loop induction variables.
- **Typed host call imports**: A family of `host_call_N` (N=0..6) imports for PVM `ecalli` instructions, where N is the number of data registers (r7..r7+N-1) to set. All take an ecalli index as the first i64 param (compile-time constant) and return r7 as i64. Variants with `b` suffix (e.g. `host_call_2b`) also capture r8 to a stack slot, retrievable via `host_call_r8() -> i64`. See `docs/src/architecture.md` "Import Calls" for the full reference. Implementation in `llvm_backend/calls.rs`.
- **PVM-in-PVM ecalli forwarding**: Inner program ecalli are forwarded to the outer PVM via the adapter WAT. Two adapter variants exist:
  - `anan-as-compiler.adapter.wat` (regular): Handles ecalli 100 (JIP-1 log) with pointer translation via `host_read_memory` + `pvm_ptr`. Traps on unknown ecalli. Imports resolved against main exports (e.g. `host_read_memory`).
  - `anan-as-compiler-replay.adapter.wat` (trace replay): Uses a scratch buffer protocol. Adapter calls outer ecalli 0 ("forward") with scratch PVM addr + inner ecalli index. Outer handler writes response `[8:new_r7][8:new_r8][4:num_memwrites][8:new_gas][entries...]` to the buffer. Adapter applies memwrites via `host_write_memory` and returns new_r7. Outer ecalli 1 ("get r8") returns the last r8 value.
- **Adapter import resolution against main exports**: `adapter_merge.rs` now resolves adapter imports that match main module export names internally, with type signature validation, instead of carrying them through as retained imports. This allows the adapter to call compiler functions like `host_read_memory` and `host_write_memory` directly.
- **Dynamic ecalli limitation**: PVM `ecalli` instruction requires a static (compile-time constant) index. The regular adapter handles only ecalli 100; other ecalli types would need individual handlers or a dispatch table. The replay adapter avoids this by using fixed outer ecalli indices (0 and 1) for the forwarding protocol.

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
- Rust unit tests in `crates/wasm-pvm/tests/` (see below for test file descriptions)
- Pre-push hook: `.githooks/pre-push` (install with `git config core.hooksPath .githooks`)

---

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| Add WASM operator | `llvm_frontend/function_builder.rs` | Add to operator match |
| Add PVM lowering (arithmetic) | `llvm_backend/alu.rs` | Binary ops, comparisons, conversions, fused bitwise (AndInv/OrInv/Xnor), CmovIz |
| Add PVM lowering (memory) | `llvm_backend/memory.rs` | Load/store, memory.size, memory.grow, bulk ops (word-sized) |
| Add PVM lowering (control flow) | `llvm_backend/control_flow.rs` | Branches, phi, switch, return |
| Add PVM lowering (calls) | `llvm_backend/calls.rs` | Direct/indirect calls, import stubs |
| Add PVM lowering (intrinsics) | `llvm_backend/intrinsics.rs` | PVM + LLVM intrinsic lowering |
| Modify emitter core | `llvm_backend/emitter.rs` | EmitterConfig (per-function config) + PvmEmitter (mutable state) |
| Add PVM instruction | `pvm/opcode.rs` + `pvm/instruction.rs` | Add enum + encode/decode wiring |
| Modify register allocator | `llvm_backend/regalloc.rs` | Live range computation, linear scan, allocatable regs |
| Modify peephole optimizer | `pvm/peephole.rs` | Add patterns, update fixup remapping |
| Fix WASM parsing | `translate/wasm_module.rs` | `WasmModule::parse()` |
| Fix compilation pipeline | `translate/mod.rs` | `compile()` |
| Fix adapter merge | `translate/adapter_merge.rs` | WAT adapter → merged WASM binary |
| Add integration test | `tests/layer{1,2,3}/*.test.ts` | Each file calls `defineSuite()` with hex args, little-endian |
| Add operator unit test | `crates/wasm-pvm/tests/operator_coverage.rs` | WASM operator → PVM opcode verification (91 tests) |
| Add emitter unit test | `crates/wasm-pvm/tests/emitter_unit.rs` | Slot allocation, labels, fixups, frame layout (19 tests) |
| Add stack spill test | `crates/wasm-pvm/tests/deep_stack_spill.rs` | Deep stack, spill across calls (8 tests) |
| Add/modify import adapter | `tests/fixtures/imports/*.adapter.wat` | WAT adapter files for complex import resolution |
| Add/modify import map | `tests/fixtures/imports/*.imports` | Text-based import maps (simple: trap, nop) |
| Fix test execution | `tests/helpers/run.ts` | `runJam()` |
| Fix test build | `tests/build.ts` + `tests/helpers/compile.ts` | Build orchestrator + compilation helpers |
| Debug execution | `tests/utils/trace-steps.ts` | Shows PC, gas, registers per step |
| Generate execution trace | `tests/utils/generate-trace.ts` | Outputs anan-as trace format to stdout |
| PVM-in-PVM trace replay | `tests/utils/trace-replay-pip.ts` | Replays trace through PVM-in-PVM pipeline |
| Verify JAM file | `tests/utils/verify-jam.ts` | Parse headers, jump table, code |
| Add/modify optimization | `translate/mod.rs` (`OptimizationFlags`) | Add flag + thread through `LoweringContext` → `PvmEmitter` |
| Toggle optimization in CLI | `wasm-pvm-cli/src/main.rs` | Add `--no-*` flag to `Compile` subcommand |
| Add property test | `crates/wasm-pvm/tests/property_tests.rs` | Proptest: compilation safety, encoding roundtrips (28 tests) |
| Understand/modify native WASM runner | `tests/helpers/wasm-runner.ts` | Native WASM runner (WAT→WASM via wabt, Bun WebAssembly) |
| Define differential test suite | `tests/helpers/suite.ts` | `defineDifferentialSuite()` + `skipDifferential` flag |
| Add/aggregate differential tests | `tests/differential/differential.test.ts` | Import suites + call `defineDifferentialSuite()` |

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
| `cross_block_cache` | `--no-cross-block-cache` | Propagate register cache across single-predecessor block boundaries | `llvm_backend/mod.rs:lower_function()` |
| `register_allocation` | `--no-register-alloc` | Linear-scan register allocation with spill-weight eviction (all functions including non-leaf, uses r5-r12 where safe) | `llvm_backend/mod.rs:lower_function()` → `regalloc.rs` |
| `aggressive_register_allocation` | `--no-aggressive-regalloc` | Lower min-use threshold from 2→1 for register allocation candidates | `llvm_backend/regalloc.rs:compute_live_intervals()` |
| `dead_function_elimination` | `--no-dead-function-elim` | Remove unreachable functions from output | `translate/mod.rs:compile_via_llvm()` |
| `fallthrough_jumps` | `--no-fallthrough-jumps` | Skip redundant Jump when target is next block in layout order | `llvm_backend/emitter.rs:emit_jump_to_label()` |
| `allocate_scratch_regs` | `--no-scratch-reg-alloc` | Allocate r5/r6 in all functions that don't clobber them (no bulk memory/funnel shifts). In non-leaf, spill/reload around calls handled automatically. | `llvm_backend/mod.rs` → `regalloc.rs` |
| `allocate_caller_saved_regs` | `--no-caller-saved-alloc` | Allocate r7/r8 in all functions. In non-leaf, invalidated after calls via arity-aware predicate. | `llvm_backend/mod.rs` → `regalloc.rs` |
| `lazy_spill` | `--no-lazy-spill` | Skip stack stores for register-allocated values; spill only when required (calls, return, phi reads, register clobber) | `llvm_backend/emitter.rs`, `llvm_backend/mod.rs`, `llvm_backend/control_flow.rs` |

**Threading path**: `CompileOptions.optimizations` → `LoweringContext.optimizations` → `EmitterConfig` fields (`register_cache_enabled`, `icmp_fusion_enabled`, `shrink_wrap_enabled`, `constant_propagation_enabled`, `cross_block_cache_enabled`, `register_allocation_enabled`, `fallthrough_jumps_enabled`, `lazy_spill_enabled`) → `PvmEmitter.config`. LLVM passes and inlining flags are passed directly to `translate_wasm_to_llvm()`. The `aggressive_register_allocation`, `allocate_scratch_regs`, and `allocate_caller_saved_regs` flags are passed directly from `LoweringContext.optimizations` to `regalloc::run()` (not through `EmitterConfig`).

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
| `0x30000` | Globals storage (each global = 4 bytes). The heap base is computed via `compute_wasm_memory_base(num_funcs, num_globals, num_passive_segments)`, which aligns the heap after the actual globals/passive-length region to the next 4KB (PVM page) boundary. |
| `0x32000` | Parameter overflow area (5th+ args for call_indirect) |
| `0x32100+` | Spilled locals base (per-function, usually empty because spills go on the stack) |
| `≈0x33000+` | WASM linear memory (4KB-aligned; actual base varies, computed via `compute_wasm_memory_base(num_funcs, num_globals, num_passive_segments)`) |
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
| r2-r3 | Scratch temps (TEMP1/TEMP2 for loading spilled operands) |
| r4 | Scratch temp (TEMP_RESULT for instruction results). With store-side coalescing, ALU/memory-load/intrinsic results are computed directly into the allocated register when available, so r4 is only used as fallback for non-allocated values. |
| r5-r6 | Scratch registers (`abi::SCRATCH1`/`SCRATCH2`). Used by bulk memory intrinsics and funnel shifts. Allocatable in all functions that don't clobber them (controlled by `allocate_scratch_regs` flag). In non-leaf functions, spill/reload around calls is handled by `spill_allocated_regs` + lazy reload. |
| r7 | Return value from calls / SPI args pointer. Allocatable in all functions (controlled by `allocate_caller_saved_regs`). In non-leaf functions, invalidated after calls via arity-aware invalidation. Overwritten by return sequence. |
| r8 | SPI args length. Allocatable in all functions (controlled by `allocate_caller_saved_regs`). In non-leaf functions, invalidated after calls. Free after prologue. |
| r9-r12 | Local variables (first 4) / callee-saved. Unused regs beyond parameter count may be assigned by linear-scan register allocation in both leaf and non-leaf functions; non-leaf allocation reserves outgoing call-arg registers and call lowering invalidates allocated mappings after calls. |

---

## Documentation

All docs live in `docs/src/` (mdbook format). Preview locally with `mdbook serve docs`.

Key pages:
- **`docs/src/architecture.md`** — ABI, calling conventions, memory layout, SPI format
- **`docs/src/learnings.md`** — Technical reference (LLVM, PVM semantics, optimizations)
- **`docs/src/internals/translation.md`** — Translation module details
- **`docs/src/internals/pvm-instructions.md`** — PVM instruction encoding
- **`docs/src/optimizations.md`** — All optimization flags with descriptions

---

## Contact

Maintainer: @tomusdrw
PVM questions: Gray Paper or PolkaVM repo
