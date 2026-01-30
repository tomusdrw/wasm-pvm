# TypeScript Tooling Scripts

**Purpose**: Integration testing and JAM execution via anan-as

## Files

| File | Lines | Role |
|------|-------|------|
| `test-all.ts` | 216 | Main test runner (62 tests) |
| `run-jam.ts` | 155 | Execute JAM files with anan-as |
| `test-cases.ts` | 150 | Test case definitions (14 suites) |

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
| Add test filtering | `test-all.ts:--filter=` argument |
| Skip AS tests | Currently auto-compiled, search for `skipped` |

## Integration Points

- **anan-as**: `vendor/anan-as/dist/build/release.js`
- **wasm-pvm-cli**: `cargo run -p wasm-pvm-cli -- compile ...`
- **Examples**: `examples-wat/` (WAT) and `examples-as/build/` (AS)

## Commands

```bash
# Run all tests
npx tsx scripts/test-all.ts

# Filter tests
npx tsx scripts/test-all.ts --filter=add

# Run single JAM
npx tsx scripts/run-jam.ts dist/add.jam --args=0500000007000000
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
