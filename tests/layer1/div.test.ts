import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "1400000005000000", expected: 4, description: "20 / 5 = 4" },
  { args: "6400000008000000", expected: 12, description: "100 / 8 = 12" },
  { args: "0a00000003000000", expected: 3, description: "10 / 3 = 3" },
];

defineSuite({
  name: "div",
  tests: tests,
});
