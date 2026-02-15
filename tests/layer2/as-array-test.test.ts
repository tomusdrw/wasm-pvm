import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "03000000aabbcc", expected: 4277796864, description: "AS: array test - args_ptr check" },
];

defineSuite({
  name: "as-array-test",
  tests: tests,
});
