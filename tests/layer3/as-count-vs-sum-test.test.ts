import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00", expected: 5010, description: "AS: count+sum hardcoded limit" },
  { args: "0105", expected: 5010, description: "AS: count+sum limit from args" },
  { args: "0205", expected: 5, description: "AS: just count, limit from args" },
  { args: "0305", expected: 10, description: "AS: just sum, limit from args" },
  { args: "0405", expected: 5010, description: "AS: count then sum, limit from args" },
  { args: "0505", expected: 655391, description: "AS: bits+sum, limit from args" },
  { args: "0602", expected: 101, description: "AS: i before/after arr access" },
  { args: "0705", expected: 10, description: "AS: sum i values (no arr access)" },
];

defineSuite({
  name: "as-count-vs-sum-test",
  tests: tests,
});
