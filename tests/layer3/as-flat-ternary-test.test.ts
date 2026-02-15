import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "05", expected: 501, description: "AS: limit=5, arr[1]=1 (flat)" },
  { args: "03", expected: 301, description: "AS: limit=3, arr[1]=1 (flat)" },
  { args: "01", expected: 101, description: "AS: limit=1, arr[1]=1 (flat)" },
];

defineSuite({
  name: "as-flat-ternary-test",
  tests: tests,
});
