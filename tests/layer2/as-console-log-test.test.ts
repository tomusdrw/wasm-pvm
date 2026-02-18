import path from "node:path";
import { describe, test, expect } from "bun:test";
import { JAM_DIR } from "../helpers/paths";
import { runJamWithOutput } from "../helpers/run";

// Tests that console.log import is mapped via adapter WAT to JIP-1 log (ecalli 100).
// The anan-as runner handles ecalli 100 as JIP-1 logging and continues execution.
// We verify:
// 1. The program compiles and runs successfully
// 2. The JIP-1 log message appears in output
// 3. The program returns the expected result (42)
describe("as-console-log-test", () => {
  const jamFile = path.join(JAM_DIR, "as-console-log-test.jam");

  test("console.log mapped to ecalli:100 (JIP-1 log) via adapter", () => {
    const result = runJamWithOutput(jamFile, "0000000000000000");
    // The program should complete successfully (status 0) since the runner
    // handles ecalli 100 as JIP-1 logging.
    expect(result.stdout).toContain("Exit code: 0");
  });

  test("JIP-1 log message appears in output", () => {
    const result = runJamWithOutput(jamFile, "0000000000000000");
    // The runner decodes the JIP-1 log and prints the AS UTF-16 string.
    expect(result.stdout).toMatch(/H.?e.?l.?l.?o/);
  });

  test("program returns expected result (42)", () => {
    const result = runJamWithOutput(jamFile, "0000000000000000");
    // Result should be 42
    expect(result.exitValue).toBe(42);
  });
});
