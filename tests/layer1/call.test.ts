import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "05000000", expected: 10, description: "double(5) = 10" },
  { args: "0a000000", expected: 20, description: "double(10) = 20" },
  { args: "00000000", expected: 0, description: "double(0) = 0" },
];

defineSuite({
  name: "call",
  tests: tests,
});
