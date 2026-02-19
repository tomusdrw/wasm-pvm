/**
 * Debug test to trace exactly where values get corrupted.
 * Stores intermediate values to memory for inspection.
 */

// Hardcoded address in globals storage area (0x30000+) for debug values.
// We use a fixed address here (instead of heap.alloc) so debug stores don't
// interfere with the heap allocator being tested, and so addresses are
// deterministic for inspection.
const DEBUG_HEAP: u32 = 0x30200;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

function createArray(): u8[] {
  const r = new Array<u8>(5);
  r[0] = 0; r[1] = 1; r[2] = 2; r[3] = 3; r[4] = 4;
  return r;
}

export function main(args_ptr: i32, args_len: i32): void {
  const RESULT_HEAP = heap.alloc(256);
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  const arr = createArray();

  // Store arr pointer for reference
  store<u32>(DEBUG_HEAP, changetype<usize>(arr));

  // Store arr's data pointer
  const dataPtr = load<u32>(changetype<usize>(arr) + 4);
  store<u32>(DEBUG_HEAP + 4, dataPtr);

  // Store arr[1] directly via data pointer for comparison
  const directValue = load<u8>(dataPtr + 1);
  store<u8>(DEBUG_HEAP + 8, directValue);

  let result: u32;

  if (step == 0) {
    // Test 0: Return debug info (arr ptr, dataPtr, directValue)
    result = (load<u32>(DEBUG_HEAP) & 0xFF) << 16
           | (load<u32>(DEBUG_HEAP + 4) & 0xFF) << 8
           | load<u8>(DEBUG_HEAP + 8);
  } else if (step == 1) {
    // Test 1: arr[1] via index operator BEFORE ternary
    const valueBefore = arr[1];
    store<u8>(DEBUG_HEAP + 9, valueBefore);  // Store for inspection

    // Now the ternary (this seems to corrupt something)
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    store<u8>(DEBUG_HEAP + 10, <u8>limit);  // Store limit

    // arr[1] AFTER ternary
    const valueAfter = arr[1];
    store<u8>(DEBUG_HEAP + 11, valueAfter);  // Store for inspection

    // Return: valueBefore*100 + valueAfter
    // Expected: 1*100 + 1 = 101
    result = <u32>(valueBefore * 100 + valueAfter);
  } else if (step == 2) {
    // Test 2: Same but encode all debug values in result
    const valueBefore = arr[1];
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    const valueAfter = arr[1];

    // Encode: valueBefore<<16 | limit<<8 | valueAfter
    // Expected: 1<<16 | 5<<8 | 1 = 0x010501 = 66817
    result = <u32>(valueBefore << 16 | limit << 8 | valueAfter);
  } else if (step == 3) {
    // Test 3: Check if arr pointer survives the ternary
    const arrPtrBefore = changetype<usize>(arr);
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    const arrPtrAfter = changetype<usize>(arr);

    // These should be equal
    result = arrPtrBefore == arrPtrAfter ? 1 : 0;
    // Expected: 1
  } else if (step == 4) {
    // Test 4: Get arr's dataPtr AFTER ternary
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    const dataPtrAfter = load<u32>(changetype<usize>(arr) + 4);
    const valueViaPtr = load<u8>(dataPtrAfter + 1);

    // This accesses arr[1] via pointer, should be 1
    result = valueViaPtr;
    // Expected: 1
  } else if (step == 5) {
    // Test 5: arr[0] after ternary (to force two-arg __get)
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    result = arr[0];
    // Expected: 0
  } else if (step == 6) {
    // Test 6: arr[2] after ternary
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    result = arr[2];
    // Expected: 2
  } else if (step == 7) {
    // Test 7: arr[3] after ternary
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    result = arr[3];
    // Expected: 3
  } else if (step == 8) {
    // Test 8: Access multiple indices without ternary
    result = <u32>(arr[0] * 1000 + arr[1] * 100 + arr[2] * 10 + arr[3]);
    // Expected: 0*1000 + 1*100 + 2*10 + 3 = 123
  } else if (step == 9) {
    // Test 9: Access multiple indices WITH ternary before each
    const l1: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    const v0 = arr[0];
    const l2: i32 = args_len > 2 ? load<u8>(args_ptr + 2) : 5;
    const v1 = arr[1];
    const l3: i32 = args_len > 3 ? load<u8>(args_ptr + 3) : 5;
    const v2 = arr[2];
    result = <u32>(v0 * 100 + v1 * 10 + v2);
    // Expected: 0*100 + 1*10 + 2 = 12
  } else {
    result = 99;
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}
