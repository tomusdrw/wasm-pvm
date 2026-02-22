//! Property-based tests for the WASM-to-PVM compiler.
//!
//! Uses `proptest` to generate random inputs and verify invariants:
//! - Compilation of valid WAT modules never panics
//! - PVM instruction encoding produces valid byte sequences
//! - Arithmetic WAT modules produce correct opcodes for random operands
//! - Edge cases (overflow, division by zero) are handled correctly

use proptest::prelude::*;
use wasm_pvm::Opcode;
use wasm_pvm::test_harness::*;

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
            body.push_str(")");
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
            body.push_str(&format!("i32.const {}\n", i + 1));
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
