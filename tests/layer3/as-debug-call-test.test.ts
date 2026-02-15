import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "01", expected: 101, description: "AS: arr[1] before*100 + after" },
  { args: "02", expected: 1281, description: "AS: limit<<8|valueAfter (valueBefore=0 bug)" },
  { args: "03", expected: 1, description: "AS: arr ptr survives ternary" },
  { args: "04", expected: 1, description: "AS: arr[1] via dataPtr after ternary" },
  { args: "05", expected: 0, description: "AS: arr[0] after ternary" },
  { args: "06", expected: 2, description: "AS: arr[2] after ternary" },
  { args: "07", expected: 3, description: "AS: arr[3] after ternary" },
  { args: "08", expected: 123, description: "AS: arr[0..3] without ternary" },
  { args: "09", expected: 12, description: "AS: arr[0..2] multi-ternary" },
];

defineSuite({
  name: "as-debug-call-test",
  tests: tests,
});
