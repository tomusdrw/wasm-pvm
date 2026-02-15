import { defineSuite } from "../helpers/suite";

const tests = [
  {
    args: "",
    expected: 1463,
    description: "AS: Subarray offset correctness test",
  },
];

defineSuite({
  name: "as-subarray-offset-test",
  tests: tests,
});
