import path from "node:path";
import { describe, test, expect } from "bun:test";
import { JAM_DIR } from "../helpers/paths";
import { buildCompilerArgs, runCompilerJam, runJamPvmInPvm } from "../helpers/pvm-in-pvm";

/**
 * PVM-in-PVM smoke tests (layer 4).
 *
 * A small set of hand-picked tests to verify the pvm-in-pvm pipeline works.
 * For comprehensive pvm-in-pvm coverage see layer5.
 */

const PVM_IN_PVM_TIMEOUT = 180_000;

describe("pvm-in-pvm smoke tests", () => {
  test("trap.jam -> inner program panics", () => {
    const trapJam = path.join(JAM_DIR, "trap.jam");
    const argsHex = buildCompilerArgs(trapJam);
    const result = runCompilerJam(argsHex);
    expect(result.status).toBe(1); // PANIC
  }, PVM_IN_PVM_TIMEOUT);

  test("add.jam with 5+7 -> inner result is 12", () => {
    const result = runJamPvmInPvm(
      path.join(JAM_DIR, "add.jam"),
      "0500000007000000",
    );
    expect(result).toBe(12);
  }, PVM_IN_PVM_TIMEOUT);

  test("as-add.jam with 3+4 -> inner result is 7", () => {
    const result = runJamPvmInPvm(
      path.join(JAM_DIR, "as-add.jam"),
      "0300000004000000",
    );
    expect(result).toBe(7);
  }, PVM_IN_PVM_TIMEOUT);
});
