import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "05000000", expected: 135, description: "add3(5,2,3) + fact(5) + sumSq(3) = 135" },
];

defineSuite({
  name: "as-tests-functions",
  tests: tests,
});
