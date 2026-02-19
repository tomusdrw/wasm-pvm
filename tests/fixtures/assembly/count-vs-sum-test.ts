/**
 * Test to compare iteration count vs array sum in same loop.
 * This will tell us if the loop runs correct iterations but array access is corrupted.
 */


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
  const RESULT_HEAP = heap.alloc(256);
  const step: i32 = args_len > 0 ? load<u8>(args_ptr) : 0;

  const arr1 = createArray(10);
  const arr2 = createArray(10);  // Spilled local

  let result: u32 = 0;

  if (step == 0) {
    // Test 0: Count + sum in same loop (hardcoded limit)
    // Returns: count * 1000 + sum
    let count: i32 = 0;
    let sum: i32 = 0;
    for (let i: i32 = 0; i < 5 && i < arr1.length; i++) {
      count++;
      sum += arr1[i];
    }
    result = <u32>(count * 1000 + sum);
    // Expected: 5*1000 + 10 = 5010
  } else if (step == 1) {
    // Test 1: Count + sum in same loop (limit from args)
    let count: i32 = 0;
    let sum: i32 = 0;
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      count++;
      sum += arr1[i];
    }
    result = <u32>(count * 1000 + sum);
    // With limit=5: Expected: 5*1000 + 10 = 5010
  } else if (step == 2) {
    // Test 2: Just count (limit from args) - sanity check
    let count: i32 = 0;
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      count++;
    }
    result = <u32>count;
    // With limit=5: Expected: 5
  } else if (step == 3) {
    // Test 3: Just sum (limit from args) - confirms bug
    let sum: i32 = 0;
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      sum += arr1[i];
    }
    result = <u32>sum;
    // With limit=5: Expected: 10 (but will get 9)
  } else if (step == 4) {
    // Test 4: Count first, then access array (two statements)
    // Maybe it's the order of operations?
    let count: i32 = 0;
    let sum: i32 = 0;
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      const val = arr1[i];  // First get the value
      count++;              // Then count
      sum += val;           // Then accumulate
    }
    result = <u32>(count * 1000 + sum);
    // Expected: 5010
  } else if (step == 5) {
    // Test 5: Record i values seen using bits + record arr[i] values
    // bits in lower 16: which i values were seen
    // sum in upper 16: sum of arr[i]
    let bits: u32 = 0;
    let sum: u32 = 0;
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      bits |= (1 << i);
      sum += arr1[i];
    }
    result = (sum << 16) | bits;
    // With limit=5: Expected bits=31, sum=10 -> (10 << 16) | 31 = 655391
  } else if (step == 6) {
    // Test 6: Store i before and after array access
    // Returns: (i_before * 100 + i_after) for last iteration
    let last_i_before: i32 = -1;
    let last_i_after: i32 = -1;
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 2;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      last_i_before = i;
      const _v = arr1[i];  // This call might corrupt i?
      last_i_after = i;
    }
    result = <u32>(last_i_before * 100 + last_i_after);
    // With limit=2: Expected: 1*100 + 1 = 101
  } else {
    // Test 7: Sum without array access, just use i values
    let sum: i32 = 0;
    const limit: i32 = args_len > 1 ? load<u8>(args_ptr + 1) : 5;
    for (let i: i32 = 0; i < limit && i < arr1.length; i++) {
      sum += i;  // Use i directly, not arr[i]
    }
    result = <u32>sum;
    // With limit=5: Expected: 0+1+2+3+4 = 10
  }

  store<i32>(RESULT_HEAP, result);
  result_ptr = RESULT_HEAP as i32;
  result_len = 4;
}
