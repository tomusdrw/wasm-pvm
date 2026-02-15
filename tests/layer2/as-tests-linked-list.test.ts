import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 60, description: "Linked list sum (recursive)" },
];

defineSuite({
  name: "as-tests-linked-list",
  tests: tests,
});
