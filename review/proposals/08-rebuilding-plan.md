# 08 - Incremental Rebuilding Plan

**Category**: Implementation  
**Goal**: Migrate from current architecture to improved design without breaking existing functionality

---

## Summary

This document provides a step-by-step plan to rebuild the compiler incrementally. The key constraint is to **maintain backward compatibility** - all 62 existing tests must continue passing throughout the migration.

---

## Migration Strategy: Strangler Fig Pattern

We'll use the Strangler Fig pattern: gradually replace parts of the old system while keeping it functional.

```
Phase 1: Add IR alongside existing code
         ↓
Phase 2: Migrate operators one by one
         ↓
Phase 3: Enable IR-based optimizations
         ↓
Phase 4: Replace register allocator
         ↓
Phase 5: Remove old code
```

---

## Phase 1: Foundation (Weeks 1-2)

### Goal
Create the new architecture alongside the existing code without changing behavior.

### Tasks

1. **Create new module structure**
   ```bash
   mkdir -p crates/wasm-pvm/src/{ir,cfg,analysis,codegen/{register,memory,pvm}}
   ```

2. **Implement IR types** (`ir/types.rs`)
   ```rust
   pub enum IrType {
       I32,
       I64,
       Void,
   }
   
   pub enum Value {
       Const(i64),
       VirtualReg(VReg),
       // ...
   }
   ```

3. **Add IR builder** (`ir/builder.rs`)
   ```rust
   pub struct IrBuilder {
       current_block: BlockId,
       instructions: Vec<IrInstruction>,
   }
   ```

4. **Create feature flag**
   ```rust
   // In lib.rs
   #[cfg(feature = "new-codegen")]
   pub mod new_codegen;
   ```

### Success Criteria
- New modules compile
- Existing tests still pass
- IR types are defined and tested

---

## Phase 2: Parallel Implementation (Weeks 3-6)

### Goal
Build the new code generation pipeline without using it yet.

### Tasks

1. **Implement WASM → IR translator**
   ```rust
   // translate/wasm_to_ir.rs
   pub fn translate_to_ir(wasm: &WasmModule) -> Result<IrModule> {
       // Translate WASM functions to IR
       // Similar to current translate_op but building IR instead of PVM
   }
   ```

2. **Implement IR → PVM instruction selector**
   ```rust
   // codegen/pvm/selector.rs
   pub fn select_instructions(ir: &IrFunction) -> Vec<PvmInstruction> {
       // Convert IR to PVM instructions
   }
   ```

3. **Add simple register allocator**
   ```rust
   // codegen/register/allocator.rs
   pub fn allocate_registers(instrs: &[PvmInstruction]) -> Allocation {
       // Linear scan or simple greedy allocation
   }
   ```

4. **Create test harness for new pipeline**
   ```rust
   #[cfg(test)]
   mod tests {
       #[test]
       fn test_new_pipeline_produces_valid_output() {
           let wasm = load_test_wasm();
           let old_result = old_compile(&wasm).unwrap();
           let new_result = new_compile(&wasm).unwrap();
           
           // Both should produce valid programs
           assert!(is_valid(&old_result));
           assert!(is_valid(&new_result));
       }
   }
   ```

### Success Criteria
- New pipeline can compile simple programs
- Output is syntactically valid
- All existing tests still pass

---

## Phase 3: Operator Migration (Weeks 7-14)

### Goal
Move operator translation from old code to new IR-based system, one operator at a time.

### Strategy

For each operator (e.g., `I32Add`):

1. **Add to IR** (`ir/instructions.rs`)
   ```rust
   pub enum IrInstruction {
       Add32 { dst: VReg, lhs: Value, rhs: Value },
       // ...
   }
   ```

2. **Add WASM → IR translation**
   ```rust
   // In wasm_to_ir.rs
   Operator::I32Add => {
       let rhs = self.pop_value();
       let lhs = self.pop_value();
       let dst = self.new_vreg();
       self.emit(IrInstruction::Add32 { dst, lhs, rhs });
       self.push_value(Value::VirtualReg(dst));
   }
   ```

3. **Add IR → PVM selection**
   ```rust
   // In selector.rs
   IrInstruction::Add32 { dst, lhs, rhs } => {
       vec![Instruction::Add32 { 
           dst: map_reg(dst), 
           src1: map_value(lhs), 
           src2: map_value(rhs) 
       }]
   }
   ```

4. **Add test and verify**
   ```rust
   #[test]
   fn test_i32_add_new_pipeline() {
       let wasm = wat!("(module (func (result i32) i32.const 5 i32.const 3 i32.add))");
       
       let old = old_compile(&wasm).unwrap();
       let new = new_compile(&wasm).unwrap();
       
       // Execute both and compare
       assert_eq!(execute(&old), execute(&new));
   }
   ```

5. **Use feature flag to enable**
   ```rust
   #[cfg(feature = "new-i32-add")]
   Operator::I32Add => self.translate_i32_add_new(),
   #[cfg(not(feature = "new-i32-add"))]
   Operator::I32Add => self.translate_i32_add_old(),
   ```

### Migration Order (by priority)

1. **Arithmetic** (Week 7): i32.add, sub, mul - simple, well-tested
2. **Constants** (Week 8): i32.const, i64.const - foundation for others
3. **Local access** (Week 9): local.get, local.set - needed for most programs
4. **Comparisons** (Week 10): i32.eq, ne, lt, gt, etc.
5. **Memory** (Week 11): i32.load, store - complex addressing
6. **Control flow** (Week 12): block, if, loop, br - most complex
7. **Calls** (Week 13): call, call_indirect - requires stack management
8. **Remaining operators** (Week 14): bitwise, shifts, conversions

### Success Criteria
- All operators have IR representations
- New pipeline produces identical output for all test cases
- Feature flags allow gradual rollout

---

## Phase 4: Optimization Integration (Weeks 15-18)

### Goal
Add optimizations to the new pipeline.

### Tasks

1. **Add constant folding**
   ```rust
   // analysis/constant_folding.rs
   pub fn fold_constants(func: &mut IrFunction) {
       for instr in &mut func.instructions {
           match instr {
               IrInstruction::Add32 { dst, lhs: Value::Const(a), rhs: Value::Const(b) } => {
                   *instr = IrInstruction::Move { 
                       dst: *dst, 
                       src: Value::Const(a.wrapping_add(*b)) 
                   };
               }
               // ...
           }
       }
   }
   ```

2. **Add dead code elimination**
   ```rust
   // analysis/dce.rs
   pub fn eliminate_dead_code(func: &mut IrFunction) {
       // Remove unused instructions
   }
   ```

3. **Add basic register coalescing**
   ```rust
   // codegen/register/coalescer.rs
   pub fn coalesce_moves(func: &mut IrFunction) {
       // Remove redundant register-to-register moves
   }
   ```

4. **Benchmark and verify**
   - Compare code size before/after optimizations
   - Verify correctness is maintained
   - Document performance improvements

### Success Criteria
- Optimizations reduce code size by 10%+ on average
- All tests still pass
- No correctness regressions

---

## Phase 5: New Register Allocator (Weeks 19-22)

### Goal
Replace ad-hoc register allocation with proper graph coloring.

### Tasks

1. **Build interference graph**
   ```rust
   // codegen/register/interference.rs
   pub struct InterferenceGraph {
       nodes: Vec<VReg>,
       edges: HashSet<(VReg, VReg)>,
   }
   
   impl InterferenceGraph {
       pub fn build(func: &IrFunction) -> Self {
           // Compute liveness and build graph
       }
   }
   ```

2. **Implement graph coloring**
   ```rust
   // codegen/register/allocator.rs
   pub fn color_graph(graph: &InterferenceGraph, num_regs: usize) -> Coloring {
       // Chaitin-style graph coloring
   }
   ```

3. **Insert spill code**
   ```rust
   pub fn insert_spills(func: &mut IrFunction, spills: &[Spill]) {
       // Add load/store for spilled values
   }
   ```

4. **A/B test against old allocator**
   ```rust
   #[test]
   fn test_new_allocator_reduces_spills() {
       let wasm = load_complex_wasm();
       
       let old = compile_with_old_allocator(&wasm);
       let new = compile_with_new_allocator(&wasm);
       
       let old_spills = count_spill_instructions(&old);
       let new_spills = count_spill_instructions(&new);
       
       assert!(new_spills <= old_spills, 
           "New allocator produced more spills: {} vs {}", new_spills, old_spills);
   }
   ```

### Success Criteria
- New allocator produces equal or fewer spills
- Compilation time acceptable (< 2x slower)
- All tests pass

---

## Phase 6: Cutover (Weeks 23-24)

### Goal
Make the new pipeline the default.

### Tasks

1. **Flip the default**
   ```rust
   // In lib.rs
   #[cfg(not(feature = "legacy-codegen"))]
   pub use new_codegen::compile;
   
   #[cfg(feature = "legacy-codegen")]
   pub use translate::compile;
   ```

2. **Update CI to test both paths**
   ```yaml
   - name: Test new codegen
     run: cargo test
     
   - name: Test legacy codegen
     run: cargo test --features legacy-codegen
   ```

3. **Monitor for issues**
   - Run full test suite
   - Compare performance
   - Watch for bug reports

4. **Document the change**
   - Update AGENTS.md
   - Add migration notes
   - Document new architecture

### Success Criteria
- New pipeline is default
- Legacy still available via feature flag
- No regressions in test suite

---

## Phase 7: Cleanup (Weeks 25-26)

### Goal
Remove old code once new pipeline is stable.

### Tasks

1. **Remove legacy codegen**
   - Delete old `translate/codegen.rs`
   - Delete old `translate/stack.rs`
   - Remove feature flags

2. **Rename new modules to standard names**
   ```bash
   mv new_codegen codegen
   mv new_translate translate
   ```

3. **Final cleanup**
   - Remove unused imports
   - Update documentation
   - Archive old design docs

4. **Celebrate** 🎉

### Success Criteria
- Old code removed
- All tests pass
- Codebase is cleaner and more maintainable

---

## Risk Mitigation

### Risk 1: Migration Takes Too Long

**Mitigation**:
- Can stop at any phase and still have functional compiler
- Each phase delivers value (better tests, cleaner code)
- Prioritize critical operators

### Risk 2: New Code Has Bugs

**Mitigation**:
- Extensive testing at each phase
- A/B testing against old implementation
- Feature flags allow quick rollback
- Keep old code until new is proven

### Risk 3: Performance Regression

**Mitigation**:
- Benchmark at each phase
- Don't enable optimizations if they slow things down
- Keep old allocator if new one isn't better

### Risk 4: Test Coverage Gaps

**Mitigation**:
- Add tests BEFORE migrating each operator
- Use property-based testing to find edge cases
- Fuzz test continuously

---

## Timeline Summary

| Phase | Weeks | Deliverable |
|-------|-------|-------------|
| 1. Foundation | 1-2 | New module structure, IR types |
| 2. Parallel Impl | 3-6 | Working new pipeline (feature-flagged) |
| 3. Migration | 7-14 | All operators on new pipeline |
| 4. Optimization | 15-18 | Constant folding, DCE |
| 5. Register Alloc | 19-22 | Graph coloring allocator |
| 6. Cutover | 23-24 | New pipeline is default |
| 7. Cleanup | 25-26 | Remove old code |
| **Total** | **26 weeks** | **~6 months** |

---

## Success Metrics

| Metric | Before | After | Target |
|--------|--------|-------|--------|
| Lines per file (max) | 2,400 | < 500 | ✅ |
| Test coverage | ~30% | 85%+ | ✅ |
| Number of bugs | 3+ open | 0 critical | ✅ |
| Compilation time | Baseline | < 1.5x | ✅ |
| Code size (generated) | Baseline | -10% | ✅ |
| Time to add new operator | Days | Hours | ✅ |

---

## Conclusion

This migration plan allows the compiler to be rebuilt incrementally while maintaining full backward compatibility. The Strangler Fig pattern ensures we can:

1. **Deliver value incrementally** - Each phase improves the codebase
2. **Manage risk** - Can stop or roll back at any point
3. **Maintain quality** - Tests ensure correctness throughout
4. **Learn and adapt** - Adjust plan based on what we learn

The result will be a compiler that is:
- Easier to understand and modify
- Better tested and more correct
- Able to support optimizations
- Ready for future enhancements

---

*End of review documents*
