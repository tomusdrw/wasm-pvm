import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 17, description: "Global variable manipulation" },
];

defineSuite({
  name: "as-tests-globals",
  tests: tests,
});
