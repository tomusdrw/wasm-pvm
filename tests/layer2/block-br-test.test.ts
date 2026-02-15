import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 10, description: "block with conditional br (skip branch)" },
  { args: "01000000", expected: 20, description: "block with conditional br (take branch)" },
  { args: "02000000", expected: 30, description: "nested blocks with br_if" },
];

defineSuite({
  name: "block-br-test",
  tests: tests,
});
