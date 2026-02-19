import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "0001", expected: 1, description: "AS: 1 iter in loop1, loop2 runs" },
  { args: "0002", expected: 101, description: "AS: 2 iters in loop1, loop2 runs" },
  { args: "0005", expected: 1001, description: "AS: 5 iters in loop1, loop2 runs" },
  { args: "000a", expected: 4501, description: "AS: 10 iters in loop1, loop2 runs" },
  { args: "03", expected: 45, description: "AS: full loop1, one loop2 iter" },
  { args: "04", expected: 90, description: "AS: full 10+10 iterations" },
];

defineSuite({
  name: "as-iteration-count-test",
  // too slow
  skipPvmInPvm: true,
  tests: tests,
});
