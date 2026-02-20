import { defineSuite } from "../helpers/suite";

const tests = [
  // Test 0: Forward non-overlapping copy, 16 bytes (2 full words).
  { args: "00000000", expected: 0x04030201, description: "forward copy 16 bytes (2 words), read first 4" },

  // Test 1: Forward non-overlapping copy, 10 bytes (1 word + 2 byte tail).
  { args: "01000000", expected: 0x08070605, description: "forward copy 10 bytes (word+tail), read middle 4" },

  // Test 2: Backward overlapping copy, 10 bytes. Read start.
  { args: "02000000", expected: 0x04030201, description: "backward copy 10 bytes (overlap), read start" },

  // Test 3: Backward overlapping copy, 10 bytes. Read end.
  { args: "03000000", expected: 0x0A090807, description: "backward copy 10 bytes (overlap), read end" },

  // Test 4: Backward overlapping copy, 16 bytes (2 full words). Read start.
  { args: "04000000", expected: 0x04030201, description: "backward copy 16 bytes (2 words, overlap), read start" },

  // Test 5: Backward overlapping copy, 16 bytes. Read end.
  { args: "05000000", expected: 0x100F0E0D, description: "backward copy 16 bytes (2 words, overlap), read end" },

  // Test 6: memory.fill with 16 bytes (word-sized fill).
  { args: "06000000", expected: 0xABABABAB, description: "memory.fill 16 bytes with 0xAB (word-sized)" },
];

defineSuite({
  name: "memory-copy-word",
  tests: tests,
});
