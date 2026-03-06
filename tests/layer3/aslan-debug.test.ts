import fs from "node:fs";
import path from "node:path";
import { describe, test, expect } from "bun:test";
import { JAM_DIR } from "../helpers/paths";

// Regression test: as-lan debug build triggers LLVM instcombine convergence
// failure without max-iterations=2 in Phase 1 optimization passes.
// This test only verifies compilation succeeds (JAM file is produced),
// not runtime correctness — the debug build uses host calls that aren't
// available in the test runner.
describe("aslan-debug", () => {
  const jamFile = path.join(JAM_DIR, "aslan-debug.jam");

  test("compilation produces a valid JAM file", () => {
    expect(fs.existsSync(jamFile)).toBe(true);
    const stat = fs.statSync(jamFile);
    expect(stat.size).toBeGreaterThan(1000);
  });
});
