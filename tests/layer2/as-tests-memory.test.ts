import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 850, description: "Byte manipulation check" },
];

defineSuite({
  name: "as-tests-memory",
  tests: tests,
});
