import fs from "node:fs";
import path from "node:path";
import { describe, test, expect } from "bun:test";
import { JAM_DIR } from "../helpers/paths";
import { runJam } from "../helpers/run";

// Regression test for issue #22: inner interpreter argument parsing.
// Dynamically constructs the inner-interpreter args format with add.jam
// embedded, and verifies as-mini-pvm-runner correctly parses the structure.

describe("inner-interpreter-args (issue #22)", () => {
  const runnerJam = path.join(JAM_DIR, "as-mini-pvm-runner.jam");
  const addJam = path.join(JAM_DIR, "add.jam");

  test("parses embedded JAM program args correctly", () => {
    const programBytes = fs.readFileSync(addJam);

    // Inner args for add.jam: 5 + 7
    const innerArgs = Buffer.from([0x05, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00]);

    const gas = BigInt(100_000_000);
    const pc = 0;

    // Format: gas(8) + pc(4) + program_len(4) + inner_args_len(4) + program + inner_args
    const header = Buffer.alloc(20);
    let offset = 0;
    header.writeBigUInt64LE(gas, offset); offset += 8;
    header.writeUInt32LE(pc, offset); offset += 4;
    header.writeUInt32LE(programBytes.length, offset); offset += 4;
    header.writeUInt32LE(innerArgs.length, offset);

    const argsHex = Buffer.concat([header, programBytes, innerArgs]).toString("hex");

    // mini-pvm-runner returns diagnostics; first u32 is the 0x11111111 marker
    const result = runJam(runnerJam, argsHex);
    expect(result).toBe(0x11111111);
  });
});
