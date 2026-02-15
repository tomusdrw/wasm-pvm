import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 1, description: "0! = 1" },
  { args: "01000000", expected: 1, description: "1! = 1" },
  { args: "05000000", expected: 120, description: "5! = 120" },
  { args: "0a000000", expected: 3628800, description: "10! = 3628800" },
];

defineSuite({
  name: "factorial",
  tests: tests,
});
