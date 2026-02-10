# 08 - V2 Planning and Roadmap

**Category**: Future Development  
**Goal**: Roadmap for V2 enhancements and optimizations

---

## Summary

The V1 compiler architecture is complete with LLVM as its foundation. The legacy direct-translator backend has been removed, leaving a clean, single-path compiler. This document outlines the roadmap for V2 enhancements.

**Current State (V1)**:
- ✅ LLVM-based IR layer
- ✅ Single clean backend
- ✅ 360+ tests passing
- ⚠️ Stack-slot allocation (conservative)
- ⚠️ Edge cases need verification

---

## V2 Goals

### Performance

1. **Implement Register Allocation**
   - Replace stack-slot approach
   - Linear scan or graph coloring for PVM's 13 registers
   - Expected 2-5x execution speedup

2. **Optimize Code Generation**
   - Type-aware slot sizes (4 bytes for i32)
   - Remove redundant load/store
   - Better br_table implementation

### Correctness

3. **Verify Edge Cases**
   - Division overflow trapping
   - Memory.copy overlapping regions
   - Fuzzing with wasm-smith

4. **WASM Spec Compliance**
   - Full spec test suite
   - Edge case coverage

### Features

5. **Explore Custom IR**
   - If LLVM coupling becomes problematic
   - Lighter weight than LLVM

6. **WASI Support**
   - ecalli instruction generation
   - Host function interface

7. **SIMD (Future)**
   - If PVM adds vector instructions

---

## Phase 1: Edge Case Verification (Weeks 1-2)

### Goal
Ensure complete WASM spec compliance for edge cases.

### Tasks

1. **Add division overflow tests**
   ```rust
   #[test]
   fn test_div_by_zero_traps() {
       let wasm = wat!("(module (func (export \"test\") (result i32) i32.const 1 i32.const 0 i32.div_s))");
       assert!(matches!(execute(&wasm), ExecutionResult::Trap));
   }
   
   #[test]
   fn test_int_min_overflow_traps() {
       let wasm = wat!("(module (func (export \"test\") (result i32) i32.const 0x80000000 i32.const -1 i32.div_s))");
       assert!(matches!(execute(&wasm), ExecutionResult::Trap));
   }
   ```

2. **Add memory.copy overlap tests**
   ```rust
   #[test]
   fn test_memory_copy_overlap() {
       // Test forward copy (dest < src)
       // Test backward copy (dest > src)
       // Verify memmove semantics
   }
   ```

3. **Fuzz testing setup**
   ```rust
   use wasm_smith::Module;
   
   #[test]
   fn fuzz_compile() {
       let mut rng = rand::thread_rng();
       for _ in 0..10000 {
           let wasm = Module::new(&mut rng);
           assert!(compile(&wasm.to_bytes()).is_ok());
       }
   }
   ```

### Success Criteria
- All edge case tests pass
- Fuzzing runs without crashes
- No spec compliance gaps identified

---

## Phase 2: Performance Benchmarking (Weeks 3-4)

### Goal
Establish baseline and identify optimization opportunities.

### Tasks

1. **Add benchmarks**
   ```rust
   use criterion::{black_box, criterion_group, criterion_main, Criterion};
   
   fn fibonacci_benchmark(c: &mut Criterion) {
       let wasm = compile_fibonacci();
       c.bench_function("fibonacci 30", |b| {
           b.iter(|| execute(black_box(&wasm), 30))
       });
   }
   ```

2. **Measure key metrics**
   - Compilation time (anan-as: ~423KB WASM)
   - Generated code size
   - Execution speed (microbenchmarks)
   - Stack frame sizes

3. **Profile hot paths**
   - Identify most common operations
   - Measure memory traffic
   - Find optimization opportunities

### Success Criteria
- Baseline measurements established
- Hot spots identified
- Optimization priorities clear

---

## Phase 3: Register Allocation (Weeks 5-10)

### Goal
Replace stack-slot approach with proper register allocation.

### Tasks

1. **Implement liveness analysis**
   ```rust
   fn analyze_liveness(func: &LlvmFunction) -> LivenessInfo {
       // For each value, determine:
       // - First use (birth)
       // - Last use (death)
       // - Interference with other values
   }
   ```

2. **Implement linear scan allocator**
   ```rust
   fn allocate_registers(
       values: &[Value],
       liveness: &LivenessInfo,
       num_regs: usize,
   ) -> Allocation {
       // Allocate PVM registers r2-r6 for values
       // Spill to slots when registers exhausted
   }
   ```

3. **Update lowering to use registers**
   ```rust
   // Instead of:
   emit(LoadIndU64 { dst: TEMP1, base: SP, offset: slot });
   emit(Add32 { dst: TEMP_RESULT, src1: TEMP1, src2: TEMP2 });
   emit(StoreIndU64 { base: SP, src: TEMP_RESULT, offset: dst_slot });
   
   // Use allocated register:
   let reg = allocation.get_register(value);
   emit(Add32 { dst: reg, src1: reg1, src2: reg2 });
   ```

4. **Test and benchmark**
   - Verify correctness unchanged
   - Measure speedup
   - Measure code size change

### Success Criteria
- All tests pass
- 2x+ execution speedup on benchmarks
- No significant code size regression

---

## Phase 4: Code Generation Optimizations (Weeks 11-14)

### Goal
Improve generated code quality beyond register allocation.

### Tasks

1. **Type-aware slot sizes**
   ```rust
   fn slot_size_for_type(ty: Type) -> i32 {
       match ty {
           Type::I32 => 4,
           Type::I64 => 8,
       }
   }
   ```

2. **Remove redundant stores**
   ```rust
   // Track which values are in registers
   // Don't store if next instruction uses same value
   ```

3. **Optimize br_table**
   ```rust
   // Binary search instead of linear
   // Or jump table if PVM supports it
   ```

4. **Peephole optimizations**
   ```rust
   // Pattern: Load followed by immediate store of same value
   // -> Remove both (dead code)
   
   // Pattern: Add 0
   // -> Remove (no-op)
   ```

### Success Criteria
- 10-20% code size reduction
- 10-30% fewer memory operations
- All tests pass

---

## Phase 5: Advanced Features (Future)

### WASI Support

1. **Implement ecalli**
   - Host function call mechanism
   - WASI syscall interface

2. **Add import handling**
   - Actual function bodies instead of stubs
   - Host-provided implementations

### Custom IR Exploration

1. **Evaluate LLVM coupling**
   - If maintenance becomes problematic
   - Consider lighter custom IR

2. **Prototype if needed**
   - Use existing `ir/` infrastructure
   - Compare complexity vs LLVM

---

## Timeline Summary

| Phase | Duration | Focus |
|-------|----------|-------|
| 1. Edge Cases | Weeks 1-2 | Spec compliance |
| 2. Benchmarking | Weeks 3-4 | Baseline measurements |
| 3. Register Allocation | Weeks 5-10 | Major performance improvement |
| 4. Optimizations | Weeks 11-14 | Code quality improvements |
| 5. Advanced Features | Ongoing | WASI, custom IR |

**Total V2 Core**: ~14 weeks for performance and correctness improvements.

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Register allocation bugs | Extensive testing, keep stack-slot as fallback |
| Performance regressions | Benchmark tracking, A/B comparisons |
| Spec compliance gaps | Fuzzing, spec test suite |
| Time overruns | Phase 1-2 are critical, others can be deferred |

---

## Success Criteria for V2

### Technical
- [ ] 2x+ execution speedup (register allocation)
- [ ] 15%+ code size reduction (optimizations)
- [ ] 100% spec compliance (edge cases)
- [ ] Fuzzing without crashes (stability)

### Quality
- [ ] Comprehensive benchmarks
- [ ] No performance regressions
- [ ] Documentation complete
- [ ] Tests for all edge cases

---

## Conclusion

V1 provides a **solid foundation** with LLVM-based architecture. V2 focuses on:

1. **Correctness** - Edge case verification, fuzzing
2. **Performance** - Register allocation, optimizations
3. **Features** - WASI, potential custom IR

The roadmap prioritizes correctness verification before major optimizations to ensure a reliable base.

---

*Planning document - V1 complete, V2 roadmap defined*
