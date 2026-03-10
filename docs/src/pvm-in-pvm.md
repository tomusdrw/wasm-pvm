# PVM-in-PVM Execution

The compiler can compile the [anan-as PVM interpreter](https://github.com/tomusdrw/anan-as) (written in AssemblyScript) to PVM bytecode, then run PVM programs *inside* this PVM interpreter that is itself running on PVM. This serves as a comprehensive integration test and stress test of the compiler.

---

## Goal

Run PVM programs (trap.jam, add.jam) through the anan-as PVM interpreter that is itself compiled to PVM bytecode and running on PVM.

**Pipeline**: `inner.wat → inner.jam` + `compiler.wasm → compiler.jam` → feed inner.jam as args to compiler.jam → outer anan-as CLI runs it all.

## Bugs Found & Fixed

### Bug 1: `HasMetadata.Yes` in anan-as entry point

**File**: `vendor/anan-as/assembly/index-compiler.ts:91`

The anan-as compiler entry point was calling:
```typescript
prepareProgram(InputKind.SPI, HasMetadata.Yes, spiProgram, [], [], [], innerArgs);
```

With `HasMetadata.Yes`, the SPI parser first calls `extractCodeAndMetadata()` which reads a varint-encoded metadata length from the start of the data. Since inner JAM programs don't have metadata, this read garbage values (e.g., the ro_data_length field), corrupting all subsequent parsing.

**Symptom**: Native WASM test failed with `"Not enough bytes left. Need: 7561472, left: 56377"` — the parser was reading the first SPI header bytes as a metadata length.

**Fix**: Changed to `HasMetadata.No` and rebuilt the vendor with `npm run asbuild:compiler`.

### Bug 2: Unknown WASM imports compiled to TRAP

**File**: `crates/wasm-pvm/src/llvm_backend/calls.rs:137-138`

The wasm-pvm compiler mapped all unknown WASM imports (anything not `host_call` or `pvm_ptr`) to PVM TRAP instructions. The anan-as compiler.wasm imports two functions:
- `env.abort` — called on unrecoverable AS runtime errors
- `env.console.log` — called during normal execution for debug logging

Since `console.log` is called in the normal success path (confirmed by native WASM test showing `console.log: 11952`), the TRAP instruction killed the PVM program before it could complete.

**Symptom**: PVM execution panicked at PC 100640 (a TRAP instruction corresponding to the `console.log` import call). The outer anan-as interpreter reported `"Unhandled host call: ecalli 0"`.

**Fix**: Changed unknown imports to be no-ops (silently skip) instead of TRAPs. The `abort` import specifically remains a TRAP since it indicates unrecoverable errors and should terminate execution.

```rust
// Before: all unknown imports → TRAP
e.emit(Instruction::Trap);

// After: only abort → TRAP, others are no-ops
let is_abort = import_name == Some("abort");
if is_abort {
    e.emit(Instruction::Trap);
}
```

## Debugging Journey

1. **Initial state**: compiler.jam panicked at PC 150403 after ~95K instructions
2. **First hypothesis** (from subagent): Jump table corruption — turned out to be incorrect; the verify-jam tool's VarU32 decoder has an endianness bug that displayed wrong values
3. **Key insight**: Ran compiler.wasm natively with the same args — it also failed! This proved the issue was in the input format, not wasm-pvm compilation
4. **Native error**: `"Not enough bytes left. Need: 7561472"` pointed to SPI parsing reading garbage lengths
5. **Found Bug 1**: `HasMetadata.Yes` → fixed to `HasMetadata.No`, rebuilt vendor
6. **After fix 1**: Native WASM worked perfectly (trap.jam → PANIC, add.jam → result 12), but PVM version still failed with `ecalli 0` at PC 100640
7. **Traced PVM execution**: Confirmed PC 100640 contains opcode 0x00 (TRAP), which is the compiled `console.log` import
8. **Confirmed**: Native WASM calls console.log during normal execution → in PVM this becomes TRAP → panic
9. **Found Bug 2**: Fixed import handling to make non-abort imports no-ops
10. **Both tests pass**: trap.jam returns inner PANIC, add.jam returns inner result 12

## Performance Notes

PVM-in-PVM tests are inherently slow (~85 seconds each) because:
- The outer anan-as interpreter executes ~525M PVM instructions
- Most of this is the inner interpreter's initialization (AS runtime setup, SPI parsing, memory page allocation)
- The actual inner program execution is tiny (~46-65K gas)
- The JS-based anan-as interpreter processes ~6M instructions/second

Tests have 180-second timeouts to accommodate this.

## PVM-in-PVM Benchmarks

| Benchmark | JAM Size | Code Size | Outer Gas | Direct Gas | Overhead |
|-----------|----------|-----------|-----------|------------|----------|
| TRAP (interpreter overhead) | 21 B | 1 B | 80,577 | - | - |
| add(5,7) | 201 B | 130 B | 1,238,302 | 39 | 31,751x |
| AS fib(10) | 708 B | 572 B | 1,753,546 | 324 | 5,412x |
| JAM-SDK fib(10)\* | 25.4 KB | 16.2 KB | 7,230,603 | 42 | 172,157x |
| Jambrains fib(10)\* | 61.1 KB | - | 6,373,683 | 1 | 6,373,683x |
| JADE fib(10)\* | 67.3 KB | 45.7 KB | 19,555,955 | 504 | 38,801x |
| aslan-fib accumulate\* | 37.1 KB | 17.6 KB | 10,511,413 | 15,968 | 658x |

\*These programs exit on unhandled host calls (ecalli). Gas cost reflects parsing/loading plus partial execution up to the first unhandled ecalli.
