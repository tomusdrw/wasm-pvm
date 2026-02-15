import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "3000000012000000", expected: 6, description: "gcd(48, 18) = 6" },
  { args: "6400000038000000", expected: 4, description: "gcd(100, 56) = 4" },
  { args: "1100000011000000", expected: 17, description: "gcd(17, 17) = 17" },
  { args: "01000000ff000000", expected: 1, description: "gcd(1, 255) = 1" },
];

defineSuite({
  name: "gcd",
  tests: tests,
});
