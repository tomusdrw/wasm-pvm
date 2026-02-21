// Peephole optimizer for PVM instructions.
//
// Runs before fixup resolution to remove redundant instructions.
// Builds an index remap table to update fixup references and label byte offsets.

use std::collections::HashSet;

use super::Instruction;
use crate::llvm_backend::{LlvmCallFixup, LlvmIndirectCallFixup};

/// Compact the instruction stream by removing entries where `keep[i]` is false.
///
/// Updates all fixup indices and label byte offsets to match the compacted stream.
/// Returns early (no-op) if nothing was removed.
fn compact_instructions(
    instructions: &mut Vec<Instruction>,
    keep: &[bool],
    fixups: &mut [(usize, usize)],
    call_fixups: &mut [LlvmCallFixup],
    indirect_call_fixups: &mut [LlvmIndirectCallFixup],
    labels: &mut [Option<usize>],
) {
    let len = keep.len();
    debug_assert_eq!(len, instructions.len());

    // Cache encoded length per instruction to avoid repeated encode() calls.
    let encoded_lengths: Vec<usize> = instructions.iter().map(|i| i.encode().len()).collect();

    // Compute byte offset for each instruction before compaction.
    let mut byte_offsets = Vec::with_capacity(len + 1);
    let mut running = 0usize;
    for &enc_len in &encoded_lengths {
        byte_offsets.push(running);
        running += enc_len;
    }
    byte_offsets.push(running);

    // Build reverse map: byte_offset → instruction_index for label resolution.
    let mut byte_to_idx = std::collections::HashMap::new();
    for (idx, &off) in byte_offsets.iter().enumerate() {
        byte_to_idx.entry(off).or_insert(idx);
    }

    // Build old→new index remap.
    let mut remap = vec![0usize; len + 1];
    let mut new_idx = 0;
    for (old_idx, &kept) in keep.iter().enumerate() {
        remap[old_idx] = new_idx;
        if kept {
            new_idx += 1;
        }
    }
    remap[len] = new_idx;

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

    // Recompute byte offsets for the compacted stream using cached lengths.
    let mut new_byte_offsets = Vec::with_capacity(instructions.len() + 1);
    let mut new_running = 0usize;
    for (old_idx, &kept) in keep.iter().enumerate() {
        if kept {
            new_byte_offsets.push(new_running);
            new_running += encoded_lengths[old_idx];
        }
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

/// Eliminate dead SP-relative stores.
///
/// A `StoreIndU64` with `base == STACK_PTR_REG` is dead if no instruction in the
/// function loads from the same SP-relative offset. Only `StoreIndU64` is targeted
/// because the compiler always uses it for stack slot writes (`store_to_slot`).
///
/// Must be called **before** `resolve_fixups()`.
pub fn eliminate_dead_stores(
    instructions: &mut Vec<Instruction>,
    fixups: &mut [(usize, usize)],
    call_fixups: &mut [LlvmCallFixup],
    indirect_call_fixups: &mut [LlvmIndirectCallFixup],
    labels: &mut [Option<usize>],
) {
    const SP: u8 = crate::abi::STACK_PTR_REG;

    let len = instructions.len();
    if len == 0 {
        return;
    }

    // Pass 1: Collect all SP-relative load offsets (the "read" set).
    let mut read_offsets = HashSet::new();
    for instr in instructions.iter() {
        match instr {
            Instruction::LoadIndU64 {
                base: SP, offset, ..
            }
            | Instruction::LoadIndU32 {
                base: SP, offset, ..
            }
            | Instruction::LoadIndU8 {
                base: SP, offset, ..
            }
            | Instruction::LoadIndU16 {
                base: SP, offset, ..
            } => {
                read_offsets.insert(*offset);
            }
            _ => {}
        }
    }

    // Pass 2: Mark SP-relative StoreIndU64 to unread offsets for removal.
    let mut keep = vec![true; len];
    for (i, instr) in instructions.iter().enumerate() {
        if let Instruction::StoreIndU64 {
            base: SP, offset, ..
        } = instr
            && !read_offsets.contains(offset)
        {
            keep[i] = false;
        }
    }

    compact_instructions(
        instructions,
        &keep,
        fixups,
        call_fixups,
        indirect_call_fixups,
        labels,
    );
}

/// Returns true if the instruction is a 32-bit producer that sign-extends its result.
/// PVM 32-bit operations write `u32SignExtend(result)` to the destination register,
/// so a subsequent `AddImm32(x, x, 0)` truncation is redundant.
fn is_32bit_sign_extending_producer(instr: &Instruction) -> bool {
    matches!(
        instr,
        Instruction::Add32 { .. }
            | Instruction::Sub32 { .. }
            | Instruction::Mul32 { .. }
            | Instruction::DivU32 { .. }
            | Instruction::DivS32 { .. }
            | Instruction::RemU32 { .. }
            | Instruction::RemS32 { .. }
            | Instruction::ShloL32 { .. }
            | Instruction::ShloR32 { .. }
            | Instruction::SharR32 { .. }
            | Instruction::AddImm32 { .. }
            | Instruction::CountSetBits32 { .. }
            | Instruction::LeadingZeroBits32 { .. }
            | Instruction::TrailingZeroBits32 { .. }
            | Instruction::SignExtend8 { .. }
            | Instruction::SignExtend16 { .. }
    )
}

/// Run peephole optimizations on a function's instruction stream.
///
/// Must be called **before** `resolve_fixups()` since it removes instructions
/// and remaps all fixup indices and label byte offsets accordingly.
pub fn optimize(
    instructions: &mut Vec<Instruction>,
    fixups: &mut [(usize, usize)],
    call_fixups: &mut [LlvmCallFixup],
    indirect_call_fixups: &mut [LlvmIndirectCallFixup],
    labels: &mut [Option<usize>],
) {
    let len = instructions.len();
    if len == 0 {
        return;
    }

    // 1. Optimize address calculations (fuse AddImm + Load/Store).
    // This updates instructions in-place and doesn't remove any, so fixups are fine.
    optimize_address_calculation(instructions, labels);

    // 2. Eliminate dead code (unused registers).
    // This marks instructions for removal.
    eliminate_dead_code(instructions, fixups, call_fixups, indirect_call_fixups, labels);

    // 3. Simple peephole patterns (redundant fallthroughs).
    // Mark instructions for removal (true = keep, false = remove).
    let len = instructions.len();
    let mut keep = vec![true; len];

    for i in 0..len {
        if !keep[i] {
            continue;
        }

        // Pattern 1: Consecutive Fallthroughs — remove all but the last.
        // Pattern 2: Fallthrough followed by Jump or Trap — remove the Fallthrough.
        if matches!(instructions[i], Instruction::Fallthrough) && i + 1 < len {
            match &instructions[i + 1] {
                Instruction::Fallthrough | Instruction::Jump { .. } | Instruction::Trap => {
                    keep[i] = false;
                }
                _ => {}
            }
        }

        // Pattern 3: Redundant truncation — remove AddImm32(x, x, 0) after a 32-bit producer.
        // PVM 32-bit operations already sign-extend their results, so truncation is a NOP.
        // The truncation target must match the producer's destination register.
        if i + 1 < len
            && is_32bit_sign_extending_producer(&instructions[i])
            && let Instruction::AddImm32 { dst, src, value: 0 } = &instructions[i + 1]
            && let Some(prod_dst) = instructions[i].dest_reg()
            && *dst == prod_dst
            && *src == prod_dst
        {
            keep[i + 1] = false;
        }
    }

    compact_instructions(
        instructions,
        &keep,
        fixups,
        call_fixups,
        indirect_call_fixups,
        labels,
    );
}


/// Optimize address calculations by fusing `AddImm` into `LoadInd`/`StoreInd` offsets.
/// Also performs simple copy propagation for `MoveReg`.
///
/// Pattern:
/// 1. `MoveReg dst=A, src=B` → Record A is alias of B.
/// 2. `AddImm dst=A, src=B, val=C` → Record A is B + C.
/// 3. `LoadInd base=A, offset=D` → Rewrite as `LoadInd base=B, offset=C+D`.
///
/// This pass assumes sequential execution within basic blocks (reset at labels/branches).
/// It updates instructions in-place.
pub fn optimize_address_calculation(
    instructions: &mut Vec<Instruction>,
    _labels: &[Option<usize>], // labels are used to reset state (block boundaries)
) {
    // Map register -> (base_register, offset)
    // entry[R] = Some((Base, Off)) means value of R is (value of Base) + Off.
    let mut state = [None; 13];
    
    // Track labels to reset state at block boundaries.
    let mut label_offsets = HashSet::new();
    for label in _labels.iter().flatten() {
        label_offsets.insert(*label);
    }

    let mut byte_offset = 0;

    for i in 0..instructions.len() {
        // If this instruction is a label target, reset state.
        if label_offsets.contains(&byte_offset) {
            state = [None; 13];
        }
        
        let encoded_len = instructions[i].encode().len();
        let instr = &mut instructions[i];

        // 1. Try to rewrite usage of registers based on state.
        match instr {
            Instruction::LoadIndU8 { base, offset, .. }
            | Instruction::LoadIndI8 { base, offset, .. }
            | Instruction::LoadIndU16 { base, offset, .. }
            | Instruction::LoadIndI16 { base, offset, .. }
            | Instruction::LoadIndU32 { base, offset, .. }
            | Instruction::LoadIndU64 { base, offset, .. }
            | Instruction::StoreIndU8 { base, offset, .. }
            | Instruction::StoreIndU16 { base, offset, .. }
            | Instruction::StoreIndU32 { base, offset, .. }
            | Instruction::StoreIndU64 { base, offset, .. } => {
                if let Some((tracked_base, tracked_off)) = state[*base as usize] {
                    // Check if offset + tracked_off fits in i32
                    if let Some(new_off) = offset.checked_add(tracked_off) {
                        *base = tracked_base;
                        *offset = new_off;
                    }
                }
            }
            Instruction::JumpInd { reg, offset, .. } => {
                if let Some((tracked_base, tracked_off)) = state[*reg as usize] {
                    if let Some(new_off) = offset.checked_add(tracked_off) {
                        *reg = tracked_base;
                        *offset = new_off;
                    }
                }
            }
            Instruction::AddImm32 { src, value, .. } 
            | Instruction::AddImm64 { src, value, .. } => {
                if let Some((tracked_base, tracked_off)) = state[*src as usize] {
                    if let Some(new_val) = value.checked_add(tracked_off) {
                        *src = tracked_base;
                        *value = new_val;
                    }
                }
            }
            _ => {}
        }

        // 2. Update state based on destination.
        let dest = instr.dest_reg();
        
        // Invalidate any state that depends on the overwritten register.
        if let Some(dst) = dest {
            for s in state.iter_mut() {
                if let Some((base, _)) = s {
                    if *base == dst {
                        *s = None;
                    }
                }
            }
        }

        // Set new state for dst.
        match instr {
            Instruction::MoveReg { dst, src } => {
                if dst != src {
                    state[*dst as usize] = state[*src as usize].or(Some((*src, 0)));
                } else {
                    state[*dst as usize] = None;
                }
            }
            Instruction::AddImm32 { dst, src, value } 
            | Instruction::AddImm64 { dst, src, value } => {
                if dst != src {
                    // dst = src + value
                    // If src is tracked (Base, Off), then dst = (Base, Off + value).
                    // Else dst = (src, value).
                    // Note: src/value might have been optimized in step 1 (folding constant).
                    // If so, src is already Base.
                    state[*dst as usize] = Some((*src, *value));
                } else {
                    // In-place update (A = A + imm).
                    // Original value of A is lost, so we cannot track A as an alias of (A + imm).
                    state[*dst as usize] = None;
                }
            }
            _ => {
                if let Some(dst) = dest {
                    state[dst as usize] = Some((dst, 0)); // Identity
                }
            }
        }
        
        byte_offset += encoded_len;
    }
}

/// Eliminate dead code (instructions defining unused registers).
///
/// Iterates backwards to track liveness.
/// Must be called **before** `resolve_fixups()` and after other optimizations.
pub fn eliminate_dead_code(
    instructions: &mut Vec<Instruction>,
    fixups: &mut [(usize, usize)],
    call_fixups: &mut [LlvmCallFixup],
    indirect_call_fixups: &mut [LlvmIndirectCallFixup],
    labels: &mut [Option<usize>],
) {
    let len = instructions.len();
    if len == 0 {
        return;
    }

    let mut keep = vec![true; len];
    let mut needed_regs = [true; 13]; // Default to all needed (conservative)
    
    // Compute byte offsets for label matching
    let mut offsets = Vec::with_capacity(len);
    let mut running = 0;
    for instr in instructions.iter() {
        offsets.push(running);
        running += instr.encode().len();
    }
    let mut label_offsets = HashSet::new();
    for label in labels.iter().flatten() {
        label_offsets.insert(*label);
    }
    
    for i in (0..len).rev() {
        // If this is a label target, reset liveness to ALL (conservative).
        if label_offsets.contains(&offsets[i]) {
            needed_regs = [true; 13];
        }
        
        let instr = &instructions[i];
        let mut remove = false;
        
        if instr.is_terminating() {
             needed_regs = [true; 13];
        } else {
            match instr {
                Instruction::StoreIndU8{..} | Instruction::StoreIndU16{..} | Instruction::StoreIndU32{..} | Instruction::StoreIndU64{..} | Instruction::Ecalli{..} | Instruction::Trap | Instruction::Sbrk{..} => {
                    // Side effects, keep.
                }
                _ => {
                    if let Some(dst) = instr.dest_reg() {
                        if !needed_regs[dst as usize] {
                            remove = true;
                        } else {
                            needed_regs[dst as usize] = false;
                        }
                    }
                }
            }
        }
        
        if remove {
            keep[i] = false;
        } else {
            // Mark sources needed
            for src in instr.src_regs() {
                if let Some(reg) = src {
                    needed_regs[reg as usize] = true;
                }
            }
        }
    }

    compact_instructions(
        instructions,
        &keep,
        fixups,
        call_fixups,
        indirect_call_fixups,
        labels,
    );
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
            Instruction::LoadImm { reg: 0, value: 42 }
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
            Instruction::Jump { offset: 0 },           // jump_instr
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

    // ── Dead store elimination tests ──



    #[test]
    fn dse_removes_unread_sp_store() {
        // Store to SP+16 but never load from SP+16 → dead store removed.
        let mut instrs = vec![
            Instruction::LoadImm { reg: 2, value: 42 },
            Instruction::StoreIndU64 {
                base: SP,
                src: 2,
                offset: 16,
            },
            Instruction::LoadImm { reg: 3, value: 99 },
        ];
        let mut fixups = vec![];
        let mut call_fixups = vec![];
        let mut indirect_call_fixups = vec![];
        let mut labels = vec![];

        eliminate_dead_stores(
            &mut instrs,
            &mut fixups,
            &mut call_fixups,
            &mut indirect_call_fixups,
            &mut labels,
        );

        assert_eq!(instrs.len(), 2);
        assert!(matches!(
            instrs[0],
            Instruction::LoadImm { reg: 2, value: 42 }
        ));
        assert!(matches!(
            instrs[1],
            Instruction::LoadImm { reg: 3, value: 99 }
        ));
    }

    #[test]
    fn dse_keeps_read_sp_store() {
        // Store to SP+8 and load from SP+8 → store is kept.
        let mut instrs = vec![
            Instruction::StoreIndU64 {
                base: SP,
                src: 2,
                offset: 8,
            },
            Instruction::LoadIndU64 {
                dst: 3,
                base: SP,
                offset: 8,
            },
        ];
        let mut fixups = vec![];
        let mut call_fixups = vec![];
        let mut indirect_call_fixups = vec![];
        let mut labels = vec![];

        eliminate_dead_stores(
            &mut instrs,
            &mut fixups,
            &mut call_fixups,
            &mut indirect_call_fixups,
            &mut labels,
        );

        assert_eq!(instrs.len(), 2);
    }

    #[test]
    fn dse_ignores_non_sp_stores() {
        // Store to reg 5 (not SP) at offset 16, never loaded → NOT removed (memory side effect).
        let mut instrs = vec![
            Instruction::StoreIndU64 {
                base: 5,
                src: 2,
                offset: 16,
            },
            Instruction::LoadImm { reg: 3, value: 0 },
        ];
        let mut fixups = vec![];
        let mut call_fixups = vec![];
        let mut indirect_call_fixups = vec![];
        let mut labels = vec![];

        eliminate_dead_stores(
            &mut instrs,
            &mut fixups,
            &mut call_fixups,
            &mut indirect_call_fixups,
            &mut labels,
        );

        assert_eq!(instrs.len(), 2);
    }

    #[test]
    fn dse_removes_unread_sp_store() {
        // Store to SP+16 but never load from SP+16 → dead store removed.
        let mut instrs = vec![
            Instruction::LoadImm { reg: 2, value: 42 },
            Instruction::StoreIndU64 {
                base: SP,
                src: 2,
                offset: 16,
            },
            Instruction::LoadImm { reg: 3, value: 99 },
        ];
        let mut fixups = vec![];
        let mut call_fixups = vec![];
        let mut indirect_call_fixups = vec![];
        let mut labels = vec![];

        eliminate_dead_stores(
            &mut instrs,
            &mut fixups,
            &mut call_fixups,
            &mut indirect_call_fixups,
            &mut labels,
        );

        assert_eq!(instrs.len(), 2);
        assert!(matches!(
            instrs[0],
            Instruction::LoadImm { reg: 2, value: 42 }
        ));
        assert!(matches!(
            instrs[1],
            Instruction::LoadImm { reg: 3, value: 99 }
        ));
    }

    #[test]
    fn dse_no_op_when_nothing_to_eliminate() {
        let mut instrs = vec![
            Instruction::StoreIndU64 {
                base: SP,
                src: 2,
                offset: 8,
            },
            Instruction::LoadIndU64 {
                dst: 3,
                base: SP,
                offset: 8,
            },
            Instruction::LoadImm { reg: 0, value: 0 },
        ];
        let original_len = instrs.len();
        let mut fixups = vec![];
        let mut call_fixups = vec![];
        let mut indirect_call_fixups = vec![];
        let mut labels = vec![];

        eliminate_dead_stores(
            &mut instrs,
            &mut fixups,
            &mut call_fixups,
            &mut indirect_call_fixups,
            &mut labels,
        );

        assert_eq!(instrs.len(), original_len);
    }

    // ── Truncation NOP removal tests ──

    #[test]
    fn removes_redundant_trunc_after_add32() {
        let mut instrs = vec![
            Instruction::Add32 {
                dst: 5,
                src1: 3,
                src2: 4,
            },
            Instruction::AddImm32 {
                dst: 5,
                src: 5,
                value: 0,
            },
            Instruction::LoadImm { reg: 0, value: 99 },
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
        assert!(matches!(instrs[0], Instruction::Add32 { dst: 5, .. }));
        assert!(matches!(instrs[1], Instruction::LoadImm { reg: 0, .. }));
    }

    #[test]
    fn removes_redundant_trunc_after_mul32() {
        let mut instrs = vec![
            Instruction::Mul32 {
                dst: 5,
                src1: 3,
                src2: 4,
            },
            Instruction::AddImm32 {
                dst: 5,
                src: 5,
                value: 0,
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

        assert_eq!(instrs.len(), 1);
        assert!(matches!(instrs[0], Instruction::Mul32 { dst: 5, .. }));
    }

    #[test]
    fn keeps_trunc_when_registers_differ() {
        // AddImm32 dst != producer dst → not redundant.
        let mut instrs = vec![
            Instruction::Add32 {
                dst: 5,
                src1: 3,
                src2: 4,
            },
            Instruction::AddImm32 {
                dst: 6,
                src: 6,
                value: 0,
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

        assert_eq!(instrs.len(), 2);
    }

    #[test]
    fn keeps_trunc_with_nonzero_value() {
        // AddImm32(x, x, 1) is NOT a truncation NOP.
        let mut instrs = vec![
            Instruction::Add32 {
                dst: 5,
                src1: 3,
                src2: 4,
            },
            Instruction::AddImm32 {
                dst: 5,
                src: 5,
                value: 1,
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

        assert_eq!(instrs.len(), 2);
    }

    #[test]
    fn keeps_trunc_after_64bit_producer() {
        // Add64 is NOT a 32-bit producer → AddImm32 truncation is meaningful.
        let mut instrs = vec![
            Instruction::Add64 {
                dst: 5,
                src1: 3,
                src2: 4,
            },
            Instruction::AddImm32 {
                dst: 5,
                src: 5,
                value: 0,
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

        assert_eq!(instrs.len(), 2);
    }

    #[test]
    fn removes_trunc_after_shlol32() {
        let mut instrs = vec![
            Instruction::ShloL32 {
                dst: 5,
                src1: 3,
                src2: 4,
            },
            Instruction::AddImm32 {
                dst: 5,
                src: 5,
                value: 0,
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

        assert_eq!(instrs.len(), 1);
    }

    #[test]
    fn keeps_trunc_across_intervening_store() {
        // StoreIndU64 between producer and truncation → pattern does NOT match.
        // This is a known limitation: we only match directly adjacent pairs.
        let mut instrs = vec![
            Instruction::Add32 {
                dst: 5,
                src1: 3,
                src2: 4,
            },
            Instruction::StoreIndU64 {
                base: 1,
                src: 5,
                offset: 8,
            },
            Instruction::AddImm32 {
                dst: 5,
                src: 5,
                value: 0,
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

        // All 3 instructions kept — the pattern requires direct adjacency.
        assert_eq!(instrs.len(), 3);
    }

    #[test]
    fn trunc_removal_remaps_fixups() {
        // Fixup at index 3 should be remapped to 2 after truncation NOP at index 1 is removed.
        let mut instrs = vec![
            Instruction::Add32 {
                dst: 5,
                src1: 3,
                src2: 4,
            },
            Instruction::AddImm32 {
                dst: 5,
                src: 5,
                value: 0,
            },
            Instruction::LoadImm { reg: 0, value: 0 },
            Instruction::Jump { offset: 0 },
        ];
        let mut fixups = vec![(3, 0)];
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
        assert_eq!(fixups[0].0, 2); // remapped from 3 to 2
    }

    #[test]
    fn dse_keeps_store_if_smaller_load_reads_same_offset() {
        // Store u64 to SP+8, load u32 from SP+8 → store is kept (offset is in read set).
        let mut instrs = vec![
            Instruction::StoreIndU64 {
                base: SP,
                src: 2,
                offset: 8,
            },
            Instruction::LoadIndU32 {
                dst: 3,
                base: SP,
                offset: 8,
            },
        ];
        let mut fixups = vec![];
        let mut call_fixups = vec![];
        let mut indirect_call_fixups = vec![];
        let mut labels = vec![];

        eliminate_dead_stores(
            &mut instrs,
            &mut fixups,
            &mut call_fixups,
            &mut indirect_call_fixups,
            &mut labels,
        );

        assert_eq!(instrs.len(), 2);
    }
}
