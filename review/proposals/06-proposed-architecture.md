# 06 - Proposed Compiler Architecture

**Category**: Architecture Redesign  
**Goal**: A maintainable, extensible, and correct compiler design

---

## Summary

This document proposes a redesigned compiler architecture that addresses the flaws identified in this review. The new architecture follows established compiler construction principles with clear separation of concerns and proper abstraction layers.

---

## Design Principles

1. **Separation of Concerns**: Each phase has a single responsibility
2. **Testability**: Every component can be tested in isolation
3. **Extensibility**: New optimizations and backends can be added easily
4. **Correctness**: Type safety and validation at every layer
5. **Performance**: Enable optimizations without compromising maintainability

---

## Proposed Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         WASM Binary Input                                │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         PHASE 1: Parsing                                   │
│  - wasmparser (existing)                                                 │
│  - Validate WASM structure                                               │
│  - Extract types, functions, globals, memory, tables                       │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         PHASE 2: IR Generation                           │
│  - Translate WASM to Intermediate Representation                           │
│  - Build Control Flow Graph (CFG)                                        │
│  - Track types and value flow                                            │
│  - Convert stack machine to register-based IR                           │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         PHASE 3: Analysis & Optimization                  │
│  - Constant folding                                                       │
│  - Dead code elimination                                                  │
│  - Common subexpression elimination                                       │
│  - Register liveness analysis                                           │
│  - Loop invariant code motion (future)                                    │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         PHASE 4: Instruction Selection                    │
│  - Choose PVM instructions for IR operations                            │
│  - Abstract instruction encoding                                        │
│  - Handle calling conventions                                           │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         PHASE 5: Register Allocation                      │
│  - Graph coloring or linear scan allocator                              │
│  - Spill code insertion                                                 │
│  - Coalescing to reduce moves                                           │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         PHASE 6: Code Emission                            │
│  - Encode PVM instructions to bytes                                     │
│  - Generate jump tables                                                 │
│  - Build masks and metadata                                             │
│  - Handle fixups and relocations                                        │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         PHASE 7: SPI Generation                           │
│  - Package RO/RW data sections                                          │
│  - Set heap/stack configuration                                         │
│  - Final JAM output                                                     │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Module Structure

```
crates/wasm-pvm/src/
├── lib.rs                      # Public API
├── error.rs                    # Error types (unchanged)
├── wasm/                       # WASM parsing wrapper
│   ├── mod.rs
│   ├── parser.rs               # Thin wrapper around wasmparser
│   └── validate.rs             # WASM validation
├── ir/                         # Intermediate Representation
│   ├── mod.rs
│   ├── types.rs                # IR type system
│   ├── instructions.rs         # IR instruction enum
│   ├── values.rs               # Value representation (SSA)
│   ├── builder.rs              # IR construction
│   └── func.rs                 # Function IR
├── cfg/                        # Control Flow Graph
│   ├── mod.rs
│   ├── block.rs                # Basic block
│   ├── graph.rs                # CFG structure
│   ├── analysis.rs             # Dominance, loops
│   └── ssa.rs                  # SSA form construction
├── analysis/                   # Optimizations
│   ├── mod.rs
│   ├── constant_folding.rs
│   ├── dce.rs                  # Dead code elimination
│   ├── cse.rs                  # Common subexpression elimination
│   └── liveness.rs             # Liveness analysis
├── codegen/                    # Code Generation
│   ├── mod.rs
│   ├── selector.rs             # Instruction selection
│   ├── register/               # Register allocation
│   │   ├── mod.rs
│   │   ├── allocator.rs
│   │   ├── spiller.rs
│   │   └── interference.rs
│   ├── memory/                 # Memory layout
│   │   ├── mod.rs
│   │   └── layout.rs
│   ├── pvm/                    # PVM-specific
│   │   ├── mod.rs
│   │   ├── instructions.rs     # PVM instruction abstraction
│   │   ├── calling.rs          # Calling convention
│   │   └── emit.rs             # Code emission
│   └── optimize/               # Peephole optimizations
│       ├── mod.rs
│       └── peephole.rs
└── spi/                        # SPI format (unchanged)
    └── mod.rs
```

---

## Phase Details

### Phase 1: Parsing (WASM → Module)

**Responsibility**: Parse WASM binary and validate structure.

**Key Components**:
```rust
pub struct WasmModule {
    pub types: Vec<FuncType>,
    pub functions: Vec<Function>,
    pub globals: Vec<Global>,
    pub memory: Option<Memory>,
    pub tables: Vec<Table>,
    pub data_segments: Vec<DataSegment>,
    pub exports: Vec<Export>,
    pub start_func: Option<u32>,
}

pub fn parse_and_validate(wasm: &[u8]) -> Result<WasmModule> {
    let mut validator = Validator::new();
    let module = Parser::new(0).parse_all(wasm);
    // ... validation and extraction ...
}
```

**Benefits**:
- Centralized validation catches errors early
- Clean abstraction over wasmparser
- Type-safe representation

---

### Phase 2: IR Generation (WASM → IR)

**Responsibility**: Convert WASM stack-based code to register-based IR.

**IR Design**:
```rust
pub enum Value {
    Const(i64),                  // Constant value
    Register(VReg),              // Virtual register
    StackSlot(usize),            // Spill slot
    Global(u32),                 // Global variable
    Local(u32),                  // Local variable
}

pub enum Instruction {
    // Arithmetic
    Add32 { dst: VReg, lhs: Value, rhs: Value },
    Sub32 { dst: VReg, lhs: Value, rhs: Value },
    Mul32 { dst: VReg, lhs: Value, rhs: Value },
    // ... other ops ...
    
    // Memory
    Load32 { dst: VReg, addr: Value, offset: i32 },
    Store32 { addr: Value, value: Value, offset: i32 },
    
    // Control flow
    Branch { target: BlockId },
    BranchIf { cond: Value, then_target: BlockId, else_target: BlockId },
    Return { value: Option<Value> },
    
    // Function calls
    Call { func: u32, args: Vec<Value>, dst: Option<VReg> },
    CallIndirect { table_idx: Value, type_idx: u32, args: Vec<Value>, dst: Option<VReg> },
}
```

**Benefits**:
- Stack machine converted to registers (easier to optimize)
- Type information preserved
- Control flow explicit (not implicit in stack)

---

### Phase 3: Control Flow Graph

**Responsibility**: Build CFG and convert to SSA form.

```rust
pub struct BasicBlock {
    pub id: BlockId,
    pub instructions: Vec<Instruction>,
    pub terminator: Terminator,
    pub predecessors: Vec<BlockId>,
    pub successors: Vec<BlockId>,
}

pub struct ControlFlowGraph {
    pub blocks: Vec<BasicBlock>,
    pub entry: BlockId,
    pub exit: BlockId,
}

pub fn build_cfg(func: &FunctionIr) -> ControlFlowGraph {
    // Split into basic blocks at branch targets
    // Link predecessors and successors
}

pub fn to_ssa(cfg: &mut ControlFlowGraph) {
    // Insert phi nodes at join points
    // Rename values to satisfy SSA property
}
```

**Benefits**:
- Enables dataflow analysis
- Makes optimizations easier
- Standard compiler technique

---

### Phase 4: Analysis & Optimization

**Responsibility**: Transform IR for better performance.

**Constant Folding**:
```rust
fn fold_constants(instr: &mut Instruction) {
    match instr {
        Instruction::Add32 { dst, lhs: Value::Const(a), rhs: Value::Const(b) } => {
            *instr = Instruction::Move { 
                dst: *dst, 
                src: Value::Const(a.wrapping_add(*b)) 
            };
        }
        // ... other patterns ...
    }
}
```

**Dead Code Elimination**:
```rust
fn eliminate_dead_code(func: &mut FunctionIr) {
    // Remove instructions whose results are unused
    // Work backwards from returns
}
```

**Benefits**:
- Smaller, faster code
- Done at IR level (backend-agnostic)
- Can add more optimizations later

---

### Phase 5: Instruction Selection

**Responsibility**: Choose target instructions for IR operations.

```rust
trait InstructionSelector {
    type Instruction;
    
    fn select_load32(&mut self, dst: VReg, addr: Value, offset: i32) -> Vec<Self::Instruction>;
    fn select_add32(&mut self, dst: VReg, lhs: Value, rhs: Value) -> Vec<Self::Instruction>;
    // ... etc ...
}

struct PvmSelector;

impl InstructionSelector for PvmSelector {
    type Instruction = PvmInstruction;
    
    fn select_add32(&mut self, dst: VReg, lhs: Value, rhs: Value) -> Vec<PvmInstruction> {
        match (lhs, rhs) {
            (Value::Const(c), _) if c == 0 => vec![PvmInstruction::Move(dst, rhs)],
            (_, Value::Const(c)) => vec![PvmInstruction::AddImm32(dst, lhs, c)],
            _ => vec![PvmInstruction::Add32(dst, lhs, rhs)],
        }
    }
}
```

**Benefits**:
- Abstract over target ISA
- Can retarget to different VMs
- Optimal instruction selection

---

### Phase 6: Register Allocation

**Responsibility**: Assign physical registers to virtual registers.

```rust
pub struct RegisterAllocator {
    physical_regs: Vec<PReg>,
    interference_graph: InterferenceGraph,
}

impl RegisterAllocator {
    pub fn allocate(&mut self, func: &mut FunctionIr) -> AllocationResult {
        // Build interference graph
        self.build_interference_graph(func);
        
        // Color the graph
        let coloring = self.color_graph();
        
        // Assign registers based on coloring
        for (vreg, preg) in coloring {
            func.assign_register(vreg, preg);
        }
        
        // Spill uncolorable registers
        let spills = self.insert_spill_code(func);
        
        AllocationResult { spills }
    }
}
```

**Benefits**:
- Optimal register usage
- Minimal spill code
- Can use different algorithms (graph coloring, linear scan)

---

### Phase 7: Code Emission

**Responsibility**: Generate final PVM bytecode.

```rust
pub fn emit_function(func: &FunctionIr, layout: &MemoryLayout) -> Vec<u8> {
    let mut emitter = PvmEmitter::new();
    
    for block in func.blocks() {
        emitter.define_label(block.id);
        
        for instr in block.instructions() {
            match instr {
                Instruction::Add32 { dst, lhs, rhs } => {
                    let dst_reg = allocated_reg(*dst);
                    let lhs_reg = allocated_reg_or_const(*lhs);
                    let rhs_reg = allocated_reg_or_const(*rhs);
                    emitter.emit(Instruction::Add32 { dst: dst_reg, src1: lhs_reg, src2: rhs_reg });
                }
                // ... etc ...
            }
        }
        
        emit_terminator(&mut emitter, block.terminator());
    }
    
    emitter.resolve_fixups();
    emitter.encode()
}
```

**Benefits**:
- Clean separation from IR
- Handles fixups systematically
- Generates correct PVM code

---

## Key Abstractions

### Virtual Registers

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VReg(pub u32);

impl VReg {
    pub fn new(id: u32) -> Self { VReg(id) }
}
```

- Unlimited virtual registers (allocated during lowering)
- Physical registers (PRegs) are target-specific
- Mapping happens in register allocation phase

### Memory Layout Abstraction

```rust
pub struct MemoryLayout {
    globals: MemoryRegion,
    spilled_locals: MemoryRegion,
    wasm_memory: MemoryRegion,
    ro_data: MemoryRegion,
    stack: MemoryRegion,
}

impl MemoryLayout {
    pub fn compute(module: &WasmModule) -> Result<Self> {
        // Calculate regions based on module needs
        // Ensure no overlaps
        // Validate alignment
    }
    
    pub fn global_addr(&self, idx: u32) -> u32 {
        self.globals.start + idx * 4
    }
    
    pub fn spilled_local_addr(&self, func: u32, local: u32) -> u32 {
        self.spilled_locals.start + func * self.spilled_locals.stride + local * 8
    }
}
```

### Error Handling

```rust
pub enum CompileError {
    // Parsing errors
    InvalidWasm(String),
    UnsupportedFeature(&'static str),
    
    // Validation errors
    TypeMismatch { expected: IrType, found: IrType },
    InvalidLocalIndex { func: u32, index: u32 },
    
    // Codegen errors
    OutOfRegisters,
    SpillFailed,
    FixupFailed,
    
    // Internal errors
    Internal(String),
}

pub type Result<T> = std::result::Result<T, CompileError>;
```

---

## Testing Strategy

Each phase can be tested independently:

1. **IR Tests**: Verify WASM → IR conversion
2. **Optimization Tests**: Check transformations are correct
3. **Register Allocation Tests**: Ensure no interference violations
4. **Integration Tests**: End-to-end compilation

See [07-testing-strategy.md](./07-testing-strategy.md) for details.

---

## Migration Path

The new architecture can be adopted incrementally:

1. **Phase 1**: Keep current codegen, add IR layer underneath
2. **Phase 2**: Move operators to IR one by one
3. **Phase 3**: Add optimizations on IR
4. **Phase 4**: Replace register allocator
5. **Phase 5**: Clean up old code

See [08-rebuilding-plan.md](./08-rebuilding-plan.md) for detailed migration steps.

---

## Benefits Summary

| Aspect | Current | Proposed |
|--------|---------|----------|
| Lines per module | 2,400 | < 500 |
| Test coverage | 30% | 80%+ |
| Optimizations | None | Constant folding, DCE, CSE |
| Register allocation | Ad-hoc | Graph coloring |
| Extensibility | Poor | Excellent |
| Debugging | Hard (no IR) | Easy (inspect IR) |
| Retargetability | None | Possible |
| Correctness | Bug-prone | Validated at each phase |

---

*Next: [07-testing-strategy.md](./07-testing-strategy.md)*
