import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "03000000aabbcc", expected: 4277919488, description: "AS: array test - args_ptr check" },
];

defineSuite({
  name: "as-array-test",
  tests: tests,
});
