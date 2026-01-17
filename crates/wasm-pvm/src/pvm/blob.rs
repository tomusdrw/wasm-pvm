use super::Instruction;

pub struct ProgramBlob {
    instructions: Vec<Instruction>,
}

impl ProgramBlob {
    #[must_use]
    pub fn new(instructions: Vec<Instruction>) -> Self {
        Self { instructions }
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let (code, mask) = self.encode_code_and_mask();
        let code_len = code.len();

        let mut blob = Vec::new();
        blob.push(0);
        blob.push(0);
        blob.extend(encode_var_u32(code_len as u32));
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
}
