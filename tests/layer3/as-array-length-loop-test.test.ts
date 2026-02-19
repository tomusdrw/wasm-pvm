import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 45, description: "AS: single array with .length loop" },
  { args: "01", expected: 45, description: "AS: two arrays, sum first only" },
  { args: "02", expected: 90, description: "AS: two arrays, .length in loops (FAIL pattern)" },
  { args: "03", expected: 90, description: "AS: two arrays, length in locals (PASS pattern)" },
  { args: "04", expected: 10, description: "AS: arr2.length after first loop" },
  { args: "05", expected: 0, description: "AS: arr2[0] after first loop" },
  { args: "07", expected: 90, description: "AS: two loops with getValue function" },
  { args: "08", expected: 10, description: "AS: manual i32.load arr2 len after loop" },
];

defineSuite({
  name: "as-array-length-loop-test",
  // slow
  skipPvmInPvm: true,
  tests: tests,
});
