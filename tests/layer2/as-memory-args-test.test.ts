import { defineSuite } from "../helpers/suite";

const tests = [
  // args_ptr_WASM = ARGS_SEGMENT_START (0xFEFF0000) - wasm_memory_base.
  // For this AS program (1 user global + mem-size slot, no passive/overflow)
  // wasm_memory_base = 0x30008 → args_ptr = 0xFEFBFFF8 = 4277927928.
  { args: "0500000007000000", expected: 4277927928, description: "AS: memory args test - args_ptr check" },
];

defineSuite({
  name: "as-memory-args-test",
  tests: tests,
});
