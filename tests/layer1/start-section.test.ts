import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 42, description: "start-section returns 42" },
];

defineSuite({
  name: "start-section",
  tests: tests,
});
