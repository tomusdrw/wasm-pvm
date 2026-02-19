import { defineSuite } from "../helpers/suite";

const tests = [
  {
    args: "",
    expected: 1107,
    description: "AS: alloc test (incremental GC) = 1107",
  },
];

defineSuite({
  name: "as-alloc-test-incremental",
  // too slow
  skipPvmInPvm: true,
  tests: tests,
});
