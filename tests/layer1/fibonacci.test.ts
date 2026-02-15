import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 0, description: "fib(0) = 0" },
  { args: "01000000", expected: 1, description: "fib(1) = 1" },
  { args: "02000000", expected: 1, description: "fib(2) = 1" },
  { args: "0a000000", expected: 55, description: "fib(10) = 55" },
  { args: "14000000", expected: 6765, description: "fib(20) = 6765" },
];

defineSuite({
  name: "fibonacci",
  tests: tests,
});
