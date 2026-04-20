import { defineSuite } from "../helpers/suite";

const tests = [
  // args_ptr_WASM = ARGS_SEGMENT_START (0xFEFF0000) - wasm_memory_base.
  // 2 user globals + mem-size slot → wasm_memory_base = 0x3000C.
  { args: "04000000000000001234abcd", expected: 4277927924, description: "AS: decoder test - args_ptr check" },
];

defineSuite({
  name: "as-decoder-test",
  // too slow
  skipPvmInPvm: true,
  tests: tests,
});
