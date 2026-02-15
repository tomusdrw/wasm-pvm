import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 16, description: "AS: life 0 steps - returns width=16" },
  { args: "01000000", expected: 16, description: "AS: life 1 step - returns width=16" },
  { args: "05000000", expected: 16, description: "AS: life 5 steps - returns width=16" },
];

defineSuite({
  name: "as-life",
  tests: tests,
});
