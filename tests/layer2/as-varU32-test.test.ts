import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "0a141e2832", expected: 60, description: "AS: varU32 decode - single byte values" },
  { args: "0102030405", expected: 6, description: "AS: varU32 decode - small values" },
];

defineSuite({
  name: "as-varU32-test",
  tests: tests,
});
