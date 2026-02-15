import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 1, description: "recursive factorial(0) = 1" },
  { args: "01000000", expected: 1, description: "recursive factorial(1) = 1" },
  { args: "05000000", expected: 120, description: "recursive factorial(5) = 120" },
  { args: "07000000", expected: 5040, description: "recursive factorial(7) = 5040" },
];

defineSuite({
  name: "recursive",
  tests: tests,
});
