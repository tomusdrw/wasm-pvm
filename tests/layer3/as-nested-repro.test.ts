import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 1, description: "AS: arr[1] no nesting" },
  { args: "01", expected: 1, description: "AS: arr[1] nested if+ternary" },
];

defineSuite({
  name: "as-nested-repro",
  tests: tests,
});
