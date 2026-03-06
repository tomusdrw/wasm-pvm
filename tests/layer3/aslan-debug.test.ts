import path from "node:path";
import { describe, test, expect } from "bun:test";
import { JAM_DIR } from "../helpers/paths";
import { verifyJamStructure } from "../helpers/verify-jam";

// Regression test: as-lan debug build triggers LLVM instcombine convergence
// failure without max-iterations=2 in Phase 1 optimization passes.
//
// This test verifies compilation succeeds by validating the JAM file structure.
// It does NOT use defineSuite because the debug build uses host calls (ecalli)
// that the test runner cannot handle, so runtime execution would always fail.
describe("aslan-debug", () => {
  const jamFile = path.join(JAM_DIR, "aslan-debug.jam");

  test("compilation produces a structurally valid JAM file", () => {
    const result = verifyJamStructure(jamFile);
    expect(result.codeLength).toBeGreaterThan(0);
    expect(result.blobCodeLength).toBeGreaterThan(0);
    expect(result.instrCount).toBeGreaterThan(0);
  });
});
