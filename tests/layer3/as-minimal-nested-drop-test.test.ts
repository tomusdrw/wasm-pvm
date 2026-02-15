import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 42, description: "AS: direct call (no nesting)" },
  { args: "0105", expected: 42, description: "AS: ternary+drop then getSecondArg" },
  { args: "0205", expected: 100, description: "AS: ternary+drop then getFirstArg" },
  { args: "03", expected: 42, description: "AS: direct call in nested if" },
  { args: "04", expected: 100042, description: "AS: multiple calls" },
];

defineSuite({
  name: "as-minimal-nested-drop-test",
  tests: tests,
});
