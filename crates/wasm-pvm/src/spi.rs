use crate::pvm::ProgramBlob;

pub struct SpiProgram {
    ro_data: Vec<u8>,
    rw_data: Vec<u8>,
    heap_pages: u16,
    stack_size: u32,
    code: ProgramBlob,
}

impl SpiProgram {
    #[must_use]
    pub fn new(code: ProgramBlob) -> Self {
        Self {
            ro_data: Vec::new(),
            rw_data: Vec::new(),
            heap_pages: 16,
            stack_size: 64 * 1024,
            code,
        }
    }

    #[must_use]
    pub fn with_stack_size(mut self, size: u32) -> Self {
        self.stack_size = size;
        self
    }

    #[must_use]
    pub fn with_heap_pages(mut self, pages: u16) -> Self {
        self.heap_pages = pages;
        self
    }

    #[must_use]
    pub fn with_ro_data(mut self, data: Vec<u8>) -> Self {
        self.ro_data = data;
        self
    }

    #[must_use]
    pub fn with_rw_data(mut self, data: Vec<u8>) -> Self {
        self.rw_data = data;
        self
    }

    #[must_use]
    pub fn code(&self) -> &ProgramBlob {
        &self.code
    }

    #[must_use]
    pub fn ro_data(&self) -> &[u8] {
        &self.ro_data
    }

    #[must_use]
    pub fn rw_data(&self) -> &[u8] {
        &self.rw_data
    }

    #[must_use]
    pub fn heap_pages(&self) -> u16 {
        self.heap_pages
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let code_blob = self.code.encode();

        let mut spi = Vec::new();

        spi.extend(encode_u24(self.ro_data.len() as u32));
        spi.extend(encode_u24(self.rw_data.len() as u32));
        spi.extend(self.heap_pages.to_le_bytes());
        spi.extend(encode_u24(self.stack_size));
        spi.extend(&self.ro_data);
        spi.extend(&self.rw_data);
        spi.extend((code_blob.len() as u32).to_le_bytes());
        spi.extend(code_blob);

        spi
    }
}

fn encode_u24(value: u32) -> [u8; 3] {
    let bytes = value.to_le_bytes();
    [bytes[0], bytes[1], bytes[2]]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pvm::Instruction;

    #[test]
    fn test_spi_encode_minimal() {
        let code = ProgramBlob::new(vec![Instruction::Trap]);
        let spi = SpiProgram::new(code);
        let encoded = spi.encode();

        assert_eq!(&encoded[0..3], &[0, 0, 0]);
        assert_eq!(&encoded[3..6], &[0, 0, 0]);
        assert_eq!(&encoded[6..8], &16u16.to_le_bytes());
        let stack_bytes = encode_u24(64 * 1024);
        assert_eq!(&encoded[8..11], &stack_bytes);
    }

    #[test]
    fn encode_u24_zero() {
        assert_eq!(encode_u24(0), [0, 0, 0]);
    }

    #[test]
    fn encode_u24_one() {
        assert_eq!(encode_u24(1), [1, 0, 0]);
    }

    #[test]
    fn encode_u24_max() {
        assert_eq!(encode_u24(0xFFFFFF), [0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn encode_u24_mid_value() {
        // 0x010203 in LE: [0x03, 0x02, 0x01]
        assert_eq!(encode_u24(0x010203), [0x03, 0x02, 0x01]);
    }

    #[test]
    fn test_spi_encode_with_ro_and_rw_data() {
        let code = ProgramBlob::new(vec![Instruction::Trap]);
        let ro_data = vec![0xAA, 0xBB, 0xCC];
        let rw_data = vec![0x11, 0x22];
        let spi = SpiProgram::new(code)
            .with_ro_data(ro_data.clone())
            .with_rw_data(rw_data.clone());
        let encoded = spi.encode();

        // Header: ro_len(3) + rw_len(3) + heap_pages(2) + stack_size(3) = 11 bytes
        assert_eq!(&encoded[0..3], &encode_u24(3)); // ro_data len
        assert_eq!(&encoded[3..6], &encode_u24(2)); // rw_data len
        assert_eq!(&encoded[6..8], &16u16.to_le_bytes()); // default heap_pages
        assert_eq!(&encoded[8..11], &encode_u24(64 * 1024)); // default stack_size

        // Data sections follow header
        assert_eq!(&encoded[11..14], &[0xAA, 0xBB, 0xCC]); // ro_data
        assert_eq!(&encoded[14..16], &[0x11, 0x22]); // rw_data
    }

    #[test]
    fn test_builder_methods() {
        let code = ProgramBlob::new(vec![Instruction::Trap]);
        let spi = SpiProgram::new(code)
            .with_heap_pages(42)
            .with_stack_size(128 * 1024);
        let encoded = spi.encode();

        assert_eq!(&encoded[6..8], &42u16.to_le_bytes());
        assert_eq!(&encoded[8..11], &encode_u24(128 * 1024));
    }
}
