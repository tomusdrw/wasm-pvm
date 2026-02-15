import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 100, description: "Manual array implementation (Sum)" },
];

defineSuite({
  name: "as-tests-arrays",
  tests: tests,
});
