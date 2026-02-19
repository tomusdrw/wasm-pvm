import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "0005", expected: 43210, description: "AS: arr[i] digits (limit from args)" },
  { args: "01", expected: 43210, description: "AS: arr[i] digits (hardcoded limit)" },
  { args: "0205", expected: 43210, description: "AS: arr[i] via memory store" },
  { args: "0305", expected: 43210, description: "AS: arr[0..4] outside loop" },
  { args: "0403", expected: 4386, description: "AS: i<<4|arr[i] encoding" },
  { args: "0505", expected: 0, description: "AS: arr[i] vs i mismatch check" },
  { args: "0603", expected: 2232576, description: "AS: i,arr[i] pairs to memory" },
  { args: "0705", expected: 1, description: "AS: just arr[1] with ternary" },
  { args: "0805", expected: 10, description: "AS: simple sum (limit from args)" },
];

defineSuite({
  name: "as-array-value-trace-test",
  // too slow
  skipPvmInPvm: true,
  tests: tests,
});
