import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 45, description: "AS: lowerBytes - source sum" },
  { args: "01", expected: 45, description: "AS: lowerBytes - subarray sum" },
  { args: "02", expected: 10045, description: "AS: lowerBytes - result len*1000 + sum" },
  { args: "03", expected: 10045, description: "AS: lowerBytes on subarray - len*1000 + sum" },
  { args: "04", expected: 123435, description: "AS: lowerBytes - element breakdown" },
  { args: "05", expected: 999, description: "AS: lowerBytes - all indices match" },
];

defineSuite({
  name: "as-lowerBytes-test",
  tests: tests,
});
