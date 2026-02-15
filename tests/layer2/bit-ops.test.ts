import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "0000000001000000", expected: 31, description: "clz(1) = 31 (leading zeros in 32-bit)" },
  { args: "0000000000000080", expected: 0, description: "clz(0x80000000) = 0 (MSB set)" },
  { args: "0100000001000000", expected: 0, description: "ctz(1) = 0 (LSB set)" },
  { args: "0100000002000000", expected: 1, description: "ctz(2) = 1" },
  { args: "02000000ffffffff", expected: 32, description: "popcnt(0xffffffff) = 32" },
  { args: "02000000f0f0f0f0", expected: 16, description: "popcnt(0xf0f0f0f0) = 16" },
];

defineSuite({
  name: "bit-ops",
  tests: tests,
});
