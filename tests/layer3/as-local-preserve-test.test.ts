import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 148, description: "AS: locals after simple call" },
  { args: "01", expected: 66, description: "AS: locals after multiple calls" },
  { args: "02", expected: 310, description: "AS: locals after 4-arg call" },
  { args: "03", expected: 23, description: "AS: locals after loop with calls" },
  { args: "04", expected: 336, description: "AS: locals after two loops with calls" },
  { args: "05", expected: 22, description: "AS: local $3 (r12) after call" },
];

defineSuite({
  name: "as-local-preserve-test",
  tests: tests,
});
