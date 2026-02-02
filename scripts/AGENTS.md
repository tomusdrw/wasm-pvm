# TypeScript Tooling Scripts

**Purpose**: Integration testing and JAM execution via anan-as

## Files

| File | Lines | Role |
|------|-------|------|
| `test-all.ts` | 216 | Main test runner (62 tests) |
| `run-jam.ts` | 155 | Execute JAM files with anan-as |
| `test-cases.ts` | 150 | Test case definitions (14 suites) |
| `trace-jam.ts` | 41 | Quick step-by-step trace (50 steps default) |
| `trace-steps.ts` | 53 | Detailed trace with final state summary (50K steps) |
| `verify-jam.ts` | 196 | Validate JAM file structure and headers |

## Key Patterns

### Test Case Format
```typescript
export const testCases: TestCase[] = [
  {
    name: 'add',
    tests: [
      { args: '0500000007000000', expected: 12, description: '5 + 7 = 12' },
    ],
  },
];
```

Args are hex-encoded little-endian u32s.

### Test Execution Flow
1. `compileWatIfAvailable()` - Auto-compile `.jam.wat` → `.jam`
2. `compileAsIfAvailable()` - Auto-compile AS `.wasm` → `.jam`
3. `runJamFile()` - Execute via anan-as library
4. Parse `Result: X` from output

### JAM Execution
```typescript
const { InputKind, HasMetadata, prepareProgram, runProgram } = ananAs;
const prog = prepareProgram(InputKind.SPI, HasMetadata.No, program, [], [], [], args);
const result = runProgram(prog, gas, pc, false, true);
```

### Result Extraction
Results read from memory chunks at `result_ptr` (r7):
```typescript
const ptr_start = Number(result.registers[7]);
const ptr_end = Number(result.registers[8]);
// Reconstruct from sparse memory chunks
```

## Where to Look

| Task | Location |
|------|----------|
| Add new test case | `test-cases.ts` - add to testCases array |
| Fix test execution | `test-all.ts:runJamFile()` |
| Fix result parsing | `run-jam.ts` - memory chunk reconstruction |
| Debug execution trace | `trace-steps.ts` - step-by-step with final state |
| Quick execution trace | `trace-jam.ts` - minimal 50-step trace |
| Verify JAM structure | `verify-jam.ts` - validate headers and blob |
| Add test filtering | `test-all.ts:--filter=` argument |
| Skip AS tests | Currently auto-compiled, search for `skipped` |

## Integration Points

- **anan-as**: `vendor/anan-as/dist/build/release.js`
- **wasm-pvm-cli**: `cargo run -p wasm-pvm-cli -- compile ...`
- **Examples**: `examples-wat/` (WAT) and `examples-as/build/` (AS)

## Commands

```bash
# Run all tests
bun scripts/test-all.ts

# Filter tests
bun scripts/test-all.ts --filter=add

# Run single JAM
bun scripts/run-jam.ts dist/add.jam --args=0500000007000000

# Quick trace (50 steps default) - for simple debugging
bun scripts/trace-jam.ts dist/add.jam 0500000007000000

# Detailed trace with final state (50K steps default) - for infinite loops
bun scripts/trace-steps.ts dist/life.jam 01000000 100000

# Verify JAM file structure
bun scripts/verify-jam.ts dist/add.jam
```

## Debugging and Tracing

### When to Use Each Tool

| Tool | Use Case | Default Steps | Output |
|------|----------|---------------|--------|
| `run-jam.ts` | Normal execution, get final result | 100M gas | Result value from memory |
| `trace-jam.ts` | Quick debug, simple programs | 50 | Step trace to console |
| `trace-steps.ts` | Complex debugging, infinite loops | 50,000 | Step trace + final state summary |
| `verify-jam.ts` | Check JAM structure validity | N/A | Header info + structure analysis |

### Trace Script Differences

**trace-jam.ts** - Minimal, quick traces:
- Default 50 steps (will OOG on complex programs)
- Raw anan-as verbose output
- Use for: Simple programs, quick sanity checks
- Syntax: `bun scripts/trace-jam.ts <jam-file> [hex-args] [max-steps]`

**trace-steps.ts** - Comprehensive debugging:
- Default 50,000 steps
- Shows final register state formatted as `r0=value, r1=value, ...`
- Use for: Infinite loops, debugging why a program fails
- Syntax: `bun scripts/trace-steps.ts <jam-file> [hex-args] [max-steps]`

### Debugging Workflow

1. **Program doesn't compile**: Check WAT syntax, verify with `wasm-validate`
2. **Program compiles but wrong result**: Use `run-jam.ts` with `--gas=high` first
3. **Program runs out of gas**: Use `trace-steps.ts` with high step limit
4. **Infinite loop suspected**: Use `trace-steps.ts` and look for PC patterns
5. **Verify binary structure**: Use `verify-jam.ts` to inspect headers

### Reading Trace Output

Trace output format (from anan-as):
```
PC = 10              ; Program counter (instruction offset)
GAS = 99             ; Gas remaining
STATUS = -1          ; -1 = running, 0 = halt, 4 = out of gas
REGISTERS = ...      ; Decimal values
REGISTERS = 0x...    ; Hex values
ARGUMENTS:           ; Instruction arguments
```

The trace shows **every step** - use grep to filter:
```bash
# Find all jumps to PC=42
bun scripts/trace-steps.ts prog.jam args 1000 | grep "PC = 42"

# Watch gas consumption
bun scripts/trace-steps.ts prog.jam args 100 | grep "GAS ="
```

## Anti-Patterns

1. **Don't add Rust tests here** - Integration tests are TypeScript-only
2. **Preserve hex arg encoding** - Little-endian u32s expected by WASM
3. **Don't delete skip logic for AS** - AS tests need pre-built WASM

## Notes

- No Jest/Mocha - uses vanilla TypeScript with execSync
- Tests compile WAT/AS on-demand if JAM missing
- anan-as library handles PVM interpretation
- Output parsing relies on `Result: X` string format
