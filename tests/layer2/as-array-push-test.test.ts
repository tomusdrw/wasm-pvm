import { defineSuite } from "../helpers/suite";

const tests = [
  {
    args: "",
    expected: 28,
    description: "AS: Array.push() sum test - should return 28",
  },
];

defineSuite({
  name: "as-array-push-test",
  tests: tests,
});
