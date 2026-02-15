import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 20, description: "AS: loadFromMemory no nesting" },
  { args: "01", expected: 20, description: "AS: loadFromMemory with ternary" },
  { args: "02", expected: 10, description: "AS: loadFromMemory index 0" },
  { args: "03", expected: 20, description: "AS: loadFromMemory no ternary" },
];

defineSuite({
  name: "as-noinline-call-test",
  tests: tests,
});
