import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 0, description: "AS: fib(0) = 0" },
  { args: "01000000", expected: 1, description: "AS: fib(1) = 1" },
  { args: "0a000000", expected: 55, description: "AS: fib(10) = 55" },
];

defineSuite({
  name: "as-fibonacci",
  tests: tests,
});
