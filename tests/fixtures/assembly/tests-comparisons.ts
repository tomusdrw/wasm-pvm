// Test for comparison operations that generate SetLtUImm and SetLtSImm
// SetLtUImm is generated for: ==, <=, >= comparisons
const RESULT_HEAP: u32 = 0x30100;

export let result_ptr: i32 = 0;
export let result_len: i32 = 0;

function writeResult(val: i32): void {
  store<i32>(RESULT_HEAP, val);
  result_ptr = RESULT_HEAP;
  result_len = 4;
}

export function main(args_ptr: i32, args_len: i32): void {
  const a = load<i32>(args_ptr);
  const b = load<i32>(args_ptr + 4);

  let result: i32 = 0;

  // Test equality (==) - generates xor + SetLtUImm
  if (a == b) {
    result = result | 1;  // bit 0: equal
  }

  // Test unsigned less than or equal (<=) - generates SetLtU + SetLtUImm
  if (a <= b) {
    result = result | 2;  // bit 1: a <= b
  }

  // Test unsigned greater than or equal (>=) - generates SetLtU + SetLtUImm
  if (a >= b) {
    result = result | 4;  // bit 2: a >= b
  }

  // Test signed less than or equal - also uses SetLtUImm for negation
  // This exercises signed comparisons
  if (a as i32 <= b as i32) {
    result = result | 8;  // bit 3: signed a <= b
  }

  // Test signed greater than or equal
  if (a as i32 >= b as i32) {
    result = result | 16; // bit 4: signed a >= b
  }

  // Combined comparison test
  // result bits: 0=equal, 1=ule, 2=uge, 3=sle, 4=sge
  writeResult(result);
}
