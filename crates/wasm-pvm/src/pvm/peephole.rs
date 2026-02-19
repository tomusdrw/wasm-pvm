// Peephole optimizer for PVM instructions.
//
// Runs before fixup resolution to remove redundant instructions.
// Builds an index remap table to update fixup references and label byte offsets.

use super::Instruction;
use crate::llvm_backend::{LlvmCallFixup, LlvmIndirectCallFixup};

/// Run peephole optimizations on a function's instruction stream.
///
/// Must be called **before** `resolve_fixups()` since it removes instructions
/// and remaps all fixup indices and label byte offsets accordingly.
pub fn optimize(
    instructions: &mut Vec<Instruction>,
    fixups: &mut Vec<(usize, usize)>,
    call_fixups: &mut Vec<LlvmCallFixup>,
    indirect_call_fixups: &mut Vec<LlvmIndirectCallFixup>,
    labels: &mut [Option<usize>],
) {
    let len = instructions.len();
    if len == 0 {
        return;
    }

    // Compute byte offset for each instruction before optimization.
    let mut byte_offsets = Vec::with_capacity(len + 1);
    let mut running = 0usize;
    for instr in instructions.iter() {
        byte_offsets.push(running);
        running += instr.encode().len();
    }
    // Sentinel: byte offset after the last instruction.
    byte_offsets.push(running);

    // Build reverse map: byte_offset → instruction_index for label resolution.
    // Labels point to instruction boundaries, so we map each boundary to its index.
    let mut byte_to_idx = std::collections::HashMap::new();
    for (idx, &off) in byte_offsets.iter().enumerate() {
        byte_to_idx.entry(off).or_insert(idx);
    }

    // Mark instructions for removal (true = keep, false = remove).
    let mut keep = vec![true; len];

    for i in 0..len {
        if !keep[i] {
            continue;
        }
        if !matches!(instructions[i], Instruction::Fallthrough) {
            continue;
        }

        // Pattern 1: Consecutive Fallthroughs — remove all but the last.
        // Pattern 2: Fallthrough followed by Jump or Trap — remove the Fallthrough.
        if i + 1 < len {
            match &instructions[i + 1] {
                Instruction::Fallthrough | Instruction::Jump { .. } | Instruction::Trap => {
                    keep[i] = false;
                }
                _ => {}
            }
        }
    }

    // Build old→new index remap.
    let mut remap = vec![0usize; len + 1]; // +1 for sentinel
    let mut new_idx = 0;
    for (old_idx, &kept) in keep.iter().enumerate() {
        remap[old_idx] = new_idx;
        if kept {
            new_idx += 1;
        }
    }
    remap[len] = new_idx; // sentinel maps to new end

    // If nothing was removed, skip the rest.
    if new_idx == len {
        return;
    }

    // Compact instructions in-place.
    let mut write = 0;
    for read in 0..len {
        if keep[read] {
            if write != read {
                instructions[write] = instructions[read].clone();
            }
            write += 1;
        }
    }
    instructions.truncate(write);

    // Recompute byte offsets for the compacted instruction stream.
    let mut new_byte_offsets = Vec::with_capacity(instructions.len() + 1);
    let mut new_running = 0usize;
    for instr in instructions.iter() {
        new_byte_offsets.push(new_running);
        new_running += instr.encode().len();
    }
    new_byte_offsets.push(new_running);

    // Update labels: map old byte offset → old instr index → new instr index → new byte offset.
    for label in labels.iter_mut().flatten() {
        if let Some(&old_idx) = byte_to_idx.get(label) {
            let new_i = remap[old_idx.min(len)];
            *label = new_byte_offsets[new_i.min(instructions.len())];
        }
    }

    // Remap all fixup indices.
    for (instr_idx, _label) in fixups.iter_mut() {
        *instr_idx = remap[*instr_idx];
    }
    for fixup in call_fixups.iter_mut() {
        fixup.return_addr_instr = remap[fixup.return_addr_instr];
        fixup.jump_instr = remap[fixup.jump_instr];
    }
    for fixup in indirect_call_fixups.iter_mut() {
        fixup.return_addr_instr = remap[fixup.return_addr_instr];
        fixup.jump_ind_instr = remap[fixup.jump_ind_instr];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collapse_consecutive_fallthroughs() {
        let mut instrs = vec![
            Instruction::Fallthrough,
            Instruction::Fallthrough,
            Instruction::Fallthrough,
            Instruction::LoadImm { reg: 0, value: 42 },
        ];
        let mut fixups = vec![];
        let mut call_fixups = vec![];
        let mut indirect_call_fixups = vec![];
        let mut labels = vec![];

        optimize(
            &mut instrs,
            &mut fixups,
            &mut call_fixups,
            &mut indirect_call_fixups,
            &mut labels,
        );

        assert_eq!(instrs.len(), 2);
        assert!(matches!(instrs[0], Instruction::Fallthrough));
        assert!(matches!(
            instrs[1],
            Instruction::LoadImm {
                reg: 0,
                value: 42
            }
        ));
    }

    #[test]
    fn remove_fallthrough_before_jump() {
        let mut instrs = vec![
            Instruction::LoadImm { reg: 0, value: 1 },
            Instruction::Fallthrough,
            Instruction::Jump { offset: 0 },
        ];
        let mut fixups = vec![(2, 0)]; // jump at index 2
        let mut call_fixups = vec![];
        let mut indirect_call_fixups = vec![];
        let mut labels = vec![];

        optimize(
            &mut instrs,
            &mut fixups,
            &mut call_fixups,
            &mut indirect_call_fixups,
            &mut labels,
        );

        assert_eq!(instrs.len(), 2);
        assert!(matches!(instrs[0], Instruction::LoadImm { .. }));
        assert!(matches!(instrs[1], Instruction::Jump { .. }));
        assert_eq!(fixups[0].0, 1); // remapped from 2 to 1
    }

    #[test]
    fn remove_fallthrough_before_trap() {
        let mut instrs = vec![Instruction::Fallthrough, Instruction::Trap];
        let mut fixups = vec![];
        let mut call_fixups = vec![];
        let mut indirect_call_fixups = vec![];
        let mut labels = vec![];

        optimize(
            &mut instrs,
            &mut fixups,
            &mut call_fixups,
            &mut indirect_call_fixups,
            &mut labels,
        );

        assert_eq!(instrs.len(), 1);
        assert!(matches!(instrs[0], Instruction::Trap));
    }

    #[test]
    fn remaps_call_fixups() {
        let mut instrs = vec![
            Instruction::Fallthrough,
            Instruction::Fallthrough,
            Instruction::LoadImm { reg: 0, value: 0 }, // return_addr_instr
            Instruction::Jump { offset: 0 },            // jump_instr
        ];
        let mut fixups = vec![];
        let mut call_fixups = vec![LlvmCallFixup {
            return_addr_instr: 2,
            jump_instr: 3,
            target_func: 0,
        }];
        let mut indirect_call_fixups = vec![];
        let mut labels = vec![];

        optimize(
            &mut instrs,
            &mut fixups,
            &mut call_fixups,
            &mut indirect_call_fixups,
            &mut labels,
        );

        // One fallthrough removed (first one before second fallthrough)
        assert_eq!(instrs.len(), 3);
        assert_eq!(call_fixups[0].return_addr_instr, 1);
        assert_eq!(call_fixups[0].jump_instr, 2);
    }

    #[test]
    fn no_op_when_nothing_to_optimize() {
        let mut instrs = vec![
            Instruction::LoadImm { reg: 0, value: 1 },
            Instruction::LoadImm { reg: 1, value: 2 },
            Instruction::Add64 {
                dst: 0,
                src1: 0,
                src2: 1,
            },
        ];
        let mut fixups = vec![];
        let mut call_fixups = vec![];
        let mut indirect_call_fixups = vec![];
        let mut labels = vec![];

        optimize(
            &mut instrs,
            &mut fixups,
            &mut call_fixups,
            &mut indirect_call_fixups,
            &mut labels,
        );

        assert_eq!(instrs.len(), 3);
    }

    #[test]
    fn updates_label_byte_offsets() {
        // Fallthrough (1 byte), Fallthrough (1 byte), LoadImm (6 bytes)
        // Label at byte offset 2 (pointing to LoadImm)
        let mut instrs = vec![
            Instruction::Fallthrough,
            Instruction::Fallthrough,
            Instruction::LoadImm { reg: 0, value: 42 },
        ];
        let mut fixups = vec![];
        let mut call_fixups = vec![];
        let mut indirect_call_fixups = vec![];
        // Label points to byte offset 2 (start of LoadImm, after two 1-byte Fallthroughs)
        let mut labels = vec![Some(2usize)];

        optimize(
            &mut instrs,
            &mut fixups,
            &mut call_fixups,
            &mut indirect_call_fixups,
            &mut labels,
        );

        // First Fallthrough removed, so: Fallthrough (1 byte), LoadImm (6 bytes)
        assert_eq!(instrs.len(), 2);
        // Label should now point to byte offset 1 (start of LoadImm after one Fallthrough)
        assert_eq!(labels[0], Some(1));
    }
}
