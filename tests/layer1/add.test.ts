import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "0500000007000000", expected: 12, description: "5 + 7 = 12" },
  { args: "00000000ffffffff", expected: 4294967295, description: "0 + MAX = MAX" },
  { args: "01000000ffffffff", expected: 0, description: "1 + MAX = 0 (overflow)" },
];

defineSuite({
  name: "add",
  tests: tests,
});
