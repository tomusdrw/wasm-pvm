/**
 * Complex allocation test - mimics anan-as interpreter patterns
 *
 * This test creates multiple arrays, typed arrays, and objects
 * to stress test memory allocation and array operations in PVM context.
 *
 * Input: 8 bytes (ignored)
 * Expected: Checksum of operations
 */

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

// Simulate the Decoder class structure
class Decoder {
  constructor(
    private readonly source: Uint8Array,
    private offset: i32 = 0,
  ) {}

  u8(): u8 {
    const v = this.source[this.offset];
    this.offset += 1;
    return v;
  }

  u16(): u16 {
    let v: u16 = this.source[this.offset];
    v |= u16(this.source[this.offset + 1]) << 8;
    this.offset += 2;
    return v;
  }

  u32(): u32 {
    let v: u32 = this.source[this.offset];
    v |= u32(this.source[this.offset + 1]) << 8;
    v |= u32(this.source[this.offset + 2]) << 16;
    v |= u32(this.source[this.offset + 3]) << 24;
    this.offset += 4;
    return v;
  }

  // This uses subarray - the pattern that potentially breaks
  bytes(len: i32): Uint8Array {
    const v = this.source.subarray(this.offset, this.offset + len);
    this.offset += len;
    return v;
  }

  // varU32 pattern with subarray
  varU32Simple(): u32 {
    // Simplified: just read single byte (< 0x80)
    const firstByte = this.source[this.offset];
    if (firstByte < 0x80) {
      this.offset += 1;
      return firstByte;
    }
    // For this test, we only handle single-byte values
    this.offset += 1;
    return firstByte;
  }
}

// lowerBytes pattern - copy Uint8Array to Array<u8>
function lowerBytes(data: Uint8Array): u8[] {
  const r = new Array<u8>(data.length);
  for (let i = 0; i < data.length; i++) {
    r[i] = data[i];
  }
  return r;
}

export function main(args_ptr: i32, args_len: i32): void {
  // Step 1: Create a Uint8Array from args (like liftBytes)
  const inputData = new Uint8Array(args_len);
  for (let i: i32 = 0; i < args_len; i++) {
    inputData[i] = load<u8>(args_ptr + i);
  }

  // Step 2: Create a larger buffer for testing
  const largeBuffer = new Uint8Array(256);
  for (let i: i32 = 0; i < 256; i++) {
    largeBuffer[i] = <u8>(i & 0xFF);
  }

  // Step 3: Create Decoder and read values
  const decoder = new Decoder(largeBuffer);
  let sum: u32 = 0;

  // Read 10 u8 values
  for (let i: i32 = 0; i < 10; i++) {
    sum += decoder.u8();
  }

  // Step 4: Test bytes() method (uses subarray)
  const decoder2 = new Decoder(largeBuffer);
  const slice = decoder2.bytes(20);
  for (let i: i32 = 0; i < slice.length; i++) {
    sum += slice[i];
  }

  // Step 5: Test lowerBytes (copies to Array)
  const decoder3 = new Decoder(largeBuffer);
  const slice2 = decoder3.bytes(10);
  const arr = lowerBytes(slice2);
  for (let i: i32 = 0; i < arr.length; i++) {
    sum += arr[i];
  }

  // Step 6: Test Array.push pattern (pre-allocated to avoid potential bug)
  const pushTest = new Array<u8>(5);
  pushTest[0] = 1;
  pushTest[1] = 2;
  pushTest[2] = 3;
  pushTest[3] = 4;
  pushTest[4] = 5;
  for (let i: i32 = 0; i < pushTest.length; i++) {
    sum += pushTest[i];
  }

  // Step 7: Multiple allocations
  for (let round: i32 = 0; round < 5; round++) {
    const tempArr = new Uint8Array(32);
    for (let i: i32 = 0; i < 32; i++) {
      tempArr[i] = <u8>((round * 32 + i) & 0xFF);
    }
    sum += <u32>tempArr[0] + <u32>tempArr[31];
  }

  // Expected:
  // u8s: 0+1+2+...+9 = 45
  // slice: 0+1+2+...+19 = 190
  // lowerBytes: 0+1+2+...+9 = 45
  // push: 1+2+3+4+5 = 15
  // rounds: (0+31) + (32+63) + (64+95) + (96+127) + (128+159) = 31+95+159+223+287 = 795
  // Total: 45 + 190 + 45 + 15 + 795 = 1090

  store<i32>(RESULT_HEAP, sum);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
