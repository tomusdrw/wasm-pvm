import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "05", expected: 1, description: "AS: arr[1] after ternary (flat)" },
];

defineSuite({
  name: "as-flat-ternary-drop",
  tests: tests,
});
