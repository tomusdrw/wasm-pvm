import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 1, description: "AS: arr[1] no ternary" },
  { args: "0105", expected: 1, description: "AS: arr[1] after ternary" },
  { args: "0205", expected: 5, description: "AS: just ternary value" },
  { args: "0305", expected: 0, description: "AS: arr[0] after ternary" },
  { args: "0405", expected: 2, description: "AS: arr[2] after ternary" },
  { args: "0505", expected: 51, description: "AS: limit*10 + arr[1]" },
  { args: "0605", expected: 1, description: "AS: arr[1] (no arr2)" },
  { args: "0705", expected: 15, description: "AS: arr[1] first, then ternary" },
  { args: "0805", expected: 1, description: "AS: direct memory after ternary" },
];

defineSuite({
  name: "as-simple-call-test",
  tests: tests,
});
