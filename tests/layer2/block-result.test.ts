import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 42, description: "block with result returns 42" },
  { args: "01000000", expected: 100, description: "block with br returns 100 (not 999)" },
];

defineSuite({
  name: "block-result",
  tests: tests,
});
