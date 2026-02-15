import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "050000000700000002000000", expected: 25, description: "((5+7)*2) | 1 >> 1 = 25" },
];

defineSuite({
  name: "as-tests-arithmetic",
  tests: tests,
});
