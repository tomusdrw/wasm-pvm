import { defineSuite } from "../helpers/suite";

const tests = [
  {
    args: "",
    expected: 1107,
    description: "AS: alloc test (minimal runtime) = 1107",
  },
];

defineSuite({
  name: "as-alloc-test-minimal",
  tests: tests,
});
