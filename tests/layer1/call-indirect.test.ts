import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "0000000005000000", expected: 10, description: "call_indirect double(5) = 10" },
  { args: "0100000005000000", expected: 15, description: "call_indirect triple(5) = 15" },
  { args: "000000000a000000", expected: 20, description: "call_indirect double(10) = 20" },
  { args: "010000000a000000", expected: 30, description: "call_indirect triple(10) = 30" },
];

defineSuite({
  name: "call-indirect",
  tests: tests,
});
