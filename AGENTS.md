# WASM-PVM Project - AI Agent Knowledge Base

**Project**: WebAssembly to PolkaVM (PVM) bytecode recompiler
**Stack**: Rust (core) + TypeScript (tests) + AssemblyScript (examples)
**Architecture**: `WASM → [inkwell] → LLVM IR → [mem2reg] → [Rust PVM backend] → PVM bytecode`
**Docs**: `PLAN.md` (roadmap), `LEARNINGS.md` (tech reference), `gp-0.7.2.md` (PVM spec)

---

## Quick Start

```bash
# Build
cargo build --release

# Build with LLVM backend
LLVM_SYS_181_PREFIX=/opt/homebrew/opt/llvm@18 cargo build --release --features llvm-backend

# Test
cargo test                                    # Unit tests (Rust)
cargo test --features llvm-backend            # Unit tests with LLVM backend
cargo test --features llvm-backend --test differential  # Differential tests
cd tests && bun test                          # Integration tests (360 tests)

# Compile WASM → JAM
cargo run -p wasm-pvm-cli -- compile tests/fixtures/wat/add.jam.wat -o dist/add.jam

# Run JAM
cd tests && bun utils/run-jam.ts ../dist/add.jam --args=0500000007000000
```

---

## Structure

```
crates/
├── wasm-pvm/              # Core library
│   └── src/
│       ├── llvm_frontend/ # WASM → LLVM IR [LLVM backend]
│       │   ├── function_builder.rs (~1350 lines - core translator)
│       │   └── mod.rs
│       ├── llvm_backend/  # LLVM IR → PVM bytecode [LLVM backend]
│       │   ├── lowering.rs (~1900 lines - core lowering)
│       │   └── mod.rs
│       ├── translate/     # Compilation orchestration
│       │   ├── mod.rs     (pipeline dispatch + SPI assembly)
│       │   ├── wasm_module.rs (shared WASM section parsing)
│       │   ├── memory_layout.rs (PVM memory address constants)
│       │   ├── codegen.rs (legacy backend - direct translation)
│       │   └── stack.rs   (legacy operand stack)
│       ├── ir/            # Legacy IR (to be removed)
│       ├── pvm/           # PVM instruction definitions
│       │   ├── instruction.rs  # Instruction enum + encoding
│       │   ├── opcode.rs       # Opcode constants
│       │   └── blob.rs         # Program blob format
│       ├── spi.rs         # JAM/SPI format encoder
│       └── error.rs       # Error types (thiserror)
└── wasm-pvm-cli/          # CLI binary
    └── src/main.rs

tests/                     # Integration tests & tooling
├── build.ts               # Test build orchestrator
├── differential.rs        # 43 differential tests (both backends)
├── utils/                 # Utility scripts (run-jam, verify-jam, trace)
├── fixtures/              # Test cases
│   ├── wat/               # WAT test programs (43 fixtures)
│   └── assembly/          # AssemblyScript examples
├── helpers/               # Test helpers
└── data/                  # Test definitions (test-cases.ts)

vendor/                    # Git submodules (anan-as)
```

---

## Domain Knowledge

### Compiler Pipeline (LLVM backend)
1. **WASM parsing**: `wasm_module.rs` parses all WASM sections into `WasmModule` struct
2. **LLVM IR generation**: `llvm_frontend/function_builder.rs` translates `wasmparser::Operator` → LLVM IR using inkwell
3. **mem2reg pass**: LLVM's `mem2reg` promotes alloca'd locals to SSA registers
4. **PVM lowering**: `llvm_backend/lowering.rs` reads LLVM IR and emits PVM bytecode
5. **SPI assembly**: `translate/mod.rs` builds entry header, dispatch tables, ro_data/rw_data

### PVM (Target)
- Register-based (13 × 64-bit registers)
- Flat control flow with jumps/branches
- Gas metering on all instructions
- Memory: addresses < 2^16 panic

### Key Design Decisions
- **PVM-specific intrinsics** for memory ops (`@__pvm_load_i32`, `@__pvm_store_i32`, etc.) — avoids `unsafe` GEP/inttoptr
- **Stack-slot approach**: every SSA value gets a dedicated memory offset from SP (correctness-first, register allocator is future work)
- **All values as i64**: PVM registers are 64-bit; simplifies translation
- **Feature flags**: `llvm-backend` enables LLVM pipeline, default uses legacy direct translation

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
- Rust differential tests in `tests/differential.rs`
- LLVM backend gated behind `#[cfg(feature = "llvm-backend")]`

---

## Where to Look

| Task | Location | Notes |
|------|----------|-------|
| Add WASM operator (LLVM) | `llvm_frontend/function_builder.rs` | Add to operator match |
| Add PVM lowering (LLVM) | `llvm_backend/lowering.rs` | Add instruction lowering |
| Add PVM instruction | `pvm/opcode.rs` + `pvm/instruction.rs` | Add enum + encoding |
| Fix WASM parsing | `translate/wasm_module.rs` | `WasmModule::parse()` |
| Fix compilation pipeline | `translate/mod.rs` | `compile_via_llvm()` / `compile_legacy()` |
| Add test case | `tests/data/test-cases.ts` | Hex args, little-endian |
| Fix test execution | `tests/helpers/run.ts` | `runJam()` |
| Debug execution | `tests/utils/trace-steps.ts` | Shows PC, gas, registers per step |
| Verify JAM file | `tests/utils/verify-jam.ts` | Parse headers, jump table, code |

---

## Anti-Patterns (Forbidden)

1. **No `unsafe` code** — Strictly forbidden by workspace lint
2. **No panics in library code** — Use `Result<>` with `Error::Internal`
3. **No floating point** — PVM lacks FP support; reject WASM floats
4. **Don't break register conventions** — Hardcoded in multiple files
5. **NEVER use --no-verify on git push** — Always ensure tests and linters pass

---

## Memory Layout (Hardcoded)

| Address | Purpose |
|---------|---------|
| `0x10000` | Read-only data (dispatch table) |
| `0x30000` | Globals storage |
| `0x30100` | User heap (results) |
| `0x40000` | Spilled locals |
| `0x50000+` | WASM linear memory base |
| `0xFEFF0000` | Arguments (`args_ptr`) |
| `0xFFFF0000` | EXIT address (HALT) |

---

## Register Allocation

| Register | Usage |
|----------|-------|
| r0 | Return address (jump table index) |
| r1 | Stack pointer |
| r2-r6 | Scratch registers (LLVM backend) / Operand stack (legacy) |
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
