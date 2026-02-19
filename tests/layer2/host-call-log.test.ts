import { describe, expect, test } from "bun:test";
import { resolve } from "node:path";
import { defineSuite } from "../helpers/suite";
import { runJamWithOutput } from "../helpers/run";
import { JAM_DIR } from "../helpers/paths";

// Standard suite test (result value check).
defineSuite({
  name: "host-call-log",
  // Uses ecalli 100 (JIP-1 logging) which the inner pvm-in-pvm interpreter can't handle.
  skipPvmInPvm: true,
  tests: [
    {
      args: "00000000",
      expected: 42,
      description: "host_call log returns 42",
    },
  ],
});

// Additional test: verify that the log host call produces expected output.
describe("host-call-log output", () => {
  test("should emit [INFO] test-log: Hello from PVM!", () => {
    const jamFile = resolve(JAM_DIR, "host-call-log.jam");
    const result = runJamWithOutput(jamFile, "00000000");
    expect(result.exitCode).toBe(0);
    expect(result.exitValue).toBe(42);
    expect(result.stdout).toContain("[INFO] test-log: Hello from PVM!");
  });
});
