# 07 - Testing Strategy

**Category**: Testing  
**Goal**: Comprehensive, automated, fast testing at every layer

---

## Summary

The current testing strategy relies heavily on integration tests (TypeScript) with minimal unit testing (Rust). This document proposes a comprehensive testing strategy with tests at every layer of the compiler.

---

## Current State Analysis

### Existing Tests

| Test Type | Count | Language | Location |
|-----------|-------|----------|----------|
| Unit tests | ~30 | Rust | `crates/wasm-pvm/src/` |
| Integration tests | 62 | TypeScript | `scripts/test-all.ts` |
| Example programs | 50+ | WAT/AS | `examples-wat/`, `examples-as/` |

### Current Test Issues

1. **Insufficient unit testing**: Only 30 Rust unit tests
2. **No IR-level tests**: Cannot test translation logic in isolation
3. **No property-based testing**: Only example-based tests
4. **Slow feedback loop**: Integration tests compile and run external interpreter
5. **TypeScript dependency**: Core compiler tested via JS/TS wrapper
6. **No mutation testing**: Don't know if tests catch real bugs
7. **No fuzzing**: Haven't found edge cases through automation

---

## Proposed Testing Strategy

### Layer 1: Unit Tests (Rust)

**Goal**: Test every component in isolation

#### IR Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ir_add32() {
        let mut builder = IrBuilder::new();
        let v1 = builder.const_i32(5);
        let v2 = builder.const_i32(3);
        let result = builder.add32(v1, v2);
        
        assert_eq!(builder.instruction_count(), 1);
        assert!(matches!(builder.get(result), Value::VirtualReg(_)));
    }
    
    #[test]
    fn test_ir_constant_folding() {
        let mut builder = IrBuilder::new();
        let v1 = builder.const_i32(5);
        let v2 = builder.const_i32(3);
        let result = builder.add32(v1, v2);
        
        // After constant folding
        let folded = fold_constants(&builder);
        assert!(matches!(folded.get(result), Value::Const(8)));
    }
}
```

#### Register Allocation Tests

```rust
#[test]
fn test_register_allocation_simple() {
    let func = build_simple_function();
    let mut allocator = LinearScanAllocator::new(13); // 13 PVM registers
    
    let allocation = allocator.allocate(&func);
    
    // No spills for simple case
    assert!(allocation.spills.is_empty());
    
    // Check no interference
    for (vreg, preg) in allocation.mapping {
        assert!(!allocator.interferes(vreg, preg));
    }
}

#[test]
fn test_spill_insertion() {
    let func = build_function_with_many_values();
    let mut allocator = LinearScanAllocator::new(5); // Only 5 registers
    
    let allocation = allocator.allocate(&func);
    
    // Should have spills
    assert!(!allocation.spills.is_empty());
    
    // Spills should be at correct locations
    for spill in &allocation.spills {
        assert!(spill.location.is_valid());
    }
}
```

#### Memory Layout Tests

```rust
#[test]
fn test_memory_layout_no_overlap() {
    let module = create_test_module();
    let layout = MemoryLayout::compute(&module).unwrap();
    
    assert!(!layout.globals.overlaps(&layout.wasm_memory));
    assert!(!layout.spilled_locals.overlaps(&layout.stack));
}

#[test]
fn test_global_address_calculation() {
    let layout = MemoryLayout::default();
    
    assert_eq!(layout.global_addr(0), 0x30000);
    assert_eq!(layout.global_addr(1), 0x30004);
    assert_eq!(layout.global_addr(63), 0x30000 + 63 * 4);
}
```

---

### Layer 2: Integration Tests (Rust)

**Goal**: Test compilation pipeline end-to-end

#### WASM → IR Tests

```rust
#[test]
fn test_wasm_to_ir_simple_add() {
    let wasm = wat::parse_str(r#"
        (module
            (func (export "test") (param i32) (result i32)
                local.get 0
                i32.const 5
                i32.add))
    "#).unwrap();
    
    let module = parse_and_validate(&wasm).unwrap();
    let ir = generate_ir(&module).unwrap();
    
    // Check IR structure
    let func = &ir.functions[0];
    assert_eq!(func.params.len(), 1);
    assert!(func.returns.is_some());
}
```

#### Full Pipeline Tests

```rust
#[test]
fn test_compile_simple_program() {
    let wasm = wat::parse_str(r#"
        (module
            (memory 1)
            (global $result_ptr (mut i32) (i32.const 0))
            (global $result_len (mut i32) (i32.const 0))
            (func (export "main") (param i32) (param i32)
                (global.set $result_ptr (i32.const 0x30100))
                (global.set $result_len (i32.const 4))))
    "#).unwrap();
    
    let program = compile(&wasm).unwrap();
    
    // Verify program structure
    assert!(!program.instructions.is_empty());
    assert!(program.jump_table.len() > 0);
}
```

---

### Layer 3: Property-Based Tests

**Goal**: Generate random valid WASM and verify properties

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_compile_does_not_panic(wasm in valid_wasm_module()) {
        // Should either succeed or fail with a proper error, never panic
        let result = std::panic::catch_unwind(|| {
            compile(&wasm)
        });
        
        prop_assert!(result.is_ok(), "Compilation panicked");
        
        if let Ok(Ok(program)) = result {
            // Verify program is well-formed
            prop_assert!(!program.instructions.is_empty());
        }
    }
    
    #[test]
    fn test_arithmetic_correctness(a in any::<i32>(), b in any::<i32>()) {
        let wasm = generate_add_program(a, b);
        let program = compile(&wasm).unwrap();
        let result = execute_on_pvm(&program, &[]);
        
        prop_assert_eq!(result, (a.wrapping_add(b)) as u32);
    }
}
```

---

### Layer 4: Fuzzing

**Goal**: Find edge cases and crashes

```rust
// Using cargo-fuzz
fuzz_target!(|data: &[u8]| {
    // Try to parse and compile any byte sequence
    if let Ok(module) = parse(&data) {
        if let Ok(ir) = generate_ir(&module) {
            // Try various optimization levels
            for opt_level in 0..=3 {
                let _ = optimize(&ir, opt_level);
            }
        }
    }
});
```

**Fuzz Targets**:
1. Random byte sequences → parser
2. Valid WASM → IR generation
3. Valid IR → optimization
4. Valid IR → register allocation
5. Full pipeline → JAM output

---

### Layer 5: Differential Tests

**Goal**: Compare output with reference implementation

```rust
#[test]
fn test_against_reference_interpreter() {
    let test_cases = load_test_suite();
    
    for case in test_cases {
        let program = compile(&case.wasm).unwrap();
        let our_result = execute(&program, &case.args);
        let ref_result = execute_with_reference(&case.wasm, &case.args);
        
        assert_eq!(our_result, ref_result, 
            "Mismatch on test case: {}", case.name);
    }
}
```

**Reference Implementations**:
1. wasmtime (WASM reference)
2. anan-as (PVM reference)
3. Manual trace comparison

---

### Layer 6: Regression Tests

**Goal**: Ensure bugs stay fixed

```rust
// For each bug in KNOWN_ISSUES.md, add a test
#[test]
fn test_issue_memory_copy_overlap() {
    // Test overlapping copy works correctly
    let wasm = generate_overlap_copy_program();
    let program = compile(&wasm).unwrap();
    
    let result = execute(&program, &[]);
    assert_eq!(result, expected_overlap_result());
}

#[test]
fn test_issue_division_overflow() {
    // Test INT_MIN / -1 traps
    let wasm = generate_div_overflow_program();
    let program = compile(&wasm).unwrap();
    
    let result = execute(&program, &[]);
    assert!(result.is_trap());
}
```

---

### Layer 7: Performance Benchmarks

**Goal**: Track performance over time

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_fibonacci(c: &mut Criterion) {
    let wasm = load_fibonacci_wasm();
    let program = compile(&wasm).unwrap();
    
    c.bench_function("fibonacci 20", |b| {
        b.iter(|| execute(black_box(&program), black_box(&[20])))
    });
}

fn benchmark_compilation(c: &mut Criterion) {
    let wasm = load_large_wasm();
    
    c.bench_function("compile large module", |b| {
        b.iter(|| compile(black_box(&wasm)))
    });
}

criterion_group!(benches, benchmark_fibonacci, benchmark_compilation);
criterion_main!(benches);
```

**Benchmark Categories**:
1. Compilation time by module size
2. Execution time by instruction count
3. Memory usage during compilation
4. Generated code size

---

### Layer 8: Continuous Integration

**Goal**: Catch issues before they reach main

```yaml
# .github/workflows/ci.yml additions
- name: Run unit tests
  run: cargo test --lib

- name: Run integration tests
  run: cargo test --test '*'

- name: Run property-based tests
  run: cargo test --features proptest

- name: Run fuzzing (quick)
  run: cargo fuzz run compile -- -max_total_time=60

- name: Run benchmarks
  run: cargo bench -- --baseline main

- name: Generate coverage
  run: cargo tarpaulin --out xml

- name: Upload coverage
  uses: codecov/codecov-action@v3
```

---

## Test Data Generation

### WASM Generator

```rust
pub struct WasmGenerator {
    rng: StdRng,
}

impl WasmGenerator {
    pub fn generate_function(&mut self) -> Function {
        let num_params = self.rng.gen_range(0..5);
        let has_result = self.rng.gen_bool(0.5);
        
        let mut builder = FunctionBuilder::new();
        
        // Generate random instructions
        for _ in 0..self.rng.gen_range(1..100) {
            let instr = self.random_instruction();
            builder.push(instr);
        }
        
        builder.build()
    }
    
    fn random_instruction(&mut self) -> Instruction {
        match self.rng.gen_range(0..10) {
            0..=2 => Instruction::Const(self.rng.gen::<i32>()),
            3..=5 => Instruction::Add,
            6..=7 => Instruction::Load(self.rng.gen_range(0..1024)),
            8..=9 => Instruction::Store(self.rng.gen_range(0..1024)),
            _ => Instruction::Nop,
        }
    }
}
```

---

## Test Organization

### Directory Structure

```
crates/wasm-pvm/
├── src/
│   └── ...
├── tests/
│   ├── unit/                   # Unit tests per module
│   │   ├── ir_tests.rs
│   │   ├── register_tests.rs
│   │   └── memory_tests.rs
│   ├── integration/            # Integration tests
│   │   ├── compile_tests.rs
│   │   └── execute_tests.rs
│   ├── fixtures/               # Test WASM files
│   │   ├── add.wat
│   │   ├── factorial.wat
│   │   └── ...
│   └── benches/                # Benchmarks
│       └── compile_bench.rs
├── fuzz/
│   └── fuzz_targets/
│       ├── compile.rs
│       ├── optimize.rs
│       └── execute.rs
└── proptest-regressions/       # Property test failures
```

---

## Coverage Goals

| Component | Target Coverage | Current |
|-----------|-----------------|---------|
| IR generation | 90% | Unknown |
| Optimizations | 85% | Unknown |
| Register allocation | 90% | Unknown |
| Instruction selection | 95% | Unknown |
| Code emission | 90% | Unknown |
| Error handling | 100% | Unknown |
| Overall | 85% | ~30% |

---

## Tools and Crates

| Tool | Purpose | Integration |
|------|---------|-------------|
| `cargo test` | Unit/integration tests | Built-in |
| `proptest` | Property-based testing | Add to dev-dependencies |
| `cargo-fuzz` | Fuzzing | Separate fuzz crate |
| `criterion` | Benchmarking | Add to dev-dependencies |
| `cargo-tarpaulin` | Code coverage | CI only |
| `wat` | WAT parsing in tests | Already have |
| `pretty_assertions` | Better test output | Dev dependency |

---

## Implementation Roadmap

### Phase 1: Immediate (Week 1)

1. Add `pretty_assertions` for better test output
2. Convert existing unit tests to use it
3. Add tests for existing but untested functions
4. Measure current coverage with `cargo-tarpaulin`

### Phase 2: Short Term (Weeks 2-4)

1. Add IR-level unit tests for all operators
2. Add register allocation tests
3. Add memory layout tests
4. Set up property-based testing framework

### Phase 3: Medium Term (Weeks 5-8)

1. Create differential test harness
2. Add fuzzing targets
3. Add regression tests for known bugs
4. Set up CI for all test types

### Phase 4: Long Term (Ongoing)

1. Maintain test coverage above 85%
2. Regular fuzzing runs (nightly CI)
3. Performance regression tracking
4. Expand test suite with community examples

---

## Success Metrics

- **Coverage**: 85%+ line coverage
- **Test Count**: 500+ unit tests, 100+ integration tests
- **Bug Detection**: All bugs in KNOWN_ISSUES.md have regression tests
- **CI Time**: < 10 minutes for full test suite
- **Fuzzing**: 1M+ iterations per target without crashes

---

*Next: [08-rebuilding-plan.md](./08-rebuilding-plan.md)*
