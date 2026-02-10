# 01 - Architectural Design Flaws

**Category**: Critical Design Issues  
**Impact**: Prevents optimizations, increases bug surface, limits extensibility

---

## Summary

The current architecture is a **naive direct translator** that converts WASM bytecode directly to PVM machine code without any intermediate layers. While this approach works for simple programs, it creates fundamental barriers to correctness, optimization, and maintainability.

---

## Flaw 1: No Intermediate Representation (IR)

**Severity**: 🔴 Critical  
**Location**: Entire codebase  
**Impact**: Blocks all optimizations, makes debugging impossible

### Problem Description

The compiler performs a direct translation: `WASM → PVM bytes`. There is no IR layer between source and target.

```
Current Flow:
WASM Parser → (WASM ops) → translate_op() → PVM Instructions → Encoding → JAM file
```

### Why This Is a Problem

1. **No Optimization Possible**: You cannot do constant folding, dead code elimination, or strength reduction without an IR
2. **No Verification**: No way to verify semantic correctness before code generation
3. **Debugging Nightmare**: Cannot inspect intermediate state
4. **Cannot Support Future Features**: Adding SIMD, threads, or other proposals requires rewriting everything

### What a Proper Compiler Does

```
Proper Flow:
WASM Parser → (WASM ops) → IR (SSA form) → Optimizations → PVM Instructions → Encoding
```

### Evidence from Codebase

The `translate_op()` function in `codegen.rs` directly emits PVM instructions:

```rust
// From codegen.rs:1440-1444
Operator::I32Add => {
    let src2 = emitter.spill_pop();
    let src1 = emitter.spill_pop();
    let dst = emitter.spill_push();
    emitter.emit(Instruction::Add32 { dst, src1, src2 });
}
```

There's no place to insert an IR or perform analysis.

### Recommended Solution

Introduce a simple IR (e.g., three-address code or SSA) between WASM parsing and code generation.

---

## Flaw 2: Monolithic Translation Module

**Severity**: 🔴 Critical  
**Location**: `translate/codegen.rs` (2,400+ lines)  
**Impact**: Unmaintainable, untestable, error-prone

### Problem Description

The `codegen.rs` file is a single massive module with:
- 600+ line `translate_op()` function with a match statement for 100+ operators
- Ad-hoc control flow handling scattered throughout
- Manual register allocation mixed with instruction selection
- Label management mixed with code emission

### Code Statistics

| Metric | Value | Industry Standard |
|--------|-------|-------------------|
| Lines per file | 2,400 | < 500 |
| Lines per function | 600+ | < 50 |
| Match arms in translate_op | 100+ | < 20 |
| Responsibilities per file | 5+ | 1 |

### Violations of Single Responsibility Principle

The file handles:
1. WASM operator dispatch
2. Register allocation
3. Stack spilling management
4. Control flow translation
5. Function prologue/epilogue generation
6. Memory address calculations

### Evidence

```rust
// clippy warning suppressed at file level
#![allow(clippy::too_many_lines)]

// translate_op is a massive match statement
fn translate_op(op: &Operator, emitter: &mut CodeEmitter, ctx: &CompileContext, total_locals: usize) 
    -> Result<()> {
    match op {
        Operator::LocalGet { local_index } => { /* 20 lines */ }
        Operator::LocalSet { local_index } => { /* 25 lines */ }
        // ... 100+ more arms
    }
}
```

### Recommended Solution

Split into focused modules:
```
translate/
  mod.rs           - Orchestration
  parser.rs        - WASM parsing wrapper
  ir.rs            - Intermediate representation
  instruction.rs   - Instruction selection
  register.rs      - Register allocation
  control_flow.rs  - Control flow graph and branches
  calling.rs       - Calling convention handling
  memory.rs        - Memory model management
```

---

## Flaw 3: Ad-Hoc Register Allocation

**Severity**: 🔴 High  
**Location**: `translate/codegen.rs`, `translate/stack.rs`  
**Impact**: Suboptimal code, high bug risk, prevents optimizations

### Problem Description

Register allocation is hardcoded with magic numbers scattered throughout the codebase. There's no concept of register liveness, allocation pressure, or spilling heuristics.

### Current Register Convention (Hardcoded)

| Register | Purpose | Hardcoded In |
|----------|---------|--------------|
| r0 | Return address (jump table) | `RETURN_ADDR_REG = 0` |
| r1 | Stack pointer / Return value | `STACK_PTR_REG = 1`, `RETURN_VALUE_REG = 1` |
| r2-r6 | Operand stack (5 slots) | `FIRST_STACK_REG = 2`, `LAST_STACK_REG = 6` |
| r7 | SPI args pointer / spill temp | `ARGS_PTR_REG = 7`, `SPILL_TEMP_REG = 7` |
| r8 | SPI args length / saved table idx | `ARGS_LEN_REG = 8`, `SAVED_TABLE_IDX_REG = 8` |
| r9-r12 | Local variables (first 4) | `FIRST_LOCAL_REG = 9` |

### Problems with This Approach

1. **Cannot Change Register Usage**: Every register number is hardcoded in multiple places
2. **No Liveness Analysis**: Spills happen based on stack depth, not actual register pressure
3. **Fixed Local Limit**: Only 4 locals in registers (r9-r12), rest spilled to memory
4. **No Interference Tracking**: No knowledge of which values interfere
5. **No Coalescing**: Cannot eliminate redundant moves

### Evidence of Fragility

```rust
// From codegen.rs - stack overflow check clobbers r7
emitter.emit(Instruction::LoadImm64 {
    reg: ARGS_PTR_REG,  // r7
    value: u64::from(limit as u32),
});

// Later in same function, r7 is also used for SPI args
// This requires careful ordering to avoid clobbering
```

### Recommended Solution

Implement a proper register allocator:

1. **Graph Coloring Allocator** (for production)
   - Build interference graph
   - Color with PVM's 13 registers
   - Spill when necessary

2. **Linear Scan Allocator** (for simplicity)
   - Track live ranges
   - Allocate registers greedily
   - Spill at interval intersections

---

## Flaw 4: Manual Control Flow Management

**Severity**: 🔴 High  
**Location**: `translate/codegen.rs` (ControlFrame, labels, fixups)  
**Impact**: Error-prone, no optimization possible, hard to debug

### Problem Description

Control flow (blocks, loops, if/else, branches) is handled through manual label allocation and fixup resolution. There's no Control Flow Graph (CFG).

### Current Mechanism

```rust
// From codegen.rs
#[derive(Debug, Clone, Copy)]
enum ControlFrame {
    Block { end_label: usize, stack_depth: usize, has_result: bool },
    Loop { start_label: usize, stack_depth: usize },
    If { else_label: usize, end_label: usize, stack_depth: usize, has_result: bool },
}

struct CodeEmitter {
    labels: Vec<Option<usize>>,        // Label ID → offset
    fixups: Vec<(usize, usize)>,      // (instr_idx, label_id)
    control_stack: Vec<ControlFrame>,  // WASM control stack
}
```

### Problems

1. **No CFG**: Cannot analyze basic blocks, dominance, or loops
2. **Manual Fixups**: Label resolution is error-prone
3. **No SSA**: Cannot track value flow across branches
4. **Ad-Hoc Stack Tracking**: Stack depth tracked manually per frame

### Evidence of Complexity

```rust
// From codegen.rs:2041-2077
Operator::End => match emitter.pop_control() {
    Some(ControlFrame::Block { end_label, stack_depth, has_result }) => {
        emitter.emit(Instruction::Fallthrough);
        emitter.define_label(end_label);
        let target_depth = if has_result { stack_depth + 1 } else { stack_depth };
        emitter.stack.set_depth(target_depth);
        emitter.pending_spill = None;
        emitter.last_spill_pop_reg = None;
    }
    // ... more arms ...
}
```

This manually manages:
- Label definition
- Stack depth restoration
- Spill state clearing
- Different logic for blocks vs if vs loops

### Recommended Solution

Build a Control Flow Graph:

```rust
struct BasicBlock {
    id: BlockId,
    instructions: Vec<IrInstruction>,
    successors: Vec<BlockId>,
    predecessors: Vec<BlockId>,
}

struct ControlFlowGraph {
    blocks: Vec<BasicBlock>,
    entry: BlockId,
    exit: BlockId,
}
```

Then use standard algorithms for:
- Dominance analysis
- Loop detection
- SSA form construction

---

## Flaw 5: Hardcoded Memory Layout

**Severity**: 🟡 Medium  
**Location**: `translate/codegen.rs`, `translate/mod.rs`  
**Impact**: Brittle, hard to change, easy to break

### Problem Description

Memory addresses are scattered as magic constants throughout the codebase. There's no abstraction for memory regions.

### Magic Constants Found

```rust
// From codegen.rs
const GLOBAL_MEMORY_BASE: i32 = 0x30000;           // Globals at 0x30000
const SPILLED_LOCALS_BASE: i32 = 0x40000;          // Spilled locals at 0x40000
const EXIT_ADDRESS: i32 = -65536;                  // 0xFFFF0000
const RO_DATA_BASE: i32 = 0x10000;                 // Dispatch table
const PARAM_OVERFLOW_BASE: i32 = 0x3FF00;           // Parameter overflow area
const STACK_SEGMENT_END: i32 = 0xFEFE_0000;        // Stack end

// Computed addresses
fn compute_wasm_memory_base(num_local_funcs: usize) -> i32 {
    let spilled_locals_end = SPILLED_LOCALS_BASE + (num_local_funcs as i32) * SPILLED_LOCALS_PER_FUNC;
    let aligned = (spilled_locals_end + 0xFFFF) & !0xFFFF;
    aligned.max(0x50000)
}
```

### Problems

1. **Scattered Definitions**: Addresses defined in multiple places
2. **No Validation**: No checks for region overlaps
3. **Hard to Modify**: Changing one address requires changes everywhere
4. **No Documentation**: Comments are the only documentation
5. **Computed Addresses**: Some addresses computed at runtime, making static analysis impossible

### Evidence of Fragility

```rust
// From mod.rs - RW data layout calculation
let wasm_to_rw_offset = wasm_memory_base as u32 - 0x30000;

// Hardcoded assumption that RW data starts at 0x30000
// What if we want to move globals?
```

### Recommended Solution

Create a memory layout abstraction:

```rust
struct MemoryLayout {
    globals: MemoryRegion,
    spilled_locals: MemoryRegion,
    wasm_linear_memory: MemoryRegion,
    ro_data: MemoryRegion,
    stack: MemoryRegion,
}

struct MemoryRegion {
    start: u32,
    size: u32,
}

impl MemoryLayout {
    fn globals_address(&self, idx: u32) -> u32 {
        self.globals.start + idx * 4
    }
    
    fn spilled_local_address(&self, func_idx: usize, local_idx: usize) -> u32 {
        // ... calculation using region bounds
    }
}
```

---

## Flaw 6: No Separation of Concerns

**Severity**: 🔴 High  
**Location**: All translation code  
**Impact**: Untestable, unmaintainable, hard to reason about

### Problem Description

The compiler conflates multiple distinct phases:

1. **Parsing**: WASM binary → WASM operators
2. **Analysis**: Understanding what the code does
3. **Translation**: WASM operators → PVM instructions
4. **Instruction Selection**: Choosing which PVM instructions to use
5. **Register Allocation**: Assigning registers to values
6. **Code Emission**: Writing bytes to output

In the current codebase, these are all mixed together.

### Example of Mixed Concerns

```rust
// From codegen.rs:1440-1444
Operator::I32Add => {
    // Translation: WASM I32Add → PVM Add32
    let src2 = emitter.spill_pop();  // Register allocation + stack management
    let src1 = emitter.spill_pop();  // Register allocation + stack management
    let dst = emitter.spill_push();  // Register allocation + stack management
    emitter.emit(Instruction::Add32 { dst, src1, src2 });  // Code emission
}
```

This single block handles:
- Understanding WASM operand order (src2, src1)
- Register allocation (spill_pop/push)
- Stack depth management
- Instruction selection (Add32)
- Code emission (emit)

### Recommended Solution

Separate into distinct phases with clean interfaces:

```rust
// Phase 1: Parse WASM (external crate: wasmparser)

// Phase 2: Build IR
let ir = build_ir(wasm_module);

// Phase 3: Analyze and optimize
let optimized = optimize(ir);

// Phase 4: Instruction selection
let pvm_ops = select_instructions(optimized);

// Phase 5: Register allocation
let allocated = allocate_registers(pvm_ops);

// Phase 6: Code emission
let bytes = emit(allocated);
```

---

## Flaw 7: Fragile Operand Stack Spilling

**Severity**: 🔴 High  
**Location**: `translate/stack.rs`, `translate/codegen.rs`  
**Impact**: Subtle bugs, hard to debug, inefficient

### Problem Description

The operand stack can hold 5 values in registers (r2-r6). Beyond that, values spill to memory. This spilling is managed manually and is prone to errors.

### Current Implementation

```rust
// From stack.rs
const STACK_REG_COUNT: usize = 5;  // r2-r6

pub const fn needs_spill(depth: usize) -> bool {
    depth >= STACK_REG_COUNT
}

pub const fn spill_offset(depth: usize) -> i32 {
    let spill_index = depth - STACK_REG_COUNT;
    (spill_index as i32) * 8
}
```

### Problems

1. **Fixed Register Count**: Cannot adapt to different register availability
2. **Depth-Based**: Spills based on depth, not liveness
3. **Manual Management**: `pending_spill`, `last_spill_pop_reg` track state manually
4. **No Optimization**: Cannot coalesce spills or remove redundant ones

### Evidence of Complexity

```rust
// From codegen.rs:191-231
fn spill_pop(&mut self) -> u8 {
    self.flush_pending_spill();
    let depth = self.stack.depth();
    if depth > 0 && StackMachine::needs_spill(depth - 1) {
        let offset = OPERAND_SPILL_BASE + StackMachine::spill_offset(depth - 1);
        let default_reg = StackMachine::reg_at_depth(depth - 1);
        let dst = if self.last_spill_pop_reg == Some(default_reg) {
            SPILL_ALT_REG  // Use alternate register
        } else {
            default_reg
        };
        self.emit(Instruction::LoadIndU64 { dst, base: STACK_PTR_REG, offset });
        self.last_spill_pop_reg = Some(dst);
        self.stack.pop();
        return dst;
    }
    self.last_spill_pop_reg = None;
    self.stack.pop()
}
```

This complexity is needed because:
- Need to track if value is in register or memory
- Need to handle case where default register was just used
- Need to track last spilled register to avoid conflicts

### Recommended Solution

Use a proper spilling algorithm:

1. **Live Range Analysis**: Determine when each value is live
2. **Spill Heuristics**: Spill values with longest remaining lifetime
3. **Splitting**: Split live ranges instead of spilling entire values
4. **Rematerialization**: Recompute values instead of reloading

---

## Flaw 8: Two-Pass Compilation is Ad-Hoc

**Severity**: 🟡 Medium  
**Location**: `translate/mod.rs`  
**Impact**: Complex, error-prone, limits extensibility

### Problem Description

The compiler uses a two-pass approach:
1. First pass: Generate code with placeholder offsets
2. Second pass: Resolve fixups with actual offsets

However, this is implemented manually with fixup lists rather than a proper linker.

### Current Mechanism

```rust
// From mod.rs
fn resolve_call_fixups(
    instructions: &mut [Instruction],
    call_fixups: &[(usize, codegen::CallFixup)],
    indirect_call_fixups: &[(usize, codegen::IndirectCallFixup)],
    function_offsets: &[usize],
) -> Result<(Vec<u32>, usize)> {
    // Manual fixup resolution
    for (instr_base, fixup) in call_fixups {
        let target_offset = function_offsets.get(fixup.target_func as usize)
            .ok_or_else(|| Error::Unsupported(...))?;
        
        // Calculate relative offset
        let jump_start_offset: usize = instructions[..jump_idx]
            .iter().map(|i| i.encode().len()).sum();
        let relative_offset = (*target_offset as i32) - (jump_start_offset as i32);
        
        // Patch the instruction
        if let Instruction::Jump { offset } = &mut instructions[jump_idx] {
            *offset = relative_offset;
        }
    }
}
```

### Problems

1. **Manual Offset Calculation**: Must compute byte offsets manually
2. **Instruction Size Dependency**: Must know encoded size of every instruction
3. **No Linker**: No separate linking phase
4. **Limited**: Cannot do more complex relocations

### Recommended Solution

Implement a proper linker or use a simpler single-pass approach with symbolic labels:

```rust
// Alternative: Symbolic assembly with separate assembler
enum AsmInstruction {
    Add32 { dst: Reg, src1: Reg, src2: Reg },
    Jump { target: Label },
    BranchEqImm { reg: Reg, value: i32, target: Label },
}

struct Assembler {
    instructions: Vec<AsmInstruction>,
    labels: HashMap<Label, usize>,
}

impl Assembler {
    fn assemble(&self) -> Vec<u8> {
        // Two-phase assembly:
        // 1. Assign offsets to labels
        // 2. Generate code with resolved labels
    }
}
```

---

## Summary Table

| Flaw | Severity | Effort to Fix | Priority |
|------|----------|---------------|----------|
| No IR | 🔴 Critical | High | 1 |
| Monolithic module | 🔴 Critical | Medium | 2 |
| Ad-hoc register allocation | 🔴 High | High | 3 |
| Manual control flow | 🔴 High | High | 4 |
| Hardcoded memory layout | 🟡 Medium | Low | 5 |
| No separation of concerns | 🔴 High | Medium | 6 |
| Fragile spilling | 🔴 High | High | 7 |
| Ad-hoc two-pass | 🟡 Medium | Low | 8 |

---

## Next Steps

See [06-proposed-architecture.md](./06-proposed-architecture.md) for a redesigned architecture that addresses these flaws.
