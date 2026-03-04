import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 0, description: "n=0: empty loops" },
  { args: "01000000", expected: 0, description: "n=1: single element (0)" },
  { args: "02000000", expected: 6, description: "n=2: 0+1+2+3 = 6" },
  { args: "03000000", expected: 36, description: "n=3: sum(0..8) = 36" },
  { args: "04000000", expected: 120, description: "n=4: sum(0..15) = 120" },
  { args: "0a000000", expected: 4950, description: "n=10: sum(0..99) = 4950" },
];

defineSuite({
  name: "regalloc-nested-loops",
  tests: tests,
});
