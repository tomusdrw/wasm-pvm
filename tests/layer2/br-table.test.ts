import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 100, description: "br_table case 0 returns 100" },
  { args: "01000000", expected: 200, description: "br_table case 1 returns 200" },
  { args: "02000000", expected: 300, description: "br_table case 2 returns 300" },
  { args: "03000000", expected: 999, description: "br_table default case returns 999" },
  { args: "04000000", expected: 999, description: "br_table out of bounds returns 999" },
];

defineSuite({
  name: "br-table",
  tests: tests,
});
