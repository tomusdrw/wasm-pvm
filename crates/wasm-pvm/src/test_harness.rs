//! Test harness for wasm-pvm unit tests
//!
//! This module provides utilities for testing the WASM to PVM compiler.
//! It is only available when running tests (`#[cfg(test)]`).
//!
//! # Example
//!
//! ```rust
//! use wasm_pvm::test_harness::*;
//!
//! #[test]
//! fn test_simple_add() {
//!     let wasm = wat_to_wasm(r#"
//!         (module
//!             (func (export "main") (param i32 i32) (result i32)
//!                 local.get 0
//!                 local.get 1
//!                 i32.add
//!             )
//!         )
//!     "#).expect("Failed to parse WAT");
//!
//!     let program = compile(&wasm).expect("Failed to compile");
//!     let code = program.code();
//!     let instructions = code.instructions();
//!
//!     // Assert on generated instructions
//!     assert_has_pattern(instructions, &[
//!         InstructionPattern::LoadImm { reg: Pat::Any, value: Pat::Any },
//!         InstructionPattern::LoadImm { reg: Pat::Any, value: Pat::Any },
//!         InstructionPattern::Add32 { dst: Pat::Any, src1: Pat::Any, src2: Pat::Any },
//!     ]);
//! }
//! ```

#![allow(
    clippy::match_same_arms,
    clippy::must_use_candidate,
    clippy::manual_assert,
    clippy::missing_panics_doc,
    clippy::uninlined_format_args
)]

use crate::pvm::{Instruction, Opcode};
use crate::{Error, Result, SpiProgram, compile};

/// Parse WAT (WebAssembly Text) format to WASM binary
pub fn wat_to_wasm(wat: &str) -> Result<Vec<u8>> {
    wat::parse_str(wat).map_err(|e| Error::Internal(format!("WAT parse error: {e}")))
}

/// Compile WAT directly to a SPI program
pub fn compile_wat(wat: &str) -> Result<SpiProgram> {
    let wasm = wat_to_wasm(wat)?;
    compile(&wasm)
}

/// Extract the instruction sequence from a SPI program
pub fn extract_instructions(program: &SpiProgram) -> Vec<Instruction> {
    program.code().instructions().to_vec()
}

/// Pattern matching for instruction fields
#[derive(Debug, Clone)]
pub enum Pat<T> {
    /// Match any value
    Any,
    /// Match exact value
    Exact(T),
    /// Match if value satisfies predicate
    Predicate(fn(&T) -> bool),
}

impl<T: PartialEq> Pat<T> {
    /// Check if a value matches this pattern
    pub fn matches(&self, value: &T) -> bool {
        match self {
            Pat::Any => true,
            Pat::Exact(expected) => value == expected,
            Pat::Predicate(pred) => pred(value),
        }
    }
}

/// Pattern for matching instructions in tests
///
/// This allows flexible matching of instruction sequences where some
/// fields can be wildcards or predicates.
#[derive(Debug, Clone)]
pub enum InstructionPattern {
    /// Match any instruction
    Any,
    /// Match a specific instruction kind with pattern fields
    LoadImm {
        reg: Pat<u8>,
        value: Pat<i32>,
    },
    LoadImm64 {
        reg: Pat<u8>,
        value: Pat<u64>,
    },
    Add32 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    Sub32 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    Mul32 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    DivU32 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    DivS32 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    RemU32 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    RemS32 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    Add64 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    Sub64 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    Mul64 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    DivU64 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    DivS64 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    RemU64 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    RemS64 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    And {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    Or {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    Xor {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    ShloL32 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    ShloR32 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    SharR32 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    ShloL64 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    ShloR64 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    SharR64 {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    SetLtU {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    SetLtS {
        dst: Pat<u8>,
        src1: Pat<u8>,
        src2: Pat<u8>,
    },
    Jump {
        offset: Pat<i32>,
    },
    JumpInd {
        reg: Pat<u8>,
        offset: Pat<i32>,
    },
    AddImm32 {
        dst: Pat<u8>,
        src: Pat<u8>,
        value: Pat<i32>,
    },
    AddImm64 {
        dst: Pat<u8>,
        src: Pat<u8>,
        value: Pat<i32>,
    },
    BranchLtU {
        reg1: Pat<u8>,
        reg2: Pat<u8>,
        offset: Pat<i32>,
    },
    BranchGeU {
        reg1: Pat<u8>,
        reg2: Pat<u8>,
        offset: Pat<i32>,
    },
    BranchEqImm {
        reg: Pat<u8>,
        value: Pat<i32>,
        offset: Pat<i32>,
    },
    BranchNeImm {
        reg: Pat<u8>,
        value: Pat<i32>,
        offset: Pat<i32>,
    },
    BranchGeSImm {
        reg: Pat<u8>,
        value: Pat<i32>,
        offset: Pat<i32>,
    },
    LoadIndU8 {
        dst: Pat<u8>,
        base: Pat<u8>,
        offset: Pat<i32>,
    },
    LoadIndI8 {
        dst: Pat<u8>,
        base: Pat<u8>,
        offset: Pat<i32>,
    },
    LoadIndU16 {
        dst: Pat<u8>,
        base: Pat<u8>,
        offset: Pat<i32>,
    },
    LoadIndI16 {
        dst: Pat<u8>,
        base: Pat<u8>,
        offset: Pat<i32>,
    },
    LoadIndU32 {
        dst: Pat<u8>,
        base: Pat<u8>,
        offset: Pat<i32>,
    },
    LoadIndU64 {
        dst: Pat<u8>,
        base: Pat<u8>,
        offset: Pat<i32>,
    },
    StoreIndU8 {
        base: Pat<u8>,
        src: Pat<u8>,
        offset: Pat<i32>,
    },
    StoreIndU16 {
        base: Pat<u8>,
        src: Pat<u8>,
        offset: Pat<i32>,
    },
    StoreIndU32 {
        base: Pat<u8>,
        src: Pat<u8>,
        offset: Pat<i32>,
    },
    StoreIndU64 {
        base: Pat<u8>,
        src: Pat<u8>,
        offset: Pat<i32>,
    },
    CountSetBits32 {
        dst: Pat<u8>,
        src: Pat<u8>,
    },
    CountSetBits64 {
        dst: Pat<u8>,
        src: Pat<u8>,
    },
    LeadingZeroBits32 {
        dst: Pat<u8>,
        src: Pat<u8>,
    },
    LeadingZeroBits64 {
        dst: Pat<u8>,
        src: Pat<u8>,
    },
    TrailingZeroBits32 {
        dst: Pat<u8>,
        src: Pat<u8>,
    },
    TrailingZeroBits64 {
        dst: Pat<u8>,
        src: Pat<u8>,
    },
    SignExtend8 {
        dst: Pat<u8>,
        src: Pat<u8>,
    },
    SignExtend16 {
        dst: Pat<u8>,
        src: Pat<u8>,
    },
    ZeroExtend16 {
        dst: Pat<u8>,
        src: Pat<u8>,
    },
    Trap,
    Fallthrough,
}

impl InstructionPattern {
    /// Check if an instruction matches this pattern
    pub fn matches(&self, instr: &Instruction) -> bool {
        use InstructionPattern as P;

        match (self, instr) {
            (P::Any, _) => true,
            (P::Trap, Instruction::Trap) => true,
            (P::Fallthrough, Instruction::Fallthrough) => true,
            (
                P::LoadImm {
                    reg: r_pat,
                    value: v_pat,
                },
                Instruction::LoadImm { reg, value },
            ) => r_pat.matches(reg) && v_pat.matches(value),
            (
                P::LoadImm64 {
                    reg: r_pat,
                    value: v_pat,
                },
                Instruction::LoadImm64 { reg, value },
            ) => r_pat.matches(reg) && v_pat.matches(value),
            (
                P::Add32 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::Add32 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::Sub32 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::Sub32 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::Mul32 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::Mul32 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::DivU32 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::DivU32 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::DivS32 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::DivS32 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::RemU32 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::RemU32 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::RemS32 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::RemS32 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::Add64 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::Add64 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::Sub64 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::Sub64 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::Mul64 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::Mul64 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::DivU64 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::DivU64 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::DivS64 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::DivS64 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::RemU64 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::RemU64 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::RemS64 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::RemS64 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::And {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::And { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::Or {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::Or { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::Xor {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::Xor { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::ShloL32 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::ShloL32 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::ShloR32 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::ShloR32 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::SharR32 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::SharR32 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::ShloL64 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::ShloL64 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::ShloR64 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::ShloR64 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::SharR64 {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::SharR64 { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::SetLtU {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::SetLtU { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (
                P::SetLtS {
                    dst: d_pat,
                    src1: s1_pat,
                    src2: s2_pat,
                },
                Instruction::SetLtS { dst, src1, src2 },
            ) => d_pat.matches(dst) && s1_pat.matches(src1) && s2_pat.matches(src2),
            (P::Jump { offset: o_pat }, Instruction::Jump { offset }) => o_pat.matches(offset),
            (
                P::JumpInd {
                    reg: r_pat,
                    offset: o_pat,
                },
                Instruction::JumpInd { reg, offset },
            ) => r_pat.matches(reg) && o_pat.matches(offset),
            (
                P::AddImm32 {
                    dst: d_pat,
                    src: s_pat,
                    value: v_pat,
                },
                Instruction::AddImm32 { dst, src, value },
            ) => d_pat.matches(dst) && s_pat.matches(src) && v_pat.matches(value),
            (
                P::AddImm64 {
                    dst: d_pat,
                    src: s_pat,
                    value: v_pat,
                },
                Instruction::AddImm64 { dst, src, value },
            ) => d_pat.matches(dst) && s_pat.matches(src) && v_pat.matches(value),
            (
                P::BranchLtU {
                    reg1: r1_pat,
                    reg2: r2_pat,
                    offset: o_pat,
                },
                Instruction::BranchLtU { reg1, reg2, offset },
            ) => r1_pat.matches(reg1) && r2_pat.matches(reg2) && o_pat.matches(offset),
            (
                P::BranchGeU {
                    reg1: r1_pat,
                    reg2: r2_pat,
                    offset: o_pat,
                },
                Instruction::BranchGeU { reg1, reg2, offset },
            ) => r1_pat.matches(reg1) && r2_pat.matches(reg2) && o_pat.matches(offset),
            (
                P::BranchEqImm {
                    reg: r_pat,
                    value: v_pat,
                    offset: o_pat,
                },
                Instruction::BranchEqImm { reg, value, offset },
            ) => r_pat.matches(reg) && v_pat.matches(value) && o_pat.matches(offset),
            (
                P::BranchNeImm {
                    reg: r_pat,
                    value: v_pat,
                    offset: o_pat,
                },
                Instruction::BranchNeImm { reg, value, offset },
            ) => r_pat.matches(reg) && v_pat.matches(value) && o_pat.matches(offset),
            (
                P::BranchGeSImm {
                    reg: r_pat,
                    value: v_pat,
                    offset: o_pat,
                },
                Instruction::BranchGeSImm { reg, value, offset },
            ) => r_pat.matches(reg) && v_pat.matches(value) && o_pat.matches(offset),
            (
                P::LoadIndU8 {
                    dst: d_pat,
                    base: b_pat,
                    offset: o_pat,
                },
                Instruction::LoadIndU8 { dst, base, offset },
            ) => d_pat.matches(dst) && b_pat.matches(base) && o_pat.matches(offset),
            (
                P::LoadIndI8 {
                    dst: d_pat,
                    base: b_pat,
                    offset: o_pat,
                },
                Instruction::LoadIndI8 { dst, base, offset },
            ) => d_pat.matches(dst) && b_pat.matches(base) && o_pat.matches(offset),
            (
                P::LoadIndU16 {
                    dst: d_pat,
                    base: b_pat,
                    offset: o_pat,
                },
                Instruction::LoadIndU16 { dst, base, offset },
            ) => d_pat.matches(dst) && b_pat.matches(base) && o_pat.matches(offset),
            (
                P::LoadIndI16 {
                    dst: d_pat,
                    base: b_pat,
                    offset: o_pat,
                },
                Instruction::LoadIndI16 { dst, base, offset },
            ) => d_pat.matches(dst) && b_pat.matches(base) && o_pat.matches(offset),
            (
                P::LoadIndU32 {
                    dst: d_pat,
                    base: b_pat,
                    offset: o_pat,
                },
                Instruction::LoadIndU32 { dst, base, offset },
            ) => d_pat.matches(dst) && b_pat.matches(base) && o_pat.matches(offset),
            (
                P::LoadIndU64 {
                    dst: d_pat,
                    base: b_pat,
                    offset: o_pat,
                },
                Instruction::LoadIndU64 { dst, base, offset },
            ) => d_pat.matches(dst) && b_pat.matches(base) && o_pat.matches(offset),
            (
                P::StoreIndU8 {
                    base: b_pat,
                    src: s_pat,
                    offset: o_pat,
                },
                Instruction::StoreIndU8 { base, src, offset },
            ) => b_pat.matches(base) && s_pat.matches(src) && o_pat.matches(offset),
            (
                P::StoreIndU16 {
                    base: b_pat,
                    src: s_pat,
                    offset: o_pat,
                },
                Instruction::StoreIndU16 { base, src, offset },
            ) => b_pat.matches(base) && s_pat.matches(src) && o_pat.matches(offset),
            (
                P::StoreIndU32 {
                    base: b_pat,
                    src: s_pat,
                    offset: o_pat,
                },
                Instruction::StoreIndU32 { base, src, offset },
            ) => b_pat.matches(base) && s_pat.matches(src) && o_pat.matches(offset),
            (
                P::StoreIndU64 {
                    base: b_pat,
                    src: s_pat,
                    offset: o_pat,
                },
                Instruction::StoreIndU64 { base, src, offset },
            ) => b_pat.matches(base) && s_pat.matches(src) && o_pat.matches(offset),
            (
                P::CountSetBits32 {
                    dst: d_pat,
                    src: s_pat,
                },
                Instruction::CountSetBits32 { dst, src },
            ) => d_pat.matches(dst) && s_pat.matches(src),
            (
                P::CountSetBits64 {
                    dst: d_pat,
                    src: s_pat,
                },
                Instruction::CountSetBits64 { dst, src },
            ) => d_pat.matches(dst) && s_pat.matches(src),
            (
                P::LeadingZeroBits32 {
                    dst: d_pat,
                    src: s_pat,
                },
                Instruction::LeadingZeroBits32 { dst, src },
            ) => d_pat.matches(dst) && s_pat.matches(src),
            (
                P::LeadingZeroBits64 {
                    dst: d_pat,
                    src: s_pat,
                },
                Instruction::LeadingZeroBits64 { dst, src },
            ) => d_pat.matches(dst) && s_pat.matches(src),
            (
                P::TrailingZeroBits32 {
                    dst: d_pat,
                    src: s_pat,
                },
                Instruction::TrailingZeroBits32 { dst, src },
            ) => d_pat.matches(dst) && s_pat.matches(src),
            (
                P::TrailingZeroBits64 {
                    dst: d_pat,
                    src: s_pat,
                },
                Instruction::TrailingZeroBits64 { dst, src },
            ) => d_pat.matches(dst) && s_pat.matches(src),
            (
                P::SignExtend8 {
                    dst: d_pat,
                    src: s_pat,
                },
                Instruction::SignExtend8 { dst, src },
            ) => d_pat.matches(dst) && s_pat.matches(src),
            (
                P::SignExtend16 {
                    dst: d_pat,
                    src: s_pat,
                },
                Instruction::SignExtend16 { dst, src },
            ) => d_pat.matches(dst) && s_pat.matches(src),
            (
                P::ZeroExtend16 {
                    dst: d_pat,
                    src: s_pat,
                },
                Instruction::ZeroExtend16 { dst, src },
            ) => d_pat.matches(dst) && s_pat.matches(src),
            _ => false,
        }
    }
}

/// Find a pattern in an instruction sequence
///
/// Returns the index of the first match, or None if not found
pub fn find_pattern(instructions: &[Instruction], pattern: &[InstructionPattern]) -> Option<usize> {
    if pattern.is_empty() {
        return Some(0);
    }

    'outer: for start in 0..=instructions.len().saturating_sub(pattern.len()) {
        for (i, pat) in pattern.iter().enumerate() {
            if !pat.matches(&instructions[start + i]) {
                continue 'outer;
            }
        }
        return Some(start);
    }
    None
}

/// Assert that an instruction sequence contains a pattern
///
/// Panics with a descriptive message if the pattern is not found
pub fn assert_has_pattern(instructions: &[Instruction], pattern: &[InstructionPattern]) {
    if find_pattern(instructions, pattern).is_none() {
        panic!(
            "Pattern not found in instruction sequence.\n\nExpected pattern:\n{}\n\nActual instructions:\n{}",
            format_patterns(pattern),
            format_instructions(instructions)
        );
    }
}

/// Assert that instructions match a pattern exactly
///
/// Panics with a descriptive message if they don't match
pub fn assert_matches(instructions: &[Instruction], pattern: &[InstructionPattern]) {
    if instructions.len() != pattern.len() {
        panic!(
            "Instruction count mismatch: expected {}, got {}.\n\nExpected pattern:\n{}\n\nActual instructions:\n{}",
            pattern.len(),
            instructions.len(),
            format_patterns(pattern),
            format_instructions(instructions)
        );
    }

    for (i, (instr, pat)) in instructions.iter().zip(pattern.iter()).enumerate() {
        if !pat.matches(instr) {
            panic!(
                "Instruction mismatch at index {}:\nExpected: {:?}\nActual:   {:?}\n\nFull pattern:\n{}\n\nFull instructions:\n{}",
                i,
                pat,
                instr,
                format_patterns(pattern),
                format_instructions(instructions)
            );
        }
    }
}

/// Format patterns for display
fn format_patterns(patterns: &[InstructionPattern]) -> String {
    patterns
        .iter()
        .map(|p| format!("  {:?}", p))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Format instructions for display
fn format_instructions(instructions: &[Instruction]) -> String {
    instructions
        .iter()
        .map(|i| format!("  {:?}", i))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Count instructions with a specific opcode
pub fn count_opcode(instructions: &[Instruction], opcode: Opcode) -> usize {
    instructions
        .iter()
        .filter(|i| i.opcode() == Some(opcode))
        .count()
}

/// Check if an instruction sequence contains a specific opcode
pub fn has_opcode(instructions: &[Instruction], opcode: Opcode) -> bool {
    instructions.iter().any(|i| i.opcode() == Some(opcode))
}

/// Filter instructions by opcode
pub fn filter_by_opcode(instructions: &[Instruction], opcode: Opcode) -> Vec<&Instruction> {
    instructions
        .iter()
        .filter(|i| i.opcode() == Some(opcode))
        .collect()
}

/// Helper extension trait for instructions
pub trait InstructionExt {
    /// Get the opcode of this instruction, if applicable
    fn opcode(&self) -> Option<Opcode>;
}

impl InstructionExt for Instruction {
    fn opcode(&self) -> Option<Opcode> {
        match self {
            Instruction::Trap => Some(Opcode::Trap),
            Instruction::Fallthrough => Some(Opcode::Fallthrough),
            Instruction::LoadImm64 { .. } => Some(Opcode::LoadImm64),
            Instruction::LoadImm { .. } => Some(Opcode::LoadImm),
            Instruction::Add32 { .. } => Some(Opcode::Add32),
            Instruction::Sub32 { .. } => Some(Opcode::Sub32),
            Instruction::Mul32 { .. } => Some(Opcode::Mul32),
            Instruction::DivU32 { .. } => Some(Opcode::DivU32),
            Instruction::DivS32 { .. } => Some(Opcode::DivS32),
            Instruction::RemU32 { .. } => Some(Opcode::RemU32),
            Instruction::RemS32 { .. } => Some(Opcode::RemS32),
            Instruction::Add64 { .. } => Some(Opcode::Add64),
            Instruction::Sub64 { .. } => Some(Opcode::Sub64),
            Instruction::Mul64 { .. } => Some(Opcode::Mul64),
            Instruction::DivU64 { .. } => Some(Opcode::DivU64),
            Instruction::DivS64 { .. } => Some(Opcode::DivS64),
            Instruction::RemU64 { .. } => Some(Opcode::RemU64),
            Instruction::RemS64 { .. } => Some(Opcode::RemS64),
            Instruction::ShloL64 { .. } => Some(Opcode::ShloL64),
            Instruction::ShloR64 { .. } => Some(Opcode::ShloR64),
            Instruction::SharR64 { .. } => Some(Opcode::SharR64),
            Instruction::AddImm32 { .. } => Some(Opcode::AddImm32),
            Instruction::AddImm64 { .. } => Some(Opcode::AddImm64),
            Instruction::Jump { .. } => Some(Opcode::Jump),
            Instruction::JumpInd { .. } => Some(Opcode::JumpInd),
            Instruction::LoadIndU32 { .. } => Some(Opcode::LoadIndU32),
            Instruction::StoreIndU32 { .. } => Some(Opcode::StoreIndU32),
            Instruction::LoadIndU64 { .. } => Some(Opcode::LoadIndU64),
            Instruction::StoreIndU64 { .. } => Some(Opcode::StoreIndU64),
            Instruction::BranchNeImm { .. } => Some(Opcode::BranchNeImm),
            Instruction::BranchEqImm { .. } => Some(Opcode::BranchEqImm),
            Instruction::BranchGeSImm { .. } => Some(Opcode::BranchGeSImm),
            Instruction::BranchGeU { .. } => Some(Opcode::BranchGeU),
            Instruction::BranchLtU { .. } => Some(Opcode::BranchLtU),
            Instruction::SetLtU { .. } => Some(Opcode::SetLtU),
            Instruction::SetLtS { .. } => Some(Opcode::SetLtS),
            Instruction::And { .. } => Some(Opcode::And),
            Instruction::Xor { .. } => Some(Opcode::Xor),
            Instruction::Or { .. } => Some(Opcode::Or),
            Instruction::SetLtUImm { .. } => Some(Opcode::SetLtUImm),
            Instruction::SetLtSImm { .. } => Some(Opcode::SetLtSImm),
            Instruction::ShloL32 { .. } => Some(Opcode::ShloL32),
            Instruction::ShloR32 { .. } => Some(Opcode::ShloR32),
            Instruction::SharR32 { .. } => Some(Opcode::SharR32),
            Instruction::CountSetBits64 { .. } => Some(Opcode::CountSetBits64),
            Instruction::CountSetBits32 { .. } => Some(Opcode::CountSetBits32),
            Instruction::LeadingZeroBits64 { .. } => Some(Opcode::LeadingZeroBits64),
            Instruction::LeadingZeroBits32 { .. } => Some(Opcode::LeadingZeroBits32),
            Instruction::TrailingZeroBits64 { .. } => Some(Opcode::TrailingZeroBits64),
            Instruction::TrailingZeroBits32 { .. } => Some(Opcode::TrailingZeroBits32),
            Instruction::SignExtend8 { .. } => Some(Opcode::SignExtend8),
            Instruction::SignExtend16 { .. } => Some(Opcode::SignExtend16),
            Instruction::ZeroExtend16 { .. } => Some(Opcode::ZeroExtend16),
            Instruction::LoadIndU8 { .. } => Some(Opcode::LoadIndU8),
            Instruction::LoadIndI8 { .. } => Some(Opcode::LoadIndI8),
            Instruction::LoadIndU16 { .. } => Some(Opcode::LoadIndU16),
            Instruction::LoadIndI16 { .. } => Some(Opcode::LoadIndI16),
            Instruction::StoreIndU8 { .. } => Some(Opcode::StoreIndU8),
            Instruction::StoreIndU16 { .. } => Some(Opcode::StoreIndU16),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wat_to_wasm_simple() {
        let wat = r#"
            (module
                (func (export "main") (result i32)
                    i32.const 42
                )
            )
        "#;
        let wasm = wat_to_wasm(wat).expect("Failed to parse WAT");
        assert!(!wasm.is_empty());
        // WASM magic number
        assert_eq!(&wasm[0..4], &[0x00, 0x61, 0x73, 0x6d]);
    }

    #[test]
    fn test_instruction_pattern_matching() {
        let instr = Instruction::Add32 {
            dst: 5,
            src1: 2,
            src2: 3,
        };

        // Exact match
        let pattern_exact = InstructionPattern::Add32 {
            dst: Pat::Exact(5),
            src1: Pat::Exact(2),
            src2: Pat::Exact(3),
        };
        assert!(pattern_exact.matches(&instr));

        // Wildcard match
        let pattern_wildcard = InstructionPattern::Add32 {
            dst: Pat::Any,
            src1: Pat::Any,
            src2: Pat::Any,
        };
        assert!(pattern_wildcard.matches(&instr));

        // Wrong instruction type
        let pattern_wrong = InstructionPattern::Sub32 {
            dst: Pat::Any,
            src1: Pat::Any,
            src2: Pat::Any,
        };
        assert!(!pattern_wrong.matches(&instr));
    }

    #[test]
    fn test_find_pattern() {
        let instructions = vec![
            Instruction::LoadImm { reg: 2, value: 5 },
            Instruction::LoadImm { reg: 3, value: 7 },
            Instruction::Add32 {
                dst: 4,
                src1: 2,
                src2: 3,
            },
            Instruction::Trap,
        ];

        let pattern = vec![
            InstructionPattern::LoadImm {
                reg: Pat::Any,
                value: Pat::Any,
            },
            InstructionPattern::Add32 {
                dst: Pat::Any,
                src1: Pat::Exact(2),
                src2: Pat::Any,
            },
        ];

        assert_eq!(find_pattern(&instructions, &pattern), Some(1));
    }

    #[test]
    fn test_count_opcode() {
        let instructions = vec![
            Instruction::LoadImm { reg: 2, value: 5 },
            Instruction::LoadImm { reg: 3, value: 7 },
            Instruction::Add32 {
                dst: 4,
                src1: 2,
                src2: 3,
            },
            Instruction::Trap,
        ];

        assert_eq!(count_opcode(&instructions, Opcode::LoadImm), 2);
        assert_eq!(count_opcode(&instructions, Opcode::Add32), 1);
        assert_eq!(count_opcode(&instructions, Opcode::Trap), 1);
        assert_eq!(count_opcode(&instructions, Opcode::Sub32), 0);
    }

    #[test]
    fn test_pat_predicate() {
        let is_positive = |v: &i32| *v > 0;
        let pat = Pat::Predicate(is_positive);

        assert!(pat.matches(&5));
        assert!(!pat.matches(&-1));
        assert!(!pat.matches(&0));
    }

    #[test]
    fn test_compile_simple_wat() {
        let wat = r#"
            (module
                (func (export "main") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.add
                )
            )
        "#;

        let program = compile_wat(wat).expect("Failed to compile");
        let code = program.code();
        let instructions = code.instructions();

        // Should have some instructions
        assert!(!instructions.is_empty());

        // Should contain at least one Add32
        assert!(has_opcode(instructions, Opcode::Add32));
    }
}
