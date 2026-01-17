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

fn encode_var_u32(mut value: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_var_u32() {
        assert_eq!(encode_var_u32(0), vec![0]);
        assert_eq!(encode_var_u32(1), vec![1]);
        assert_eq!(encode_var_u32(127), vec![127]);
        assert_eq!(encode_var_u32(128), vec![0x80, 0x01]);
        assert_eq!(encode_var_u32(300), vec![0xAC, 0x02]);
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
