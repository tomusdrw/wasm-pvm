import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "", expected: 42, description: "main (PC=0) returns 42" },
  {
    args: "",
    expected: 99,
    description: "main2 (PC=5) returns 99",
    pc: 5,
  },
];

defineSuite({
  name: "entry-points",
  tests: tests,
});
