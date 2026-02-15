import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 1, description: "AS: nested if, step 0, just arr[1]" },
  { args: "0105", expected: 501, description: "AS: nested if, step 1, ternary+arr[1]" },
  { args: "02", expected: 2, description: "AS: nested if, step 2" },
];

defineSuite({
  name: "as-nested-if-test",
  tests: tests,
});
