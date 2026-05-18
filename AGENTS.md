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
- **`experiments/analysis.md`** — If you touched any optimization pass, its gate, or the LLVM pipeline, re-run `./experiments/opt_impact.sh` (~75–90 min full, or `--fixtures` for ~15 min) and refresh `experiments/results/` plus the `analysis.md` snapshot. See `experiments/README.md` for the workflow.

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
│       │   ├── control_flow.rs  # Branches, phi nodes (incl. slot-based parallel-move resolver for >5 copies), switch, return (~550 lines)
│       │   ├── calls.rs         # Direct/indirect calls, import stubs (~190 lines)
│       │   ├── intrinsics.rs    # PVM + LLVM intrinsic lowering (~440 lines)
│       │   └── regalloc.rs      # Linear-scan register allocator (all functions, spill-weight eviction) (~1060 lines)
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
3. **LLVM IR generation**: `llvm_frontend/function_builder.rs` translates `wasmparser::Operator` → LLVM IR using inkwell. **Important**: LLVM's `IRBuilder` constant-folds at instruction *creation* time — `LLVMBuildAdd(3, 5)` produces the constant `8` directly, never an `add` instruction. This means no instruction with all-constant operands survives IR construction, even without optimization passes. This has implications for any optimization that tries to detect constant-valued instructions (see learnings.md "Rematerialization" entry).
4. **LLVM optimization passes** (three phases):
   - Phase 1: `mem2reg` (SSA promotion), `instcombine<max-iterations=20>`, `simplifycfg` (pre-inline cleanup)
   - Phase 2: `cgscc(inline)` (function inlining, optional)
   - Phase 3: `instcombine<max-iterations=20>`, `simplifycfg`, `gvn` (redundancy elimination), `simplifycfg`, `dce` (dead code removal). The `max-iterations=20` cap (was `2` before #212) prevents LLVM hard-aborts on `--trap-floats` IR shapes with many `@llvm.trap()`+`unreachable` clusters; see `docs/src/learnings.md` "instcombine Convergence".
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

This section lists architectural facts and load-bearing invariants. For implementation detail on individual optimizations, see `docs/src/optimizations.md`; for non-obvious technical findings (including "do not retry these" dead-ends), see `docs/src/learnings.md`.

- **Unified entry ABI**: `main(args_ptr: i32, args_len: i32) -> i64`. The i64 return packs ptr (low 32) | len (high 32). PVM epilogue unpacks to `r7 = ptr + wasm_memory_base`, `r8 = r7 + len`. WAT uses `(i64.const 17179869184)` for the common ptr=0/len=4 case; AS uses a `writeResult` helper.
- **PVM-specific intrinsics** (`@__pvm_load_*`, `@__pvm_store_*`) for memory — avoids `unsafe` GEP/inttoptr.
- **All values as i64**: PVM registers are 64-bit. Simplifies translation.
- **LLVM backend**: inkwell (LLVM 18) is required for the `compiler` feature.
- **Stack-slot + linear-scan regalloc**: every SSA value gets an SP-relative slot (correctness-first); `regalloc.rs` then assigns hot values to physical registers via spill-weight eviction (`use_count × 10^loop_depth − spans_calls × 2.0`). Allocatable: r5–r6 (when no bulk-mem/funnel-shift in the function), r7–r8 (leaf only), r9–r12 (callee-saved beyond param count). Non-leaf outgoing arg regs are reserved; existing call lowering (`spill_allocated_regs` + arity-aware invalidation + lazy reload) handles spill/reload around calls.
- **Per-block register cache**: `slot_cache`/`reg_to_slot` in `PvmEmitter` do store-load forwarding. Cleared at block boundaries and after calls/ecalli. Propagated across a block boundary when the successor has a single predecessor and no phis (cross-block cache). See `docs/src/optimizations.md`.
- **Lazy spill** (when enabled): allocated values stay in registers; the dirty flag drives flush on clobber / call / return / phi read. DSE protects allocated slot offsets. See `docs/src/optimizations.md`.
- **Phi-copy resolution**: fast temp-pool path for ≤5 copies (`emit_phi_copies_legacy` / `emit_phi_copies_regaware`); slot-based parallel-move resolver (`emit_phi_copies_via_slots`) for more or for swap cycles. `topo_order_phase2` reorders Phase-2 copies to handle temp/dest aliasing on r7/r8. See `control_flow.rs` and `docs/src/learnings.md` "Phi-Copy Resolution".
- **Leaf-function detection**: `is_real_call()` in `emitter.rs` excludes PVM/LLVM intrinsics; leaf functions skip the callee-save prologue/epilogue.
- **Callee-save shrink wrapping**: only used r9–r12 registers are saved/restored. Frame header: `8 (ra) + 8 × used_callee_regs`.
- **Indirect-call fusion**: `LoadImmJumpInd` (opcode 180) combines return-address setup and `JumpInd`.
- **Trimmed RW data**: `build_rw_data()` strips trailing zeros before SPI encoding (heap pages are zero-initialized anyway).
- **`heap_pages` computed after rw_data**: in `compile_via_llvm()`, after `build_rw_data()`. Reserves `+1` page at the heap boundary for the first `memory.grow`/sbrk (required for PVM-in-PVM).
- **Block layout for fallthrough bias**: `compute_block_layout` in `llvm_backend/mod.rs` reorders blocks so each block's preferred successor (`else_bb` for cond `br`, `default_bb` for switch, single dest for uncond) follows it. Regalloc walks the same order via `block_order` so live intervals match emission. Trampoline paths in `lower_br`/`lower_switch` may diverge from the layout but stay correct.
- **Cross-block snapshot invalidation set**: the snapshot taken before a terminator invalidates TEMP1/TEMP2 *and* TEMP_RESULT + emitter-scope SCRATCH1/SCRATCH2 (= r4/r7/r8), because phi copies use those as Phase-1 temps. Successors restoring the snapshot would otherwise see `alloc_reg_slot` entries pointing at registers a phi-copy already overwrote. See `docs/src/learnings.md` "Cross-Block Snapshot Must Mirror Terminator-Clobber Set".
- **Store/load-side coalescing**: `result_reg()` / `operand_reg()` in `emitter.rs` use the allocated register directly as instr dst/src, eliminating MoveRegs. Dst-conflict fallback (`apply_dst_conflict_fallback`) routes through TEMP1/TEMP2 when the operand reg equals an allocated dst; for `dst == TEMP_RESULT` the alias is kept (PVM reads both srcs before writing dst). Exclusions: `lower_select`/`emit_pvm_memory_grow`/`lower_abs` (TEMP_RESULT used across control flow), div/rem (trap code clobbers SCRATCH1), `bitreverse` (clobbers TEMP_RESULT mid-sequence). See `docs/src/optimizations.md` "Store-Side Coalescing" / "Load-Side Coalescing".
- **Typed host call imports**: `host_call_N` (N=0..6) sets r7..r7+N−1 then ecallis; `b`-suffixed variants also capture r8 (retrieve via `host_call_r8()`). See `docs/src/architecture.md` "Import Calls".
- **`ecalli:N` in import maps**: `.imports` files accept `name = ecalli:N` alongside `trap` and `nop`. Args load into r7..r12 before the `Ecalli`.
- **PVM-in-PVM ecalli forwarding** (two adapter WATs):
  - `anan-as-compiler.adapter.wat`: handles ecalli 100 (JIP-1 log) via `host_read_memory` + `pvm_ptr`; traps on unknown ecalli.
  - `anan-as-compiler-replay.adapter.wat`: scratch-buffer protocol. Outer ecalli 0 forwards (response: `[8:new_r7][8:new_r8][4:num_memwrites][8:new_gas][entries…]`); outer ecalli 1 returns the last r8.
- **Adapter import resolution against main exports**: `adapter_merge.rs` resolves matching imports internally with type-signature validation, letting the adapter call compiler functions (`host_read_memory`, `host_write_memory`) directly.
- **Dynamic ecalli limitation**: PVM `ecalli` requires a compile-time-constant index. Workaround: per-ecalli handlers, or the replay adapter's fixed-index forwarding.
- **Inline threshold**: `OptimizationFlags.inline_threshold: Option<u32>`, default `Some(5)`. Functions with more LLVM IR instrs than the threshold are marked `noinline`. Use `None` (CLI `225`) for LLVM defaults.
- **`--trap-floats`** (feature gate, not optimization): replaces every f32/f64 op with `@llvm.trap()` + LLVM unreachable so compilation finishes past the float wall. Do NOT use bare `unreachable` (simplifycfg deletes float-only if-arms as UB) and do NOT set `self.unreachable = true` in the frontend (leaves phis without an incoming edge). See `docs/src/trap-floats.md` and `docs/src/learnings.md` "Trap-Floats Lowering".
- **Libcall recognition**: replaces `__multi3` and `__udivti3` bodies with PVM-friendly versions (name + signature + body-scan gated). `__multi3` shrinks to ~8 instrs; `__udivti3` dispatches `(a_hi | b_hi) == 0` fast path (5 instrs) vs slow path (forwards to `specialized_div_rem`). See `llvm_frontend/libcall_recognition.rs` and `docs/src/optimizations.md`.
- **Operator-error location wrapping**: `Error::Located { func_idx, func_name, op_offset, cause }` (frontend attaches WASM byte offset; backend says "during PVM lowering"). `Error::AdapterMerge { context, cause }` wraps function-body and element re-encoding in `adapter_merge.rs`. Both guard against double-wrapping. Names from `WasmModule::local_function_display_name`.
- **Dead ends** (do not retry): rematerialization, callee-saved state preservation after calls, per-phi early-expiration guard, non-leaf r7/r8 allocation. All four share the same root cause — `operand_reg()` may use a value's register as both source and destination during address computation, clobbering it. See `docs/src/learnings.md`.
- **Configurable optimizations**: see `docs/src/optimizations.md` for the full flag list. All default enabled; CLI `--no-*` flags disable individually.

---

## Conventions

### Code Style
- `rustfmt` defaults, `clippy` warnings = errors
- `unsafe_code = "deny"` (workspace lint)
- `thiserror` for errors, `tracing` for logging
- Unit tests inline under `#[cfg(test)]`
- **Prefer `BTreeMap`/`BTreeSet` over `HashMap`/`HashSet`**. The compiler must be reproducible: the same input WASM must produce byte-identical output every run. Rust's default `HashMap`/`HashSet` use a per-process-randomized hasher, so iteration order varies across invocations; any iteration whose side effects reach the emitted bytes (e.g. instruction emission, register/offset assignment, live-interval extension) would leak that randomness into the output. `BTreeMap`/`BTreeSet` iterate in key order and sidestep the problem entirely. Use `HashMap` only when the key type has no `Ord` impl (e.g. `inkwell::basic_block::BasicBlock`); in those cases, keep the map purely for `get`/`insert`/`contains_key` lookups, and if you must iterate, collect into a `Vec` sorted by a deterministic derived key (position, index) first. Avoid relying on "iteration happens to be commutative" — a future edit can break that silently.

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
| Add PVM lowering (control flow) | `llvm_backend/control_flow.rs` | Branches, phi (incl. `topo_order_phase2` for temp/dest alias-safe Phase 2 emission), switch, return |
| Modify per-function block emission order | `llvm_backend/mod.rs:compute_block_layout()` | Greedy fallthrough-biased trace. Shared with regalloc via `block_order` parameter so live intervals match emission order. |
| Add PVM lowering (calls) | `llvm_backend/calls.rs` | Direct/indirect calls, import stubs |
| Add PVM lowering (intrinsics) | `llvm_backend/intrinsics.rs` | PVM + LLVM intrinsic lowering (incl. min/max, bswap, bitreverse, ctlz/cttz/ctpop, abs, fshl/fshr, `{u,s}{add,sub}.sat`) |
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
| Add/modify import map | `tests/fixtures/imports/*.imports` | Text-based import maps (trap, nop, ecalli:N) |
| Fix test execution | `tests/helpers/run.ts` | `runJam()` (u32 result), `runJamBytes()` (raw `Uint8Array` — use for hash / multi-byte outputs) |
| Byte-level native WASM run | `tests/helpers/wasm-runner.ts` | `runWasmNativeBytes()` — raw bytes variant of `runWasmNative` for differential tests that compare buffers |
| Hand-crafted crypto example | `tests/fixtures/wat/blake2b.jam.wat` + `tests/layer3/blake2b.test.ts` | RFC 7693 blake2b (unkeyed, variable output 1..=64) with 3-way agreement tests vs `@noble/hashes` |
| Hand-crafted hash example (SHA-2) | `tests/fixtures/wat/sha512.jam.wat` + `tests/layer3/sha512.test.ts` | FIPS 180-4 SHA-512 (fixed 64-byte output) with 3-way agreement tests vs `@noble/hashes/sha2`. Input cap 32 KB (capped to keep the hex CLI encoding under Linux's 128 KB per-argv-string limit). |
| Fix test build | `tests/build.ts` + `tests/helpers/compile.ts` | Build orchestrator + compilation helpers |
| Debug execution | `tests/utils/trace-steps.ts` | Shows PC, gas, registers per step |
| Generate execution trace | `tests/utils/generate-trace.ts` | Outputs anan-as trace format to stdout |
| PVM-in-PVM trace replay | `tests/utils/trace-replay-pip.ts` | Replays trace through PVM-in-PVM pipeline |
| Verify JAM file | `tests/utils/verify-jam.ts` | Parse headers, jump table, code |
| Add/modify optimization | `translate/mod.rs` (`OptimizationFlags`) | Add flag + thread through `LoweringContext` → `PvmEmitter` |
| Add a compiler-builtins libcall replacement | `llvm_frontend/libcall_recognition.rs` + `translate/wasm_module.rs` (`scan_libcall_targets`, `LibcallTargets`) | Pick name; add to the parse-time table with signature/body-scan gates; write `emit_<kind>_body` synthesizer using `WasmToLlvm` accessors. Cost / win analysis lives in `docs/src/learnings.md` "Libcall Recognition". |
| Toggle optimization in CLI | `wasm-pvm-cli/src/main.rs` | Add `--no-*` flag to `Compile` subcommand |
| Add property test | `crates/wasm-pvm/tests/property_tests.rs` | Proptest: compilation safety, encoding roundtrips (28 tests) |
| Understand/modify native WASM runner | `tests/helpers/wasm-runner.ts` | Native WASM runner (WAT→WASM via wabt, Bun WebAssembly) |
| Define differential test suite | `tests/helpers/suite.ts` | `defineDifferentialSuite()` + `skipDifferential` flag |
| Add/aggregate differential tests | `tests/differential/differential.test.ts` | Import suites + call `defineDifferentialSuite()` |
| Modify trap-floats lowering | `llvm_frontend/function_builder.rs::emit_float_trap` + `float_op_stack_effect` | Frontend emits `@llvm.trap()` + LLVM unreachable; backend lowers `llvm.trap` in `llvm_backend/intrinsics.rs::lower_llvm_intrinsic`. See `docs/src/trap-floats.md`. |
| Add/edit trap-floats tests | `crates/wasm-pvm/tests/float_handling.rs` (Rust unit) + `tests/layer1/trap-floats.test.ts` (CLI + runtime trap) | Compile-time + run-time coverage |
| Diagnostic location wrapping | `Error::Located` (in `error.rs`) wrapped at `function_builder.rs::translate_function` (frontend, `op_offset = Some(_)`) and `llvm_backend::lower_function` (backend, `op_offset = None`); `Error::AdapterMerge` wrapped in `translate/adapter_merge.rs::{wrap_adapter_err, encode_function_body, encode_function_body_main}` and around the per-element `encode_element` calls in `build_merged_module` (other adapter-merge failures stay unwrapped — they carry an inline `"main"`/`"adapter"` label) | Function name from `WasmModule::local_function_display_name`, op byte offset from `into_iter_with_offsets()` |

---

## Optimization Flags

See `docs/src/optimizations.md` for the canonical list of flags, what each controls, and measured impact. The flags live on `OptimizationFlags` in `translate/mod.rs` (re-exported from `lib.rs`); each defaults to enabled, with `--no-*` CLI counterparts in `wasm-pvm-cli/src/main.rs`.

**Threading path**: `CompileOptions.optimizations` → `LoweringContext.optimizations` → `EmitterConfig` (`*_enabled` fields) → `PvmEmitter.config`. `llvm_passes` / `inlining` / `inline_threshold` / `mergefunc` go directly to `translate_wasm_to_llvm()`; `aggressive_register_allocation` / `allocate_scratch_regs` / `allocate_caller_saved_regs` go directly to `regalloc::run()`.

**Adding a new optimization**: add a field to `OptimizationFlags`, thread it through `LoweringContext` → `EmitterConfig`, guard the codegen with `e.config.<flag>`, add a `--no-*` CLI flag, document it in `docs/src/optimizations.md`.

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
| `0x30000` | Mem-size slot (4 bytes, fixed position, only when `memory.size`/`grow`/`init` used), then user globals (per-global width: 4 B for i32/f32, 8 B for i64/f64; see `docs/src/learnings.md` "Global Storage Width"), passive segment length slots (4 bytes each), then (when any local function takes >4 params) a 256-byte parameter overflow area. |
| `region_end+` | WASM linear memory. Computed via `compute_wasm_memory_base(&global_widths, num_passive_segments, has_mem_size_global, needs_param_overflow)`. **No 4KB alignment** is applied: anan-as page-aligns the rw_data tail (`heapZerosStart`) independently, so the base sits tightly against the end of the globals/overflow region. For a bare memory-only program the base lands at `0x30004`; for a typical AS program (1 i32 global × 4B, mem-size, no overflow) at `0x30008`. Collapses the 4KB leading-zero page that every memory-using program used to pay for (~4 KB saving per fixture with data segments). |
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
| r7 | Return value from calls / SPI args pointer. Allocatable in leaf functions only (controlled by `allocate_caller_saved_regs`). Non-leaf allocation infeasible — see `docs/src/learnings.md` "Non-Leaf r7/r8 Allocation". Overwritten by return sequence. |
| r8 | SPI args length. Allocatable in leaf functions only (controlled by `allocate_caller_saved_regs`). Free after prologue. |
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
