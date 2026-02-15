import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000ff00000001000000", expected: 510, description: "rotl(0xff, 1) = 0x1fe" },
  { args: "00000000ff00000008000000", expected: 65280, description: "rotl(0xff, 8) shifts left 8" },
  { args: "01000000ff00000001000000", expected: 2147483775, description: "rotr(0xff, 1) rotates right" },
  { args: "01000000cdab000010000000", expected: 2882338816, description: "rotr(0xabcd, 16) swaps to high bytes" },
];

defineSuite({
  name: "rotate",
  tests: tests,
});
