import { defineSuite } from "../helpers/suite";

const tests = [
  {
    args: "",
    expected: 30,
    description: "AS: Uint8Array.subarray() test - should return 30",
  },
];

defineSuite({
  name: "as-subarray-test",
  tests: tests,
});
