//! Property-based tests for the WASM-to-PVM compiler.
//!
//! Uses `proptest` to generate random inputs and verify invariants:
//! - Compilation of valid WAT modules never panics
//! - PVM instruction encoding produces valid byte sequences
//! - Arithmetic WAT modules produce correct opcodes for random operands
//! - Edge cases (overflow, division by zero) are handled correctly

use std::fmt::Write;

use proptest::prelude::*;
use wasm_pvm::test_harness::*;
use wasm_pvm::{Instruction, Opcode};

#[allow(clippy::too_many_lines)]
fn instruction_roundtrip_strategy() -> impl Strategy<Value = Instruction> {
    let reg = 0u8..13;

    prop_oneof![
        Just(Instruction::Trap),
        Just(Instruction::Fallthrough),
        (reg.clone(), any::<u64>()).prop_map(|(reg, value)| Instruction::LoadImm64 { reg, value }),
        any::<i32>().prop_map(|offset| Instruction::Jump { offset }),
        any::<u32>().prop_map(|index| Instruction::Ecalli { index }),
        (reg.clone(), reg.clone(), any::<i32>(), any::<i32>()).prop_map(
            |(base, dst, value, offset)| {
                Instruction::LoadImmJumpInd {
                    base,
                    dst,
                    value,
                    offset,
                }
            }
        ),
        (0u8..41, reg.clone(), reg.clone(), reg.clone()).prop_map(|(kind, dst, src1, src2)| {
            match kind {
                0 => Instruction::Add32 { dst, src1, src2 },
                1 => Instruction::Sub32 { dst, src1, src2 },
                2 => Instruction::Mul32 { dst, src1, src2 },
                3 => Instruction::DivU32 { dst, src1, src2 },
                4 => Instruction::DivS32 { dst, src1, src2 },
                5 => Instruction::RemU32 { dst, src1, src2 },
                6 => Instruction::RemS32 { dst, src1, src2 },
                7 => Instruction::ShloL32 { dst, src1, src2 },
                8 => Instruction::ShloR32 { dst, src1, src2 },
                9 => Instruction::SharR32 { dst, src1, src2 },
                10 => Instruction::Add64 { dst, src1, src2 },
                11 => Instruction::Sub64 { dst, src1, src2 },
                12 => Instruction::Mul64 { dst, src1, src2 },
                13 => Instruction::DivU64 { dst, src1, src2 },
                14 => Instruction::DivS64 { dst, src1, src2 },
                15 => Instruction::RemU64 { dst, src1, src2 },
                16 => Instruction::RemS64 { dst, src1, src2 },
                17 => Instruction::ShloL64 { dst, src1, src2 },
                18 => Instruction::ShloR64 { dst, src1, src2 },
                19 => Instruction::SharR64 { dst, src1, src2 },
                20 => Instruction::SetLtU { dst, src1, src2 },
                21 => Instruction::SetLtS { dst, src1, src2 },
                22 => Instruction::CmovIz {
                    dst,
                    src: src1,
                    cond: src2,
                },
                23 => Instruction::CmovNz {
                    dst,
                    src: src1,
                    cond: src2,
                },
                24 => Instruction::And { dst, src1, src2 },
                25 => Instruction::Xor { dst, src1, src2 },
                26 => Instruction::Or { dst, src1, src2 },
                27 => Instruction::MulUpperSS { dst, src1, src2 },
                28 => Instruction::MulUpperUU { dst, src1, src2 },
                29 => Instruction::MulUpperSU { dst, src1, src2 },
                30 => Instruction::RotL64 { dst, src1, src2 },
                31 => Instruction::RotL32 { dst, src1, src2 },
                32 => Instruction::RotR64 { dst, src1, src2 },
                33 => Instruction::RotR32 { dst, src1, src2 },
                34 => Instruction::AndInv { dst, src1, src2 },
                35 => Instruction::OrInv { dst, src1, src2 },
                36 => Instruction::Xnor { dst, src1, src2 },
                37 => Instruction::Max { dst, src1, src2 },
                38 => Instruction::MaxU { dst, src1, src2 },
                39 => Instruction::Min { dst, src1, src2 },
                40 => Instruction::MinU { dst, src1, src2 },
                _ => unreachable!("kind bounded to 0..41"),
            }
        }),
        (0u8..12, reg.clone(), reg.clone()).prop_map(|(kind, dst, src)| match kind {
            0 => Instruction::MoveReg { dst, src },
            1 => Instruction::Sbrk { dst, src },
            2 => Instruction::CountSetBits64 { dst, src },
            3 => Instruction::CountSetBits32 { dst, src },
            4 => Instruction::LeadingZeroBits64 { dst, src },
            5 => Instruction::LeadingZeroBits32 { dst, src },
            6 => Instruction::TrailingZeroBits64 { dst, src },
            7 => Instruction::TrailingZeroBits32 { dst, src },
            8 => Instruction::SignExtend8 { dst, src },
            9 => Instruction::SignExtend16 { dst, src },
            10 => Instruction::ZeroExtend16 { dst, src },
            11 => Instruction::ReverseBytes { dst, src },
            _ => unreachable!("kind bounded to 0..12"),
        }),
        (0u8..42, reg.clone(), reg.clone(), any::<i32>()).prop_map(|(kind, lo, hi, value)| {
            match kind {
                0 => Instruction::AddImm32 {
                    dst: lo,
                    src: hi,
                    value,
                },
                1 => Instruction::AddImm64 {
                    dst: lo,
                    src: hi,
                    value,
                },
                2 => Instruction::AndImm {
                    dst: lo,
                    src: hi,
                    value,
                },
                3 => Instruction::XorImm {
                    dst: lo,
                    src: hi,
                    value,
                },
                4 => Instruction::OrImm {
                    dst: lo,
                    src: hi,
                    value,
                },
                5 => Instruction::MulImm32 {
                    dst: lo,
                    src: hi,
                    value,
                },
                6 => Instruction::MulImm64 {
                    dst: lo,
                    src: hi,
                    value,
                },
                7 => Instruction::SetLtUImm {
                    dst: lo,
                    src: hi,
                    value,
                },
                8 => Instruction::SetLtSImm {
                    dst: lo,
                    src: hi,
                    value,
                },
                9 => Instruction::ShloLImm32 {
                    dst: lo,
                    src: hi,
                    value,
                },
                10 => Instruction::ShloRImm32 {
                    dst: lo,
                    src: hi,
                    value,
                },
                11 => Instruction::SharRImm32 {
                    dst: lo,
                    src: hi,
                    value,
                },
                12 => Instruction::ShloLImm64 {
                    dst: lo,
                    src: hi,
                    value,
                },
                13 => Instruction::ShloRImm64 {
                    dst: lo,
                    src: hi,
                    value,
                },
                14 => Instruction::SharRImm64 {
                    dst: lo,
                    src: hi,
                    value,
                },
                15 => Instruction::NegAddImm32 {
                    dst: lo,
                    src: hi,
                    value,
                },
                16 => Instruction::NegAddImm64 {
                    dst: lo,
                    src: hi,
                    value,
                },
                17 => Instruction::SetGtUImm {
                    dst: lo,
                    src: hi,
                    value,
                },
                18 => Instruction::SetGtSImm {
                    dst: lo,
                    src: hi,
                    value,
                },
                19 => Instruction::CmovIzImm {
                    dst: lo,
                    cond: hi,
                    value,
                },
                20 => Instruction::CmovNzImm {
                    dst: lo,
                    cond: hi,
                    value,
                },
                21 => Instruction::LoadIndU8 {
                    dst: lo,
                    base: hi,
                    offset: value,
                },
                22 => Instruction::LoadIndI8 {
                    dst: lo,
                    base: hi,
                    offset: value,
                },
                23 => Instruction::LoadIndU16 {
                    dst: lo,
                    base: hi,
                    offset: value,
                },
                24 => Instruction::LoadIndI16 {
                    dst: lo,
                    base: hi,
                    offset: value,
                },
                25 => Instruction::LoadIndU32 {
                    dst: lo,
                    base: hi,
                    offset: value,
                },
                26 => Instruction::LoadIndI32 {
                    dst: lo,
                    base: hi,
                    offset: value,
                },
                27 => Instruction::LoadIndU64 {
                    dst: lo,
                    base: hi,
                    offset: value,
                },
                28 => Instruction::StoreIndU8 {
                    base: hi,
                    src: lo,
                    offset: value,
                },
                29 => Instruction::StoreIndU16 {
                    base: hi,
                    src: lo,
                    offset: value,
                },
                30 => Instruction::StoreIndU32 {
                    base: hi,
                    src: lo,
                    offset: value,
                },
                31 => Instruction::StoreIndU64 {
                    base: hi,
                    src: lo,
                    offset: value,
                },
                32 => Instruction::ShloLImmAlt32 {
                    dst: lo,
                    src: hi,
                    value,
                },
                33 => Instruction::ShloRImmAlt32 {
                    dst: lo,
                    src: hi,
                    value,
                },
                34 => Instruction::SharRImmAlt32 {
                    dst: lo,
                    src: hi,
                    value,
                },
                35 => Instruction::ShloLImmAlt64 {
                    dst: lo,
                    src: hi,
                    value,
                },
                36 => Instruction::ShloRImmAlt64 {
                    dst: lo,
                    src: hi,
                    value,
                },
                37 => Instruction::SharRImmAlt64 {
                    dst: lo,
                    src: hi,
                    value,
                },
                38 => Instruction::RotRImm64 {
                    dst: lo,
                    src: hi,
                    value,
                },
                39 => Instruction::RotRImmAlt64 {
                    dst: lo,
                    src: hi,
                    value,
                },
                40 => Instruction::RotRImm32 {
                    dst: lo,
                    src: hi,
                    value,
                },
                41 => Instruction::RotRImmAlt32 {
                    dst: lo,
                    src: hi,
                    value,
                },
                _ => unreachable!("kind bounded to 0..42"),
            }
        }),
        (0u8..13, reg.clone(), any::<i32>()).prop_map(|(kind, reg, value)| match kind {
            0 => Instruction::LoadImm { reg, value },
            1 => Instruction::JumpInd { reg, offset: value },
            2 => Instruction::LoadU8 {
                dst: reg,
                address: value,
            },
            3 => Instruction::LoadI8 {
                dst: reg,
                address: value,
            },
            4 => Instruction::LoadU16 {
                dst: reg,
                address: value,
            },
            5 => Instruction::LoadI16 {
                dst: reg,
                address: value,
            },
            6 => Instruction::LoadU32 {
                dst: reg,
                address: value,
            },
            7 => Instruction::LoadI32 {
                dst: reg,
                address: value,
            },
            8 => Instruction::LoadU64 {
                dst: reg,
                address: value,
            },
            9 => Instruction::StoreU8 {
                src: reg,
                address: value,
            },
            10 => Instruction::StoreU16 {
                src: reg,
                address: value,
            },
            11 => Instruction::StoreU32 {
                src: reg,
                address: value,
            },
            12 => Instruction::StoreU64 {
                src: reg,
                address: value,
            },
            _ => unreachable!("kind bounded to 0..13"),
        }),
        (0u8..11, reg.clone(), any::<i32>(), any::<i32>()).prop_map(
            |(kind, reg, value, offset)| match kind {
                0 => Instruction::LoadImmJump { reg, value, offset },
                1 => Instruction::BranchEqImm { reg, value, offset },
                2 => Instruction::BranchNeImm { reg, value, offset },
                3 => Instruction::BranchLtUImm { reg, value, offset },
                4 => Instruction::BranchLeUImm { reg, value, offset },
                5 => Instruction::BranchGeUImm { reg, value, offset },
                6 => Instruction::BranchGtUImm { reg, value, offset },
                7 => Instruction::BranchLtSImm { reg, value, offset },
                8 => Instruction::BranchLeSImm { reg, value, offset },
                9 => Instruction::BranchGeSImm { reg, value, offset },
                10 => Instruction::BranchGtSImm { reg, value, offset },
                _ => unreachable!("kind bounded to 0..11"),
            }
        ),
        (0u8..6, reg.clone(), reg.clone(), any::<i32>()).prop_map(|(kind, reg1, reg2, offset)| {
            match kind {
                0 => Instruction::BranchEq { reg1, reg2, offset },
                1 => Instruction::BranchNe { reg1, reg2, offset },
                2 => Instruction::BranchLtU { reg1, reg2, offset },
                3 => Instruction::BranchLtS { reg1, reg2, offset },
                4 => Instruction::BranchGeU { reg1, reg2, offset },
                5 => Instruction::BranchGeS { reg1, reg2, offset },
                _ => unreachable!("kind bounded to 0..6"),
            }
        }),
        (0u8..4, reg.clone(), any::<i32>(), any::<i32>()).prop_map(
            |(kind, base, offset, value)| match kind {
                0 => Instruction::StoreImmIndU8 {
                    base,
                    offset,
                    value,
                },
                1 => Instruction::StoreImmIndU16 {
                    base,
                    offset,
                    value,
                },
                2 => Instruction::StoreImmIndU32 {
                    base,
                    offset,
                    value,
                },
                3 => Instruction::StoreImmIndU64 {
                    base,
                    offset,
                    value,
                },
                _ => unreachable!("kind bounded to 0..4"),
            }
        ),
        (0u8..4, any::<i32>(), any::<i32>()).prop_map(|(kind, address, value)| match kind {
            0 => Instruction::StoreImmU8 { address, value },
            1 => Instruction::StoreImmU16 { address, value },
            2 => Instruction::StoreImmU32 { address, value },
            3 => Instruction::StoreImmU64 { address, value },
            _ => unreachable!("kind bounded to 0..4"),
        }),
        (any::<u8>(), prop::collection::vec(any::<u8>(), 0..4))
            .prop_filter("opcode must not map to a known Opcode", |(opcode, _)| {
                Opcode::from_u8(*opcode).is_none()
            })
            .prop_map(|(opcode, mut tail)| {
                let mut raw_bytes = vec![opcode];
                raw_bytes.append(&mut tail);
                Instruction::Unknown { opcode, raw_bytes }
            }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn instruction_decode_encode_roundtrip(instr in instruction_roundtrip_strategy()) {
        let encoded = instr.encode();
        let (decoded, consumed) = Instruction::decode(&encoded).expect("decode should succeed");
        prop_assert_eq!(consumed, encoded.len(), "consumed length mismatch");
        prop_assert_eq!(decoded, instr, "decode(encode(instr)) mismatch");
    }
}

// =============================================================================
// Compilation Safety: valid WAT modules never panic during compilation
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    /// i32.add with random constant operands compiles without panicking.
    #[test]
    fn wasm_compile_i32_add_const(a in any::<i32>(), b in any::<i32>()) {
        let wat = format!(
            r#"(module
                (memory 1)
                (func (export "main") (param i32 i32) (result i32)
                    i32.const {a}
                    i32.const {b}
                    i32.add
                )
            )"#
        );
        let result = compile_wat(&wat);
        prop_assert!(result.is_ok(), "Compilation failed for a={a}, b={b}: {:?}", result.err());
    }

    /// i32.sub with random constant operands compiles without panicking.
    #[test]
    fn wasm_compile_i32_sub_const(a in any::<i32>(), b in any::<i32>()) {
        let wat = format!(
            r#"(module
                (memory 1)
                (func (export "main") (param i32 i32) (result i32)
                    i32.const {a}
                    i32.const {b}
                    i32.sub
                )
            )"#
        );
        let result = compile_wat(&wat);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    /// i32.mul with random constant operands compiles without panicking.
    #[test]
    fn wasm_compile_i32_mul_const(a in any::<i32>(), b in any::<i32>()) {
        let wat = format!(
            r#"(module
                (memory 1)
                (func (export "main") (param i32 i32) (result i32)
                    i32.const {a}
                    i32.const {b}
                    i32.mul
                )
            )"#
        );
        let result = compile_wat(&wat);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    /// i64 arithmetic with random constants compiles without panicking.
    #[test]
    fn wasm_compile_i64_add_const(a in any::<i64>(), b in any::<i64>()) {
        let wat = format!(
            r#"(module
                (memory 1)
                (func (export "main") (param i32 i32) (result i64)
                    i64.const {a}
                    i64.const {b}
                    i64.add
                )
            )"#
        );
        let result = compile_wat(&wat);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    /// Shift amounts can be any value (WASM masks them to type width).
    #[test]
    fn wasm_compile_i32_shl_any_amount(shift in any::<i32>()) {
        let wat = format!(
            r#"(module
                (memory 1)
                (func (export "main") (param i32) (result i32)
                    local.get 0
                    i32.const {shift}
                    i32.shl
                )
            )"#
        );
        let result = compile_wat(&wat);
        prop_assert!(result.is_ok(), "Compilation failed: {:?}", result.err());
    }

    /// Memory load with random offset compiles (offset is compile-time constant).
    #[test]
    fn wasm_compile_memory_load_offset(offset in 0u32..65536) {
        let wat = format!(
            r#"(module
                (memory 1)
                (func (export "main") (param i32) (result i32)
                    local.get 0
                    i32.load offset={offset}
                )
            )"#
        );
        let result = compile_wat(&wat);
        prop_assert!(result.is_ok(), "Compilation failed for offset={offset}: {:?}", result.err());
    }

    /// Memory store with random offset compiles.
    #[test]
    fn wasm_compile_memory_store_offset(offset in 0u32..65536) {
        let wat = format!(
            r#"(module
                (memory 1)
                (func (export "main") (param i32 i32) (result i32)
                    local.get 0
                    local.get 1
                    i32.store offset={offset}
                    i32.const 0
                )
            )"#
        );
        let result = compile_wat(&wat);
        prop_assert!(result.is_ok(), "Compilation failed for offset={offset}: {:?}", result.err());
    }

    /// Branch table with random number of targets compiles.
    #[test]
    fn wasm_compile_br_table(num_targets in 1usize..10) {
        let targets: Vec<String> = (0..num_targets).map(|_| "0".to_string()).collect();
        let target_list = targets.join(" ");
        let wat = format!(
            r#"(module
                (memory 1)
                (func (export "main") (param i32) (result i32)
                    (block
                        local.get 0
                        br_table {target_list} 0
                    )
                    local.get 0
                )
            )"#
        );
        let result = compile_wat(&wat);
        prop_assert!(result.is_ok(), "Compilation failed for {} targets: {:?}", num_targets, result.err());
    }

    /// Multiple locals of various counts compile correctly.
    #[test]
    fn wasm_compile_many_locals(num_locals in 1usize..20) {
        let locals: String = (0..num_locals).map(|_| "(local i32)").collect::<Vec<_>>().join(" ");
        let wat = format!(
            r#"(module
                (memory 1)
                (func (export "main") (param i32) (result i32)
                    {locals}
                    local.get 0
                )
            )"#
        );
        let result = compile_wat(&wat);
        prop_assert!(result.is_ok(), "Compilation failed for {} locals: {:?}", num_locals, result.err());
    }
}

// =============================================================================
// Instruction Encoding Properties
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Three-reg instructions always encode to exactly 3 bytes.
    #[test]
    fn three_reg_encoding_length(dst in 0u8..13, src1 in 0u8..13, src2 in 0u8..13) {
        let instr = wasm_pvm::Instruction::Add32 { dst, src1, src2 };
        let encoded = instr.encode();
        prop_assert_eq!(encoded.len(), 3, "Three-reg should be 3 bytes, got {}", encoded.len());
    }

    /// Two-reg instructions always encode to exactly 2 bytes.
    #[test]
    fn two_reg_encoding_length(dst in 0u8..13, src in 0u8..13) {
        let instr = wasm_pvm::Instruction::MoveReg { dst, src };
        let encoded = instr.encode();
        prop_assert_eq!(encoded.len(), 2, "Two-reg should be 2 bytes, got {}", encoded.len());
    }

    /// LoadImm encodes to 2 + imm_len bytes where imm_len depends on value magnitude.
    #[test]
    fn load_imm_encoding_range(value in any::<i32>()) {
        let instr = wasm_pvm::Instruction::LoadImm { reg: 2, value };
        let encoded = instr.encode();
        // 2 bytes base (opcode + reg) + 0-4 imm bytes
        prop_assert!(encoded.len() >= 2 && encoded.len() <= 6,
            "LoadImm should be 2-6 bytes, got {} for value={value}", encoded.len());
    }

    /// LoadImm64 always encodes to exactly 10 bytes (opcode + reg + 8 bytes).
    #[test]
    fn load_imm64_encoding_length(value in any::<u64>()) {
        let instr = wasm_pvm::Instruction::LoadImm64 { reg: 3, value };
        let encoded = instr.encode();
        prop_assert_eq!(encoded.len(), 10, "LoadImm64 should be 10 bytes");
    }

    /// Register nibbles are properly masked to 4 bits in three-reg encoding.
    #[test]
    fn three_reg_register_masking(dst in 0u8..16, src1 in 0u8..16, src2 in 0u8..16) {
        let instr = wasm_pvm::Instruction::Add32 { dst, src1, src2 };
        let encoded = instr.encode();
        // byte[1] = (src2 & 0x0F) << 4 | (src1 & 0x0F)
        prop_assert_eq!(encoded[1] & 0x0F, src1 & 0x0F, "src1 nibble mismatch");
        prop_assert_eq!(encoded[1] >> 4, src2 & 0x0F, "src2 nibble mismatch");
        // byte[2] = dst & 0x0F
        prop_assert_eq!(encoded[2], dst & 0x0F, "dst byte mismatch");
    }

    /// Ecalli encoding roundtrips correctly for any u32 index.
    #[test]
    fn ecalli_roundtrip(index in any::<u32>()) {
        let instr = wasm_pvm::Instruction::Ecalli { index };
        let encoded = instr.encode();
        prop_assert_eq!(encoded[0], Opcode::Ecalli as u8);
        // Decode: the remaining bytes are the variable-length encoded index
        let imm = &encoded[1..];
        let mut bytes = [0u8; 4];
        bytes[..imm.len()].copy_from_slice(imm);
        let decoded = u32::from_le_bytes(bytes);
        prop_assert_eq!(decoded, index, "Ecalli roundtrip failed");
    }

    /// Jump instruction encodes offset correctly for any i32.
    #[test]
    fn jump_offset_roundtrip(offset in any::<i32>()) {
        let instr = wasm_pvm::Instruction::Jump { offset };
        let encoded = instr.encode();
        prop_assert_eq!(encoded[0], Opcode::Jump as u8);
        let decoded = i32::from_le_bytes(encoded[1..5].try_into().unwrap());
        prop_assert_eq!(decoded, offset);
    }

    /// LoadImmJump encoding preserves reg, value, and offset for any inputs.
    #[test]
    fn load_imm_jump_roundtrip(
        reg in 0u8..13,
        value in any::<i32>(),
        offset in any::<i32>(),
    ) {
        let instr = wasm_pvm::Instruction::LoadImmJump { reg, value, offset };
        let encoded = instr.encode();
        prop_assert_eq!(encoded[0], Opcode::LoadImmJump as u8);
        // byte[1]: (imm_len << 4) | (reg & 0x0F)
        prop_assert_eq!(encoded[1] & 0x0F, reg & 0x0F, "reg mismatch");
        let imm_len = (encoded[1] >> 4) as usize;
        // Decode value from imm_len bytes (sign-extended)
        let mut imm_bytes = [0u8; 4];
        imm_bytes[..imm_len].copy_from_slice(&encoded[2..2 + imm_len]);
        // Sign-extend if needed
        if imm_len > 0 && (imm_bytes[imm_len - 1] & 0x80) != 0 {
            for b in imm_bytes.iter_mut().skip(imm_len) {
                *b = 0xFF;
            }
        }
        let decoded_value = i32::from_le_bytes(imm_bytes);
        prop_assert_eq!(decoded_value, value, "value mismatch");
        // Offset is last 4 bytes
        let offset_start = 2 + imm_len;
        let decoded_offset = i32::from_le_bytes(
            encoded[offset_start..offset_start + 4].try_into().unwrap()
        );
        prop_assert_eq!(decoded_offset, offset, "offset mismatch");
    }

    /// CmovIz/CmovNz use ThreeReg encoding with correct register placement.
    #[test]
    fn cmov_encoding(dst in 0u8..13, src in 0u8..13, cond in 0u8..13) {
        let cmov_iz = wasm_pvm::Instruction::CmovIz { dst, src, cond };
        let enc_iz = cmov_iz.encode();
        prop_assert_eq!(enc_iz[0], Opcode::CmovIz as u8);
        prop_assert_eq!(enc_iz[1] & 0x0F, src & 0x0F, "CmovIz src nibble");
        prop_assert_eq!(enc_iz[1] >> 4, cond & 0x0F, "CmovIz cond nibble");
        prop_assert_eq!(enc_iz[2], dst & 0x0F, "CmovIz dst byte");

        let cmov_nz = wasm_pvm::Instruction::CmovNz { dst, src, cond };
        let enc_nz = cmov_nz.encode();
        prop_assert_eq!(enc_nz[0], Opcode::CmovNz as u8);
        prop_assert_eq!(enc_nz[1] & 0x0F, src & 0x0F, "CmovNz src nibble");
        prop_assert_eq!(enc_nz[1] >> 4, cond & 0x0F, "CmovNz cond nibble");
        prop_assert_eq!(enc_nz[2], dst & 0x0F, "CmovNz dst byte");
    }

    /// StoreImmInd encoding: opcode, register in low nibble, offset_len in high nibble,
    /// then offset bytes, then value bytes.
    #[test]
    fn store_imm_ind_encoding(
        base in 0u8..13,
        offset in any::<i32>(),
        value in any::<i32>(),
    ) {
        let variants: Vec<(Opcode, wasm_pvm::Instruction)> = vec![
            (Opcode::StoreImmIndU8, wasm_pvm::Instruction::StoreImmIndU8 { base, offset, value }),
            (Opcode::StoreImmIndU16, wasm_pvm::Instruction::StoreImmIndU16 { base, offset, value }),
            (Opcode::StoreImmIndU32, wasm_pvm::Instruction::StoreImmIndU32 { base, offset, value }),
            (Opcode::StoreImmIndU64, wasm_pvm::Instruction::StoreImmIndU64 { base, offset, value }),
        ];
        for (expected_opcode, instr) in &variants {
            let encoded = instr.encode();
            prop_assert_eq!(encoded[0], *expected_opcode as u8, "opcode mismatch for {:?}", expected_opcode);
            prop_assert_eq!(encoded[1] & 0x0F, base & 0x0F, "register mismatch for {:?}", expected_opcode);
            // Minimum: 2 bytes (opcode + reg byte). Max: 2 + 4 (offset) + 4 (value) = 10.
            prop_assert!(encoded.len() >= 2 && encoded.len() <= 10,
                "StoreImmInd should be 2-10 bytes, got {}", encoded.len());
        }
    }

    /// TwoRegOneImm instructions (ALU immediate) encode registers and imm correctly.
    #[test]
    fn two_reg_one_imm_encoding(dst in 0u8..13, src in 0u8..13, value in any::<i32>()) {
        // Test all TwoRegOneImm opcodes use the same encoding format
        let instrs = vec![
            wasm_pvm::Instruction::AndImm { dst, src, value },
            wasm_pvm::Instruction::XorImm { dst, src, value },
            wasm_pvm::Instruction::OrImm { dst, src, value },
            wasm_pvm::Instruction::MulImm32 { dst, src, value },
            wasm_pvm::Instruction::MulImm64 { dst, src, value },
            wasm_pvm::Instruction::ShloLImm32 { dst, src, value },
            wasm_pvm::Instruction::ShloRImm32 { dst, src, value },
            wasm_pvm::Instruction::SharRImm32 { dst, src, value },
            wasm_pvm::Instruction::ShloLImm64 { dst, src, value },
            wasm_pvm::Instruction::ShloRImm64 { dst, src, value },
            wasm_pvm::Instruction::SharRImm64 { dst, src, value },
            wasm_pvm::Instruction::NegAddImm32 { dst, src, value },
            wasm_pvm::Instruction::NegAddImm64 { dst, src, value },
            wasm_pvm::Instruction::SetGtUImm { dst, src, value },
            wasm_pvm::Instruction::SetGtSImm { dst, src, value },
        ];
        for instr in &instrs {
            let encoded = instr.encode();
            // byte[1] = (src & 0x0F) << 4 | (dst & 0x0F)
            prop_assert_eq!(encoded[1] & 0x0F, dst & 0x0F, "dst nibble for {:?}", instr);
            prop_assert_eq!(encoded[1] >> 4, src & 0x0F, "src nibble for {:?}", instr);
            // Decode immediate from remaining bytes
            let imm_bytes = &encoded[2..];
            let mut raw = [0u8; 4];
            raw[..imm_bytes.len()].copy_from_slice(imm_bytes);
            let decoded = i32::from_le_bytes(raw);
            // For sign-extension to work, we need to handle sign bit
            if imm_bytes.len() < 4 && value < 0 {
                // Variable-length encoding truncates; verify the sign-extended value
                // is correct by checking the encoded bytes match the low bytes of value
                let value_bytes = value.to_le_bytes();
                for (i, b) in imm_bytes.iter().enumerate() {
                    prop_assert_eq!(*b, value_bytes[i], "imm byte {} mismatch for {:?}", i, instr);
                }
            } else {
                prop_assert_eq!(decoded, value, "imm roundtrip for {:?}", instr);
            }
        }
    }
}

// =============================================================================
// Opcode Correctness: compiled WAT contains expected PVM opcodes
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(30))]

    /// i32.add with various constant operands always produces an Add32 or AddImm32 opcode.
    #[test]
    fn add_produces_add_opcode(a in any::<i32>(), b in any::<i32>()) {
        let wat = format!(
            r#"(module
                (memory 1)
                (func (export "main") (param i32) (result i32)
                    local.get 0
                    i32.const {a}
                    i32.add
                    i32.const {b}
                    i32.add
                )
            )"#
        );
        let program = compile_wat(&wat).expect("compile");
        let instructions = extract_instructions(&program);
        prop_assert!(
            has_opcode(&instructions, Opcode::Add32)
                || has_opcode(&instructions, Opcode::AddImm32)
                || has_opcode(&instructions, Opcode::Add64),
            "Expected Add opcode in compiled output for a={a}, b={b}"
        );
    }

    /// Division by param always produces a division opcode with div-by-zero trap guard.
    #[test]
    fn div_produces_div_opcode_and_trap_guard(_divisor in any::<i32>()) {
        let wat = r#"(module
            (memory 1)
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.div_u
            )
        )"#;
        let program = compile_wat(wat).expect("compile");
        let instructions = extract_instructions(&program);
        // Division should produce DivU32 with a trap guard for div-by-zero.
        // The divisor is a runtime parameter, so the guard must always be present.
        prop_assert!(
            has_opcode(&instructions, Opcode::DivU32),
            "Expected DivU32 opcode"
        );
        prop_assert!(
            has_opcode(&instructions, Opcode::Trap)
                || has_opcode(&instructions, Opcode::BranchEqImm)
                || has_opcode(&instructions, Opcode::BranchNeImm),
            "Expected div-by-zero trap guard"
        );
    }

    /// Division by constant zero compiles (LLVM may fold to trap).
    #[test]
    fn div_by_zero_const_compiles(_a in any::<i32>()) {
        let wat = r#"(module
            (memory 1)
            (func (export "main") (param i32) (result i32)
                local.get 0
                i32.const 0
                i32.div_u
            )
        )"#;
        let program = compile_wat(wat).expect("compile");
        let instructions = extract_instructions(&program);
        // LLVM folds div-by-zero-const to a trap; the emitted code must contain a Trap
        prop_assert!(
            has_opcode(&instructions, Opcode::Trap),
            "Division by constant 0 should produce a Trap"
        );
    }

    /// Comparison operations produce set or branch instructions.
    #[test]
    fn comparison_produces_set_or_branch(_a in any::<i32>()) {
        let wat = r#"(module
            (memory 1)
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.lt_u
            )
        )"#;
        let program = compile_wat(wat).expect("compile");
        let instructions = extract_instructions(&program);
        prop_assert!(
            has_opcode(&instructions, Opcode::SetLtU)
                || has_opcode(&instructions, Opcode::SetLtUImm)
                || has_opcode(&instructions, Opcode::BranchLtU)
                || has_opcode(&instructions, Opcode::BranchLtUImm),
            "Expected comparison opcode"
        );
    }

    /// Bitwise operations produce the correct opcode family.
    #[test]
    fn bitwise_and_produces_and_opcode(_a in any::<u32>()) {
        let wat = r#"(module
            (memory 1)
            (func (export "main") (param i32 i32) (result i32)
                local.get 0
                local.get 1
                i32.and
            )
        )"#;
        let program = compile_wat(wat).expect("compile");
        let instructions = extract_instructions(&program);
        prop_assert!(
            has_opcode(&instructions, Opcode::And),
            "Expected And opcode"
        );
    }
}

// =============================================================================
// Edge Cases: boundary values and special conditions
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Nested loops with random depth compile correctly.
    #[test]
    fn wasm_compile_nested_loops(depth in 1usize..5) {
        // Each loop has no result type; only the outermost function returns i32.
        let mut body = String::new();
        for _ in 0..depth {
            body.push_str("(block (loop ");
        }
        body.push_str("nop");
        for _ in 0..depth {
            body.push_str("))");
        }

        let wat = format!(
            r#"(module
                (memory 1)
                (func (export "main") (param i32) (result i32)
                    {body}
                    local.get 0
                )
            )"#
        );
        let result = compile_wat(&wat);
        prop_assert!(result.is_ok(), "Nested loops (depth={}) failed: {:?}", depth, result.err());
    }

    /// Nested blocks with random depth compile correctly.
    #[test]
    fn wasm_compile_nested_blocks(depth in 1usize..10) {
        let mut body = String::new();
        for _ in 0..depth {
            body.push_str("(block ");
        }
        body.push_str("nop");
        for _ in 0..depth {
            body.push(')');
        }

        let wat = format!(
            r#"(module
                (memory 1)
                (func (export "main") (param i32) (result i32)
                    {body}
                    local.get 0
                )
            )"#
        );
        let result = compile_wat(&wat);
        prop_assert!(result.is_ok(), "Nested blocks (depth={}) failed: {:?}", depth, result.err());
    }

    /// Chain of i32 operations with random length compiles correctly.
    #[test]
    fn wasm_compile_operation_chain(chain_len in 1usize..15) {
        let ops = ["i32.add", "i32.sub", "i32.mul", "i32.and", "i32.or", "i32.xor"];
        let mut body = String::from("local.get 0\n");
        for i in 0..chain_len {
            writeln!(body, "i32.const {}", i + 1).unwrap();
            body.push_str(ops[i % ops.len()]);
            body.push('\n');
        }

        let wat = format!(
            r#"(module
                (memory 1)
                (func (export "main") (param i32) (result i32)
                    {body}
                )
            )"#
        );
        let result = compile_wat(&wat);
        prop_assert!(result.is_ok(), "Chain (len={chain_len}) failed: {:?}", result.err());
    }

    /// Global variables with any i32 initial value compile correctly.
    #[test]
    fn wasm_compile_global_init(init_val in any::<i32>()) {
        let wat = format!(
            r#"(module
                (memory 1)
                (global $g (mut i32) (i32.const {init_val}))
                (func (export "main") (param i32) (result i32)
                    global.get $g
                )
            )"#
        );
        let result = compile_wat(&wat);
        prop_assert!(result.is_ok(), "Global init={init_val} failed: {:?}", result.err());
    }

    /// Functions with varying param counts compile correctly.
    #[test]
    fn wasm_compile_varying_params(num_params in 1usize..8) {
        let params: String = (0..num_params).map(|_| "(param i32)").collect::<Vec<_>>().join(" ");
        let wat = format!(
            r#"(module
                (memory 1)
                (func (export "main") {params} (result i32)
                    local.get 0
                )
            )"#
        );
        let result = compile_wat(&wat);
        prop_assert!(result.is_ok(), "Params count={num_params} failed: {:?}", result.err());
    }
}

// =============================================================================
// Immediate Encoding Properties
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// AddImm32 encodes the immediate value correctly.
    #[test]
    fn add_imm32_encoding(value in any::<i32>()) {
        let instr = wasm_pvm::Instruction::AddImm32 { dst: 2, src: 3, value };
        let encoded = instr.encode();
        prop_assert_eq!(encoded[0], Opcode::AddImm32 as u8);
        // Reg byte: (src << 4) | dst = (3 << 4) | 2 = 0x32
        prop_assert_eq!(encoded[1], 0x32);
        // Decode the immediate
        let imm_bytes = &encoded[2..];
        let mut padded = [0u8; 4];
        padded[..imm_bytes.len()].copy_from_slice(imm_bytes);
        // Sign-extend if needed
        if !imm_bytes.is_empty() && imm_bytes[imm_bytes.len() - 1] & 0x80 != 0 {
            for b in &mut padded[imm_bytes.len()..] {
                *b = 0xFF;
            }
        }
        let decoded = i32::from_le_bytes(padded);
        prop_assert_eq!(decoded, value, "AddImm32 immediate roundtrip failed");
    }

    /// LoadImm immediate encoding preserves values correctly.
    #[test]
    fn load_imm_roundtrip(value in any::<i32>()) {
        let instr = wasm_pvm::Instruction::LoadImm { reg: 5, value };
        let encoded = instr.encode();
        prop_assert_eq!(encoded[0], Opcode::LoadImm as u8);
        prop_assert_eq!(encoded[1], 5);
        // Decode
        let imm_bytes = &encoded[2..];
        let mut padded = [0u8; 4];
        padded[..imm_bytes.len()].copy_from_slice(imm_bytes);
        if !imm_bytes.is_empty() && imm_bytes[imm_bytes.len() - 1] & 0x80 != 0 {
            for b in &mut padded[imm_bytes.len()..] {
                *b = 0xFF;
            }
        }
        let decoded = i32::from_le_bytes(padded);
        prop_assert_eq!(decoded, value, "LoadImm roundtrip failed for value={}", value);
    }

    /// LoadImm64 preserves all 64 bits correctly.
    #[test]
    fn load_imm64_roundtrip(value in any::<u64>()) {
        let instr = wasm_pvm::Instruction::LoadImm64 { reg: 7, value };
        let encoded = instr.encode();
        prop_assert_eq!(encoded[0], Opcode::LoadImm64 as u8);
        prop_assert_eq!(encoded[1], 7);
        let decoded = u64::from_le_bytes(encoded[2..10].try_into().unwrap());
        prop_assert_eq!(decoded, value);
    }

    /// CmovIzImm/CmovNzImm TwoRegOneImm encoding roundtrip.
    #[test]
    fn cmov_imm_encoding(cond in 0u8..13, dst in 0u8..13, value in any::<i32>()) {
        let variants: Vec<(Opcode, wasm_pvm::Instruction)> = vec![
            (Opcode::CmovIzImm, wasm_pvm::Instruction::CmovIzImm { cond, dst, value }),
            (Opcode::CmovNzImm, wasm_pvm::Instruction::CmovNzImm { cond, dst, value }),
        ];

        for (opcode, instr) in &variants {
            let encoded = instr.encode();
            prop_assert_eq!(encoded[0], *opcode as u8, "wrong opcode for {:?}", opcode);
            prop_assert!(encoded.len() >= 2, "too short for {:?}", opcode);

            // Decode registers from byte 1
            let decoded_cond = (encoded[1] >> 4) & 0x0F;
            let decoded_dst = encoded[1] & 0x0F;
            prop_assert_eq!(decoded_cond, cond, "cond mismatch for {:?}", opcode);
            prop_assert_eq!(decoded_dst, dst, "dst mismatch for {:?}", opcode);

            // Decode immediate value
            let imm_bytes = &encoded[2..];
            let mut padded = if value < 0 { [0xFFu8; 4] } else { [0u8; 4] };
            padded[..imm_bytes.len()].copy_from_slice(imm_bytes);
            let decoded_val = i32::from_le_bytes(padded);
            prop_assert_eq!(decoded_val, value, "value mismatch for {:?}", opcode);
        }
    }

    /// StoreImm TwoImm encoding roundtrip: both immediates are recoverable.
    #[test]
    fn store_imm_encoding(address in any::<i32>(), value in any::<i32>()) {
        let variants: Vec<(Opcode, wasm_pvm::Instruction)> = vec![
            (Opcode::StoreImmU8, wasm_pvm::Instruction::StoreImmU8 { address, value }),
            (Opcode::StoreImmU16, wasm_pvm::Instruction::StoreImmU16 { address, value }),
            (Opcode::StoreImmU32, wasm_pvm::Instruction::StoreImmU32 { address, value }),
            (Opcode::StoreImmU64, wasm_pvm::Instruction::StoreImmU64 { address, value }),
        ];

        for (opcode, instr) in &variants {
            let encoded = instr.encode();
            prop_assert_eq!(encoded[0], *opcode as u8, "wrong opcode for {:?}", opcode);
            prop_assert!(encoded.len() >= 2, "too short for {:?}", opcode);

            // Decode: low nibble of byte 1 = address immediate length
            let addr_len = (encoded[1] & 0x0F) as usize;
            prop_assert!(addr_len <= 4, "addr_len out of range for {:?}", opcode);

            // Decode first immediate (address)
            let mut addr_bytes = if address < 0 { [0xFFu8; 4] } else { [0u8; 4] };
            addr_bytes[..addr_len].copy_from_slice(&encoded[2..2 + addr_len]);
            let decoded_addr = i32::from_le_bytes(addr_bytes);
            prop_assert_eq!(decoded_addr, address, "address mismatch for {:?}", opcode);

            // Decode second immediate (value)
            let val_start = 2 + addr_len;
            let val_len = encoded.len() - val_start;
            let mut val_bytes = if value < 0 { [0xFFu8; 4] } else { [0u8; 4] };
            val_bytes[..val_len].copy_from_slice(&encoded[val_start..]);
            let decoded_val = i32::from_le_bytes(val_bytes);
            prop_assert_eq!(decoded_val, value, "value mismatch for {:?}", opcode);
        }
    }
}
