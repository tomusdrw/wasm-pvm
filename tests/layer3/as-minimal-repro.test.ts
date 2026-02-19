import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 1, description: "AS: arr[1] no ternary (step 0)" },
  { args: "01", expected: 1, description: "AS: arr[1] after ternary (step 1)" },
  { args: "02", expected: 0, description: "AS: arr[0] after ternary" },
  { args: "03", expected: 2, description: "AS: arr[2] after ternary" },
  { args: "04", expected: 3, description: "AS: arr[3] no ternary" },
  { args: "06", expected: 1, description: "AS: arr_only[1] deep nesting" },
];

defineSuite({
  name: "as-minimal-repro",
  // too slow
  skipPvmInPvm: true,
  tests: tests,
});
