import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "0500000007000000", expected: 4277796864, description: "AS: memory args test - args_ptr check" },
];

defineSuite({
  name: "as-memory-args-test",
  tests: tests,
});
