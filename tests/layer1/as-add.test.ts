import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "0500000007000000", expected: 12, description: "AS: 5 + 7 = 12" },
  { args: "0a00000014000000", expected: 30, description: "AS: 10 + 20 = 30" },
];

defineSuite({
  name: "as-add",
  tests: tests,
});
