import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "04000000000000001234abcd", expected: 4277796864, description: "AS: decoder test - args_ptr check" },
];

defineSuite({
  name: "as-decoder-test",
  tests: tests,
});
