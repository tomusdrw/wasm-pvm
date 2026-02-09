#!/usr/bin/env bun
/**
 * Verify that a JAM/SPI file is structurally valid.
 *
 * Usage: bun tests/utils/verify-jam.ts <jam-file>
 */

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ananAsPath = path.join(__dirname, '../../vendor/anan-as/build/release.js');

async function main() {
  const jamFile = process.argv[2];

  if (!jamFile) {
    console.error('Usage: verify-jam.ts <jam-file>');
    process.exit(1);
  }

  if (!fs.existsSync(jamFile)) {
    console.error(`Error: File not found: ${jamFile}`);
    process.exit(1);
  }

  const fileSize = fs.statSync(jamFile).size;
  console.log(`File: ${jamFile}`);
  console.log(`Size: ${fileSize} bytes (${(fileSize / 1024).toFixed(1)} KB)`);
  console.log();

  const data = fs.readFileSync(jamFile);

  // Parse SPI header
  console.log('=== SPI Header ===');
  let offset = 0;

  const roLength = data[offset] | (data[offset + 1] << 8) | (data[offset + 2] << 16);
  offset += 3;
  console.log(`RO Data Length: ${roLength} bytes`);

  const rwLength = data[offset] | (data[offset + 1] << 8) | (data[offset + 2] << 16);
  offset += 3;
  console.log(`RW Data Length: ${rwLength} bytes`);

  const heapPages = data[offset] | (data[offset + 1] << 8);
  offset += 2;
  console.log(`Heap Pages: ${heapPages} (${heapPages * 4} KB)`);

  const stackSize = data[offset] | (data[offset + 1] << 8) | (data[offset + 2] << 16);
  offset += 3;
  console.log(`Stack Size: ${stackSize} bytes`);

  const roDataStart = offset;
  offset += roLength;
  console.log(`RO Data: offset ${roDataStart}, ${roLength} bytes`);

  const rwDataStart = offset;
  offset += rwLength;
  console.log(`RW Data: offset ${rwDataStart}, ${rwLength} bytes`);

  const codeLength = data[offset] | (data[offset + 1] << 8) | (data[offset + 2] << 16) | (data[offset + 3] << 24);
  offset += 4;
  console.log(`Code Length: ${codeLength} bytes`);

  console.log();
  console.log('=== PVM Blob ===');
  const blobStart = offset;

  const { value: jumpTableLength, bytesRead: jtlBytes } = readVarU32(data, offset);
  offset += jtlBytes;
  console.log(`Jump Table Length: ${jumpTableLength} entries`);

  const jumpTableItemBytes = data[offset];
  offset += 1;
  console.log(`Jump Table Item Bytes: ${jumpTableItemBytes}`);

  const { value: blobCodeLength, bytesRead: bclBytes } = readVarU32(data, offset);
  offset += bclBytes;
  console.log(`Blob Code Length: ${blobCodeLength} bytes`);

  const jumpTableStart = offset;
  const jumpTableSize = jumpTableLength * jumpTableItemBytes;
  offset += jumpTableSize;
  console.log(`Jump Table: offset ${jumpTableStart - blobStart}, ${jumpTableSize} bytes`);

  if (jumpTableLength > 0 && jumpTableItemBytes > 0) {
    console.log(`  First entries:`);
    for (let i = 0; i < Math.min(10, jumpTableLength); i++) {
      let entry = 0;
      for (let j = 0; j < jumpTableItemBytes; j++) {
        entry |= data[jumpTableStart + i * jumpTableItemBytes + j] << (j * 8);
      }
      console.log(`    [${i}] = ${entry} (0x${entry.toString(16)})`);
    }
    if (jumpTableLength > 10) {
      console.log(`    ... and ${jumpTableLength - 10} more`);
    }
  }

  const codeStart = offset;
  offset += blobCodeLength;
  console.log(`Code: offset ${codeStart - blobStart}, ${blobCodeLength} bytes`);

  const maskLength = Math.ceil(blobCodeLength / 8);
  const maskStart = offset;
  console.log(`Mask: offset ${maskStart - blobStart}, ${maskLength} bytes`);

  let instrCount = 0;
  for (let i = 0; i < maskLength && i < data.length - maskStart; i++) {
    const byte = data[maskStart + i];
    let b = byte;
    while (b) {
      instrCount += b & 1;
      b >>= 1;
    }
  }
  console.log(`Instruction Count: ~${instrCount}`);

  const totalParsed = offset + maskLength - blobStart;
  console.log();
  console.log('=== Verification ===');
  console.log(`Total blob parsed: ${totalParsed} bytes`);
  console.log(`Expected code length: ${codeLength} bytes`);

  if (Math.abs(totalParsed - codeLength) < 10) {
    console.log('Status: OK - Structure appears valid');
  } else {
    console.log(`Status: WARNING - Size mismatch (diff: ${totalParsed - codeLength})`);
  }

  console.log();
  console.log('=== anan-as Validation ===');
  try {
    const ananAs = await import(ananAsPath);

    const program = ananAs.prepareProgram(
      ananAs.InputKind.SPI,
      ananAs.HasMetadata.No,
      Array.from(data),
      [],
      [],
      [],
      []
    );

    console.log('prepareProgram: OK');
    console.log(`Program loaded successfully`);
  } catch (err: any) {
    console.log(`prepareProgram: FAILED - ${err.message}`);
  }
}

function readVarU32(data: Buffer, offset: number): { value: number; bytesRead: number } {
  const firstByte = data[offset];

  if (firstByte < 0x80) {
    return { value: firstByte, bytesRead: 1 };
  } else if (firstByte < 0xc0) {
    const value = ((firstByte - 0x80) << 8) | data[offset + 1];
    return { value, bytesRead: 2 };
  } else if (firstByte < 0xe0) {
    const value = ((firstByte - 0xc0) << 16) | (data[offset + 1] << 8) | data[offset + 2];
    return { value, bytesRead: 3 };
  } else {
    const value = ((firstByte - 0xe0) << 24) | (data[offset + 1] << 16) | (data[offset + 2] << 8) | data[offset + 3];
    return { value, bytesRead: 4 };
  }
}

main().catch(err => {
  console.error('Error:', err);
  process.exit(1);
});
