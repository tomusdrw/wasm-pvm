import path from "node:path";
import fs from "node:fs";
import os from "node:os";
import { execFileSync } from "node:child_process";
import { describe, test, expect } from "bun:test";
import { JAM_DIR, PROJECT_ROOT } from "../helpers/paths";

/**
 * PVM-in-PVM trace replay tests (layer 4).
 *
 * Generates a trace from a JAM file via generate-trace.ts, then replays it
 * through PVM-in-PVM via trace-replay-pip.ts, verifying the termination matches.
 */

const GENERATE_TRACE = path.join(PROJECT_ROOT, "tests/utils/generate-trace.ts");
const TRACE_REPLAY_PIP = path.join(PROJECT_ROOT, "tests/utils/trace-replay-pip.ts");
const TIMEOUT = 300_000;

function traceReplay(jamName: string, argsHex?: string, gas?: number): void {
  const jamFile = path.join(JAM_DIR, `${jamName}.jam`);
  if (!fs.existsSync(jamFile)) {
    throw new Error(`JAM file not found: ${jamFile}`);
  }

  // Generate trace
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "trace-replay-"));
  const traceFile = path.join(tmpDir, `${jamName}.trace`);
  try {
    const genArgs = ["run", GENERATE_TRACE, jamFile];
    if (argsHex) genArgs.push(argsHex);
    if (gas !== undefined) genArgs.push("--gas", gas.toString());
    const traceOutput = execFileSync("bun", genArgs, {
      cwd: PROJECT_ROOT,
      encoding: "utf8",
      timeout: 60_000,
      maxBuffer: 10 * 1024 * 1024,
    });
    fs.writeFileSync(traceFile, traceOutput);

    // Replay through PVM-in-PVM
    const replayOutput = execFileSync("bun", ["run", TRACE_REPLAY_PIP, traceFile], {
      cwd: PROJECT_ROOT,
      encoding: "utf8",
      timeout: TIMEOUT,
      maxBuffer: 10 * 1024 * 1024,
    });

    // Verify replay passed
    expect(replayOutput).toContain("Verification: PASSED");
  } finally {
    fs.rmSync(tmpDir, { recursive: true, force: true });
  }
}

describe("pvm-in-pvm trace replay", () => {
  test("host-call-log.jam trace replay", () => {
    traceReplay("host-call-log", "48656c6c6f", 10_000_000);
  }, TIMEOUT);

  test("add.jam trace replay (5+7)", () => {
    traceReplay("add", "0500000007000000");
  }, TIMEOUT);

  test("as-add.jam trace replay (3+4)", () => {
    traceReplay("as-add", "0300000004000000");
  }, TIMEOUT);
});
