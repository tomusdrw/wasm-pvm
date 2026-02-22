// Shared helper to collect successor basic blocks from an LLVM terminator instruction.
//
// Used by both the register allocator (for back-edge detection) and the emitter
// (for single-predecessor map computation).

use inkwell::basic_block::BasicBlock;
use inkwell::values::{InstructionOpcode, InstructionValue};

/// Collect successor basic blocks from a terminator instruction.
pub fn collect_successors(term: InstructionValue<'_>) -> Vec<BasicBlock<'_>> {
    let mut successors = Vec::new();
    match term.get_opcode() {
        InstructionOpcode::Br => {
            let num_ops = term.get_num_operands();
            if num_ops == 1 {
                // Unconditional: operand 0 is dest_bb
                if let Some(bb) = term
                    .get_operand(0)
                    .and_then(inkwell::values::Operand::block)
                {
                    successors.push(bb);
                }
            } else {
                // Conditional: operand 1 = false_bb, operand 2 = true_bb
                if let Some(bb) = term
                    .get_operand(1)
                    .and_then(inkwell::values::Operand::block)
                {
                    successors.push(bb);
                }
                if let Some(bb) = term
                    .get_operand(2)
                    .and_then(inkwell::values::Operand::block)
                {
                    successors.push(bb);
                }
            }
        }
        InstructionOpcode::Switch => {
            // Operand 1 = default_bb, then pairs of (case_val, case_bb)
            if let Some(bb) = term
                .get_operand(1)
                .and_then(inkwell::values::Operand::block)
            {
                successors.push(bb);
            }
            let num_ops = term.get_num_operands();
            let mut i = 3; // case_bb starts at operand 3
            while i < num_ops {
                if let Some(bb) = term
                    .get_operand(i)
                    .and_then(inkwell::values::Operand::block)
                {
                    successors.push(bb);
                }
                i += 2;
            }
        }
        // Return, Unreachable — no successors
        _ => {}
    }
    successors
}
