# PVM-in-PVM Debugging Status

## Goal
Debug the pvm-in-pvm scenario: running anan-as (PVM interpreter) compiled to PVM via
wasm-pvm, executing inner PVM programs correctly.

## Current Status: MAJOR BUG FIXED - More issues remain

### Summary
Found and fixed a critical `memory.copy` (memmove) bug in wasm-pvm's code generation.
The fix resolves the Arena page aliasing issue that caused inner program args to be zeros.
However, the inner interpreter still PANICs on the add(3,7) test, suggesting there are
additional bugs to investigate.

## Bug History (fixed across multiple sessions)

### Bug 1: emit_call_indirect r8 clobber
**Status**: FIXED
- `emit_call_indirect` was clobbering r8 (used as SPILL_ALT_REG)

### Bug 2: PC never advances (stuck at 0)
**Status**: FIXED
- Comparison operations had wrong semantics

### Bug 3: Local variable zero-initialization
**Status**: FIXED
- Local variables weren't being zero-initialized in some cases

### Bug 4: AS u8 arithmetic
**Status**: FIXED
- Byte arithmetic operations weren't correctly masking to u8

### Bug 5: memory.copy overlapping regions (memmove) - NEW
**Status**: FIXED (this session)
- **Root cause**: WASM `memory.copy` was implemented as a forward-only byte loop
- **Impact**: When `dest > src` with overlapping regions, the copy corrupted source data
- **Symptom**: `Array.unshift()` internally uses `memory.copy` to shift elements right.
  The forward copy overwrote elements before reading them, causing all Arena pages
  to share the same backing data pointer (all Uint8Array.wrap views aliased).
- **Fix**: Added backward copy path in `codegen.rs` MemoryCopy handler. When `dest > src`,
  the copy now starts from the end and works backward (like `memmove`).
- **File**: `crates/wasm-pvm/src/translate/codegen.rs` lines 2481-2599

## Current Issue

After the memmove fix:
- Inner program's args page at 0xFEFF0000 now correctly contains `[03 00 00 00 07 00 00 00]`
- All 5 direct tests (350 total) still pass
- The inner interpreter still PANICs at PC 56 with exitCode 0
- Registers after PANIC: r3=0, r4=0 (should be non-zero if args were loaded)
- r7 = 0xFEFA0000 (should be 0xFEFF0000 initially, but may have been modified)
- The PANIC with exitCode=0 means a memory fault at address < 0x10000 (reserved memory)

### Next Steps
1. Add step-by-step tracing to see when/how registers change during inner execution
2. Investigate if there's another copy/memory corruption issue
3. Check if the inner program's Decoder or other subsystem also uses `unshift`/`memory.copy`
4. Run the full pvm-in-pvm test suite to see overall improvement from the memmove fix

## Test Results

### Direct tests
- 350/350 pass (cargo test)

### PVM-in-PVM tests
- Not yet re-run with the memmove fix (was 0/77 before)
- add(3,7) single test: args page now correct but inner interpreter still PANICs

## Key Files Modified
- `crates/wasm-pvm/src/translate/codegen.rs` - Fixed MemoryCopy to handle overlapping regions
- `vendor/anan-as/assembly/index-compiler.ts` - Currently has debug code, needs restoration
