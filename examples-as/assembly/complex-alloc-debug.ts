/**
 * Debug version of complex allocation test
 *
 * Returns result at each step to identify where things go wrong.
 * Input: step number (0-4)
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

  bytes(len: i32): Uint8Array {
    const v = this.source.subarray(this.offset, this.offset + len);
    this.offset += len;
    return v;
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
  // Read step number from args
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  // Create large buffer [0,1,2,...,255]
  const largeBuffer = new Uint8Array(256);
  for (let i: i32 = 0; i < 256; i++) {
    largeBuffer[i] = <u8>(i & 0xFF);
  }

  let result: u32 = 0;

  if (step == 0) {
    // Step 0: Test Decoder.u8() reading 10 bytes
    // Expected: 0+1+2+...+9 = 45
    const decoder = new Decoder(largeBuffer);
    for (let i: i32 = 0; i < 10; i++) {
      result += decoder.u8();
    }
  } else if (step == 1) {
    // Step 1: Test Decoder.bytes() with subarray
    // Expected: 0+1+2+...+19 = 190
    const decoder = new Decoder(largeBuffer);
    const slice = decoder.bytes(20);
    for (let i: i32 = 0; i < slice.length; i++) {
      result += slice[i];
    }
  } else if (step == 2) {
    // Step 2: Test lowerBytes (copy subarray to Array)
    // Expected: 0+1+2+...+9 = 45
    const decoder = new Decoder(largeBuffer);
    const slice = decoder.bytes(10);
    const arr = lowerBytes(slice);
    for (let i: i32 = 0; i < arr.length; i++) {
      result += arr[i];
    }
  } else if (step == 3) {
    // Step 3: Test pre-allocated array with direct indexing
    // Expected: 1+2+3+4+5 = 15
    const pushTest = new Array<u8>(5);
    pushTest[0] = 1;
    pushTest[1] = 2;
    pushTest[2] = 3;
    pushTest[3] = 4;
    pushTest[4] = 5;
    for (let i: i32 = 0; i < pushTest.length; i++) {
      result += pushTest[i];
    }
  } else if (step == 4) {
    // Step 4: Test multiple allocations
    // round 0: 0+31=31, round 1: 32+63=95, round 2: 64+95=159,
    // round 3: 96+127=223, round 4: 128+159=287
    // Expected: 31+95+159+223+287 = 795
    for (let round: i32 = 0; round < 5; round++) {
      const tempArr = new Uint8Array(32);
      for (let i: i32 = 0; i < 32; i++) {
        tempArr[i] = <u8>((round * 32 + i) & 0xFF);
      }
      result += <u32>tempArr[0] + <u32>tempArr[31];
    }
  } else {
    // Step 5+: Return total sum
    // Expected: 45 + 190 + 45 + 15 + 795 = 1090
    const decoder1 = new Decoder(largeBuffer);
    for (let i: i32 = 0; i < 10; i++) {
      result += decoder1.u8();
    }

    const decoder2 = new Decoder(largeBuffer);
    const slice = decoder2.bytes(20);
    for (let i: i32 = 0; i < slice.length; i++) {
      result += slice[i];
    }

    const decoder3 = new Decoder(largeBuffer);
    const slice2 = decoder3.bytes(10);
    const arr = lowerBytes(slice2);
    for (let i: i32 = 0; i < arr.length; i++) {
      result += arr[i];
    }

    const pushTest = new Array<u8>(5);
    pushTest[0] = 1;
    pushTest[1] = 2;
    pushTest[2] = 3;
    pushTest[3] = 4;
    pushTest[4] = 5;
    for (let i: i32 = 0; i < pushTest.length; i++) {
      result += pushTest[i];
    }

    for (let round: i32 = 0; round < 5; round++) {
      const tempArr = new Uint8Array(32);
      for (let i: i32 = 0; i < 32; i++) {
        tempArr[i] = <u8>((round * 32 + i) & 0xFF);
      }
      result += <u32>tempArr[0] + <u32>tempArr[31];
    }
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
