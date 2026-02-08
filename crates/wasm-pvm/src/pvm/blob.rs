use super::Instruction;

pub struct ProgramBlob {
    instructions: Vec<Instruction>,
    jump_table: Vec<u32>,
}

impl ProgramBlob {
    #[must_use]
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Self {
            instructions,
            jump_table: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_jump_table(mut self, jump_table: Vec<u32>) -> Self {
        self.jump_table = jump_table;
        self
    }

    #[must_use]
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let (code, mask) = self.encode_code_and_mask();
        let code_len = code.len();

        let mut blob = Vec::new();

        blob.extend(encode_var_u32(self.jump_table.len() as u32));

        let item_len: u8 = if self.jump_table.is_empty() { 0 } else { 4 };
        blob.push(item_len);

        blob.extend(encode_var_u32(code_len as u32));

        for &addr in &self.jump_table {
            blob.extend(addr.to_le_bytes());
        }

        blob.extend(code);
        blob.extend(mask);

        blob
    }

    fn encode_code_and_mask(&self) -> (Vec<u8>, Vec<u8>) {
        let mut code = Vec::new();
        let mut mask_bits = Vec::new();

        for instr in &self.instructions {
            let encoded = instr.encode();
            let start_offset = code.len();

            code.extend(&encoded);

            for i in 0..encoded.len() {
                mask_bits.push(i == 0);
            }

            if instr.is_terminating() && start_offset + encoded.len() < code.len() {}
        }

        let mask = pack_mask(&mask_bits);
        (code, mask)
    }
}

fn pack_mask(bits: &[bool]) -> Vec<u8> {
    let mut packed = Vec::new();
    for chunk in bits.chunks(8) {
        let mut byte: u8 = 0;
        for (i, &bit) in chunk.iter().enumerate() {
            if bit {
                byte |= 1 << i;
            }
        }
        packed.push(byte);
    }
    packed
}

fn encode_var_u32(value: u32) -> Vec<u8> {
    if value == 0 {
        return vec![0];
    }

    let value = u64::from(value);
    let max_encoded: u64 = 1 << (7 * 8);

    if value >= max_encoded {
        let mut dest = vec![0xff];
        dest.extend(&value.to_le_bytes());
        return dest;
    }

    let mut min_encoded = max_encoded >> 7;
    for l in (0..=7).rev() {
        if value >= min_encoded {
            let mut dest = vec![0u8; l + 1];
            let max_val = 1u64 << (8 * l);
            let first_byte = (1u64 << 8) - (1u64 << (8 - l)) + value / max_val;
            dest[0] = first_byte as u8;

            let mut rest = value % max_val;
            for item in dest.iter_mut().skip(1) {
                *item = rest as u8;
                rest >>= 8;
            }
            return dest;
        }
        min_encoded >>= 7;
    }

    vec![value as u8]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_var_u32() {
        assert_eq!(encode_var_u32(0), vec![0]);
        assert_eq!(encode_var_u32(1), vec![1]);
        assert_eq!(encode_var_u32(127), vec![127]);
        assert_eq!(encode_var_u32(128), vec![0x80, 0x80]);
        assert_eq!(encode_var_u32(145), vec![0x80, 0x91]);
        assert_eq!(encode_var_u32(300), vec![0x81, 0x2c]);
        assert_eq!(encode_var_u32(16383), vec![0xbf, 0xff]);
        assert_eq!(encode_var_u32(16384), vec![0xc0, 0x00, 0x40]);
    }

    #[test]
    fn test_pack_mask() {
        assert_eq!(pack_mask(&[true, false, false]), vec![0b0000_0001]);
        assert_eq!(
            pack_mask(&[true, false, false, false, false, false, false, false, true]),
            vec![0b0000_0001, 0b0000_0001]
        );
    }

    #[test]
    fn test_load_imm64_mask() {
        // LoadImm64 encodes to 10 bytes, Trap to 1 byte
        let instructions = vec![
            Instruction::LoadImm64 {
                reg: 7,
                value: 0xFEFD_0000,
            },
            Instruction::Trap,
        ];

        let blob = ProgramBlob::new(instructions);
        let (code, mask) = blob.encode_code_and_mask();

        // LoadImm64 = 10 bytes, Trap = 1 byte → total 11 bytes
        assert_eq!(code.len(), 11, "code length should be 11");

        // Check that the opcode is correct
        assert_eq!(code[0], 20, "first byte should be LoadImm64 opcode (20)");
        assert_eq!(code[10], 0, "byte 10 should be Trap opcode (0)");

        // Check mask: bit 0 = 1 (LoadImm64 start), bits 1-9 = 0, bit 10 = 1 (Trap start)
        let mask_bits: Vec<bool> = (0..11)
            .map(|pc| {
                let byte_idx = pc / 8;
                let bit_idx = pc % 8;
                (mask[byte_idx] >> bit_idx) & 1 == 1
            })
            .collect();

        assert!(mask_bits[0], "PC 0 should be instruction start");
        for pc in 1..10 {
            assert!(
                !mask_bits[pc],
                "PC {} should NOT be instruction start",
                pc
            );
        }
        assert!(mask_bits[10], "PC 10 should be instruction start (Trap)");
    }
}
