import { defineSuite } from "../helpers/suite";

const tests = [
  // args_ptr_WASM = ARGS_SEGMENT_START (0xFEFF0000) - wasm_memory_base.
  // 1 user global + mem-size slot → wasm_memory_base = 0x30008.
  { args: "0400000002000000deadbeef1234", expected: 4277927928, description: "AS: nested memory test - args_ptr check" },
];

defineSuite({
  name: "as-nested-memory-test",
  tests: tests,
});
