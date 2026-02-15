import { defineSuite } from "../helpers/suite";

const tests = [
  // Test 0: dst > src with overlap — requires backward copy (memmove).
  // Copy 4 bytes from src=2 to dst=4. Read from addr 4.
  // Correct: [03 04 05 06] = 0x06050403. Forward-copy bug: [03 04 03 04].
  { args: "00000000", expected: 0x06050403, description: "memory.copy overlap dst>src (memmove backward)" },

  // Test 1: dst < src with overlap — forward copy is correct.
  // Copy 4 bytes from src=4 to dst=2. Read from addr 2.
  // Correct: [05 06 07 08] = 0x08070605.
  { args: "01000000", expected: 0x08070605, description: "memory.copy overlap dst<src (forward)" },

  // Test 2: Non-overlapping copy.
  // Copy 4 bytes from src=0 to dst=8. Read from addr 8.
  // Correct: [01 02 03 04] = 0x04030201.
  { args: "02000000", expected: 0x04030201, description: "memory.copy non-overlapping" },
];

defineSuite({
  name: "memory-copy-overlap",
  tests: tests,
});
