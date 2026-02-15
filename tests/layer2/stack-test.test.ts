import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 30, description: "stack operations: 10*2 + 10 = 30" },
  { args: "01000000", expected: 50, description: "stack operations: 20*2 + 10 = 50" },
];

defineSuite({
  name: "stack-test",
  tests: tests,
});
