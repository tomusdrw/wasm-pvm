import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "000101", expected: 1, description: "AS: 1 && 1 = true" },
  { args: "000100", expected: 0, description: "AS: 1 && 0 = false" },
  { args: "000001", expected: 0, description: "AS: 0 && 1 = false" },
  { args: "010309", expected: 1, description: "AS: 3<5 && 9<10 = true" },
  { args: "010509", expected: 0, description: "AS: 5<5 && 9<10 = false" },
  { args: "020102", expected: 1, description: "AS: 1<2 && 1<10 = true" },
  { args: "020202", expected: 0, description: "AS: 2<2 && 2<10 = false" },
  { args: "0305", expected: 5, description: "AS: loop count limit=5" },
  { args: "030f", expected: 10, description: "AS: loop count limit=15 (capped at 10)" },
  { args: "04", expected: 1234, description: "AS: loop iterations 0,1,2,3,4" },
  { args: "060102", expected: 1, description: "AS: 1 < 2 = true" },
  { args: "060202", expected: 0, description: "AS: 2 < 2 = false" },
  { args: "08", expected: 1, description: "AS: 1<2 && 1<10 hardcoded" },
];

defineSuite({
  name: "as-if-result-test",
  tests: tests,
});
