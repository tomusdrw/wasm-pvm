import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "0a141e2832", expected: 60, description: "AS: Decoder subarray pattern - sum of first 3" },
  { args: "0102030405", expected: 6, description: "AS: Decoder subarray pattern - small values" },
];

defineSuite({
  name: "as-decoder-subarray-test",
  tests: tests,
});
