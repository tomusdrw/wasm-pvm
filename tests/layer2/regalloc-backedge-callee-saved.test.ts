import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 6, description: "n=0: init only, a+b+c = 6" },
  { args: "01000000", expected: 11, description: "n=1: one call iteration" },
  { args: "02000000", expected: 23, description: "n=2: two iterations with call" },
  { args: "03000000", expected: 48, description: "n=3: three iterations" },
  { args: "05000000", expected: 187, description: "n=5: five iterations" },
  { args: "0a000000", expected: 4303, description: "n=10: ten iterations" },
];

defineSuite({
  name: "regalloc-backedge-callee-saved",
  tests: tests,
});
