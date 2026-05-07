/**
 * End-to-end test for `--trap-floats` mode and the location-aware diagnostic.
 *
 * Drives the `wasm-pvm` CLI directly (instead of going through `defineSuite`)
 * because:
 * 1. The default build of the float fixture is *expected* to fail; auto-
 *    compilation by `build.ts` would block the whole test suite.
 * 2. We need to assert on the CLI's stderr to verify the new diagnostic
 *    surface (function name + byte offset).
 *
 * Fixture lives in `tests/fixtures/wat-trap-floats/` (outside the auto-built
 * `tests/fixtures/wat/` tree).
 */

import { describe, test, expect, beforeAll } from "bun:test";
import { execFileSync, execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {
  ANAN_AS_CLI,
  CLI_BINARY,
  FIXTURES_DIR,
  JAM_DIR,
  PROJECT_ROOT,
} from "../helpers/paths";

const WAT_PATH = path.join(FIXTURES_DIR, "wat-trap-floats/float-trap.wat");
const JAM_PATH = path.join(JAM_DIR, "float-trap.jam");

// Ensure the JAM dir exists; build.ts normally creates it but we run before
// the orchestrator may have touched the trap-floats subtree.
beforeAll(() => {
  fs.mkdirSync(JAM_DIR, { recursive: true });
  // Always start clean so a stale JAM from a prior run can't make a failing
  // compile look like a success.
  if (fs.existsSync(JAM_PATH)) fs.unlinkSync(JAM_PATH);
});

describe("trap-floats CLI behavior", () => {
  test("default mode rejects f64.add with location diagnostic", () => {
    let exitCode = 0;
    let stderr = "";
    try {
      execFileSync(CLI_BINARY, ["compile", WAT_PATH, "-o", JAM_PATH], {
        cwd: PROJECT_ROOT,
        encoding: "utf8",
        stdio: ["pipe", "pipe", "pipe"],
      });
    } catch (err: any) {
      exitCode = err.status ?? 1;
      stderr = (err.stderr ?? "").toString();
    }

    expect(exitCode).not.toBe(0);

    // The new diagnostic must mention the function name and a byte offset.
    expect(stderr).toContain("'main'");
    expect(stderr).toMatch(/byte offset 0x[0-9a-f]+/);

    // And it should still classify the failure as an unsupported feature.
    // anyhow's `Caused by:` chain shows both the wrapper and the inner error.
    expect(stderr).toContain("Unsupported");

    // No JAM file should exist after a failed compile.
    expect(fs.existsSync(JAM_PATH)).toBe(false);
  });

  test("--trap-floats compiles and writes a JAM", () => {
    execFileSync(
      CLI_BINARY,
      ["compile", WAT_PATH, "-o", JAM_PATH, "--trap-floats"],
      {
        cwd: PROJECT_ROOT,
        encoding: "utf8",
        stdio: ["pipe", "pipe", "pipe"],
      },
    );
    expect(fs.existsSync(JAM_PATH)).toBe(true);
    expect(fs.statSync(JAM_PATH).size).toBeGreaterThan(0);
  });

  test("compiled JAM runs cleanly when the float branch is skipped", () => {
    // First byte of args = 0 → safe path → no float op executed.
    const cmd = `node ${ANAN_AS_CLI} run --spi --no-logs --gas=100000000 ${JAM_PATH} 0x00`;
    const stdout = execSync(cmd, {
      cwd: PROJECT_ROOT,
      encoding: "utf8",
      stdio: ["pipe", "pipe", "pipe"],
    });
    // anan-as prints `Status: 0` on clean halt, `Status: <non-zero>` on trap
    // or panic. The safe path must show clean status and a non-empty Result.
    expect(stdout).toMatch(/Status:\s*0\b/);
    expect(stdout).toMatch(/Result:\s*\[0x[0-9a-fA-F]+\]/);
  });

  test("compiled JAM traps when the float branch is taken", () => {
    // First byte of args = 1 → trap path → @llvm.trap → PVM Trap instruction.
    const cmd = `node ${ANAN_AS_CLI} run --spi --no-logs --gas=100000000 ${JAM_PATH} 0x01`;
    const stdout = execSync(cmd, {
      cwd: PROJECT_ROOT,
      encoding: "utf8",
      stdio: ["pipe", "pipe", "pipe"],
    });
    // anan-as reports a non-zero PVM `Status:` on trap and emits an empty
    // `Result: [0x]` (no bytes returned because execution didn't reach the
    // ptr/len pack). Both signals together pin down the trap behavior.
    expect(stdout).toMatch(/Status:\s*[1-9]/);
    expect(stdout).toContain("Result: [0x]");
  });
});
