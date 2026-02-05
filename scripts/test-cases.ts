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
];
