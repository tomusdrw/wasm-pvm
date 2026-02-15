import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 1, description: "AS: 0! = 1" },
  { args: "05000000", expected: 120, description: "AS: 5! = 120" },
  { args: "07000000", expected: 5040, description: "AS: 7! = 5040" },
];

defineSuite({
  name: "as-factorial",
  tests: tests,
});
