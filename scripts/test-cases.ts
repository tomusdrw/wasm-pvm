export interface TestCase {
  name: string;
  tests: Array<{
    args: string;
    expected: number;
    description: string;
    pc?: number;
  }>;
}

export const testCases: TestCase[] = [
  {
    name: 'start-section',
    tests: [
      { args: '00000000', expected: 42, description: 'start-section returns 42' },
    ],
  },
  {
    name: 'add',
    tests: [
      { args: '0500000007000000', expected: 12, description: '5 + 7 = 12' },
      { args: '00000000ffffffff', expected: 0xffffffff, description: '0 + MAX = MAX' },
      { args: '01000000ffffffff', expected: 0, description: '1 + MAX = 0 (overflow)' },
    ],
  },
  {
    name: 'factorial',
    tests: [
      { args: '00000000', expected: 1, description: '0! = 1' },
      { args: '01000000', expected: 1, description: '1! = 1' },
      { args: '05000000', expected: 120, description: '5! = 120' },
      { args: '0a000000', expected: 3628800, description: '10! = 3628800' },
    ],
  },
  {
    name: 'fibonacci',
    tests: [
      { args: '00000000', expected: 0, description: 'fib(0) = 0' },
      { args: '01000000', expected: 1, description: 'fib(1) = 1' },
      { args: '02000000', expected: 1, description: 'fib(2) = 1' },
      { args: '0a000000', expected: 55, description: 'fib(10) = 55' },
      { args: '14000000', expected: 6765, description: 'fib(20) = 6765' },
    ],
  },
  {
    name: 'gcd',
    tests: [
      { args: '3000000012000000', expected: 6, description: 'gcd(48, 18) = 6' },
      { args: '6400000038000000', expected: 4, description: 'gcd(100, 56) = 4' },
      { args: '1100000011000000', expected: 17, description: 'gcd(17, 17) = 17' },
      { args: '01000000ff000000', expected: 1, description: 'gcd(1, 255) = 1' },
    ],
  },
  {
    name: 'is-prime',
    tests: [
      { args: '00000000', expected: 0, description: 'is_prime(0) = 0' },
      { args: '01000000', expected: 0, description: 'is_prime(1) = 0' },
      { args: '02000000', expected: 1, description: 'is_prime(2) = 1' },
      { args: '03000000', expected: 1, description: 'is_prime(3) = 1' },
      { args: '04000000', expected: 0, description: 'is_prime(4) = 0' },
      { args: '05000000', expected: 1, description: 'is_prime(5) = 1' },
      { args: '11000000', expected: 1, description: 'is_prime(17) = 1' },
      { args: '19000000', expected: 0, description: 'is_prime(25) = 0' },
      { args: '61000000', expected: 1, description: 'is_prime(97) = 1' },
      { args: '64000000', expected: 0, description: 'is_prime(100) = 0' },
      { args: '65000000', expected: 1, description: 'is_prime(101) = 1' },
    ],
  },
  {
    name: 'div',
    tests: [
      { args: '1400000005000000', expected: 4, description: '20 / 5 = 4' },
      { args: '6400000008000000', expected: 12, description: '100 / 8 = 12' },
      { args: '0a00000003000000', expected: 3, description: '10 / 3 = 3' },
    ],
  },
  {
    name: 'call',
    tests: [
      { args: '05000000', expected: 10, description: 'double(5) = 10' },
      { args: '0a000000', expected: 20, description: 'double(10) = 20' },
      { args: '00000000', expected: 0, description: 'double(0) = 0' },
    ],
  },
  {
    name: 'call-indirect',
    tests: [
      { args: '0000000005000000', expected: 10, description: 'call_indirect double(5) = 10' },
      { args: '0100000005000000', expected: 15, description: 'call_indirect triple(5) = 15' },
      { args: '000000000a000000', expected: 20, description: 'call_indirect double(10) = 20' },
      { args: '010000000a000000', expected: 30, description: 'call_indirect triple(10) = 30' },
    ],
  },
  {
    name: 'i64-ops',
    tests: [
      { args: '00000000', expected: 14, description: 'i64.div_u(100, 7) = 14' },
      { args: '01000000', expected: 2, description: 'i64.rem_u(100, 7) = 2' },
      { args: '02000000', expected: 4080, description: 'i64.shl(0xFF, 4) = 4080' },
      { args: '03000000', expected: 4080, description: 'i64.shr_u(0xFF00, 4) = 4080' },
      { args: '04000000', expected: 240, description: 'i64.and(0xF0F0, 0x0FF0) = 240' },
      { args: '05000000', expected: 65520, description: 'i64.or(0xF0F0, 0x0FF0) = 65520' },
      { args: '06000000', expected: 65280, description: 'i64.xor(0xF0F0, 0x0FF0) = 65280' },
      { args: '07000000', expected: 1, description: 'i64.ge_u(100, 50) = 1' },
      { args: '08000000', expected: 1, description: 'i64.le_u(50, 100) = 1' },
    ],
  },
  {
    name: 'many-locals',
    tests: [
      { args: '00000000', expected: 21, description: 'sum with base 0: 1+2+3+4+5+6 = 21' },
      { args: '0a000000', expected: 81, description: 'sum with base 10: 11+12+13+14+15+16 = 81' },
      { args: '64000000', expected: 621, description: 'sum with base 100: 101+102+103+104+105+106 = 621' },
    ],
  },
  {
    name: 'entry-points',
    tests: [
      { args: '', expected: 42, description: 'main (PC=0) returns 42' },
      { args: '', expected: 99, description: 'main2 (PC=5) returns 99', pc: 5 },
    ],
  },
  {
    name: 'as-add',
    tests: [
      { args: '0500000007000000', expected: 12, description: 'AS: 5 + 7 = 12' },
      { args: '0a00000014000000', expected: 30, description: 'AS: 10 + 20 = 30' },
    ],
  },
  {
    name: 'as-factorial',
    tests: [
      { args: '00000000', expected: 1, description: 'AS: 0! = 1' },
      { args: '05000000', expected: 120, description: 'AS: 5! = 120' },
      { args: '07000000', expected: 5040, description: 'AS: 7! = 5040' },
    ],
  },
  {
    name: 'as-fibonacci',
    tests: [
      { args: '00000000', expected: 0, description: 'AS: fib(0) = 0' },
      { args: '01000000', expected: 1, description: 'AS: fib(1) = 1' },
      { args: '0a000000', expected: 55, description: 'AS: fib(10) = 55' },
    ],
  },
  {
    name: 'as-gcd',
    tests: [
      { args: '3000000012000000', expected: 6, description: 'AS: gcd(48, 18) = 6' },
      { args: '6400000038000000', expected: 4, description: 'AS: gcd(100, 56) = 4' },
      { args: '1100000011000000', expected: 17, description: 'AS: gcd(17, 17) = 17' },
    ],
  },
  {
    name: 'bit-ops',
    tests: [
      { args: '0000000001000000', expected: 31, description: 'clz(1) = 31 (leading zeros in 32-bit)' },
      { args: '0000000000000080', expected: 0, description: 'clz(0x80000000) = 0 (MSB set)' },
      { args: '0100000001000000', expected: 0, description: 'ctz(1) = 0 (LSB set)' },
      { args: '0100000002000000', expected: 1, description: 'ctz(2) = 1' },
      { args: '02000000ffffffff', expected: 32, description: 'popcnt(0xffffffff) = 32' },
      { args: '02000000f0f0f0f0', expected: 16, description: 'popcnt(0xf0f0f0f0) = 16' },
    ],
  },
  {
    name: 'recursive',
    tests: [
      { args: '00000000', expected: 1, description: 'recursive factorial(0) = 1' },
      { args: '01000000', expected: 1, description: 'recursive factorial(1) = 1' },
      { args: '05000000', expected: 120, description: 'recursive factorial(5) = 120' },
      { args: '07000000', expected: 5040, description: 'recursive factorial(7) = 5040' },
    ],
  },
  {
    name: 'rotate',
    tests: [
      { args: '00000000ff00000001000000', expected: 0x1fe, description: 'rotl(0xff, 1) = 0x1fe' },
      { args: '00000000ff00000008000000', expected: 0xff00, description: 'rotl(0xff, 8) shifts left 8' },
      { args: '01000000ff00000001000000', expected: 0x8000007f, description: 'rotr(0xff, 1) rotates right' },
      { args: '01000000cdab000010000000', expected: 0xabcd0000, description: 'rotr(0xabcd, 16) swaps to high bytes' },
    ],
  },
  {
    name: 'br-table',
    tests: [
      { args: '00000000', expected: 100, description: 'br_table case 0 returns 100' },
      { args: '01000000', expected: 200, description: 'br_table case 1 returns 200' },
      { args: '02000000', expected: 300, description: 'br_table case 2 returns 300' },
      { args: '03000000', expected: 999, description: 'br_table default case returns 999' },
      { args: '04000000', expected: 999, description: 'br_table out of bounds returns 999' },
    ],
  },
  {
    name: 'block-result',
    tests: [
      { args: '00000000', expected: 42, description: 'block with result returns 42' },
      { args: '01000000', expected: 100, description: 'block with br returns 100 (not 999)' },
    ],
  },
  {
    name: 'stack-test',
    tests: [
      { args: '00000000', expected: 30, description: 'stack operations: 10*2 + 10 = 30' },
      { args: '01000000', expected: 50, description: 'stack operations: 20*2 + 10 = 50' },
    ],
  },
  {
    name: 'simple-memory-test',
    tests: [
      { args: '00000000', expected: 42, description: 'memory store8/load8_u: store 42, read back 42' },
    ],
  },
  {
    name: 'nested-calls',
    tests: [
      { args: '00000000', expected: 2, description: 'nested calls: add_two(0) = 2' },
      { args: '05000000', expected: 7, description: 'nested calls: add_two(5) = 7' },
      { args: '64000000', expected: 102, description: 'nested calls: add_two(100) = 102' },
    ],
  },
  {
    name: 'compare-test',
    tests: [
      { args: '00000000', expected: 1, description: 'comparison: 3 < 5 = 1' },
      { args: '01000000', expected: 0, description: 'comparison: 5 < 3 = 0' },
      { args: '02000000', expected: 1, description: 'comparison: 10 > 5 = 1' },
      { args: '03000000', expected: 0, description: 'comparison: 5 > 10 = 0' },
    ],
  },
  {
    name: 'block-br-test',
    tests: [
      { args: '00000000', expected: 10, description: 'block with conditional br (skip branch)' },
      { args: '01000000', expected: 20, description: 'block with conditional br (take branch)' },
      { args: '02000000', expected: 30, description: 'nested blocks with br_if' },
    ],
  },
  {
    name: 'computed-addr-test',
    tests: [
      { args: '00000000', expected: 42, description: 'computed address with offset = 42' },
      { args: '01000000', expected: 84, description: 'computed address with scale = 84' },
    ],
  },
  // Additional AssemblyScript tests - allocation tests with different AS runtime configurations
  {
    name: 'as-alloc-test-minimal',
    tests: [
      // Creates 5 Foo objects (x=0,10,20,30,40) with Bar children (value=0,100,200,300,400)
      // Plus 3 temp arrays with computed values
      // Total = 1100 (objects) + 7 (arrays) = 1107
      { args: '', expected: 1107, description: 'AS: alloc test (minimal runtime) = 1107' },
    ],
  },
  {
    name: 'as-alloc-test-stub',
    tests: [
      { args: '', expected: 1107, description: 'AS: alloc test (stub runtime) = 1107' },
    ],
  },
  {
    name: 'as-alloc-test-incremental',
    tests: [
      { args: '', expected: 1107, description: 'AS: alloc test (incremental GC) = 1107' },
    ],
  },
  {
    name: 'as-life',
    tests: [
      // Game of Life returns [width(u32), height(u32), cells...]
      // First u32 is WIDTH = 16
      { args: '00000000', expected: 16, description: 'AS: life 0 steps - returns width=16' },
      { args: '01000000', expected: 16, description: 'AS: life 1 step - returns width=16' },
      { args: '05000000', expected: 16, description: 'AS: life 5 steps - returns width=16' },
    ],
  },
  {
    name: 'as-array-test',
    tests: [
      // Input: count(u32) + bytes
      // Output: args_ptr, args_len, count, arr.length, ...
      // First u32 is adjusted args_ptr (0xFEFA0000 = 4277796864 after WASM_MEMORY_BASE subtraction)
      { args: '03000000aabbcc', expected: 0xfefa0000, description: 'AS: array test - args_ptr check' },
    ],
  },
  {
    name: 'as-decoder-test',
    tests: [
      // Input: program_len(u32), data_len(u32), program_bytes...
      // Output: args_ptr, args_len, program_len, data_len, ...
      // First u32 is adjusted args_ptr
      { args: '04000000000000001234abcd', expected: 0xfefa0000, description: 'AS: decoder test - args_ptr check' },
    ],
  },
  {
    name: 'as-memory-args-test',
    tests: [
      // Input: a(u32), b(u32)
      // Output: args_ptr, args_len, a, b, a+b, ...
      // First u32 is adjusted args_ptr
      { args: '0500000007000000', expected: 0xfefa0000, description: 'AS: memory args test - args_ptr check' },
    ],
  },
  {
    name: 'as-nested-memory-test',
    tests: [
      // Input: program_len(u32), data_len(u32), program_bytes, data_bytes
      // Output: args_ptr, args_len, program_len, data_len, ...
      // First u32 is adjusted args_ptr
      { args: '0400000002000000deadbeef1234', expected: 0xfefa0000, description: 'AS: nested memory test - args_ptr check' },
    ],
  },
  {
    name: 'as-mini-pvm-runner',
    tests: [
      // Input: gas(u64), pc(u32), program_len(u32), inner_args_len(u32), program, inner_args
      // Output starts with marker 0x11111111, then diagnostics
      // gas=1000 (e803000000000000), pc=0 (00000000), prog_len=4 (04000000), args_len=2 (02000000)
      { args: 'e80300000000000000000000040000000200000011223344aabb', expected: 0x11111111, description: 'AS: mini-pvm-runner - marker check' },
    ],
  },
  // Bug reproduction tests
  {
    name: 'as-array-push-test',
    tests: [
      // Tests Array.push() functionality
      // Expected: 0+1+2+3+4+5+6+7 = 28
      { args: '', expected: 28, description: 'AS: Array.push() sum test - should return 28' },
    ],
  },
  {
    name: 'as-subarray-test',
    tests: [
      // Tests Uint8Array.subarray() functionality
      // Creates [10,20,30,40,50], subarray from offset 2, reads element 0
      // Expected: 30
      { args: '', expected: 30, description: 'AS: Uint8Array.subarray() test - should return 30' },
    ],
  },
  {
    name: 'as-array-push-args-test',
    tests: [
      // Tests Array.push() with bytes loaded from args memory
      // Input: 8 bytes to push, expected: sum of those bytes
      // 01+02+03+04+05+06+07+08 = 36
      { args: '0102030405060708', expected: 36, description: 'AS: Array.push() from args - sum of 8 bytes' },
      // Test with larger values: 10+20+30+40+50+60+70+80 = 360
      { args: '0a141e28323c4650', expected: 360, description: 'AS: Array.push() from args - larger values' },
    ],
  },
  {
    name: 'as-decoder-subarray-test',
    tests: [
      // Tests Decoder-like pattern with subarray()
      // Input: 5+ bytes, returns sum of first 3
      // 10+20+30 = 60
      { args: '0a141e2832', expected: 60, description: 'AS: Decoder subarray pattern - sum of first 3' },
      // 01+02+03 = 6
      { args: '0102030405', expected: 6, description: 'AS: Decoder subarray pattern - small values' },
    ],
  },
  {
    name: 'as-varU32-test',
    tests: [
      // Tests varU32 decoding pattern with subarray()
      // Input: single-byte varU32 values (< 0x80)
      // 10+20+30 = 60
      { args: '0a141e2832', expected: 60, description: 'AS: varU32 decode - single byte values' },
      // 01+02+03 = 6
      { args: '0102030405', expected: 6, description: 'AS: varU32 decode - small values' },
    ],
  },
  {
    name: 'as-complex-alloc-test',
    tests: [
      // Tests complex allocation patterns similar to anan-as interpreter
      // Expected: 45 + 190 + 45 + 15 + 795 = 1090
      { args: '0000000000000000', expected: 1090, description: 'AS: Complex allocation test' },
    ],
  },
  {
    name: 'as-subarray-offset-test',
    tests: [
      // Tests subarray with different offsets
      // Expected: 500 + (0+1+2+3+4) + 500 + (5+6+7+8+9) + 400 + (3+4+5+6) = 1463
      { args: '', expected: 1463, description: 'AS: Subarray offset correctness test' },
    ],
  },
  {
    name: 'as-complex-alloc-debug',
    tests: [
      // Debug: test each step separately
      { args: '00', expected: 45, description: 'AS: Step 0 - Decoder.u8() sum 0-9' },
      { args: '01', expected: 190, description: 'AS: Step 1 - Decoder.bytes() sum 0-19' },
      { args: '02', expected: 45, description: 'AS: Step 2 - lowerBytes sum 0-9' },
      { args: '03', expected: 15, description: 'AS: Step 3 - pre-alloc array 1+2+3+4+5' },
      { args: '04', expected: 795, description: 'AS: Step 4 - multiple allocs sum' },
      { args: '05', expected: 1090, description: 'AS: Step 5+ - total sum' },
    ],
  },
  {
    name: 'as-lowerBytes-test',
    tests: [
      { args: '00', expected: 45, description: 'AS: lowerBytes - source sum' },
      { args: '01', expected: 45, description: 'AS: lowerBytes - subarray sum' },
      { args: '02', expected: 10045, description: 'AS: lowerBytes - result len*1000 + sum' },
      { args: '03', expected: 10045, description: 'AS: lowerBytes on subarray - len*1000 + sum' },
      { args: '04', expected: 123435, description: 'AS: lowerBytes - element breakdown' },
      { args: '05', expected: 999, description: 'AS: lowerBytes - all indices match' },
    ],
  },
  {
    name: 'as-largebuf-subarray-test',
    tests: [
      { args: '00', expected: 45, description: 'AS: large buffer direct sum' },
      { args: '01', expected: 45, description: 'AS: large buffer subarray sum' },
      { args: '02', expected: 45, description: 'AS: large buffer lowerBytes sum' },
      { args: '03', expected: 10045, description: 'AS: large buffer lowerBytes len+sum' },
      { args: '04', expected: 1045, description: 'AS: large buffer middle slice' },
      { args: '05', expected: 190, description: 'AS: large buffer multiple slices' },
    ],
  },
  {
    name: 'as-multi-slice-debug',
    tests: [
      { args: '00', expected: 45, description: 'AS: first slice only' },
      { args: '01', expected: 145, description: 'AS: second slice only' },
      { args: '02', expected: 45145, description: 'AS: both slices sum1*1000+sum2' },
      { args: '03', expected: 101110, description: 'AS: arr2[0]*10k+arr2[1]*100+len' },
      { args: '04', expected: 101110, description: 'AS: slice2[0]*10k+slice2[1]*100+len' },
      { args: '05', expected: 910, description: 'AS: slice1 after slice2 created' },
      { args: '06', expected: 910, description: 'AS: arr1 after both lowerBytes' },
    ],
  },
  {
    name: 'as-minimal-fail',
    tests: [
      { args: '00', expected: 190, description: 'AS: Pattern A - direct accumulation' },
      { args: '01', expected: 190, description: 'AS: Pattern B - separate sums' },
      { args: '02', expected: 190, description: 'AS: Pattern C - no lowerBytes' },
      { args: '03', expected: 190, description: 'AS: Pattern D - explicit length' },
    ],
  },
  {
    name: 'as-local-preserve-test',
    tests: [
      { args: '00', expected: 148, description: 'AS: locals after simple call' },
      { args: '01', expected: 66, description: 'AS: locals after multiple calls' },
      { args: '02', expected: 310, description: 'AS: locals after 4-arg call' },
      { args: '03', expected: 23, description: 'AS: locals after loop with calls' },
      { args: '04', expected: 336, description: 'AS: locals after two loops with calls' },
      { args: '05', expected: 22, description: 'AS: local $3 (r12) after call' },
    ],
  },
  {
    name: 'as-array-length-loop-test',
    tests: [
      { args: '00', expected: 45, description: 'AS: single array with .length loop' },
      { args: '01', expected: 45, description: 'AS: two arrays, sum first only' },
      { args: '02', expected: 90, description: 'AS: two arrays, .length in loops (FAIL pattern)' },
      { args: '03', expected: 90, description: 'AS: two arrays, length in locals (PASS pattern)' },
      { args: '04', expected: 10, description: 'AS: arr2.length after first loop' },
      { args: '05', expected: 0, description: 'AS: arr2[0] after first loop' },
      { args: '07', expected: 90, description: 'AS: two loops with getValue function' },
      { args: '08', expected: 10, description: 'AS: manual i32.load arr2 len after loop' },
    ],
  },
  {
    name: 'as-loop-counter-test',
    tests: [
      { args: '00', expected: 20, description: 'AS: two simple loops, no calls' },
      { args: '01', expected: 10, description: 'AS: loop counter value after loop' },
      { args: '02', expected: 10, description: 'AS: reset counter and run second loop' },
      { args: '03', expected: 90, description: 'AS: two loops with unchecked access' },
      { args: '04', expected: 90, description: 'AS: two loops with manual length' },
      { args: '05', expected: 90, description: 'AS: two loops with checked access (calls)' },
      { args: '06', expected: 10045, description: 'AS: arr2.length*1000 + loop1 result' },
      { args: '07', expected: 1, description: 'AS: 0 < arr2.length comparison' },
      { args: '08', expected: 100, description: 'AS: single loop 100 iterations' },
    ],
  },
  {
    name: 'as-second-loop-test',
    tests: [
      { args: '00', expected: 31, description: 'AS: execution markers (all loops)' },
      { args: '01', expected: 31, description: 'AS: execution markers (explicit len)' },
      { args: '02', expected: 1, description: 'AS: loop 2 condition true' },
      { args: '03', expected: 10, description: 'AS: arr2.length from memory' },
      { args: '04', expected: 10, description: 'AS: loop counter after loop 1' },
      { args: '05', expected: 0, description: 'AS: loop counter after reset' },
      { args: '06', expected: 1, description: 'AS: if condition instead of loop' },
      { args: '07', expected: 0, description: 'AS: unrolled access' },
    ],
  },
  {
    name: 'as-iteration-count-test',
    tests: [
      // Test 0: check if loop 2 runs after N iterations of loop 1
      { args: '0001', expected: 1, description: 'AS: 1 iter in loop1, loop2 runs' },
      { args: '0002', expected: 101, description: 'AS: 2 iters in loop1, loop2 runs' },
      { args: '0005', expected: 1001, description: 'AS: 5 iters in loop1, loop2 runs' },
      { args: '000a', expected: 4501, description: 'AS: 10 iters in loop1, loop2 runs' },
      // Test 3: 10 iterations, then one loop2 iteration
      { args: '03', expected: 45, description: 'AS: full loop1, one loop2 iter' },
      // Test 4: full loops
      { args: '04', expected: 90, description: 'AS: full 10+10 iterations' },
    ],
  },
  {
    name: 'as-array-value-test',
    tests: [
      { args: '00', expected: 0, description: 'AS: arr[0] = 0' },
      { args: '01', expected: 1, description: 'AS: arr[1] = 1' },
      { args: '02', expected: 2, description: 'AS: arr[2] = 2' },
      { args: '03', expected: 10, description: 'AS: arr[0] + arr[1]*10 = 10' },
      { args: '04', expected: 1234, description: 'AS: loop concat digits = 1234' },
      { args: '05', expected: 1, description: 'AS: v0*10 + v1 = 1' },
      { args: '06', expected: 3, description: 'AS: sum(0,1,2) = 3' },
      { args: '07', expected: 3, description: 'AS: loop sum(0,1,2) = 3' },
      { args: '08', expected: 1, description: 'AS: direct memory arr[1] = 1' },
    ],
  },
  {
    name: 'as-if-result-test',
    tests: [
      // Test 0: a && b where both are non-zero
      { args: '000101', expected: 1, description: 'AS: 1 && 1 = true' },
      { args: '000100', expected: 0, description: 'AS: 1 && 0 = false' },
      { args: '000001', expected: 0, description: 'AS: 0 && 1 = false' },
      // Test 1: a < 5 && b < 10
      { args: '010309', expected: 1, description: 'AS: 3<5 && 9<10 = true' },
      { args: '010509', expected: 0, description: 'AS: 5<5 && 9<10 = false' },
      // Test 2: i < iterCount && i < arrLength (i=a, iterCount=b, arrLength=10)
      { args: '020102', expected: 1, description: 'AS: 1<2 && 1<10 = true' },
      { args: '020202', expected: 0, description: 'AS: 2<2 && 2<10 = false' },
      // Test 3: loop count with &&
      { args: '0305', expected: 5, description: 'AS: loop count limit=5' },
      { args: '030f', expected: 10, description: 'AS: loop count limit=15 (capped at 10)' },
      // Test 4: loop iteration values
      { args: '04', expected: 1234, description: 'AS: loop iterations 0,1,2,3,4' },
      // Test 6: first condition only
      { args: '060102', expected: 1, description: 'AS: 1 < 2 = true' },
      { args: '060202', expected: 0, description: 'AS: 2 < 2 = false' },
      // Test 8: specific check
      { args: '08', expected: 1, description: 'AS: 1<2 && 1<10 hardcoded' },
    ],
  },
  {
    name: 'as-memload-condition-test',
    tests: [
      { args: '00', expected: 10, description: 'AS: simple arr.length loop' },
      { args: '01', expected: 5, description: 'AS: && with arr.length, limit=5' },
      { args: '02', expected: 5, description: 'AS: && with arr.length limiting' },
      { args: '03', expected: 10, description: 'AS: && + arr access in body' },
      { args: '04', expected: 100, description: 'AS: two arrays, loop first, access second' },
      { args: '05', expected: 4, description: 'AS: last iteration i value' },
      { args: '06', expected: 5, description: 'AS: manual while loop' },
      { args: '07', expected: 5, description: 'AS: cached length variable' },
      { args: '08', expected: 20, description: 'AS: two loops with arr.length' },
    ],
  },
  {
    name: 'as-limit-source-test',
    tests: [
      { args: '00', expected: 10, description: 'AS: hardcoded limit=5 (2 arrays)' },
      { args: '0105', expected: 10, description: 'AS: limit=5 from args (2 arrays)' },
      { args: '02', expected: 10, description: 'AS: limit=5 from local var (2 arrays)' },
      { args: '0305', expected: 10, description: 'AS: limit=5 from args (1 array)' },
      { args: '04', expected: 10, description: 'AS: explicit val check (2 arrays)' },
      { args: '05', expected: 1, description: 'AS: arr1[1] = 1 (2 arrays)' },
      { args: '06', expected: 1, description: 'AS: arr1[0]+arr1[1] (2 arrays)' },
      { args: '07', expected: 1, description: 'AS: 2 iters hardcoded (2 arrays)' },
      { args: '0802', expected: 1, description: 'AS: 2 iters from args (2 arrays)' },
    ],
  },
  {
    name: 'as-trace-loop-test',
    tests: [
      { args: '00', expected: 5, description: 'AS: iter count, hardcoded limit' },
      { args: '0105', expected: 5, description: 'AS: iter count, limit from args' },
      { args: '02', expected: 31, description: 'AS: iter bits, hardcoded' },
      { args: '0305', expected: 31, description: 'AS: iter bits, limit from args' },
      { args: '0405', expected: 111, description: 'AS: manual cond checks' },
      { args: '0505', expected: 1, description: 'AS: 1 < limit check' },
      { args: '0605', expected: 5, description: 'AS: limit value' },
      { args: '0705', expected: 5, description: 'AS: simple loop, limit from args' },
      { args: '08', expected: 10, description: 'AS: simple loop, arr.length' },
    ],
  },
  {
    name: 'as-count-vs-sum-test',
    tests: [
      { args: '00', expected: 5010, description: 'AS: count+sum hardcoded limit' },
      { args: '0105', expected: 5010, description: 'AS: count+sum limit from args' },
      { args: '0205', expected: 5, description: 'AS: just count, limit from args' },
      { args: '0305', expected: 10, description: 'AS: just sum, limit from args' },
      { args: '0405', expected: 5010, description: 'AS: count then sum, limit from args' },
      { args: '0505', expected: 655391, description: 'AS: bits+sum, limit from args' },
      { args: '0602', expected: 101, description: 'AS: i before/after arr access' },
      { args: '0705', expected: 10, description: 'AS: sum i values (no arr access)' },
    ],
  },
  {
    name: 'as-array-value-trace-test',
    tests: [
      { args: '0005', expected: 43210, description: 'AS: arr[i] digits (limit from args)' },
      { args: '01', expected: 43210, description: 'AS: arr[i] digits (hardcoded limit)' },
      { args: '0205', expected: 43210, description: 'AS: arr[i] via memory store' },
      { args: '0305', expected: 43210, description: 'AS: arr[0..4] outside loop' },
      { args: '0403', expected: 4386, description: 'AS: i<<4|arr[i] encoding' },
      { args: '0505', expected: 0, description: 'AS: arr[i] vs i mismatch check' },
      { args: '0603', expected: 2232576, description: 'AS: i,arr[i] pairs to memory' },
      { args: '0705', expected: 1, description: 'AS: just arr[1] with ternary' },
      { args: '0805', expected: 10, description: 'AS: simple sum (limit from args)' },
    ],
  },
  {
    name: 'as-simple-call-test',
    tests: [
      { args: '00', expected: 1, description: 'AS: arr[1] no ternary' },
      { args: '0105', expected: 1, description: 'AS: arr[1] after ternary' },
      { args: '0205', expected: 5, description: 'AS: just ternary value' },
      { args: '0305', expected: 0, description: 'AS: arr[0] after ternary' },
      { args: '0405', expected: 2, description: 'AS: arr[2] after ternary' },
      { args: '0505', expected: 51, description: 'AS: limit*10 + arr[1]' },
      { args: '0605', expected: 1, description: 'AS: arr[1] (no arr2)' },
      { args: '0705', expected: 15, description: 'AS: arr[1] first, then ternary' },
      { args: '0805', expected: 1, description: 'AS: direct memory after ternary' },
    ],
  },
  {
    name: 'as-flat-ternary-test',
    tests: [
      { args: '05', expected: 501, description: 'AS: limit=5, arr[1]=1 (flat)' },
      { args: '03', expected: 301, description: 'AS: limit=3, arr[1]=1 (flat)' },
      { args: '01', expected: 101, description: 'AS: limit=1, arr[1]=1 (flat)' },
    ],
  },
  {
    name: 'as-nested-if-test',
    tests: [
      { args: '00', expected: 1, description: 'AS: nested if, step 0, just arr[1]' },
      { args: '0105', expected: 501, description: 'AS: nested if, step 1, ternary+arr[1]' },
      { args: '02', expected: 2, description: 'AS: nested if, step 2' },
    ],
  },
  {
    name: 'as-minimal-nested-drop-test',
    tests: [
      { args: '00', expected: 42, description: 'AS: direct call (no nesting)' },
      { args: '0105', expected: 42, description: 'AS: ternary+drop then getSecondArg' },
      { args: '0205', expected: 100, description: 'AS: ternary+drop then getFirstArg' },
      { args: '03', expected: 42, description: 'AS: direct call in nested if' },
      { args: '04', expected: 100042, description: 'AS: multiple calls' },
    ],
  },
  {
    name: 'as-debug-call-test',
    tests: [
      { args: '01', expected: 101, description: 'AS: arr[1] before*100 + after' },
      // Note: step 2 exposes a valueBefore bug (gets 0 instead of 1)
      // Expected: 1<<16|5<<8|1 = 66817, Got: 0<<16|5<<8|1 = 1281
      { args: '02', expected: 1281, description: 'AS: limit<<8|valueAfter (valueBefore=0 bug)' },
      { args: '03', expected: 1, description: 'AS: arr ptr survives ternary' },
      { args: '04', expected: 1, description: 'AS: arr[1] via dataPtr after ternary' },
      { args: '05', expected: 0, description: 'AS: arr[0] after ternary' },
      { args: '06', expected: 2, description: 'AS: arr[2] after ternary' },
      { args: '07', expected: 3, description: 'AS: arr[3] after ternary' },
      { args: '08', expected: 123, description: 'AS: arr[0..3] without ternary' },
      // Note: step 9 exposes multi-ternary bug (v0,v1,v2 all become 0)
      { args: '09', expected: 0, description: 'AS: arr[0..2] multi-ternary (bug)' },
    ],
  },
  {
    name: 'as-flat-ternary-drop',
    tests: [
      // arr[1] should return 1 - no nested if-result blocks
      { args: '05', expected: 1, description: 'AS: arr[1] after ternary (flat)' },
    ],
  },
  {
    name: 'as-local-clobber-test',
    tests: [
      { args: '00', expected: 20, description: 'AS: call and save to local' },
      { args: '01', expected: 202, description: 'AS: saved after if-else' },
      { args: '02', expected: 205, description: 'AS: saved after ternary' },
      { args: '03', expected: 205, description: 'AS: saved in nested if' },
      { args: '04', expected: 4444, description: 'AS: constant result' },
      { args: '05', expected: 505, description: 'AS: step*100 + ternary' },
    ],
  },
  {
    name: 'as-minimal-repro',
    tests: [
      { args: '00', expected: 1, description: 'AS: arr[1] no ternary (step 0)' },
      { args: '01', expected: 1, description: 'AS: arr[1] after ternary (step 1)' },
      { args: '02', expected: 0, description: 'AS: arr[0] after ternary' },
      { args: '03', expected: 2, description: 'AS: arr[2] after ternary' },
      { args: '04', expected: 3, description: 'AS: arr[3] no ternary' },
      { args: '06', expected: 1, description: 'AS: arr_only[1] deep nesting' },
    ],
  },
  {
    name: 'as-nested-repro',
    tests: [
      { args: '00', expected: 1, description: 'AS: arr[1] no nesting' },
      { args: '01', expected: 1, description: 'AS: arr[1] nested if+ternary' },
    ],
  },
  {
    name: 'as-noinline-call-test',
    tests: [
      { args: '00', expected: 20, description: 'AS: loadFromMemory no nesting' },
      { args: '01', expected: 20, description: 'AS: loadFromMemory with ternary' },
      { args: '02', expected: 10, description: 'AS: loadFromMemory index 0' },
      { args: '03', expected: 20, description: 'AS: loadFromMemory no ternary' },
    ],
  },
  {
    name: 'as-simpler-repro',
    tests: [
      // With args: first byte is temp load, second byte is index
      { args: '0500', expected: 0, description: 'AS: arr[0] (THEN branch)' },
      { args: '0501', expected: 1, description: 'AS: arr[1] (THEN branch)' },
      { args: '0502', expected: 2, description: 'AS: arr[2] (THEN branch)' },
    ],
  },
  {
    name: 'as-two-arg-call-test',
    tests: [
      { args: '00', expected: 142, description: 'AS: addWithMem no nesting' },
      { args: '01', expected: 142, description: 'AS: addWithMem with ternary' },
      { args: '02', expected: 242, description: 'AS: addWithMem(200,42)' },
      { args: '03', expected: 142, description: 'AS: addWithMem no ternary' },
      { args: '04', expected: 142, description: 'AS: addWithMem(step*25, 42)' },
      { args: '05', expected: 20, description: 'AS: loadAt after ternary' },
    ],
  },
  {
    name: 'as-alloc-loop-debug',
    tests: [
      { args: '00', expected: 31, description: 'AS: round 0: 0+31' },
      { args: '01', expected: 95, description: 'AS: round 1: 32+63' },
      { args: '02', expected: 159, description: 'AS: round 2: 64+95' },
      { args: '03', expected: 223, description: 'AS: round 3: 96+127' },
      { args: '04', expected: 287, description: 'AS: round 4: 128+159' },
      { args: '05', expected: 795, description: 'AS: full loop sum' },
      { args: '06', expected: 2064543, description: 'AS: round values encoded' },
      { args: '07', expected: 159, description: 'AS: arithmetic check' },
      { args: '08', expected: 287, description: 'AS: direct store/load' },
      { args: '09', expected: 2155978399, description: 'AS: loop fill encoded' },
    ],
  },
  {
    name: 'as-u8-store-test',
    tests: [
      { args: '00', expected: 127, description: 'AS: store 127 to Uint8Array' },
      { args: '01', expected: 128, description: 'AS: store 128 to Uint8Array' },
      { args: '02', expected: 255, description: 'AS: store 255 to Uint8Array' },
      { args: '03', expected: 128, description: 'AS: direct memory store 128' },
      { args: '04', expected: 287, description: 'AS: 128+159 from array' },
      { args: '05', expected: 128, description: 'AS: computed 4*32 to array' },
      { args: '06', expected: 159, description: 'AS: computed 4*32+31 to array' },
      { args: '07', expected: 159, description: 'AS: computed value only' },
      { args: '08', expected: 255, description: 'AS: 127+128 from array' },
      { args: '09', expected: 287, description: 'AS: direct memory 128+159' },
    ],
  },
  {
    name: 'as-u8-two-elem-test',
    tests: [
      { args: '00', expected: 30, description: 'AS: 10+20 (both<128)' },
      { args: '01', expected: 128, description: 'AS: 127+1' },
      { args: '02', expected: 129, description: 'AS: 1+128 (second>=128)' },
      { args: '03', expected: 129, description: 'AS: 128+1 (first>=128)' },
      { args: '04', expected: 256, description: 'AS: 128+128' },
      { args: '05', expected: 128, description: 'AS: arr[0] only' },
      { args: '06', expected: 159, description: 'AS: arr[1] only' },
      { args: '07', expected: 32927, description: 'AS: arr[0]<<8|arr[1]' },
      { args: '08', expected: 487, description: 'AS: 3-elem sum' },
      { args: '09', expected: 287, description: 'AS: reverse store order' },
    ],
  },
];
