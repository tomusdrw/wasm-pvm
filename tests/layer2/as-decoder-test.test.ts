import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "04000000000000001234abcd", expected: 4277919488, description: "AS: decoder test - args_ptr check" },
];

defineSuite({
  name: "as-decoder-test",
  // too slow
  skipPvmInPvm: true,
  tests: tests,
});
