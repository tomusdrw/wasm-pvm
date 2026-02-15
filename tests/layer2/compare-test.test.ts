import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 1, description: "comparison: 3 < 5 = 1" },
  { args: "01000000", expected: 0, description: "comparison: 5 < 3 = 0" },
  { args: "02000000", expected: 1, description: "comparison: 10 > 5 = 1" },
  { args: "03000000", expected: 0, description: "comparison: 5 > 10 = 0" },
];

defineSuite({
  name: "compare-test",
  tests: tests,
});
