import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 142, description: "AS: addWithMem no nesting" },
  { args: "01", expected: 142, description: "AS: addWithMem with ternary" },
  { args: "02", expected: 242, description: "AS: addWithMem(200,42)" },
  { args: "03", expected: 142, description: "AS: addWithMem no ternary" },
  { args: "04", expected: 142, description: "AS: addWithMem(step*25, 42)" },
  { args: "05", expected: 20, description: "AS: loadAt after ternary" },
];

defineSuite({
  name: "as-two-arg-call-test",
  tests: tests,
});
