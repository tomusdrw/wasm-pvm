# WASM-to-PVM Translation Module

**Purpose**: Core WASM bytecode translation to PVM instructions

## Files

| File | Lines | Role |
|------|-------|------|
| `codegen.rs` | 2402 | Instruction translation, register allocation, calling conventions |
| `mod.rs` | 683 | Compilation orchestration, WASM parsing, SPI generation |
| `stack.rs` | 152 | Operand stack to register mapping |

## Key Patterns

### Translation Flow
1. `mod.rs:compile()` parses WASM sections
2. `translate_function()` creates `CodeEmitter`
3. `translate_op()` dispatches WASM operators to PVM instructions
4. `emit_epilogue()` sets up result_ptr/result_len for SPI

### Register Conventions
```rust
const ARGS_PTR_REG: u8 = 7;      // r7 - SPI args pointer
const ARGS_LEN_REG: u8 = 8;      // r8 - SPI args length
const STACK_PTR_REG: u8 = 1;     // r1 - PVM stack pointer
const RETURN_VALUE_REG: u8 = 1;  // r1 - function return value
const RETURN_ADDR_REG: u8 = 0;   // r0 - jump table index
const FIRST_LOCAL_REG: u8 = 9;   // r9 - first local variable
```

### Memory Layout (Hardcoded)
- `0x30000` - Globals storage
- `0x40000` - Spilled locals
- `0x50000` - WASM linear memory base
- `0x30100` - User heap (result storage)
- `0xFEFF0000` - Arguments (args_ptr)

### Instruction Emission
All PVM instructions go through:
```rust
emitter.emit(Instruction::Add32 { dst, src1, src2 });
```

### Stack Management
- Operand stack uses r2-r6 (5 slots)
- Deep stacks spill to memory at `spill_offset(depth)`
- Use `emitter.spill_push()` / `emitter.spill_pop()` for operands

## Complex Functions

### codegen.rs
- `translate_op()` - 600+ line match on all WASM operators
- `emit_call()` - Stack frame setup with overflow check
- `emit_call_indirect()` - Dispatch table + signature validation
- `emit_epilogue()` - SPI result setup for main function

### mod.rs
- `compile()` - Main entry, parses all WASM sections
- `resolve_call_fixups()` - Links function calls to targets
- `build_rw_data()` - Builds RW data section from WASM segments

## Where to Look

| Task | Location |
|------|----------|
| Add WASM operator | `codegen.rs:translate_op()` match arm |
| Fix register allocation | `codegen.rs:emit_*()` functions |
| Add PVM instruction | `pvm/instruction.rs` + `codegen.rs` |
| Fix calling convention | `emit_call()`, `emit_epilogue()` |
| Add global handling | `mod.rs:compile()` globals section |

## Anti-Patterns (This Module)

1. **Never add `unsafe`** - Workspace forbids it
2. **No panics** - Use `Result<>` with `Error::Internal`
3. **Don't break register constants** - Hardcoded in multiple places
4. **Preserve spill logic** - Operand stack spilling is fragile

## Notes

- `codegen.rs` has `#![allow(clippy::too_many_lines)]` - refactoring welcome but test thoroughly
- Control flow uses `ControlFrame` enum with label fixups
- Two-phase compilation: collect offsets, then resolve calls
- All tests in `scripts/test-all.ts` (TypeScript, not Rust)
