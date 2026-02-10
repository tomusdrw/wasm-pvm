use crate::ir::IrInstruction;
use crate::pvm::Instruction;
use crate::{Error, Result};
use wasmparser::FunctionBody;

use super::memory_layout::{
    self, EXIT_ADDRESS, GLOBAL_MEMORY_BASE, OPERAND_SPILL_BASE, PARAM_OVERFLOW_BASE, RO_DATA_BASE,
};
use super::stack::StackMachine;

pub const ARGS_PTR_REG: u8 = 7;
pub const ARGS_LEN_REG: u8 = 8;
const FIRST_LOCAL_REG: u8 = 9;
const MAX_LOCAL_REGS: usize = 4;
pub const RETURN_ADDR_REG: u8 = 0;
pub const STACK_PTR_REG: u8 = 1;
const RETURN_VALUE_REG: u8 = 7;
const SAVED_TABLE_IDX_REG: u8 = 8;

pub struct CompileContext {
    pub num_params: usize,
    pub num_locals: usize,
    pub num_globals: usize,
    pub result_ptr_global: Option<u32>,
    pub result_len_global: Option<u32>,
    pub is_main: bool,
    pub has_return: bool,
    /// When true, the entry function returns (ptr, len) as multi-value return
    /// instead of using `result_ptr`/`result_len` globals.
    pub entry_returns_ptr_len: bool,
    pub function_offsets: Vec<usize>,
    pub function_signatures: Vec<(usize, bool)>,
    pub func_idx: usize,
    pub function_table: Vec<u32>,
    pub type_signatures: Vec<(usize, usize)>,
    pub num_imported_funcs: usize,
    /// Names of imported functions (for stubbing abort, console.log, etc.)
    pub imported_func_names: Vec<String>,
    /// Stack size limit in bytes (for stack overflow detection)
    pub stack_size: u32,
    /// Initial memory size in 64KB pages (from WASM memory section)
    pub initial_memory_pages: u32,
    /// Maximum memory size in pages that can be allocated
    /// Calculated from `heap_pages` allocation or WASM max memory
    pub max_memory_pages: u32,
    /// Base address for WASM linear memory (computed dynamically based on # of functions)
    pub wasm_memory_base: i32,
}

#[derive(Debug, Clone)]
pub struct CallFixup {
    pub return_addr_instr: usize,
    pub jump_instr: usize,
    pub target_func: u32,
}

#[derive(Debug, Clone)]
pub struct IndirectCallFixup {
    pub return_addr_instr: usize,
    pub jump_ind_instr: usize,
}

#[derive(Debug, Clone, Copy)]
enum ControlFrame {
    Block {
        end_label: usize,
        stack_depth: usize,
        has_result: bool,
    },
    Loop {
        start_label: usize,
        stack_depth: usize,
    },
    If {
        else_label: usize,
        end_label: usize,
        stack_depth: usize,
        has_result: bool,
    },
}

/// Alternate register for second spilled operand in binary operations
/// Using r8 because r7 is the primary spill temp, and r8 is free during computation
/// (only used at function call boundaries for args length)
const SPILL_ALT_REG: u8 = 8;

struct CodeEmitter {
    instructions: Vec<Instruction>,
    labels: Vec<Option<usize>>,
    fixups: Vec<(usize, usize)>,
    control_stack: Vec<ControlFrame>,
    stack: StackMachine,
    call_fixups: Vec<CallFixup>,
    indirect_call_fixups: Vec<IndirectCallFixup>,
    pending_spill: Option<usize>,
    /// Tracks the register used by the last `spill_pop` if it was a spilled value
    last_spill_pop_reg: Option<u8>,
}

impl CodeEmitter {
    fn new() -> Self {
        Self {
            instructions: Vec::new(),
            labels: Vec::new(),
            fixups: Vec::new(),
            control_stack: Vec::new(),
            stack: StackMachine::new(),
            call_fixups: Vec::new(),
            indirect_call_fixups: Vec::new(),
            pending_spill: None,
            last_spill_pop_reg: None,
        }
    }

    fn alloc_label(&mut self) -> usize {
        let id = self.labels.len();
        self.labels.push(None);
        id
    }

    fn define_label(&mut self, label: usize) {
        // PVM requires that jump targets be valid basic block starts.
        // A basic block starts after a terminating instruction.
        // If the previous instruction is not a terminator, we must emit FALLTHROUGH
        // to create a valid basic block boundary.
        if self
            .instructions
            .last()
            .is_some_and(|last| !last.is_terminating())
        {
            self.emit(Instruction::Fallthrough);
        }
        self.labels[label] = Some(self.current_offset());
    }

    fn current_offset(&self) -> usize {
        self.instructions.iter().map(|i| i.encode().len()).sum()
    }

    fn emit(&mut self, instr: Instruction) {
        self.instructions.push(instr);
    }

    fn flush_pending_spill(&mut self) {
        if let Some(spill_depth) = self.pending_spill.take() {
            let offset = OPERAND_SPILL_BASE + StackMachine::spill_offset(spill_depth);
            self.emit(Instruction::StoreIndU64 {
                base: STACK_PTR_REG,
                src: StackMachine::reg_at_depth(spill_depth),
                offset,
            });
        }
    }

    fn spill_push(&mut self) -> u8 {
        self.flush_pending_spill();
        self.last_spill_pop_reg = None; // Clear spill tracking on push
        let depth = self.stack.depth();
        let reg = self.stack.push();
        debug_assert!(
            (2..=7).contains(&reg),
            "spill_push: unexpected register {reg} at depth {depth}",
        );
        if StackMachine::needs_spill(depth) {
            // Mark this depth for spilling - the actual spill happens
            // after the caller writes the value to the register
            self.pending_spill = Some(depth);
        }
        reg
    }

    #[allow(dead_code)]
    fn spill_finalize(&mut self) {
        self.flush_pending_spill();
    }

    fn spill_pop(&mut self) -> u8 {
        self.flush_pending_spill();
        let depth = self.stack.depth();
        debug_assert!(depth > 0, "spill_pop: stack is empty");
        if depth > 0 && StackMachine::needs_spill(depth - 1) {
            let offset = OPERAND_SPILL_BASE + StackMachine::spill_offset(depth - 1);
            // Use alternate register if we just popped another spilled value into the default register
            let default_reg = StackMachine::reg_at_depth(depth - 1);
            let dst = if self.last_spill_pop_reg == Some(default_reg) {
                SPILL_ALT_REG
            } else {
                default_reg
            };
            debug_assert!(
                (2..=SPILL_ALT_REG).contains(&dst),
                "spill_pop: unexpected register {dst} at depth {depth}",
            );
            self.emit(Instruction::LoadIndU64 {
                dst,
                base: STACK_PTR_REG,
                offset,
            });
            self.last_spill_pop_reg = Some(dst);
            self.stack.pop();
            return dst;
        }
        self.last_spill_pop_reg = None;
        self.stack.pop()
    }

    fn emit_jump_to_label(&mut self, label: usize) {
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, label));
        self.emit(Instruction::Jump { offset: 0 });
    }

    fn emit_branch_ne_imm_to_label(&mut self, reg: u8, value: i32, label: usize) {
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, label));
        self.emit(Instruction::BranchNeImm {
            reg,
            value,
            offset: 0,
        });
    }

    fn emit_branch_eq_imm_to_label(&mut self, reg: u8, value: i32, label: usize) {
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, label));
        self.emit(Instruction::BranchEqImm {
            reg,
            value,
            offset: 0,
        });
    }

    /// Emit a branch if reg1 > reg2 (unsigned comparison)
    /// Implemented as `BranchLtU`: `BranchLtU { reg1: A, reg2: B }` branches if B < A
    /// So to branch if reg1 > reg2, we need B < A where B=reg2, A=reg1
    /// i.e., `BranchLtU { reg1, reg2 }` branches if reg2 < reg1, which is reg1 > reg2 ✓
    fn emit_branch_gtu(&mut self, reg1: u8, reg2: u8, label: usize) {
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, label));
        // BranchLtU { reg1, reg2 } branches if reg2 < reg1, i.e., reg1 > reg2
        self.emit(Instruction::BranchLtU {
            reg1,
            reg2,
            offset: 0,
        });
    }

    /// Alias for `emit_jump_to_label`
    fn emit_jump(&mut self, label: usize) {
        self.emit_jump_to_label(label);
    }

    /// Emit a trap if the divisor register is zero (WASM spec: div/rem by zero must trap).
    fn emit_div_by_zero_check(&mut self, divisor_reg: u8) {
        let ok_label = self.alloc_label();
        self.emit_branch_ne_imm_to_label(divisor_reg, 0, ok_label);
        self.emit(Instruction::Trap);
        self.define_label(ok_label);
    }

    /// Emit a trap if dividend == `i32::MIN` and divisor == -1 (signed overflow).
    /// WASM spec requires trap for `i32.div_s` when result would be 2^31.
    fn emit_i32_signed_div_overflow_check(&mut self, dividend_reg: u8, divisor_reg: u8) {
        let no_overflow = self.alloc_label();
        self.emit_branch_ne_imm_to_label(dividend_reg, i32::MIN, no_overflow);
        self.emit_branch_ne_imm_to_label(divisor_reg, -1, no_overflow);
        self.emit(Instruction::Trap);
        self.define_label(no_overflow);
    }

    /// Emit a trap if dividend == `i64::MIN` and divisor == -1 (signed overflow).
    /// WASM spec requires trap for `i64.div_s` when result would be 2^63.
    /// Note: this clobbers `divisor_reg` on the slow path but reloads it with -1.
    fn emit_i64_signed_div_overflow_check(&mut self, dividend_reg: u8, divisor_reg: u8) {
        let no_overflow = self.alloc_label();
        // Fast path: if divisor != -1, no overflow possible
        self.emit_branch_ne_imm_to_label(divisor_reg, -1, no_overflow);

        // Slow path: divisor == -1, check if dividend == i64::MIN
        // Safe to clobber divisor_reg since we know its value (-1)
        let reload = self.alloc_label();
        self.emit(Instruction::LoadImm64 {
            reg: divisor_reg,
            value: i64::MIN as u64,
        });
        // XOR dividend with i64::MIN: result is 0 iff dividend == i64::MIN
        self.emit(Instruction::Xor {
            dst: divisor_reg,
            src1: divisor_reg,
            src2: dividend_reg,
        });
        self.emit_branch_ne_imm_to_label(divisor_reg, 0, reload);
        self.emit(Instruction::Trap);

        // Reload divisor value (-1) since we clobbered it
        self.define_label(reload);
        self.emit(Instruction::LoadImm {
            reg: divisor_reg,
            value: -1,
        });

        self.define_label(no_overflow);
    }

    fn resolve_fixups(&mut self) -> Result<()> {
        for (instr_idx, label_id) in &self.fixups {
            let target_offset = self.labels[*label_id]
                .ok_or_else(|| Error::Unsupported("unresolved label".to_string()))?;

            let instr_start: usize = self.instructions[..*instr_idx]
                .iter()
                .map(|i| i.encode().len())
                .sum();

            let relative_offset = (target_offset as i32) - (instr_start as i32);

            match &mut self.instructions[*instr_idx] {
                Instruction::Jump { offset }
                | Instruction::BranchNeImm { offset, .. }
                | Instruction::BranchEqImm { offset, .. }
                | Instruction::BranchGeSImm { offset, .. }
                | Instruction::BranchGeU { offset, .. }
                | Instruction::BranchLtU { offset, .. } => {
                    *offset = relative_offset;
                }
                _ => {
                    return Err(Error::Unsupported(
                        "cannot fixup non-jump instruction".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn push_block(&mut self, has_result: bool) -> usize {
        let end_label = self.alloc_label();
        let stack_depth = self.stack.depth();
        self.control_stack.push(ControlFrame::Block {
            end_label,
            stack_depth,
            has_result,
        });
        end_label
    }

    fn push_loop(&mut self) -> usize {
        let start_label = self.alloc_label();
        let stack_depth = self.stack.depth();
        self.define_label(start_label);
        self.control_stack.push(ControlFrame::Loop {
            start_label,
            stack_depth,
        });
        start_label
    }

    fn pop_control(&mut self) -> Option<ControlFrame> {
        self.control_stack.pop()
    }

    fn get_branch_info(&self, depth: u32) -> Option<(usize, usize, bool)> {
        let idx = self.control_stack.len().checked_sub(1 + depth as usize)?;
        let frame = self.control_stack.get(idx)?;
        match frame {
            ControlFrame::Block {
                end_label,
                stack_depth,
                has_result,
            }
            | ControlFrame::If {
                end_label,
                stack_depth,
                has_result,
                ..
            } => Some((*end_label, *stack_depth, *has_result)),
            ControlFrame::Loop {
                start_label,
                stack_depth,
            } => Some((*start_label, *stack_depth, false)),
        }
    }

    fn push_if(&mut self, cond_reg: u8, has_result: bool) {
        let else_label = self.alloc_label();
        let end_label = self.alloc_label();
        let stack_depth = self.stack.depth();
        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, else_label));
        self.emit(Instruction::BranchEqImm {
            reg: cond_reg,
            value: 0,
            offset: 0,
        });
        self.control_stack.push(ControlFrame::If {
            else_label,
            end_label,
            stack_depth,
            has_result,
        });
    }

    fn emit_call(
        &mut self,
        target_func_idx: u32,
        num_args: usize,
        has_return: bool,
        stack_size: u32,
        func_idx: usize,
        total_locals: usize,
    ) {
        // Calculate how many operand stack values will remain after popping args
        // These are values that belong to the caller and must be preserved
        let stack_depth_before_args = self.stack.depth().saturating_sub(num_args);
        let num_spilled_locals = total_locals.saturating_sub(MAX_LOCAL_REGS);

        // Frame layout on stack (growing down):
        // [sp+0]: return address (r0)
        // [sp+8..40]: locals r9-r12 (4 * 8 = 32 bytes)
        // [sp+40..40+S*8]: spilled locals (S = num_spilled_locals, 8 bytes each)
        // [sp+40+S*8..]: caller's operand stack values (stack_depth_before_args * 8 bytes)
        let spilled_frame_bytes = (num_spilled_locals * 8) as i32;
        let operand_stack_start = 40 + spilled_frame_bytes;
        let frame_size = operand_stack_start + (stack_depth_before_args * 8) as i32;

        // Stack overflow check: new_sp = sp - frame_size
        // If new_sp < stack_limit, trap.
        // NOTE: This clobbers r7 (ARGS_PTR_REG) with the limit value.
        // We must:
        // 1. Flush any pending spill (r7 may hold a deferred spill value at depth >= 5)
        // 2. Use LoadImm64 (not LoadImm) because the limit is in the 0xFExx_xxxx
        //    range which is negative as i32. LoadImm sign-extends to i64, producing
        //    0xFFFFFFFF_FExx_xxxx which breaks the unsigned comparison with the
        //    zero-extended stack pointer (0x00000000_FExx_xxxx).
        let limit = memory_layout::stack_limit(stack_size);
        let continue_label = self.alloc_label();

        self.flush_pending_spill();
        self.emit(Instruction::LoadImm64 {
            reg: ARGS_PTR_REG,
            value: u64::from(limit as u32),
        });
        self.emit(Instruction::AddImm64 {
            dst: SPILL_ALT_REG,
            src: STACK_PTR_REG,
            value: -frame_size,
        });

        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, continue_label));
        self.emit(Instruction::BranchGeU {
            reg1: ARGS_PTR_REG,
            reg2: SPILL_ALT_REG,
            offset: 0,
        });

        // Stack overflow: emit TRAP
        self.emit(Instruction::Trap);

        // Continue with normal call
        self.emit(Instruction::Fallthrough);
        self.define_label(continue_label);

        // Now actually decrement the stack pointer
        self.emit(Instruction::AddImm64 {
            dst: STACK_PTR_REG,
            src: STACK_PTR_REG,
            value: -frame_size,
        });

        // Save return address
        self.emit(Instruction::StoreIndU64 {
            base: STACK_PTR_REG,
            src: RETURN_ADDR_REG,
            offset: 0,
        });

        // Save locals r9-r12
        for i in 0..MAX_LOCAL_REGS {
            let reg = FIRST_LOCAL_REG + i as u8;
            self.emit(Instruction::StoreIndU64 {
                base: STACK_PTR_REG,
                src: reg,
                offset: (8 + i * 8) as i32,
            });
        }

        // Save spilled locals (index >= MAX_LOCAL_REGS) from global memory to stack frame.
        // These live at fixed addresses (SPILLED_LOCALS_BASE + func_idx*512 + offset)
        // and would be clobbered by recursive calls to the same function.
        for i in 0..num_spilled_locals {
            let mem_offset = spilled_local_offset(func_idx, MAX_LOCAL_REGS + i);
            // Load spilled local address into temp register
            self.emit(Instruction::LoadImm {
                reg: SPILL_ALT_REG,
                value: mem_offset,
            });
            // Load spilled local value from global memory
            self.emit(Instruction::LoadIndU64 {
                dst: SPILL_ALT_REG,
                base: SPILL_ALT_REG,
                offset: 0,
            });
            // Store to stack frame
            self.emit(Instruction::StoreIndU64 {
                base: STACK_PTR_REG,
                src: SPILL_ALT_REG,
                offset: (40 + i * 8) as i32,
            });
        }

        // Save caller's operand stack values (those below the arguments)
        // For values in registers (depth < 5): save directly from register
        // For spilled values (depth >= 5): load from old spill area, then save to frame
        for i in 0..stack_depth_before_args {
            if StackMachine::needs_spill(i) {
                // Spilled value: load from old spill area (relative to old SP, which is new_sp + frame_size)
                let spill_offset = frame_size + OPERAND_SPILL_BASE + StackMachine::spill_offset(i);
                self.emit(Instruction::LoadIndU64 {
                    dst: SPILL_ALT_REG,
                    base: STACK_PTR_REG,
                    offset: spill_offset,
                });
                self.emit(Instruction::StoreIndU64 {
                    base: STACK_PTR_REG,
                    src: SPILL_ALT_REG,
                    offset: (operand_stack_start + (i * 8) as i32),
                });
            } else {
                // Value in register: save directly
                let reg = StackMachine::reg_at_depth(i);
                self.emit(Instruction::StoreIndU64 {
                    base: STACK_PTR_REG,
                    src: reg,
                    offset: (operand_stack_start + (i * 8) as i32),
                });
            }
        }

        // Pop arguments and copy to local registers (or overflow area) for the callee.
        // Locals 0-3 fit in registers r9-r12. Locals 4+ go to PARAM_OVERFLOW_BASE,
        // which the callee's prologue copies to its own spilled local addresses.
        for i in 0..num_args {
            let src = self.spill_pop();
            let local_idx = num_args - 1 - i;
            if local_idx < MAX_LOCAL_REGS {
                let dst = FIRST_LOCAL_REG + local_idx as u8;
                self.emit(Instruction::AddImm64 { dst, src, value: 0 });
            } else {
                // Write to parameter overflow area
                let overflow_offset =
                    PARAM_OVERFLOW_BASE + ((local_idx - MAX_LOCAL_REGS) * 8) as i32;
                self.emit(Instruction::LoadImm {
                    reg: SPILL_ALT_REG,
                    value: overflow_offset,
                });
                self.emit(Instruction::StoreIndU64 {
                    base: SPILL_ALT_REG,
                    src,
                    offset: 0,
                });
            }
        }

        let return_addr_instr_idx = self.instructions.len();
        self.emit(Instruction::LoadImm64 {
            reg: RETURN_ADDR_REG,
            value: 0,
        });

        let jump_instr_idx = self.instructions.len();
        self.emit(Instruction::Jump { offset: 0 });

        // Return point
        self.emit(Instruction::Fallthrough);

        // Copy return value to operand stack (before restoring caller's stack)
        // We use a temporary approach: put it in r7, then we'll copy to the right place
        // after restoring the caller's operand stack
        let return_in_r7 = has_return;

        // Restore return address
        self.emit(Instruction::LoadIndU64 {
            dst: RETURN_ADDR_REG,
            base: STACK_PTR_REG,
            offset: 0,
        });

        // Restore locals r9-r12
        for i in 0..MAX_LOCAL_REGS {
            let reg = FIRST_LOCAL_REG + i as u8;
            self.emit(Instruction::LoadIndU64 {
                dst: reg,
                base: STACK_PTR_REG,
                offset: (8 + i * 8) as i32,
            });
        }

        // Restore spilled locals from stack frame back to global memory.
        // We use r7 (ARGS_PTR_REG) as a temp for the address, but r7 also holds
        // the callee's return value. Save r7 to [sp+0] (return addr slot, already
        // restored to r0) and restore it after.
        if num_spilled_locals > 0 {
            self.emit(Instruction::StoreIndU64 {
                base: STACK_PTR_REG,
                src: RETURN_VALUE_REG,
                offset: 0, // reuse return address slot
            });
        }
        for i in 0..num_spilled_locals {
            let mem_offset = spilled_local_offset(func_idx, MAX_LOCAL_REGS + i);
            // Load saved value from stack frame
            self.emit(Instruction::LoadIndU64 {
                dst: SPILL_ALT_REG,
                base: STACK_PTR_REG,
                offset: (40 + i * 8) as i32,
            });
            // Load spilled local address into ARGS_PTR_REG (r7) as temp
            self.emit(Instruction::LoadImm {
                reg: ARGS_PTR_REG,
                value: mem_offset,
            });
            // Store value back to global memory
            self.emit(Instruction::StoreIndU64 {
                base: ARGS_PTR_REG,
                src: SPILL_ALT_REG,
                offset: 0,
            });
        }
        if num_spilled_locals > 0 {
            self.emit(Instruction::LoadIndU64 {
                dst: RETURN_VALUE_REG,
                base: STACK_PTR_REG,
                offset: 0,
            });
        }

        // Restore caller's operand stack values
        // For values in registers (depth < 5): restore directly to register
        // For spilled values (depth >= 5): load from frame, then store to old spill area
        for i in 0..stack_depth_before_args {
            if StackMachine::needs_spill(i) {
                // Load from call frame
                self.emit(Instruction::LoadIndU64 {
                    dst: SPILL_ALT_REG,
                    base: STACK_PTR_REG,
                    offset: (operand_stack_start + (i * 8) as i32),
                });
                // Store to old spill area (relative to old SP, which is new_sp + frame_size)
                let spill_offset = frame_size + OPERAND_SPILL_BASE + StackMachine::spill_offset(i);
                self.emit(Instruction::StoreIndU64 {
                    base: STACK_PTR_REG,
                    src: SPILL_ALT_REG,
                    offset: spill_offset,
                });
            } else {
                // Value goes to register: restore directly
                let reg = StackMachine::reg_at_depth(i);
                self.emit(Instruction::LoadIndU64 {
                    dst: reg,
                    base: STACK_PTR_REG,
                    offset: (operand_stack_start + (i * 8) as i32),
                });
            }
        }

        self.emit(Instruction::AddImm64 {
            dst: STACK_PTR_REG,
            src: STACK_PTR_REG,
            value: frame_size,
        });

        // Now that caller's operand stack is restored, push the return value if any
        if return_in_r7 {
            let dst = self.spill_push();
            self.emit(Instruction::AddImm64 {
                dst,
                src: RETURN_VALUE_REG,
                value: 0,
            });
        }

        self.call_fixups.push(CallFixup {
            return_addr_instr: return_addr_instr_idx,
            jump_instr: jump_instr_idx,
            target_func: target_func_idx,
        });
    }

    fn emit_call_indirect(
        &mut self,
        num_args: usize,
        has_return: bool,
        stack_size: u32,
        expected_type_index: u32,
        func_idx: usize,
        total_locals: usize,
    ) {
        let table_idx_reg = self.spill_pop();

        let stack_depth_before_args = self.stack.depth().saturating_sub(num_args);
        let num_spilled_locals = total_locals.saturating_sub(MAX_LOCAL_REGS);
        let spilled_frame_bytes = (num_spilled_locals * 8) as i32;
        let operand_stack_start = 40 + spilled_frame_bytes;
        let frame_size = operand_stack_start + (stack_depth_before_args * 8) as i32;

        // Save table index to r8 (SAVED_TABLE_IDX_REG)
        self.emit(Instruction::AddImm32 {
            dst: SAVED_TABLE_IDX_REG,
            src: table_idx_reg,
            value: 0,
        });

        // Save table index to [SP - frame_size - 8] immediately, BEFORE the
        // stack overflow check which clobbers r8 (SPILL_ALT_REG = SAVED_TABLE_IDX_REG).
        // After the prologue decrements SP by frame_size, this location
        // becomes [new_SP - 8], safely below the frame.
        self.emit(Instruction::StoreIndU64 {
            base: STACK_PTR_REG,
            src: SAVED_TABLE_IDX_REG,
            offset: -frame_size - 8,
        });

        // Stack overflow check: new_sp = sp - frame_size
        // If new_sp < stack_limit, trap.
        let limit = memory_layout::stack_limit(stack_size);
        let continue_label = self.alloc_label();

        // NOTE: This clobbers r7 (ARGS_PTR_REG) with the limit value.
        // Flush any pending spill first (r7 may hold a deferred spill value).
        // Must use LoadImm64 to avoid sign-extension of the negative i32 limit.
        self.flush_pending_spill();
        self.emit(Instruction::LoadImm64 {
            reg: ARGS_PTR_REG,
            value: u64::from(limit as u32),
        });
        self.emit(Instruction::AddImm64 {
            dst: SPILL_ALT_REG,
            src: STACK_PTR_REG,
            value: -frame_size,
        });

        let fixup_idx = self.instructions.len();
        self.fixups.push((fixup_idx, continue_label));
        self.emit(Instruction::BranchGeU {
            reg1: ARGS_PTR_REG,
            reg2: SPILL_ALT_REG,
            offset: 0,
        });

        // Stack overflow: emit TRAP
        self.emit(Instruction::Trap);

        // Continue with normal call
        self.emit(Instruction::Fallthrough);
        self.define_label(continue_label);

        // Now actually decrement the stack pointer
        self.emit(Instruction::AddImm64 {
            dst: STACK_PTR_REG,
            src: STACK_PTR_REG,
            value: -frame_size,
        });

        self.emit(Instruction::StoreIndU64 {
            base: STACK_PTR_REG,
            src: RETURN_ADDR_REG,
            offset: 0,
        });

        for i in 0..MAX_LOCAL_REGS {
            let reg = FIRST_LOCAL_REG + i as u8;
            self.emit(Instruction::StoreIndU64 {
                base: STACK_PTR_REG,
                src: reg,
                offset: (8 + i * 8) as i32,
            });
        }

        // Save spilled locals (same as emit_call)
        for i in 0..num_spilled_locals {
            let mem_offset = spilled_local_offset(func_idx, MAX_LOCAL_REGS + i);
            self.emit(Instruction::LoadImm {
                reg: SPILL_ALT_REG,
                value: mem_offset,
            });
            self.emit(Instruction::LoadIndU64 {
                dst: SPILL_ALT_REG,
                base: SPILL_ALT_REG,
                offset: 0,
            });
            self.emit(Instruction::StoreIndU64 {
                base: STACK_PTR_REG,
                src: SPILL_ALT_REG,
                offset: (40 + i * 8) as i32,
            });
        }

        // Save caller's operand stack values (same as emit_call)
        for i in 0..stack_depth_before_args {
            if StackMachine::needs_spill(i) {
                let spill_offset = frame_size + OPERAND_SPILL_BASE + StackMachine::spill_offset(i);
                self.emit(Instruction::LoadIndU64 {
                    dst: SPILL_ALT_REG,
                    base: STACK_PTR_REG,
                    offset: spill_offset,
                });
                self.emit(Instruction::StoreIndU64 {
                    base: STACK_PTR_REG,
                    src: SPILL_ALT_REG,
                    offset: (operand_stack_start + (i * 8) as i32),
                });
            } else {
                let reg = StackMachine::reg_at_depth(i);
                self.emit(Instruction::StoreIndU64 {
                    base: STACK_PTR_REG,
                    src: reg,
                    offset: (operand_stack_start + (i * 8) as i32),
                });
            }
        }

        // Pop arguments and copy to local registers (or overflow area) for the callee.
        // For call_indirect, we don't know the callee's func_idx, so overflow params
        // go to a fixed temporary area (PARAM_OVERFLOW_BASE). The callee's prologue
        // copies them to its own spilled local addresses.
        for i in 0..num_args {
            let src = self.spill_pop();
            let local_idx = num_args - 1 - i;
            if local_idx < MAX_LOCAL_REGS {
                let dst = FIRST_LOCAL_REG + local_idx as u8;
                self.emit(Instruction::AddImm64 { dst, src, value: 0 });
            } else {
                // Write to parameter overflow area
                let overflow_offset =
                    PARAM_OVERFLOW_BASE + ((local_idx - MAX_LOCAL_REGS) * 8) as i32;
                self.emit(Instruction::LoadImm {
                    reg: SPILL_ALT_REG,
                    value: overflow_offset,
                });
                self.emit(Instruction::StoreIndU64 {
                    base: SPILL_ALT_REG,
                    src,
                    offset: 0,
                });
            }
        }

        // Restore the table index (r8) from [new_SP - 8].
        // The table index was saved at [old_SP - frame_size - 8] before the prologue,
        // which is [new_SP - 8] after SP was decremented by frame_size.
        self.emit(Instruction::LoadIndU64 {
            dst: SAVED_TABLE_IDX_REG,
            base: STACK_PTR_REG,
            offset: -8,
        });

        // Dispatch table entries are now 8 bytes each (4 bytes jump addr + 4 bytes type index)
        // Multiply table index by 8: x*2*2*2 = x*8
        self.emit(Instruction::Add32 {
            dst: SAVED_TABLE_IDX_REG,
            src1: SAVED_TABLE_IDX_REG,
            src2: SAVED_TABLE_IDX_REG,
        });
        self.emit(Instruction::Add32 {
            dst: SAVED_TABLE_IDX_REG,
            src1: SAVED_TABLE_IDX_REG,
            src2: SAVED_TABLE_IDX_REG,
        });
        self.emit(Instruction::Add32 {
            dst: SAVED_TABLE_IDX_REG,
            src1: SAVED_TABLE_IDX_REG,
            src2: SAVED_TABLE_IDX_REG,
        });

        // Add RO_DATA_BASE to get dispatch table address
        self.emit(Instruction::AddImm32 {
            dst: SAVED_TABLE_IDX_REG,
            src: SAVED_TABLE_IDX_REG,
            value: RO_DATA_BASE,
        });

        // Load type index from dispatch table (at offset 4)
        // Use ARGS_PTR_REG (r7) as temp since we're about to overwrite it anyway
        self.emit(Instruction::LoadIndU32 {
            dst: ARGS_PTR_REG,
            base: SAVED_TABLE_IDX_REG,
            offset: 4, // type index is at offset 4
        });

        // Validate type signature: compare with expected type index
        // If mismatch, TRAP
        let sig_ok_label = self.alloc_label();
        self.emit(Instruction::BranchEqImm {
            reg: ARGS_PTR_REG,
            value: expected_type_index as i32,
            offset: 0, // will be fixed up
        });
        let fixup_idx = self.instructions.len() - 1;
        self.fixups.push((fixup_idx, sig_ok_label));

        // Signature mismatch - TRAP
        self.emit(Instruction::Trap);
        self.emit(Instruction::Fallthrough);

        // Signature OK - continue with call
        self.define_label(sig_ok_label);

        // Load jump address from dispatch table (at offset 0)
        self.emit(Instruction::LoadIndU32 {
            dst: SAVED_TABLE_IDX_REG,
            base: SAVED_TABLE_IDX_REG,
            offset: 0,
        });

        let return_addr_instr_idx = self.instructions.len();
        self.emit(Instruction::LoadImm64 {
            reg: RETURN_ADDR_REG,
            value: 0,
        });

        let jump_ind_instr_idx = self.instructions.len();
        self.emit(Instruction::JumpInd {
            reg: SAVED_TABLE_IDX_REG,
            offset: 0,
        });

        self.emit(Instruction::Fallthrough);

        self.indirect_call_fixups.push(IndirectCallFixup {
            return_addr_instr: return_addr_instr_idx,
            jump_ind_instr: jump_ind_instr_idx,
        });

        let return_in_r7 = has_return;

        self.emit(Instruction::LoadIndU64 {
            dst: RETURN_ADDR_REG,
            base: STACK_PTR_REG,
            offset: 0,
        });

        for i in 0..MAX_LOCAL_REGS {
            let reg = FIRST_LOCAL_REG + i as u8;
            self.emit(Instruction::LoadIndU64 {
                dst: reg,
                base: STACK_PTR_REG,
                offset: (8 + i * 8) as i32,
            });
        }

        // Restore spilled locals from stack frame back to global memory.
        // Save/restore r7 (return value) around this since we use it as temp.
        if num_spilled_locals > 0 {
            self.emit(Instruction::StoreIndU64 {
                base: STACK_PTR_REG,
                src: RETURN_VALUE_REG,
                offset: 0,
            });
        }
        for i in 0..num_spilled_locals {
            let mem_offset = spilled_local_offset(func_idx, MAX_LOCAL_REGS + i);
            self.emit(Instruction::LoadIndU64 {
                dst: SPILL_ALT_REG,
                base: STACK_PTR_REG,
                offset: (40 + i * 8) as i32,
            });
            self.emit(Instruction::LoadImm {
                reg: ARGS_PTR_REG,
                value: mem_offset,
            });
            self.emit(Instruction::StoreIndU64 {
                base: ARGS_PTR_REG,
                src: SPILL_ALT_REG,
                offset: 0,
            });
        }
        if num_spilled_locals > 0 {
            self.emit(Instruction::LoadIndU64 {
                dst: RETURN_VALUE_REG,
                base: STACK_PTR_REG,
                offset: 0,
            });
        }

        // Restore caller's operand stack values (same as emit_call)
        for i in 0..stack_depth_before_args {
            if StackMachine::needs_spill(i) {
                self.emit(Instruction::LoadIndU64 {
                    dst: SPILL_ALT_REG,
                    base: STACK_PTR_REG,
                    offset: (operand_stack_start + (i * 8) as i32),
                });
                let spill_offset = frame_size + OPERAND_SPILL_BASE + StackMachine::spill_offset(i);
                self.emit(Instruction::StoreIndU64 {
                    base: STACK_PTR_REG,
                    src: SPILL_ALT_REG,
                    offset: spill_offset,
                });
            } else {
                let reg = StackMachine::reg_at_depth(i);
                self.emit(Instruction::LoadIndU64 {
                    dst: reg,
                    base: STACK_PTR_REG,
                    offset: (operand_stack_start + (i * 8) as i32),
                });
            }
        }

        self.emit(Instruction::AddImm64 {
            dst: STACK_PTR_REG,
            src: STACK_PTR_REG,
            value: frame_size,
        });

        if return_in_r7 {
            let dst = self.spill_push();
            self.emit(Instruction::AddImm64 {
                dst,
                src: RETURN_VALUE_REG,
                value: 0,
            });
        }
    }
}

pub struct FunctionTranslation {
    pub instructions: Vec<Instruction>,
    pub call_fixups: Vec<CallFixup>,
    pub indirect_call_fixups: Vec<IndirectCallFixup>,
}

pub fn translate_function(
    body: &FunctionBody,
    ctx: &CompileContext,
) -> Result<FunctionTranslation> {
    let (func_locals, ir) = crate::ir::build_ir(body)?;
    translate_function_ir(&ir, func_locals, ctx)
}

pub fn translate_function_ir(
    ir: &[IrInstruction],
    func_locals: usize,
    ctx: &CompileContext,
) -> Result<FunctionTranslation> {
    let mut emitter = CodeEmitter::new();

    let total_locals = ctx.num_params + func_locals;

    emit_prologue(&mut emitter, ctx, total_locals);

    for instr in ir {
        translate_ir_op(instr, &mut emitter, ctx, total_locals)?;
    }

    emit_epilogue(&mut emitter, ctx, ctx.has_return);

    emitter.resolve_fixups()?;

    Ok(FunctionTranslation {
        instructions: emitter.instructions,
        call_fixups: emitter.call_fixups,
        indirect_call_fixups: emitter.indirect_call_fixups,
    })
}

fn emit_prologue(emitter: &mut CodeEmitter, ctx: &CompileContext, total_locals: usize) {
    if ctx.is_main && ctx.num_params >= 1 {
        // For main(), SPI convention passes:
        //   r7 = args_ptr (raw PVM address, e.g., 0xFEFF0000)
        //   r8 = args_len
        //
        // WASM code expects to use `local.get $args_ptr` which reads from r9 (local 0).
        // All memory load/store operations add WASM_MEMORY_BASE (0x50000) to translate
        // WASM addresses to PVM addresses.
        //
        // To make this work, we:
        // 1. Subtract WASM_MEMORY_BASE from args_ptr so loads read from correct location:
        //    adjusted_ptr = 0xFEFF0000 - 0x50000 = 0xFEFA0000
        //    load(adjusted_ptr) → load(0xFEFA0000 + 0x50000) = load(0xFEFF0000) ✓
        // 2. Copy the adjusted value to r9 (local 0 / $args_ptr)
        // 3. Copy args_len to r10 (local 1 / $args_len) if present
        //
        // IMPORTANT: Use AddImm64 (not AddImm32) to avoid sign-extension issues.
        // AddImm32 with a negative value would sign-extend the result to 64 bits,
        // corrupting the address (e.g., 0xFEFA0000 → 0xFFFFFFFFFEFA0000).

        // Adjust args_ptr in r7
        emitter.emit(Instruction::AddImm64 {
            dst: ARGS_PTR_REG,
            src: ARGS_PTR_REG,
            value: -ctx.wasm_memory_base,
        });

        // Copy adjusted args_ptr to r9 (local 0)
        // IMPORTANT: Use AddImm64 (not AddImm32) to preserve the full 64-bit value.
        // AddImm32 sign-extends the result, which would corrupt addresses like 0xFEFA0000.
        emitter.emit(Instruction::AddImm64 {
            dst: FIRST_LOCAL_REG, // r9
            src: ARGS_PTR_REG,    // r7
            value: 0,
        });

        // Copy args_len to r10 (local 1) if there's a second parameter
        // Use AddImm64 for consistency, although args_len is typically small
        if ctx.num_params >= 2 {
            emitter.emit(Instruction::AddImm64 {
                dst: FIRST_LOCAL_REG + 1, // r10
                src: ARGS_LEN_REG,        // r8
                value: 0,
            });
        }
    }

    // Copy overflow parameters (5th+) from PARAM_OVERFLOW_BASE to spilled local addresses.
    // The caller writes params 4+ to PARAM_OVERFLOW_BASE + (idx-4)*8 for both direct
    // and indirect calls. We copy them to this function's spilled local addresses.
    if !ctx.is_main && ctx.num_params > MAX_LOCAL_REGS {
        for param_idx in MAX_LOCAL_REGS..ctx.num_params {
            let overflow_offset = PARAM_OVERFLOW_BASE + ((param_idx - MAX_LOCAL_REGS) * 8) as i32;
            let spilled_offset = spilled_local_offset(ctx.func_idx, param_idx);
            // Load from overflow area
            emitter.emit(Instruction::LoadImm {
                reg: SPILL_ALT_REG,
                value: overflow_offset,
            });
            emitter.emit(Instruction::LoadIndU64 {
                dst: SPILL_ALT_REG,
                base: SPILL_ALT_REG,
                offset: 0,
            });
            // Store to spilled local address
            emitter.emit(Instruction::LoadImm {
                reg: ARGS_PTR_REG,
                value: spilled_offset,
            });
            emitter.emit(Instruction::StoreIndU64 {
                base: ARGS_PTR_REG,
                src: SPILL_ALT_REG,
                offset: 0,
            });
        }
    }

    // Zero-initialize non-parameter local variables as required by WebAssembly spec.
    // Parameters (locals 0..num_params) are initialized by caller or code above.
    // Remaining locals (num_params..total_locals) must be zero-initialized.
    let start_local = ctx.num_params;
    // Initialize register-based locals (0..MAX_LOCAL_REGS)
    let end_local = total_locals.min(MAX_LOCAL_REGS);
    for local_idx in start_local..end_local {
        emitter.emit(Instruction::LoadImm {
            reg: FIRST_LOCAL_REG + local_idx as u8,
            value: 0,
        });
    }

    // Zero-initialize spilled locals (index >= MAX_LOCAL_REGS).
    // These live at fixed global memory addresses and may retain values from
    // previous calls to the same function (e.g. recursive calls).
    if total_locals > MAX_LOCAL_REGS {
        let start_spilled = if start_local > MAX_LOCAL_REGS {
            start_local
        } else {
            MAX_LOCAL_REGS
        };
        if start_spilled < total_locals {
            // Load 0 into r8 once, then use it for all stores
            emitter.emit(Instruction::LoadImm {
                reg: SPILL_ALT_REG,
                value: 0,
            });
            for local_idx in start_spilled..total_locals {
                let offset = spilled_local_offset(ctx.func_idx, local_idx);
                // Load address into r7 (safe in prologue)
                emitter.emit(Instruction::LoadImm {
                    reg: ARGS_PTR_REG,
                    value: offset,
                });
                // Store 0 (from r8) to the spilled local address
                emitter.emit(Instruction::StoreIndU64 {
                    base: ARGS_PTR_REG,
                    src: SPILL_ALT_REG,
                    offset: 0,
                });
            }
        }
    }
}

fn emit_main_exit(emitter: &mut CodeEmitter, ctx: &CompileContext) {
    if ctx.entry_returns_ptr_len {
        // New convention: main returns (ptr, len) as multi-value return.
        // Stack has [ptr, len] with len on top.
        let len_reg = emitter.spill_pop();
        let ptr_reg = emitter.spill_pop();
        // r7 = ptr + WASM_MEMORY_BASE (translate WASM address to PVM address)
        emitter.emit(Instruction::AddImm32 {
            dst: ARGS_PTR_REG,
            src: ptr_reg,
            value: ctx.wasm_memory_base,
        });
        // r8 = r7 + len (end pointer)
        emitter.emit(Instruction::Add32 {
            dst: ARGS_LEN_REG,
            src1: ARGS_PTR_REG,
            src2: len_reg,
        });
    } else {
        // Legacy convention: read result_ptr and result_len from globals
        if let Some(ptr_idx) = ctx.result_ptr_global {
            let offset = (ptr_idx as i32) * 4 + GLOBAL_MEMORY_BASE;
            emitter.emit(Instruction::LoadImm {
                reg: 2,
                value: offset,
            });
            emitter.emit(Instruction::LoadIndU32 {
                dst: ARGS_PTR_REG,
                base: 2,
                offset: 0,
            });
            // Translate result_ptr from WASM address to PVM address
            emitter.emit(Instruction::AddImm32 {
                dst: ARGS_PTR_REG,
                src: ARGS_PTR_REG,
                value: ctx.wasm_memory_base,
            });
        }
        if let Some(len_idx) = ctx.result_len_global {
            let offset = (len_idx as i32) * 4 + GLOBAL_MEMORY_BASE;
            emitter.emit(Instruction::LoadImm {
                reg: 2,
                value: offset,
            });
            emitter.emit(Instruction::LoadIndU32 {
                dst: ARGS_LEN_REG,
                base: 2,
                offset: 0,
            });
            // Calculate end pointer: r8 = r7 + r8 (ptr + len)
            emitter.emit(Instruction::Add32 {
                dst: ARGS_LEN_REG,
                src1: ARGS_PTR_REG,
                src2: ARGS_LEN_REG,
            });
        }
    }
    emitter.emit(Instruction::LoadImm {
        reg: 2,
        value: EXIT_ADDRESS,
    });
    emitter.emit(Instruction::JumpInd { reg: 2, offset: 0 });
}

fn emit_epilogue(emitter: &mut CodeEmitter, ctx: &CompileContext, has_return: bool) {
    if ctx.is_main {
        emit_main_exit(emitter, ctx);
    } else {
        // Only pop return value if there's something on the stack
        // (there might not be if an explicit 'return' already handled it)
        if has_return && emitter.stack.depth() > 0 {
            let ret_val = emitter.spill_pop();
            emitter.emit(Instruction::AddImm32 {
                dst: RETURN_VALUE_REG,
                src: ret_val,
                value: 0,
            });
        }
        emitter.emit(Instruction::JumpInd {
            reg: RETURN_ADDR_REG,
            offset: 0,
        });
    }
}

fn local_reg(idx: usize) -> Option<u8> {
    if idx < MAX_LOCAL_REGS {
        Some(FIRST_LOCAL_REG + idx as u8)
    } else {
        None
    }
}

fn spilled_local_offset(func_idx: usize, local_idx: usize) -> i32 {
    let local_offset = ((local_idx - MAX_LOCAL_REGS) as i32) * 8;
    memory_layout::spilled_local_addr(func_idx, local_offset)
}

fn global_offset(idx: u32) -> i32 {
    memory_layout::global_addr(idx)
}

fn translate_ir_op(
    op: &IrInstruction,
    emitter: &mut CodeEmitter,
    ctx: &CompileContext,
    total_locals: usize,
) -> Result<()> {
    match op {
        IrInstruction::LocalGet(local_index) => {
            let idx = *local_index as usize;
            if let Some(reg) = local_reg(idx) {
                let dst = emitter.spill_push();
                emitter.emit(Instruction::AddImm64 {
                    dst,
                    src: reg,
                    value: 0,
                });
            } else {
                let offset = spilled_local_offset(ctx.func_idx, idx);
                let dst = emitter.spill_push();
                emitter.emit(Instruction::LoadImm {
                    reg: dst,
                    value: offset,
                });
                emitter.emit(Instruction::LoadIndU64 {
                    dst,
                    base: dst,
                    offset: 0,
                });
            }
        }
        IrInstruction::LocalSet(local_index) => {
            let idx = *local_index as usize;
            if let Some(reg) = local_reg(idx) {
                let src = emitter.spill_pop();
                emitter.emit(Instruction::AddImm64 {
                    dst: reg,
                    src,
                    value: 0,
                });
            } else {
                let offset = spilled_local_offset(ctx.func_idx, idx);
                let src = emitter.spill_pop();
                // Use SPILL_ALT_REG (r8) as temp to avoid clobbering operand stack registers (r2-r6).
                // src is always r2-r7 (never r8) for a single pop, so r8 is safe.
                emitter.emit(Instruction::LoadImm {
                    reg: SPILL_ALT_REG,
                    value: offset,
                });
                emitter.emit(Instruction::StoreIndU64 {
                    base: SPILL_ALT_REG,
                    src,
                    offset: 0,
                });
            }
        }
        IrInstruction::LocalTee(local_index) => {
            let idx = *local_index as usize;
            let stack_depth = emitter.stack.depth();

            // Get the source register for the top of stack
            // If the top is at a spill depth, it might be:
            // 1. In r7 with a pending spill (not yet written to memory)
            // 2. Already spilled to memory (if there was an intervening operation)
            let src = if stack_depth > 0 && StackMachine::needs_spill(stack_depth - 1) {
                // Check if there's a pending spill for this depth
                if emitter.pending_spill == Some(stack_depth - 1) {
                    // Value is still in r7, not yet spilled
                    StackMachine::reg_at_depth(stack_depth - 1) // r7
                } else {
                    // Value was already spilled to memory, load it
                    let spill_offset =
                        OPERAND_SPILL_BASE + StackMachine::spill_offset(stack_depth - 1);
                    emitter.emit(Instruction::LoadIndU64 {
                        dst: SPILL_ALT_REG,
                        base: STACK_PTR_REG,
                        offset: spill_offset,
                    });
                    SPILL_ALT_REG
                }
            } else {
                emitter.stack.peek(0)
            };

            if let Some(reg) = local_reg(idx) {
                emitter.emit(Instruction::AddImm64 {
                    dst: reg,
                    src,
                    value: 0,
                });
            } else {
                let offset = spilled_local_offset(ctx.func_idx, idx);
                // Use r8 (SPILL_ALT_REG) as temp to avoid clobbering operand stack registers
                // Note: src might be r7 or r8, so we need to handle both cases
                let temp = if src == SPILL_ALT_REG {
                    // src is r8, use a different register for the address
                    // We'll store r8's value first, then restore after
                    // Actually, let's just use r7 which should be safe here
                    7
                } else {
                    SPILL_ALT_REG
                };
                emitter.emit(Instruction::LoadImm {
                    reg: temp,
                    value: offset,
                });
                emitter.emit(Instruction::StoreIndU64 {
                    base: temp,
                    src,
                    offset: 0,
                });
            }
        }
        IrInstruction::GlobalGet(global_index) => {
            let offset = global_offset(*global_index);
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LoadImm {
                reg: dst,
                value: offset,
            });
            emitter.emit(Instruction::LoadIndU32 {
                dst,
                base: dst,
                offset: 0,
            });
        }
        IrInstruction::GlobalSet(global_index) => {
            let offset = global_offset(*global_index);
            let src = emitter.spill_pop();
            // Use SPILL_ALT_REG (r8) as temp to avoid clobbering operand stack registers (r2-r6).
            // src is always r2-r7 (never r8) for a single pop, so r8 is safe.
            emitter.emit(Instruction::LoadImm {
                reg: SPILL_ALT_REG,
                value: offset,
            });
            emitter.emit(Instruction::StoreIndU32 {
                base: SPILL_ALT_REG,
                src,
                offset: 0,
            });
        }
        IrInstruction::I32Load { offset } | IrInstruction::I64Load32U { offset } => {
            let addr = emitter.spill_pop();
            let dst = emitter.spill_push();
            // Add WASM_MEMORY_BASE to translate WASM address to PVM address
            emitter.emit(Instruction::LoadIndU32 {
                dst,
                base: addr,
                offset: *offset as i32 + ctx.wasm_memory_base,
            });
        }
        IrInstruction::I32Store { offset } | IrInstruction::I64Store32 { offset } => {
            let value = emitter.spill_pop();
            let addr = emitter.spill_pop();
            // Add WASM_MEMORY_BASE to translate WASM address to PVM address
            emitter.emit(Instruction::StoreIndU32 {
                base: addr,
                src: value,
                offset: *offset as i32 + ctx.wasm_memory_base,
            });
        }
        IrInstruction::I64Load { offset } => {
            let addr = emitter.spill_pop();
            let dst = emitter.spill_push();
            // Add WASM_MEMORY_BASE to translate WASM address to PVM address
            emitter.emit(Instruction::LoadIndU64 {
                dst,
                base: addr,
                offset: *offset as i32 + ctx.wasm_memory_base,
            });
        }
        IrInstruction::I64Store { offset } => {
            let value = emitter.spill_pop();
            let addr = emitter.spill_pop();
            // Add WASM_MEMORY_BASE to translate WASM address to PVM address
            emitter.emit(Instruction::StoreIndU64 {
                base: addr,
                src: value,
                offset: *offset as i32 + ctx.wasm_memory_base,
            });
        }
        IrInstruction::I32Const(value) => {
            let reg = emitter.spill_push();
            emitter.emit(Instruction::LoadImm { reg, value: *value });
        }
        IrInstruction::I64Const(value) => {
            let reg = emitter.spill_push();
            emitter.emit(Instruction::LoadImm64 {
                reg,
                value: *value as u64,
            });
        }
        IrInstruction::I32Add => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Add32 { dst, src1, src2 });
        }
        IrInstruction::I32Sub => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Sub32 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        IrInstruction::I32Mul => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Mul32 { dst, src1, src2 });
        }
        IrInstruction::I32DivU => {
            let src2 = emitter.spill_pop(); // divisor
            let src1 = emitter.spill_pop(); // dividend
            emitter.emit_div_by_zero_check(src2);
            let dst = emitter.spill_push();
            emitter.emit(Instruction::DivU32 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        IrInstruction::I32DivS => {
            let src2 = emitter.spill_pop(); // divisor
            let src1 = emitter.spill_pop(); // dividend
            emitter.emit_div_by_zero_check(src2);
            emitter.emit_i32_signed_div_overflow_check(src1, src2);
            let dst = emitter.spill_push();
            emitter.emit(Instruction::DivS32 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        IrInstruction::I32RemU => {
            let src2 = emitter.spill_pop(); // divisor
            let src1 = emitter.spill_pop(); // dividend
            emitter.emit_div_by_zero_check(src2);
            let dst = emitter.spill_push();
            emitter.emit(Instruction::RemU32 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        IrInstruction::I32RemS => {
            let src2 = emitter.spill_pop(); // divisor
            let src1 = emitter.spill_pop(); // dividend
            emitter.emit_div_by_zero_check(src2);
            let dst = emitter.spill_push();
            emitter.emit(Instruction::RemS32 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        IrInstruction::I64Eq => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Xor {
                dst,
                src1: a,
                src2: b,
            });
            emitter.emit(Instruction::SetLtUImm {
                dst,
                src: dst,
                value: 1,
            });
        }
        IrInstruction::I32Eq => {
            // i32.eq must only compare the lower 32 bits.
            // i32.const sign-extends (LoadImm) while i32.load zero-extends (LoadIndU32),
            // so the upper 32 bits may differ. Truncate XOR result to 32 bits via AddImm32.
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Xor {
                dst,
                src1: a,
                src2: b,
            });
            emitter.emit(Instruction::AddImm32 {
                dst,
                src: dst,
                value: 0,
            });
            emitter.emit(Instruction::SetLtUImm {
                dst,
                src: dst,
                value: 1,
            });
        }
        IrInstruction::I64Ne => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Xor {
                dst,
                src1: a,
                src2: b,
            });
            let zero = emitter.spill_push();
            emitter.emit(Instruction::LoadImm {
                reg: zero,
                value: 0,
            });
            let _ = emitter.spill_pop();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: dst,
                src2: zero,
            });
        }
        IrInstruction::I32Ne => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Xor {
                dst,
                src1: a,
                src2: b,
            });
            emitter.emit(Instruction::AddImm32 {
                dst,
                src: dst,
                value: 0,
            });
            let zero = emitter.spill_push();
            emitter.emit(Instruction::LoadImm {
                reg: zero,
                value: 0,
            });
            let _ = emitter.spill_pop();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: dst,
                src2: zero,
            });
        }
        IrInstruction::I32And | IrInstruction::I64And => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::And {
                dst,
                src1: a,
                src2: b,
            });
        }
        IrInstruction::I32Or | IrInstruction::I64Or => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Or {
                dst,
                src1: a,
                src2: b,
            });
        }
        IrInstruction::I32Xor | IrInstruction::I64Xor => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Xor {
                dst,
                src1: a,
                src2: b,
            });
        }
        IrInstruction::I32Shl => {
            let shift = emitter.spill_pop();
            let value = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::ShloL32 {
                dst,
                src1: shift,
                src2: value,
            });
        }
        IrInstruction::I32ShrU => {
            let shift = emitter.spill_pop();
            let value = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::ShloR32 {
                dst,
                src1: shift,
                src2: value,
            });
        }
        IrInstruction::I32ShrS => {
            let shift = emitter.spill_pop();
            let value = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SharR32 {
                dst,
                src1: shift,
                src2: value,
            });
        }
        IrInstruction::I64Shl => {
            let shift = emitter.spill_pop();
            let value = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::ShloL64 {
                dst,
                src1: shift,
                src2: value,
            });
        }
        IrInstruction::I64ShrU => {
            let shift = emitter.spill_pop();
            let value = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::ShloR64 {
                dst,
                src1: shift,
                src2: value,
            });
        }
        IrInstruction::I64ShrS => {
            let shift = emitter.spill_pop();
            let value = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SharR64 {
                dst,
                src1: shift,
                src2: value,
            });
        }
        IrInstruction::Nop => {}
        IrInstruction::Unreachable => {
            emitter.emit(Instruction::Trap);
        }
        IrInstruction::Drop => {
            let _ = emitter.spill_pop();
        }
        IrInstruction::I64Add => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Add64 { dst, src1, src2 });
        }
        IrInstruction::I64Sub => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Sub64 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        IrInstruction::I64Mul => {
            let src2 = emitter.spill_pop();
            let src1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::Mul64 { dst, src1, src2 });
        }
        IrInstruction::I64DivU => {
            let src2 = emitter.spill_pop(); // divisor
            let src1 = emitter.spill_pop(); // dividend
            emitter.emit_div_by_zero_check(src2);
            let dst = emitter.spill_push();
            emitter.emit(Instruction::DivU64 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        IrInstruction::I64DivS => {
            let src2 = emitter.spill_pop(); // divisor
            let src1 = emitter.spill_pop(); // dividend
            emitter.emit_div_by_zero_check(src2);
            emitter.emit_i64_signed_div_overflow_check(src1, src2);
            let dst = emitter.spill_push();
            emitter.emit(Instruction::DivS64 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        IrInstruction::I64RemU => {
            let src2 = emitter.spill_pop(); // divisor
            let src1 = emitter.spill_pop(); // dividend
            emitter.emit_div_by_zero_check(src2);
            let dst = emitter.spill_push();
            emitter.emit(Instruction::RemU64 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        IrInstruction::I64RemS => {
            let src2 = emitter.spill_pop(); // divisor
            let src1 = emitter.spill_pop(); // dividend
            emitter.emit_div_by_zero_check(src2);
            let dst = emitter.spill_push();
            emitter.emit(Instruction::RemS64 {
                dst,
                src1: src2,
                src2: src1,
            });
        }
        IrInstruction::I64GtU => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: a,
                src2: b,
            });
        }
        IrInstruction::I32GtU => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            // Normalize both operands to sign-extended 32-bit for correct 64-bit comparison.
            // Sign-extension preserves both signed and unsigned 32-bit ordering.
            emitter.emit(Instruction::AddImm32 {
                dst: a,
                src: a,
                value: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: b,
                src: b,
                value: 0,
            });
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: a,
                src2: b,
            });
        }
        IrInstruction::I64GtS => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtS {
                dst,
                src1: a,
                src2: b,
            });
        }
        IrInstruction::I32GtS => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            emitter.emit(Instruction::AddImm32 {
                dst: a,
                src: a,
                value: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: b,
                src: b,
                value: 0,
            });
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtS {
                dst,
                src1: a,
                src2: b,
            });
        }
        // For WASM i32.lt_X: push a, push b, lt → result = a < b
        // Pop b (top), pop a (second)
        // PVM SetLt semantics: dst = src2 < src1 (verified in anan-as: reg[b] < reg[a])
        // For a < b, we need SetLt { src1: b, src2: a } so it computes a < b
        IrInstruction::I64LtU => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: b,
                src2: a,
            });
        }
        IrInstruction::I32LtU => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            emitter.emit(Instruction::AddImm32 {
                dst: a,
                src: a,
                value: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: b,
                src: b,
                value: 0,
            });
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: b,
                src2: a,
            });
        }
        IrInstruction::I64LtS => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtS {
                dst,
                src1: b,
                src2: a,
            });
        }
        IrInstruction::I32LtS => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            emitter.emit(Instruction::AddImm32 {
                dst: a,
                src: a,
                value: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: b,
                src: b,
                value: 0,
            });
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtS {
                dst,
                src1: b,
                src2: a,
            });
        }
        IrInstruction::I64GeU => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: b,
                src2: a,
            });
            let one = emitter.spill_push();
            emitter.emit(Instruction::LoadImm { reg: one, value: 1 });
            let _ = emitter.spill_pop();
            emitter.emit(Instruction::Xor {
                dst,
                src1: dst,
                src2: one,
            });
        }
        IrInstruction::I32GeU => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            emitter.emit(Instruction::AddImm32 {
                dst: a,
                src: a,
                value: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: b,
                src: b,
                value: 0,
            });
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: b,
                src2: a,
            });
            let one = emitter.spill_push();
            emitter.emit(Instruction::LoadImm { reg: one, value: 1 });
            let _ = emitter.spill_pop();
            emitter.emit(Instruction::Xor {
                dst,
                src1: dst,
                src2: one,
            });
        }
        IrInstruction::I64GeS => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtS {
                dst,
                src1: b,
                src2: a,
            });
            let one = emitter.spill_push();
            emitter.emit(Instruction::LoadImm { reg: one, value: 1 });
            let _ = emitter.spill_pop();
            emitter.emit(Instruction::Xor {
                dst,
                src1: dst,
                src2: one,
            });
        }
        IrInstruction::I32GeS => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            emitter.emit(Instruction::AddImm32 {
                dst: a,
                src: a,
                value: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: b,
                src: b,
                value: 0,
            });
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtS {
                dst,
                src1: b,
                src2: a,
            });
            let one = emitter.spill_push();
            emitter.emit(Instruction::LoadImm { reg: one, value: 1 });
            let _ = emitter.spill_pop();
            emitter.emit(Instruction::Xor {
                dst,
                src1: dst,
                src2: one,
            });
        }
        IrInstruction::I64LeU => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: a,
                src2: b,
            });
            let one = emitter.spill_push();
            emitter.emit(Instruction::LoadImm { reg: one, value: 1 });
            let _ = emitter.spill_pop();
            emitter.emit(Instruction::Xor {
                dst,
                src1: dst,
                src2: one,
            });
        }
        IrInstruction::I32LeU => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            emitter.emit(Instruction::AddImm32 {
                dst: a,
                src: a,
                value: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: b,
                src: b,
                value: 0,
            });
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtU {
                dst,
                src1: a,
                src2: b,
            });
            let one = emitter.spill_push();
            emitter.emit(Instruction::LoadImm { reg: one, value: 1 });
            let _ = emitter.spill_pop();
            emitter.emit(Instruction::Xor {
                dst,
                src1: dst,
                src2: one,
            });
        }
        IrInstruction::I64LeS => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtS {
                dst,
                src1: a,
                src2: b,
            });
            let one = emitter.spill_push();
            emitter.emit(Instruction::LoadImm { reg: one, value: 1 });
            let _ = emitter.spill_pop();
            emitter.emit(Instruction::Xor {
                dst,
                src1: dst,
                src2: one,
            });
        }
        IrInstruction::I32LeS => {
            let b = emitter.spill_pop();
            let a = emitter.spill_pop();
            emitter.emit(Instruction::AddImm32 {
                dst: a,
                src: a,
                value: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: b,
                src: b,
                value: 0,
            });
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtS {
                dst,
                src1: a,
                src2: b,
            });
            let one = emitter.spill_push();
            emitter.emit(Instruction::LoadImm { reg: one, value: 1 });
            let _ = emitter.spill_pop();
            emitter.emit(Instruction::Xor {
                dst,
                src1: dst,
                src2: one,
            });
        }
        IrInstruction::I64Eqz => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SetLtUImm { dst, src, value: 1 });
        }
        IrInstruction::I32Eqz => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            // Normalize to 32-bit: upper bits may be garbage from mixed i32 sources
            emitter.emit(Instruction::AddImm32 { dst, src, value: 0 });
            emitter.emit(Instruction::SetLtUImm {
                dst,
                src: dst,
                value: 1,
            });
        }
        IrInstruction::Block { has_result } => {
            emitter.push_block(*has_result);
        }
        IrInstruction::Loop => {
            emitter.emit(Instruction::Fallthrough);
            emitter.push_loop();
        }
        IrInstruction::If { has_result } => {
            let cond = emitter.spill_pop();
            emitter.push_if(cond, *has_result);
        }
        IrInstruction::Else => {
            if let Some(ControlFrame::If {
                else_label,
                end_label,
                stack_depth,
                has_result,
            }) = emitter.pop_control()
            {
                emitter.emit_jump_to_label(end_label);
                emitter.emit(Instruction::Fallthrough);
                emitter.define_label(else_label);
                // Always reset depth: the true branch may have ended with br/return
                // leaving the depth wrong. The else branch starts at the if's entry depth.
                emitter.stack.set_depth(stack_depth);
                // Clear stale spill state from unreachable code after br/return
                emitter.pending_spill = None;
                emitter.last_spill_pop_reg = None;
                emitter.control_stack.push(ControlFrame::Block {
                    end_label,
                    stack_depth,
                    has_result,
                });
            }
        }
        IrInstruction::End => match emitter.pop_control() {
            Some(ControlFrame::Block {
                end_label,
                stack_depth,
                has_result,
            }) => {
                emitter.emit(Instruction::Fallthrough);
                emitter.define_label(end_label);
                // Always reset depth: branches (br/return) inside the block may have
                // left the depth wrong. At the merge point, depth must match the
                // block's entry depth (+ 1 if the block produces a result).
                let target_depth = if has_result {
                    stack_depth + 1
                } else {
                    stack_depth
                };
                emitter.stack.set_depth(target_depth);
                // Clear stale spill state from unreachable code after br/return
                emitter.pending_spill = None;
                emitter.last_spill_pop_reg = None;
            }
            Some(ControlFrame::If {
                else_label,
                end_label,
                stack_depth,
                has_result,
            }) => {
                emitter.emit(Instruction::Fallthrough);
                emitter.define_label(else_label);
                emitter.define_label(end_label);
                // Always reset depth (same reasoning as Block above).
                let target_depth = if has_result {
                    stack_depth + 1
                } else {
                    stack_depth
                };
                emitter.stack.set_depth(target_depth);
                // Clear stale spill state from unreachable code after br/return
                emitter.pending_spill = None;
                emitter.last_spill_pop_reg = None;
            }
            Some(ControlFrame::Loop { .. }) => {
                emitter.emit(Instruction::Fallthrough);
            }
            None => {}
        },
        IrInstruction::Br(relative_depth) => {
            if let Some((target, target_depth, has_result)) =
                emitter.get_branch_info(*relative_depth)
            {
                if has_result && emitter.stack.depth() > target_depth {
                    let src = StackMachine::reg_at_depth(emitter.stack.depth() - 1);
                    let dst = StackMachine::reg_at_depth(target_depth);
                    if src != dst {
                        emitter.emit(Instruction::AddImm64 { dst, src, value: 0 });
                    }
                }
                emitter.emit_jump_to_label(target);
            }
        }
        IrInstruction::BrIf(relative_depth) => {
            let cond = emitter.spill_pop();
            if let Some((target, target_depth, has_result)) =
                emitter.get_branch_info(*relative_depth)
            {
                if has_result && emitter.stack.depth() > target_depth {
                    let end_label = emitter.alloc_label();
                    emitter.emit_branch_eq_imm_to_label(cond, 0, end_label);
                    let src = StackMachine::reg_at_depth(emitter.stack.depth() - 1);
                    let dst = StackMachine::reg_at_depth(target_depth);
                    if src != dst {
                        emitter.emit(Instruction::AddImm64 { dst, src, value: 0 });
                    }
                    emitter.emit_jump_to_label(target);
                    emitter.emit(Instruction::Fallthrough);
                    emitter.define_label(end_label);
                } else {
                    emitter.emit_branch_ne_imm_to_label(cond, 0, target);
                }
            }
        }
        IrInstruction::BrTable { targets, default } => {
            let index_reg = emitter.spill_pop();
            let target_depths = targets;
            let default_depth = *default;

            for (i, &depth) in target_depths.iter().enumerate() {
                if let Some((target, target_depth, has_result)) = emitter.get_branch_info(depth) {
                    let next_label = emitter.alloc_label();
                    emitter.emit_branch_ne_imm_to_label(index_reg, i as i32, next_label);
                    if has_result && emitter.stack.depth() > target_depth {
                        let src = StackMachine::reg_at_depth(emitter.stack.depth() - 1);
                        let dst = StackMachine::reg_at_depth(target_depth);
                        if src != dst {
                            emitter.emit(Instruction::AddImm64 { dst, src, value: 0 });
                        }
                    }
                    emitter.emit_jump_to_label(target);
                    emitter.emit(Instruction::Fallthrough);
                    emitter.define_label(next_label);
                }
            }

            if let Some((target, target_depth, has_result)) = emitter.get_branch_info(default_depth)
            {
                if has_result && emitter.stack.depth() > target_depth {
                    let src = StackMachine::reg_at_depth(emitter.stack.depth() - 1);
                    let dst = StackMachine::reg_at_depth(target_depth);
                    if src != dst {
                        emitter.emit(Instruction::AddImm64 { dst, src, value: 0 });
                    }
                }
                emitter.emit_jump_to_label(target);
            }
        }
        IrInstruction::Return => {
            // For the main entry function, return means exit the program
            // For other functions, return to caller via jump table
            if ctx.is_main {
                emit_main_exit(emitter, ctx);
            } else {
                // Handle return value if present
                if ctx.has_return {
                    let ret_val = emitter.spill_pop();
                    emitter.emit(Instruction::AddImm32 {
                        dst: RETURN_VALUE_REG,
                        src: ret_val,
                        value: 0,
                    });
                }
                // Return to caller via jump table
                emitter.emit(Instruction::JumpInd {
                    reg: RETURN_ADDR_REG,
                    offset: 0,
                });
            }
        }
        IrInstruction::Call(function_index) => {
            let (num_args, has_return) = ctx
                .function_signatures
                .get(*function_index as usize)
                .copied()
                .unwrap_or((0, false));

            // Check if this is a call to an imported function
            if (*function_index as usize) < ctx.num_imported_funcs {
                let import_name = ctx
                    .imported_func_names
                    .get(*function_index as usize)
                    .map_or("unknown", String::as_str);

                // Pop arguments (they're on the stack)
                for _ in 0..num_args {
                    emitter.spill_pop();
                }

                // Handle specific imports:
                // - "abort": emit TRAP
                // - others: no-op (just discard arguments)
                if import_name == "abort" {
                    emitter.emit(Instruction::Trap);
                }

                // Push dummy return value (0) to maintain stack balance
                if has_return {
                    let dst = emitter.spill_push();
                    emitter.emit(Instruction::LoadImm { reg: dst, value: 0 });
                }
            } else {
                // Convert global function index to local function index for emit_call
                let local_func_idx = *function_index - ctx.num_imported_funcs as u32;
                emitter.emit_call(
                    local_func_idx,
                    num_args,
                    has_return,
                    ctx.stack_size,
                    ctx.func_idx,
                    total_locals,
                );
            }
        }
        IrInstruction::CallIndirect {
            type_idx,
            table_idx,
        } => {
            if *table_idx != 0 {
                return Err(Error::Unsupported(format!(
                    "call_indirect with table index {table_idx}"
                )));
            }
            let (num_args, num_results) = ctx
                .type_signatures
                .get(*type_idx as usize)
                .copied()
                .unwrap_or((0, 0));
            let has_return = num_results > 0;
            emitter.emit_call_indirect(
                num_args,
                has_return,
                ctx.stack_size,
                *type_idx,
                ctx.func_idx,
                total_locals,
            );
        }
        IrInstruction::MemorySize => {
            // Load current memory size from compiler-managed global
            let dst = emitter.spill_push();
            let global_addr = memory_layout::memory_size_global_offset(ctx.num_globals);
            emitter.emit(Instruction::LoadImm {
                reg: dst,
                value: global_addr,
            });
            emitter.emit(Instruction::LoadIndU32 {
                dst,
                base: dst,
                offset: 0,
            });
        }
        IrInstruction::MemoryGrow => {
            // memory.grow(delta) - tries to grow memory by `delta` pages
            // Return: previous size in pages if success, -1 if failure
            //
            // Algorithm:
            // 1. Save delta to a temp register (since pop/push might reuse same reg)
            // 2. Load current size from compiler global
            // 3. Calculate new_size = current + delta
            // 4. If new_size > max_pages, return -1
            // 5. Store new_size to compiler global
            // 6. Return old size

            // Save the current stack depth BEFORE we pop - we need this to know
            // which registers are safe to use as temps
            let stack_depth_before = emitter.stack.depth();
            let delta = emitter.spill_pop();
            let dst = emitter.spill_push();
            let global_addr = memory_layout::memory_size_global_offset(ctx.num_globals);

            // IMPORTANT: delta and dst might be the same register!
            // We need to save delta to a different register before loading current size.
            // BUT we must not clobber any registers that are currently on the stack!
            //
            // Stack uses r2-r6 for depths 0-4. After the pop, depth is (stack_depth_before - 1),
            // so registers r2 to r(2 + stack_depth_before - 2) are still in use.
            // We need to use a register that's NOT in use by the stack.
            //
            // Using r4/r5 is unsafe if stack depth >= 3!
            // We use r7 and r8 which are designated scratch registers in our convention
            // (saved/restored by entry prologue if needed).
            // Safe temp registers: r7 (ARGS_PTR_REG) and r8 (ARGS_LEN_REG) treated as scratch
            // For the 3rd temp, we need another safe register.
            // Stack uses r2-r6. If depth is high, r6 is used.
            // Locals start at r9.
            // r0 is return addr (unsafe to clobber).
            // r1 is SP.
            //
            // We'll use a high local register that is unlikely to be used, or spill?
            // Actually, we can reuse 'delta' register if we are careful.

            // Let's use r13? No, only 13 registers (0-12).
            // r12 is local 3.

            // Let's check if we can reuse registers.
            // We need: delta_reg, dst, new_size_reg, max_reg.

            // dst is target for 'current size'.

            // Let's use r7 for delta copy.
            // dst (stack top).
            // r8 for new_size.
            // r7 for max (reuse r7).

            let scratch_1 = 7u8;
            let scratch_2 = 8u8;

            // Move delta to scratch_1 if delta == dst
            if delta == dst {
                emitter.emit(Instruction::AddImm32 {
                    dst: scratch_1,
                    src: delta,
                    value: 0,
                });
            }
            let delta_reg = if delta == dst { scratch_1 } else { delta };

            // Load current memory size into dst
            emitter.emit(Instruction::LoadImm {
                reg: dst,
                value: global_addr,
            });
            emitter.emit(Instruction::LoadIndU32 {
                dst,
                base: dst,
                offset: 0,
            });

            // Calculate new_size = current + delta using scratch_2
            let new_size_reg = scratch_2;
            emitter.emit(Instruction::Add32 {
                dst: new_size_reg,
                src1: dst,
                src2: delta_reg,
            });

            // Check if new_size > max_pages
            let fail_label = emitter.alloc_label();
            let end_label = emitter.alloc_label();

            // Use scratch_1 for max_pages comparison (delta_reg no longer needed)
            let max_reg = scratch_1;

            // Load max_pages for comparison
            emitter.emit(Instruction::LoadImm {
                reg: max_reg,
                value: ctx.max_memory_pages as i32,
            });

            // Branch to fail if new_size > max_pages (unsigned comparison)
            // BranchGtU: jump if new_size_reg > max_reg
            emitter.emit_branch_gtu(new_size_reg, max_reg, fail_label);

            // Success path: store new_size, return old size (already in dst)
            // Store new_size to compiler global (reuse max_reg for address)
            emitter.emit(Instruction::LoadImm {
                reg: max_reg,
                value: global_addr,
            });
            emitter.emit(Instruction::StoreIndU32 {
                base: max_reg,
                src: new_size_reg,
                offset: 0,
            });

            // Actually grow PVM memory via SBRK instruction.
            // Compute delta_bytes = (new_size - old_size) * 65536 in max_reg (scratch).
            // new_size is in new_size_reg, old_size is in dst.
            emitter.emit(Instruction::Sub32 {
                dst: max_reg,
                src1: new_size_reg,
                src2: dst,
            });
            // Shift left by 16 to multiply by 65536 (WASM page size)
            {
                let shift_amount = scratch_2; // reuse r8 for shift amount
                emitter.emit(Instruction::LoadImm {
                    reg: shift_amount,
                    value: 16,
                });
                emitter.emit(Instruction::ShloL32 {
                    dst: max_reg,
                    src1: max_reg,
                    src2: shift_amount,
                });
            }
            // SBRK: src=max_reg (bytes to allocate), dst=max_reg (receives old break, discarded)
            emitter.emit(Instruction::Sbrk {
                dst: max_reg,
                src: max_reg,
            });

            // dst already has old size, jump to end
            emitter.emit_jump(end_label);

            // Failure path: return -1
            emitter.define_label(fail_label);
            emitter.emit(Instruction::LoadImm {
                reg: dst,
                value: -1,
            });

            emitter.define_label(end_label);
            // Silence unused variable warning
            let _ = stack_depth_before;
            // Result is in dst (either old size on success, or -1 on failure)
        }
        IrInstruction::MemoryFill => {
            // memory.fill(dest, value, size) - fills size bytes at dest with value
            let size = emitter.spill_pop();
            let value = emitter.spill_pop();
            let dest = emitter.spill_pop();

            // Use a loop to fill memory byte by byte
            // while (size > 0) { mem[dest] = value; dest++; size--; }
            let loop_start = emitter.alloc_label();
            let loop_end = emitter.alloc_label();

            // Add WASM_MEMORY_BASE to dest for PVM address translation
            emitter.emit(Instruction::AddImm32 {
                dst: dest,
                src: dest,
                value: ctx.wasm_memory_base,
            });

            // loop_start:
            emitter.define_label(loop_start);

            // if (size == 0) goto loop_end
            emitter.emit_branch_eq_imm_to_label(size, 0, loop_end);

            // mem[dest] = value (store byte)
            emitter.emit(Instruction::StoreIndU8 {
                base: dest,
                src: value,
                offset: 0,
            });

            // dest++
            emitter.emit(Instruction::AddImm32 {
                dst: dest,
                src: dest,
                value: 1,
            });

            // size--
            emitter.emit(Instruction::AddImm32 {
                dst: size,
                src: size,
                value: -1,
            });

            // goto loop_start
            emitter.emit_jump_to_label(loop_start);

            // loop_end:
            emitter.emit(Instruction::Fallthrough);
            emitter.define_label(loop_end);
        }
        IrInstruction::MemoryCopy => {
            // memory.copy(dest, src, size) - like memmove, handles overlapping regions
            // When dest > src, we must copy backward to avoid overwriting source data
            let size = emitter.spill_pop();
            let src = emitter.spill_pop();
            let dest = emitter.spill_pop();

            let temp = ARGS_PTR_REG;

            let backward_setup = emitter.alloc_label();
            let forward_loop = emitter.alloc_label();
            let backward_loop = emitter.alloc_label();
            let end = emitter.alloc_label();

            // If dest > src (unsigned), use backward copy to handle overlap
            emitter.emit_branch_gtu(dest, src, backward_setup);

            // === FORWARD COPY (dest <= src) ===
            emitter.emit(Instruction::AddImm32 {
                dst: dest,
                src: dest,
                value: ctx.wasm_memory_base,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: src,
                src,
                value: ctx.wasm_memory_base,
            });

            // forward_loop:
            emitter.define_label(forward_loop);
            emitter.emit_branch_eq_imm_to_label(size, 0, end);
            emitter.emit(Instruction::LoadIndU8 {
                dst: temp,
                base: src,
                offset: 0,
            });
            emitter.emit(Instruction::StoreIndU8 {
                base: dest,
                src: temp,
                offset: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: dest,
                src: dest,
                value: 1,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: src,
                src,
                value: 1,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: size,
                src: size,
                value: -1,
            });
            emitter.emit_jump_to_label(forward_loop);

            // === BACKWARD COPY (dest > src) ===
            // backward_setup:
            emitter.emit(Instruction::Fallthrough);
            emitter.define_label(backward_setup);
            emitter.emit(Instruction::AddImm32 {
                dst: dest,
                src: dest,
                value: ctx.wasm_memory_base,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: src,
                src,
                value: ctx.wasm_memory_base,
            });
            // Start from the end: dest += size, src += size
            emitter.emit(Instruction::Add32 {
                dst: dest,
                src1: dest,
                src2: size,
            });
            emitter.emit(Instruction::Add32 {
                dst: src,
                src1: src,
                src2: size,
            });

            // backward_loop:
            emitter.define_label(backward_loop);
            emitter.emit_branch_eq_imm_to_label(size, 0, end);
            // Pre-decrement (pointers start past the end)
            emitter.emit(Instruction::AddImm32 {
                dst: dest,
                src: dest,
                value: -1,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: src,
                src,
                value: -1,
            });
            emitter.emit(Instruction::LoadIndU8 {
                dst: temp,
                base: src,
                offset: 0,
            });
            emitter.emit(Instruction::StoreIndU8 {
                base: dest,
                src: temp,
                offset: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: size,
                src: size,
                value: -1,
            });
            emitter.emit_jump_to_label(backward_loop);

            // end:
            emitter.emit(Instruction::Fallthrough);
            emitter.define_label(end);
        }
        IrInstruction::Select => {
            let cond = emitter.spill_pop();
            let val2 = emitter.spill_pop();
            let val1 = emitter.spill_pop();
            let dst = emitter.spill_push();
            let else_label = emitter.alloc_label();
            let end_label = emitter.alloc_label();
            emitter.emit_branch_eq_imm_to_label(cond, 0, else_label);
            emitter.emit(Instruction::AddImm64 {
                dst,
                src: val1,
                value: 0,
            });
            emitter.emit_jump_to_label(end_label);
            emitter.emit(Instruction::Fallthrough);
            emitter.define_label(else_label);
            emitter.emit(Instruction::AddImm64 {
                dst,
                src: val2,
                value: 0,
            });
            emitter.emit(Instruction::Fallthrough);
            emitter.define_label(end_label);
        }
        IrInstruction::I32Clz => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LeadingZeroBits32 { dst, src });
        }
        IrInstruction::I64Clz => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LeadingZeroBits64 { dst, src });
        }
        IrInstruction::I32Ctz => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::TrailingZeroBits32 { dst, src });
        }
        IrInstruction::I64Ctz => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::TrailingZeroBits64 { dst, src });
        }
        IrInstruction::I32Popcnt => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::CountSetBits32 { dst, src });
        }
        IrInstruction::I64Popcnt => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::CountSetBits64 { dst, src });
        }
        IrInstruction::I32WrapI64 => {
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::AddImm32 { dst, src, value: 0 });
        }
        IrInstruction::I64ExtendI32S => {
            // Sign-extend from bit 31 to 64 bits.
            // AddImm32 { value: 0 } takes the lower 32 bits and sign-extends to 64.
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::AddImm32 { dst, src, value: 0 });
        }
        IrInstruction::I32Extend8S => {
            // Sign-extend the lowest 8 bits of i32 to full i32
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SignExtend8 { dst, src });
        }
        IrInstruction::I32Extend16S => {
            // Sign-extend the lowest 16 bits of i32 to full i32
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SignExtend16 { dst, src });
        }
        IrInstruction::I64Extend8S => {
            // Sign-extend the lowest 8 bits of i64 to full i64
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SignExtend8 { dst, src });
        }
        IrInstruction::I64Extend16S => {
            // Sign-extend the lowest 16 bits of i64 to full i64
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::SignExtend16 { dst, src });
        }
        IrInstruction::I64Extend32S => {
            // Sign-extend the lowest 32 bits of i64 to full i64.
            // AddImm32 { value: 0 } takes the lower 32 bits and sign-extends to 64.
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::AddImm32 { dst, src, value: 0 });
        }
        IrInstruction::I64ExtendI32U => {
            // Zero-extend from 32 bits to 64 bits: clear upper 32 bits.
            // Shift left 32 then logical shift right 32.
            let src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LoadImm {
                reg: SPILL_ALT_REG,
                value: 32,
            });
            emitter.emit(Instruction::ShloL64 {
                dst,
                src1: SPILL_ALT_REG,
                src2: src,
            });
            emitter.emit(Instruction::ShloR64 {
                dst,
                src1: SPILL_ALT_REG,
                src2: dst,
            });
        }
        // Float truncation stubs - PVM doesn't support floats, but these may appear
        // in dead code from AssemblyScript stdlib. We stub them to allow compilation.
        // If actually called, the result will be incorrect (returns 0).
        IrInstruction::I32TruncSatF64U | IrInstruction::I32TruncSatF64S => {
            // f64 -> i32 truncation (saturating)
            // Pop the f64 input (treated as i64 in our integer-only world)
            let _src = emitter.spill_pop();
            // Push 0 as result
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LoadImm64 { reg: dst, value: 0 });
        }
        IrInstruction::I32TruncSatF32U | IrInstruction::I32TruncSatF32S => {
            // f32 -> i32 truncation (saturating)
            let _src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LoadImm64 { reg: dst, value: 0 });
        }
        IrInstruction::I64TruncSatF64U | IrInstruction::I64TruncSatF64S => {
            // f64 -> i64 truncation (saturating)
            let _src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LoadImm64 { reg: dst, value: 0 });
        }
        IrInstruction::I64TruncSatF32U | IrInstruction::I64TruncSatF32S => {
            // f32 -> i64 truncation (saturating)
            let _src = emitter.spill_pop();
            let dst = emitter.spill_push();
            emitter.emit(Instruction::LoadImm64 { reg: dst, value: 0 });
        }
        IrInstruction::I32Load8U { offset } | IrInstruction::I64Load8U { offset } => {
            let addr = emitter.spill_pop();
            let dst = emitter.spill_push();
            // Add WASM_MEMORY_BASE to translate WASM address to PVM address
            emitter.emit(Instruction::LoadIndU8 {
                dst,
                base: addr,
                offset: *offset as i32 + ctx.wasm_memory_base,
            });
        }
        IrInstruction::I32Load8S { offset } | IrInstruction::I64Load8S { offset } => {
            let addr = emitter.spill_pop();
            let dst = emitter.spill_push();
            // Add WASM_MEMORY_BASE to translate WASM address to PVM address
            emitter.emit(Instruction::LoadIndI8 {
                dst,
                base: addr,
                offset: *offset as i32 + ctx.wasm_memory_base,
            });
        }
        IrInstruction::I32Load16U { offset } | IrInstruction::I64Load16U { offset } => {
            let addr = emitter.spill_pop();
            let dst = emitter.spill_push();
            // Add WASM_MEMORY_BASE to translate WASM address to PVM address
            emitter.emit(Instruction::LoadIndU16 {
                dst,
                base: addr,
                offset: *offset as i32 + ctx.wasm_memory_base,
            });
        }
        IrInstruction::I32Load16S { offset } | IrInstruction::I64Load16S { offset } => {
            let addr = emitter.spill_pop();
            let dst = emitter.spill_push();
            // Add WASM_MEMORY_BASE to translate WASM address to PVM address
            emitter.emit(Instruction::LoadIndI16 {
                dst,
                base: addr,
                offset: *offset as i32 + ctx.wasm_memory_base,
            });
        }
        IrInstruction::I64Load32S { offset } => {
            let addr = emitter.spill_pop();
            let dst = emitter.spill_push();
            // Add WASM_MEMORY_BASE to translate WASM address to PVM address
            // LoadIndU32 zero-extends, then AddImm32 sign-extends from bit 31.
            emitter.emit(Instruction::LoadIndU32 {
                dst,
                base: addr,
                offset: *offset as i32 + ctx.wasm_memory_base,
            });
            emitter.emit(Instruction::AddImm32 {
                dst,
                src: dst,
                value: 0,
            });
        }
        IrInstruction::I32Store8 { offset } | IrInstruction::I64Store8 { offset } => {
            let val = emitter.spill_pop();
            let addr = emitter.spill_pop();
            // Add WASM_MEMORY_BASE to translate WASM address to PVM address
            emitter.emit(Instruction::StoreIndU8 {
                base: addr,
                src: val,
                offset: *offset as i32 + ctx.wasm_memory_base,
            });
        }
        IrInstruction::I32Store16 { offset } | IrInstruction::I64Store16 { offset } => {
            let val = emitter.spill_pop();
            let addr = emitter.spill_pop();
            // Add WASM_MEMORY_BASE to translate WASM address to PVM address
            emitter.emit(Instruction::StoreIndU16 {
                base: addr,
                src: val,
                offset: *offset as i32 + ctx.wasm_memory_base,
            });
        }
        IrInstruction::I32Rotl => {
            let n = emitter.spill_pop();
            let value = emitter.spill_pop();
            let result = emitter.spill_push();
            emitter.emit(Instruction::AddImm32 {
                dst: 7,
                src: value,
                value: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: 8,
                src: n,
                value: 0,
            });
            emitter.emit(Instruction::ShloL32 {
                dst: result,
                src1: 8,
                src2: 7,
            });
            emitter.emit(Instruction::LoadImm { reg: n, value: 32 });
            emitter.emit(Instruction::Sub32 {
                dst: 8,
                src1: 8,
                src2: n,
            });
            emitter.emit(Instruction::ShloR32 {
                dst: 7,
                src1: 8,
                src2: 7,
            });
            emitter.emit(Instruction::Or {
                dst: result,
                src1: result,
                src2: 7,
            });
            // Mask to 32 bits for i32 operation (0xFFFFFFFF as i32 is -1)
            let mask = emitter.spill_push();
            emitter.emit(Instruction::LoadImm {
                reg: mask,
                value: -1i32,
            });
            let _ = emitter.spill_pop();
            emitter.emit(Instruction::And {
                dst: result,
                src1: result,
                src2: mask,
            });
        }
        IrInstruction::I32Rotr => {
            let n = emitter.spill_pop();
            let value = emitter.spill_pop();
            let result = emitter.spill_push();
            emitter.emit(Instruction::AddImm32 {
                dst: 7,
                src: value,
                value: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: 8,
                src: n,
                value: 0,
            });
            emitter.emit(Instruction::ShloR32 {
                dst: result,
                src1: 8,
                src2: 7,
            });
            emitter.emit(Instruction::LoadImm { reg: n, value: 32 });
            emitter.emit(Instruction::Sub32 {
                dst: 8,
                src1: 8,
                src2: n,
            });
            emitter.emit(Instruction::ShloL32 {
                dst: 7,
                src1: 8,
                src2: 7,
            });
            emitter.emit(Instruction::Or {
                dst: result,
                src1: result,
                src2: 7,
            });
            // Mask to 32 bits for i32 operation (0xFFFFFFFF as i32 is -1)
            let mask = emitter.spill_push();
            emitter.emit(Instruction::LoadImm {
                reg: mask,
                value: -1i32,
            });
            let _ = emitter.spill_pop();
            emitter.emit(Instruction::And {
                dst: result,
                src1: result,
                src2: mask,
            });
        }
        IrInstruction::I64Rotl => {
            let n = emitter.spill_pop();
            let value = emitter.spill_pop();
            let result = emitter.spill_push();
            emitter.emit(Instruction::AddImm32 {
                dst: 7,
                src: value,
                value: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: 8,
                src: n,
                value: 0,
            });
            emitter.emit(Instruction::ShloL64 {
                dst: result,
                src1: 8,
                src2: 7,
            });
            emitter.emit(Instruction::LoadImm { reg: n, value: 64 });
            emitter.emit(Instruction::Sub64 {
                dst: 8,
                src1: 8,
                src2: n,
            });
            emitter.emit(Instruction::ShloR64 {
                dst: 7,
                src1: 8,
                src2: 7,
            });
            emitter.emit(Instruction::Or {
                dst: result,
                src1: result,
                src2: 7,
            });
        }
        IrInstruction::I64Rotr => {
            let n = emitter.spill_pop();
            let value = emitter.spill_pop();
            let result = emitter.spill_push();
            emitter.emit(Instruction::AddImm32 {
                dst: 7,
                src: value,
                value: 0,
            });
            emitter.emit(Instruction::AddImm32 {
                dst: 8,
                src: n,
                value: 0,
            });
            emitter.emit(Instruction::ShloR64 {
                dst: result,
                src1: 8,
                src2: 7,
            });
            emitter.emit(Instruction::LoadImm { reg: n, value: 64 });
            emitter.emit(Instruction::Sub64 {
                dst: 8,
                src1: 8,
                src2: n,
            });
            emitter.emit(Instruction::ShloL64 {
                dst: 7,
                src1: 8,
                src2: 7,
            });
            emitter.emit(Instruction::Or {
                dst: result,
                src1: result,
                src2: 7,
            });
        }
    }
    Ok(())
}
