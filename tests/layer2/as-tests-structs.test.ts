import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 32, description: "Struct emulation (Dot Product)" },
];

defineSuite({
  name: "as-tests-structs",
  tests: tests,
});
