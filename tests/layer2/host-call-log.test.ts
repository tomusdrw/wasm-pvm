import { describe, expect, test } from "bun:test";
import { resolve } from "node:path";
import { defineSuite } from "../helpers/suite";
import { getSuite } from "../data/test-cases";
import { runJamWithOutput } from "../helpers/run";
import { JAM_DIR } from "../helpers/paths";

// Standard suite test (result value check).
defineSuite(getSuite("host-call-log"));

// Additional test: verify that the log host call produces expected output.
describe("host-call-log output", () => {
  test("should emit [INFO] test-log: Hello from PVM!", () => {
    const jamFile = resolve(JAM_DIR, "host-call-log.jam");
    const result = runJamWithOutput(jamFile, "00000000");
    expect(result.exitValue).toBe(42);
    expect(result.stdout).toContain("[INFO] test-log: Hello from PVM!");
  });
});
