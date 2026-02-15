import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 10, description: "AS: simple arr.length loop" },
  { args: "01", expected: 5, description: "AS: && with arr.length, limit=5" },
  { args: "02", expected: 5, description: "AS: && with arr.length limiting" },
  { args: "03", expected: 10, description: "AS: && + arr access in body" },
  { args: "04", expected: 100, description: "AS: two arrays, loop first, access second" },
  { args: "05", expected: 4, description: "AS: last iteration i value" },
  { args: "06", expected: 5, description: "AS: manual while loop" },
  { args: "07", expected: 5, description: "AS: cached length variable" },
  { args: "08", expected: 20, description: "AS: two loops with arr.length" },
];

defineSuite({
  name: "as-memload-condition-test",
  tests: tests,
});
