import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "3000000012000000", expected: 6, description: "AS: gcd(48, 18) = 6" },
  { args: "6400000038000000", expected: 4, description: "AS: gcd(100, 56) = 4" },
  { args: "1100000011000000", expected: 17, description: "AS: gcd(17, 17) = 17" },
];

defineSuite({
  name: "as-gcd",
  tests: tests,
});
