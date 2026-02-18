// SPI encoding uses u32 lengths but writes u24. Truncation is checked or expected.
#![allow(clippy::cast_possible_truncation)]

use crate::pvm::ProgramBlob;

pub struct SpiProgram {
    metadata: Vec<u8>,
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
            metadata: Vec::new(),
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
    pub fn with_metadata(mut self, data: Vec<u8>) -> Self {
        self.metadata = data;
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
    pub fn metadata(&self) -> &[u8] {
        &self.metadata
    }

    /// Encode the full SPI program with metadata prefix.
    ///
    /// Format: `[varint: metadata_len][metadata_bytes][SPI header + data + code]`
    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let spi = self.encode_spi();

        let mut output = Vec::new();
        output.extend(crate::pvm::encode_var_u32(self.metadata.len() as u32));
        output.extend(&self.metadata);
        output.extend(spi);

        output
    }

    /// Encode just the raw SPI program without the metadata prefix.
    #[must_use]
    pub fn encode_spi(&self) -> Vec<u8> {
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

        // First byte is varint-encoded metadata length (0 = empty metadata).
        assert_eq!(encoded[0], 0, "metadata length varint should be 0");

        // SPI header starts at offset 1.
        assert_eq!(&encoded[1..4], &[0, 0, 0], "ro_data_len");
        assert_eq!(&encoded[4..7], &[0, 0, 0], "rw_data_len");
        assert_eq!(&encoded[7..9], &16u16.to_le_bytes(), "heap_pages");
        let stack_bytes = encode_u24(64 * 1024);
        assert_eq!(&encoded[9..12], &stack_bytes, "stack_size");
    }

    #[test]
    fn test_spi_encode_with_metadata() {
        let code = ProgramBlob::new(vec![Instruction::Trap]);
        let metadata = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let spi = SpiProgram::new(code).with_metadata(metadata.clone());
        let encoded = spi.encode();

        // Metadata length varint: 4 encodes as [4].
        assert_eq!(encoded[0], 4, "metadata length varint should be 4");

        // Metadata bytes follow.
        assert_eq!(&encoded[1..5], &metadata, "metadata content");

        // SPI header starts after metadata.
        assert_eq!(&encoded[5..8], &[0, 0, 0], "ro_data_len after metadata");
    }

    #[test]
    fn test_encode_spi_no_metadata() {
        let code = ProgramBlob::new(vec![Instruction::Trap]);
        let spi = SpiProgram::new(code);

        let raw_spi = spi.encode_spi();
        let with_meta = spi.encode();

        // encode() should be encode_spi() prefixed with varint(0).
        assert_eq!(with_meta[0], 0);
        assert_eq!(&with_meta[1..], &raw_spi[..]);
    }
}
