import { defineSuite } from "../helpers/suite";

const tests = [
  { args: "00000000", expected: 33620481, description: "memory.copy forward overlap: copy 4 bytes from 0x50000 to 0x50002" },
  { args: "01000000", expected: 134678021, description: "memory.copy backward overlap: copy 4 bytes from 0x50004 to 0x50002" },
];

defineSuite({
  name: "memory-copy-overlap",
  tests: tests,
});
