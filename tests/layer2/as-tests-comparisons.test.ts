import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "0500000005000000", expected: 31, description: "AS: 5 == 5, all comparison bits set" },
  { args: "0300000007000000", expected: 10, description: "AS: 3 < 7, <= and signed comparisons" },
  { args: "0a00000005000000", expected: 20, description: "AS: 10 > 5, >= comparisons" },
];

defineSuite({
  name: "as-tests-comparisons",
  tests: tests,
});
