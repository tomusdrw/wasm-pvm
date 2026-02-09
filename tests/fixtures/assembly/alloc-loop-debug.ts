/**
 * Debug test for the multiple allocation loop issue.
 * Step 4 of complex-alloc-debug fails: expected 795, got 539 (diff: 256)
 *
 * Input: step (0-9)
 */

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

export function main(args_ptr: i32, args_len: i32): void {
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  let result: u32 = 0;

  if (step == 0) {
    // Test single allocation round 0: expected 0+31=31
    const tempArr = new Uint8Array(32);
    for (let i: i32 = 0; i < 32; i++) {
      tempArr[i] = <u8>((0 * 32 + i) & 0xFF);
    }
    result = <u32>tempArr[0] + <u32>tempArr[31];
  } else if (step == 1) {
    // Test single allocation round 1: expected 32+63=95
    const tempArr = new Uint8Array(32);
    for (let i: i32 = 0; i < 32; i++) {
      tempArr[i] = <u8>((1 * 32 + i) & 0xFF);
    }
    result = <u32>tempArr[0] + <u32>tempArr[31];
  } else if (step == 2) {
    // Test single allocation round 2: expected 64+95=159
    const tempArr = new Uint8Array(32);
    for (let i: i32 = 0; i < 32; i++) {
      tempArr[i] = <u8>((2 * 32 + i) & 0xFF);
    }
    result = <u32>tempArr[0] + <u32>tempArr[31];
  } else if (step == 3) {
    // Test single allocation round 3: expected 96+127=223
    const tempArr = new Uint8Array(32);
    for (let i: i32 = 0; i < 32; i++) {
      tempArr[i] = <u8>((3 * 32 + i) & 0xFF);
    }
    result = <u32>tempArr[0] + <u32>tempArr[31];
  } else if (step == 4) {
    // Test single allocation round 4: expected 128+159=287
    const tempArr = new Uint8Array(32);
    for (let i: i32 = 0; i < 32; i++) {
      tempArr[i] = <u8>((4 * 32 + i) & 0xFF);
    }
    result = <u32>tempArr[0] + <u32>tempArr[31];
  } else if (step == 5) {
    // Test full loop: expected 31+95+159+223+287=795
    for (let round: i32 = 0; round < 5; round++) {
      const tempArr = new Uint8Array(32);
      for (let i: i32 = 0; i < 32; i++) {
        tempArr[i] = <u8>((round * 32 + i) & 0xFF);
      }
      result += <u32>tempArr[0] + <u32>tempArr[31];
    }
  } else if (step == 6) {
    // Debug: return round values individually as bits
    // Format: round0[0]<<24 | round0[31]<<16 | round4[0]<<8 | round4[31]
    let r0_0: u32 = 0;
    let r0_31: u32 = 0;
    let r4_0: u32 = 0;
    let r4_31: u32 = 0;

    for (let round: i32 = 0; round < 5; round++) {
      const tempArr = new Uint8Array(32);
      for (let i: i32 = 0; i < 32; i++) {
        tempArr[i] = <u8>((round * 32 + i) & 0xFF);
      }
      if (round == 0) {
        r0_0 = tempArr[0];
        r0_31 = tempArr[31];
      } else if (round == 4) {
        r4_0 = tempArr[0];
        r4_31 = tempArr[31];
      }
    }
    // Expected: 0<<24 | 31<<16 | 128<<8 | 159 = 0x001F809F = 2064543
    result = (r0_0 << 24) | (r0_31 << 16) | (r4_0 << 8) | r4_31;
  } else if (step == 7) {
    // Debug: just compute (4*32+31) & 0xFF to check arithmetic
    // Expected: 159
    result = <u32>((4 * 32 + 31) & 0xFF);
  } else if (step == 8) {
    // Debug: store to Uint8Array and read back (no loop)
    const tempArr = new Uint8Array(32);
    tempArr[0] = 128;
    tempArr[31] = 159;
    result = <u32>tempArr[0] + <u32>tempArr[31];  // Expected: 287
  } else if (step == 9) {
    // Debug: loop-based fill then read (single array, round=4 values)
    const tempArr = new Uint8Array(32);
    for (let i: i32 = 0; i < 32; i++) {
      tempArr[i] = <u8>((4 * 32 + i) & 0xFF);
    }
    // Return all values encoded: [0]<<24 | [1]<<16 | [30]<<8 | [31]
    // Expected: 128<<24 | 129<<16 | 158<<8 | 159 = 0x80819E9F
    result = (<u32>tempArr[0] << 24) | (<u32>tempArr[1] << 16) | (<u32>tempArr[30] << 8) | <u32>tempArr[31];
  } else {
    result = 99;
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
