import fs from "node:fs";

/**
 * Parse and verify the structural integrity of a JAM/SPI file.
 * Throws if the file is missing, too small, or structurally invalid.
 */
export function verifyJamStructure(jamFile: string): {
  roLength: number;
  rwLength: number;
  heapPages: number;
  stackSize: number;
  codeLength: number;
  jumpTableLength: number;
  blobCodeLength: number;
  instrCount: number;
} {
  if (!fs.existsSync(jamFile)) {
    throw new Error(`JAM file not found: ${jamFile}`);
  }

  const data = fs.readFileSync(jamFile);
  if (data.length < 12) {
    throw new Error(`JAM file too small (${data.length} bytes)`);
  }

  let offset = 0;

  // Skip metadata prefix (varint length + metadata bytes)
  const { value: metadataLength, bytesRead: metaVarBytes } = readVarU32(data, offset);
  offset += metaVarBytes + metadataLength;

  if (offset + 11 > data.length) {
    throw new Error(`File too small after metadata (offset ${offset}, length ${data.length})`);
  }

  // Parse SPI header (3+3+2+3 = 11 bytes)
  const roLength = data[offset] | (data[offset + 1] << 8) | (data[offset + 2] << 16);
  offset += 3;

  const rwLength = data[offset] | (data[offset + 1] << 8) | (data[offset + 2] << 16);
  offset += 3;

  const heapPages = data[offset] | (data[offset + 1] << 8);
  offset += 2;

  const stackSize = data[offset] | (data[offset + 1] << 8) | (data[offset + 2] << 16);
  offset += 3;

  // Validate RO/RW data regions fit
  if (offset + roLength + rwLength + 4 > data.length) {
    throw new Error(
      `SPI header claims RO(${roLength})+RW(${rwLength}) but file is only ${data.length} bytes (header at ${offset - 11})`
    );
  }

  offset += roLength;
  offset += rwLength;

  const codeLength =
    data[offset] | (data[offset + 1] << 8) | (data[offset + 2] << 16) | (data[offset + 3] << 24);
  offset += 4;

  if (codeLength === 0) {
    throw new Error("Code length is 0");
  }

  // Parse PVM blob header
  const { value: jumpTableLength, bytesRead: jtlBytes } = readVarU32(data, offset);
  offset += jtlBytes;

  const jumpTableItemBytes = data[offset];
  offset += 1;

  const { value: blobCodeLength, bytesRead: bclBytes } = readVarU32(data, offset);
  offset += bclBytes;

  if (blobCodeLength === 0) {
    throw new Error("Blob code length is 0");
  }

  // Skip jump table
  offset += jumpTableLength * jumpTableItemBytes;

  // Skip code
  offset += blobCodeLength;

  // Count instructions from mask
  const maskLength = Math.ceil(blobCodeLength / 8);
  let instrCount = 0;
  for (let i = 0; i < maskLength && offset + i < data.length; i++) {
    let b = data[offset + i];
    while (b) {
      instrCount += b & 1;
      b >>= 1;
    }
  }

  if (instrCount === 0) {
    throw new Error("No instructions found in mask");
  }

  return {
    roLength,
    rwLength,
    heapPages,
    stackSize,
    codeLength,
    jumpTableLength,
    blobCodeLength,
    instrCount,
  };
}

function readVarU32(
  data: Buffer,
  offset: number
): { value: number; bytesRead: number } {
  if (offset >= data.length) {
    throw new RangeError(`readVarU32: offset ${offset} out of bounds (length ${data.length})`);
  }

  const firstByte = data[offset];

  if (firstByte < 0x80) {
    return { value: firstByte, bytesRead: 1 };
  } else if (firstByte < 0xc0) {
    if (offset + 2 > data.length) {
      throw new RangeError(`readVarU32: need 2 bytes at offset ${offset}, only ${data.length - offset} available`);
    }
    const value = ((firstByte - 0x80) << 8) | data[offset + 1];
    return { value, bytesRead: 2 };
  } else if (firstByte < 0xe0) {
    if (offset + 3 > data.length) {
      throw new RangeError(`readVarU32: need 3 bytes at offset ${offset}, only ${data.length - offset} available`);
    }
    const value =
      ((firstByte - 0xc0) << 16) | data[offset + 1] | (data[offset + 2] << 8);
    return { value, bytesRead: 3 };
  } else {
    if (offset + 4 > data.length) {
      throw new RangeError(`readVarU32: need 4 bytes at offset ${offset}, only ${data.length - offset} available`);
    }
    const value =
      ((firstByte - 0xe0) << 24) |
      data[offset + 1] |
      (data[offset + 2] << 8) |
      (data[offset + 3] << 16);
    return { value, bytesRead: 4 };
  }
}
