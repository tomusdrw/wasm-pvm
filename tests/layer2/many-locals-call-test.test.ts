import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 4, description: "single call, pc=4" },
  { args: "01000000", expected: 20995, description: "loop 5x call_indirect + gas, pc=20 gas=995" },
  { args: "02000000", expected: 78, description: "local preservation after call" },
  { args: "0300000001000000", expected: 42, description: "dynamic table idx=1, 3*(1+13)=42" },
];

defineSuite({
  name: "many-locals-call-test",
  tests: tests,
});
