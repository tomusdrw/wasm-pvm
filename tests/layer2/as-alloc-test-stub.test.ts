import { defineSuite } from "../helpers/suite";

const tests = [
  {
    args: "",
    expected: 1107,
    description: "AS: alloc test (stub runtime) = 1107",
  },
];

defineSuite({
  name: "as-alloc-test-stub",
  // too slow
  skipPvmInPvm: true,
  tests: tests,
});
