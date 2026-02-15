import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 42, description: "computed address with offset = 42" },
  { args: "01000000", expected: 84, description: "computed address with scale = 84" },
];

defineSuite({
  name: "computed-addr-test",
  tests: tests,
});
