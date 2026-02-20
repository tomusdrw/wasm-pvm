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

  // Test 7: memory.copy with len=0 (no-op edge case).
  { args: "07000000", expected: 0x00000000, description: "memory.copy len=0 (no-op)" },

  // Test 8: memory.fill with len=0 (no-op edge case).
  { args: "08000000", expected: 0x00000000, description: "memory.fill len=0 (no-op)" },

  // Test 9: memory.copy with len=3 (pure byte tail, no word loop).
  { args: "09000000", expected: 0x00030201, description: "memory.copy 3 bytes (byte-only tail)" },

  // Test 10: memory.fill with len=5 (pure byte tail, no word loop).
  { args: "0a000000", expected: 0x42424242, description: "memory.fill 5 bytes (byte-only tail)" },

  // Test 11: memory.fill with val > 0xFF (masking test).
  { args: "0b000000", expected: 0xABABABAB, description: "memory.fill val=0x1AB masks to 0xAB" },
];

defineSuite({
  name: "memory-copy-word",
  tests: tests,
});
