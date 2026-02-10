/**
 * Test to trace what arr[i] values are returned during loop iterations.
 * This will help identify if specific indices return wrong values.
 */

const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

function createArray(len: i32): u8[] {
  const r = new Array<u8>(len);
  for (let i: i32 = 0; i < len; i++) {
    r[i] = <u8>i;
  }
  return r;
}

export function main(args_ptr: i32, args_len: i32): void {
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  const arr1 = createArray(10);
  const arr2 = createArray(10);  // Spilled local

  let result: u32 = 0;

  if (step == 0) {
    // Test 0: Record each arr[i] value in separate digit positions (limit from args)
    // Returns: v4*10000 + v3*1000 + v2*100 + v1*10 + v0
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    let collected: u32 = 0;
    let multiplier: u32 = 1;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      collected += arr1[i] * multiplier;
      multiplier *= 10;
    }
    result = collected;
    // With limit=5: Expected: 4*10000 + 3*1000 + 2*100 + 1*10 + 0*1 = 43210
  } else if (step == 1) {
    // Test 1: Same but hardcoded limit
    let collected: u32 = 0;
    let multiplier: u32 = 1;
    for (let i: i32 = 0; i < 5 && i < arr1.length; i++) {
      collected += arr1[i] * multiplier;
      multiplier *= 10;
    }
    result = collected;
    // Expected: 43210
  } else if (step == 2) {
    // Test 2: Store arr[i] to separate memory locations, then read back
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      store<u8>(0x30200 + i, arr1[i]);
    }
    // Read back and encode
    result = load<u8>(0x30200) + load<u8>(0x30201) * 10 + load<u8>(0x30202) * 100 +
             load<u8>(0x30203) * 1000 + load<u8>(0x30204) * 10000;
    // Expected: 43210
  } else if (step == 3) {
    // Test 3: Read arr[0..4] outside of loop (no loop dependency)
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    // Just to use limit and ensure ternary is evaluated
    if (limit > 0) {
      result = arr1[0] + arr1[1] * 10 + arr1[2] * 100 + arr1[3] * 1000 + arr1[4] * 10000;
    }
    // Expected: 43210
  } else if (step == 4) {
    // Test 4: Check what i and arr[i] are at each iteration (encode both)
    // Use: (i << 4 | arr[i]) for each, concatenate
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 3;
    let collected: u32 = 0;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      const val = arr1[i];
      collected = (collected << 8) | ((i << 4) | val);
    }
    result = collected;
    // With limit=3: iterations 0,1,2
    // i=0, arr[0]=0: 0x00
    // i=1, arr[1]=1: 0x11
    // i=2, arr[2]=2: 0x22
    // Result: 0x001122 = 4386
  } else if (step == 5) {
    // Test 5: Compare i vs arr[i] - they should be equal
    // If not equal, encode which iteration had mismatch
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    let mismatches: u32 = 0;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      if (<i32>arr1[i] != i) {
        mismatches |= (1 << i);  // Set bit for mismatched index
      }
    }
    result = mismatches;
    // Expected: 0 (no mismatches)
  } else if (step == 6) {
    // Test 6: Store i before array access, store arr[i], compare
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 3;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      store<u8>(0x30200 + i * 2, <u8>i);       // Store i
      store<u8>(0x30201 + i * 2, arr1[i]);     // Store arr[i]
    }
    // Read back: encode as pairs
    // i=0: (stored_i, arr_i) at 0x30200, 0x30201
    // i=1: (stored_i, arr_i) at 0x30202, 0x30203
    // i=2: (stored_i, arr_i) at 0x30204, 0x30205
    const p0 = load<u8>(0x30200) * 16 + load<u8>(0x30201);  // 0*16+0 = 0x00 = 0
    const p1 = load<u8>(0x30202) * 16 + load<u8>(0x30203);  // 1*16+1 = 0x11 = 17
    const p2 = load<u8>(0x30204) * 16 + load<u8>(0x30205);  // 2*16+2 = 0x22 = 34
    result = p0 + p1 * 256 + p2 * 65536;
    // Expected: 0 + 17*256 + 34*65536 = 4352 + 2228224 = 2232576
  } else if (step == 7) {
    // Test 7: Just arr[1] access with limit from args (isolate single access)
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    if (limit > 1) {
      result = arr1[1];
    }
    // Expected: 1
  } else {
    // Test 8: Sum without multiplier, just to confirm
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    let sum: u32 = 0;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      sum += arr1[i];
    }
    result = sum;
    // Expected: 10
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}
