import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 20, description: "AS: call and save to local" },
  { args: "01", expected: 202, description: "AS: saved after if-else" },
  { args: "02", expected: 205, description: "AS: saved after ternary" },
  { args: "03", expected: 205, description: "AS: saved in nested if" },
  { args: "04", expected: 4444, description: "AS: constant result" },
  { args: "05", expected: 505, description: "AS: step*100 + ternary" },
];

defineSuite({
  name: "as-local-clobber-test",
  tests: tests,
});
