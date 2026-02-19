import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 190, description: "AS: Pattern A - direct accumulation" },
  { args: "01", expected: 190, description: "AS: Pattern B - separate sums" },
  { args: "02", expected: 190, description: "AS: Pattern C - no lowerBytes" },
  { args: "03", expected: 190, description: "AS: Pattern D - explicit length" },
];

defineSuite({
  name: "as-minimal-fail",
  // too slow
  skipPvmInPvm: true,
  tests: tests,
});
