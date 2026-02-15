import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 21, description: "sum with base 0: 1+2+3+4+5+6 = 21" },
  { args: "0a000000", expected: 81, description: "sum with base 10: 11+12+13+14+15+16 = 81" },
  { args: "64000000", expected: 621, description: "sum with base 100: 101+102+103+104+105+106 = 621" },
];

defineSuite({
  name: "many-locals",
  tests: tests,
});
