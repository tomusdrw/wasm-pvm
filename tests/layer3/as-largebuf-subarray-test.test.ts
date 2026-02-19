import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 45, description: "AS: large buffer direct sum" },
  { args: "01", expected: 45, description: "AS: large buffer subarray sum" },
  { args: "02", expected: 45, description: "AS: large buffer lowerBytes sum" },
  { args: "03", expected: 10045, description: "AS: large buffer lowerBytes len+sum" },
  { args: "04", expected: 1045, description: "AS: large buffer middle slice" },
  { args: "05", expected: 190, description: "AS: large buffer multiple slices" },
];

defineSuite({
  name: "as-largebuf-subarray-test",
  // too slow
  skipPvmInPvm: true,
  tests: tests,
});
