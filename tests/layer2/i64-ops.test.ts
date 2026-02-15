import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 14, description: "i64.div_u(100, 7) = 14" },
  { args: "01000000", expected: 2, description: "i64.rem_u(100, 7) = 2" },
  { args: "02000000", expected: 4080, description: "i64.shl(0xFF, 4) = 4080" },
  { args: "03000000", expected: 4080, description: "i64.shr_u(0xFF00, 4) = 4080" },
  { args: "04000000", expected: 240, description: "i64.and(0xF0F0, 0x0FF0) = 240" },
  { args: "05000000", expected: 65520, description: "i64.or(0xF0F0, 0x0FF0) = 65520" },
  { args: "06000000", expected: 65280, description: "i64.xor(0xF0F0, 0x0FF0) = 65280" },
  { args: "07000000", expected: 1, description: "i64.ge_u(100, 50) = 1" },
  { args: "08000000", expected: 1, description: "i64.le_u(50, 100) = 1" },
];

defineSuite({
  name: "i64-ops",
  tests: tests,
});
