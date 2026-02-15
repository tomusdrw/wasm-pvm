import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 20, description: "AS: two simple loops, no calls" },
  { args: "01", expected: 10, description: "AS: loop counter value after loop" },
  { args: "02", expected: 10, description: "AS: reset counter and run second loop" },
  { args: "03", expected: 90, description: "AS: two loops with unchecked access" },
  { args: "04", expected: 90, description: "AS: two loops with manual length" },
  { args: "05", expected: 90, description: "AS: two loops with checked access (calls)" },
  { args: "06", expected: 10045, description: "AS: arr2.length*1000 + loop1 result" },
  { args: "07", expected: 1, description: "AS: 0 < arr2.length comparison" },
  { args: "08", expected: 100, description: "AS: single loop 100 iterations" },
];

defineSuite({
  name: "as-loop-counter-test",
  tests: tests,
});
