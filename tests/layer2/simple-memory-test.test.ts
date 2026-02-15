import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 42, description: "memory store8/load8_u: store 42, read back 42" },
];

defineSuite({
  name: "simple-memory-test",
  tests: tests,
});
