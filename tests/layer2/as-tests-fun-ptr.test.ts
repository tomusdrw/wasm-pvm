import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 50, description: "Function pointers / Indirect calls" },
];

defineSuite({
  name: "as-tests-fun-ptr",
  tests: tests,
});
