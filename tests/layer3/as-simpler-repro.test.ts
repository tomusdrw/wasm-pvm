import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "0500", expected: 0, description: "AS: arr[0] (THEN branch)" },
  { args: "0501", expected: 1, description: "AS: arr[1] (THEN branch)" },
  { args: "0502", expected: 2, description: "AS: arr[2] (THEN branch)" },
];

defineSuite({
  name: "as-simpler-repro",
  tests: tests,
});
