import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "05000000", expected: 22, description: "input=5 -> 2 (else) + 5 (while) + 15 (nested) = 22" },
  { args: "0B000000", expected: 27, description: "input=11 -> 1 (if) + 11 (while) + 15 (nested) = 27" },
];

defineSuite({
  name: "as-tests-control-flow",
  tests: tests,
});
