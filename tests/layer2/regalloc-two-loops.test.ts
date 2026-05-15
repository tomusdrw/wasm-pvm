import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 76, description: "n=0: init only, e+f+g+h = 76" },
  { args: "01000000", expected: 211, description: "n=1: one iteration each loop" },
  { args: "02000000", expected: 720, description: "n=2: two iterations each loop" },
  { args: "03000000", expected: 2851, description: "n=3: three iterations" },
  { args: "05000000", expected: 58958, description: "n=5: five iterations" },
  { args: "0a000000", expected: 165809572, description: "n=10: ten iterations" },
];

defineSuite({
  name: "regalloc-two-loops",
  tests: tests,
});
