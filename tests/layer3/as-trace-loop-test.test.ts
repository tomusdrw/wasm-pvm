import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 5, description: "AS: iter count, hardcoded limit" },
  { args: "0105", expected: 5, description: "AS: iter count, limit from args" },
  { args: "02", expected: 31, description: "AS: iter bits, hardcoded" },
  { args: "0305", expected: 31, description: "AS: iter bits, limit from args" },
  { args: "0405", expected: 111, description: "AS: manual cond checks" },
  { args: "0505", expected: 1, description: "AS: 1 < limit check" },
  { args: "0605", expected: 5, description: "AS: limit value" },
  { args: "0705", expected: 5, description: "AS: simple loop, limit from args" },
  { args: "08", expected: 10, description: "AS: simple loop, arr.length" },
];

defineSuite({
  name: "as-trace-loop-test",
  // slow
  skipPvmInPvm: true,
  tests: tests,
});
