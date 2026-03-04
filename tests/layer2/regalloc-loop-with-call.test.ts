import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 0, description: "n=0: empty loop" },
  { args: "01000000", expected: 1, description: "n=1: 1*1 = 1" },
  { args: "05000000", expected: 55, description: "n=5: 1+4+9+16+25 = 55" },
  { args: "0a000000", expected: 385, description: "n=10: sum of squares 1..10 = 385" },
];

defineSuite({
  name: "regalloc-loop-with-call",
  tests: tests,
});
