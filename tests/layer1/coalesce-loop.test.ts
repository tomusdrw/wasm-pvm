import { defineSuite } from "../helpers/suite";

// sum(1..n): simple accumulator loop with 2 phi nodes (i, sum).
// Tests that loop phi early interval expiration produces correct results.
const tests = [
  { args: "0a00000000000000", expected: 55, description: "sum(1..10) = 55" },
  { args: "0100000000000000", expected: 1, description: "sum(1..1) = 1" },
  { args: "6400000000000000", expected: 5050, description: "sum(1..100) = 5050" },
];

defineSuite({
  name: "coalesce-loop",
  tests: tests,
});
