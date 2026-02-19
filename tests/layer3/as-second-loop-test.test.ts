import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 31, description: "AS: execution markers (all loops)" },
  { args: "01", expected: 31, description: "AS: execution markers (explicit len)" },
  { args: "02", expected: 1, description: "AS: loop 2 condition true" },
  { args: "03", expected: 10, description: "AS: arr2.length from memory" },
  { args: "04", expected: 10, description: "AS: loop counter after loop 1" },
  { args: "05", expected: 0, description: "AS: loop counter after reset" },
  { args: "06", expected: 1, description: "AS: if condition instead of loop" },
  { args: "07", expected: 0, description: "AS: unrolled access" },
];

defineSuite({
  name: "as-second-loop-test",
  // slow
  skipPvmInPvm: true,
  tests: tests,
});
