# Future Optimization Ideas

## High Impact

### 1. Register Allocation (The "Holy Grail")
**Impact:** ~50-70% code size reduction
**Difficulty:** Very High
**Description:**
Currently, every SSA value gets a dedicated stack slot. This means almost every instruction involves `LoadInd` + Op + `StoreInd`.
Implementing a register allocator (Linear Scan or Graph Coloring) would allow keeping hot values in registers (`r2`-`r6`, `r9`-`r12`), eliminating most memory traffic.
*   **Phase 1**: Local register allocation (within a basic block).
*   **Phase 2**: Global register allocation (across blocks).

### 2. Shared Runtime for Bulk Operations
**Impact:** ~10-20% for memory-heavy modules
**Difficulty:** Medium
**Description:**
`memory.copy` and `memory.fill` are currently expanded inline, consuming ~100-150 bytes per call.
*   **Idea**: Inject a small, hand-optimized PVM assembly "runtime" function for `memcpy`/`memset` at the beginning of the code section.
*   Replace inline expansion with a `Call` to this runtime function.
*   Only inject if the module actually uses these instructions.

## Medium Impact

### 3. Reserved Global Base Register
**Impact:** 5-10%
**Difficulty:** Medium
**Description:**
Almost every load/store operation loads `wasm_memory_base` (or `GLOBAL_MEMORY_BASE`) into a temporary register.
*   **Idea**: Reserve a register (e.g., `r6`) to permanently hold `wasm_memory_base`.
*   Pass this value in `r6` from the entry point (or initialize it once in prologue).
*   Eliminates one `LoadImm64` (10 bytes) per memory access.

### 4. Advanced Peephole Optimizations
**Impact:** 5-10%
**Difficulty:** Low/Medium
**Description:**
*   **Load Forwarding**: If `StoreInd reg, offset` is followed by `LoadInd reg2, offset`, replace the load with `MoveReg` (or nothing if reg==reg2). (Partially handled by current register cache, but could be strictly enforced in peephole).
*   **Redundant Move Elimination**: `MoveReg A, B` followed by `MoveReg B, A`.
*   **Compare+Branch Fusion**: Verify if `ICmp` fusion can be extended to more predicates or complex conditions.

## Low Impact / Nice to Have

### 5. Stack Slot Packing
**Impact:** < 5%
**Difficulty:** Medium
**Description:**
*   Stack slots are currently 8-byte aligned for simplicity.
*   We could pack `i32`/`i16`/`i8` values tighter to reduce stack frame size.
*   PVM stack offsets are 32-bit, so "reachability" isn't an issue, but smaller frames might help cache locality (if PVM implementation cares).

### 6. Instruction Selection Refinements
**Impact:** < 5%
**Difficulty:** Medium
**Description:**
*   Use `AddImm` vs `Add` more intelligently based on constant thresholds.
*   Review all intrinsic lowerings for shorter instruction sequences.

### 7. Custom Calling Convention for Internal Functions
**Impact:** Variable
**Difficulty:** High
**Description:**
*   Internal functions (not exported) don't strictly need to follow the JAM/PVM ABI.
*   We could pass arguments in more registers (r2-r6) instead of spilling to stack/overflow area.
