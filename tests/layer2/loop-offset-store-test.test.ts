import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 20995, description: "loop no call_indirect, pc=20 gas=995" },
  { args: "01000000", expected: 20995, description: "loop with call_indirect, pc=20 gas=995" },
  { args: "02000000", expected: 4, description: "single call + offset store, pc=4" },
  { args: "03000000", expected: 8, description: "two calls + offset store, pc=8" },
  { args: "04000000", expected: 995, description: "loop gas only, gas=995" },
  { args: "05000000", expected: 20, description: "loop pc only with call_indirect" },
  { args: "06000000", expected: 20, description: "loop pc only no call_indirect" },
];

defineSuite({
  name: "loop-offset-store-test",
  tests: tests,
});
