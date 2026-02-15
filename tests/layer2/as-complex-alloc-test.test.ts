import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "0000000000000000", expected: 1090, description: "AS: Complex allocation test" },
];

defineSuite({
  name: "as-complex-alloc-test",
  tests: tests,
});
