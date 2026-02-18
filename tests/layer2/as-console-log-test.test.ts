import path from "node:path";
import { describe, test, expect } from "bun:test";
import { JAM_DIR } from "../helpers/paths";
import { runJamWithOutput } from "../helpers/run";

// Tests that console.log import is mapped via import map (ecalli:3:ptr).
// The anan-as runner halts at unhandled ecalli with "Finished with status: 3",
// where 3 is the PVM host-call status. We verify:
// 1. The program compiles successfully (JAM file exists)
// 2. When run, it halts at ecalli with status 3
// 3. r7 contains a PVM address (>= 0x50000 wasm_memory_base), proving ptr conversion works
describe("as-console-log-test", () => {
  const jamFile = path.join(JAM_DIR, "as-console-log-test.jam");

  test("console.log mapped to ecalli:3 via import map", () => {
    const result = runJamWithOutput(jamFile, "0000000000000000");
    // The PVM runner halts at ecalli with status 3 (host-call).
    expect(result.stdout).toContain("Finished with status: 3");
  });

  test("ecalli receives PVM-converted pointer (>= 0x50000)", () => {
    const result = runJamWithOutput(jamFile, "0000000000000000");
    // At the time of ecalli, r7 should contain a PVM address (wasm_memory_base + wasm_ptr).
    // wasm_memory_base is 0x50000, so r7 should be >= 0x50000.
    // The final registers show r7's value. Parse it from the output.
    const finalRegsMatch = result.stdout.match(
      /Registers: \[(\d+(?:, \d+)*)\]/
    );
    expect(finalRegsMatch).not.toBeNull();
    if (finalRegsMatch) {
      const regs = finalRegsMatch[1].split(", ").map(Number);
      // r7 is index 7
      expect(regs[7]).toBeGreaterThanOrEqual(0x50000);
    }
  });
});
