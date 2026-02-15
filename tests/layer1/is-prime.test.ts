import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 0, description: "is_prime(0) = 0" },
  { args: "01000000", expected: 0, description: "is_prime(1) = 0" },
  { args: "02000000", expected: 1, description: "is_prime(2) = 1" },
  { args: "03000000", expected: 1, description: "is_prime(3) = 1" },
  { args: "04000000", expected: 0, description: "is_prime(4) = 0" },
  { args: "05000000", expected: 1, description: "is_prime(5) = 1" },
  { args: "11000000", expected: 1, description: "is_prime(17) = 1" },
  { args: "19000000", expected: 0, description: "is_prime(25) = 0" },
  { args: "61000000", expected: 1, description: "is_prime(97) = 1" },
  { args: "64000000", expected: 0, description: "is_prime(100) = 0" },
  { args: "65000000", expected: 1, description: "is_prime(101) = 1" },
];

defineSuite({
  name: "is-prime",
  tests: tests,
});
