import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 2, description: "nested calls: add_two(0) = 2" },
  { args: "05000000", expected: 7, description: "nested calls: add_two(5) = 7" },
  { args: "64000000", expected: 102, description: "nested calls: add_two(100) = 102" },
];

defineSuite({
  name: "nested-calls",
  tests: tests,
});
