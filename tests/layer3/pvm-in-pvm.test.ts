import fs from "node:fs";
import path from "node:path";
import { execSync } from "node:child_process";
import { describe, test, expect } from "bun:test";
import { JAM_DIR, PROJECT_ROOT, ANAN_AS_CLI } from "../helpers/paths";

/**
 * PVM-in-PVM tests (Issue #23).
 *
 * Runs inner JAM programs through the anan-as PVM interpreter that is itself
 * compiled to PVM bytecode. The pipeline is:
 *   inner.wat → inner.jam  (compiled by wasm-pvm)
 *   compiler.wasm → compiler.jam  (anan-as interpreter compiled to PVM)
 *   compiler.jam receives: gas + pc + program_len + inner_args_len + program + inner_args
 *   compiler.jam returns:  status(1) + exitCode(4) + gas(8) + pc(4) + result(?)
 */

const COMPILER_JAM = path.join(JAM_DIR, "anan-as-compiler.jam");

/** Inner interpreter status codes from anan-as. */
const STATUS_HALT = 0;
const STATUS_PANIC = 1;

/** High gas budget for the outer interpreter (10 billion). */
const OUTER_GAS = "10000000000";

/** Gas budget for the inner program. */
const INNER_GAS = BigInt(100_000_000);

interface InnerResult {
  status: number;
  exitCode: number;
  gasLeft: bigint;
  pc: number;
  resultBytes: Buffer;
}

/**
 * Strip the metadata prefix from a JAM/SPI buffer.
 * Format: varint(metadata_len) + metadata_bytes + raw_spi
 * Returns the raw SPI portion (after metadata).
 */
function stripMetadata(buf: Buffer): Buffer {
  let offset = 0;
  // Decode varint: if first byte < 128, it's a 1-byte length
  const firstByte = buf[offset];
  let metadataLen = 0;
  if (firstByte < 128) {
    metadataLen = firstByte;
    offset = 1;
  } else {
    // Multi-byte varint (same encoding as PVM blob varint)
    // For typical metadata sizes this is sufficient
    const leadingOnes = Math.clz32(~(firstByte << 24));
    const extraBytes = leadingOnes;
    const mask = (1 << (8 - leadingOnes)) - 1;
    metadataLen = firstByte & mask;
    for (let i = 1; i <= extraBytes; i++) {
      metadataLen = metadataLen * 256 + buf[offset + i];
    }
    offset = 1 + extraBytes;
  }
  return buf.subarray(offset + metadataLen);
}

/**
 * Build the args buffer for the anan-as compiler.jam.
 * Format: gas(8LE) + pc(4LE) + program_len(4LE) + inner_args_len(4LE) + program + inner_args
 */
function buildCompilerArgs(
  innerJamPath: string,
  innerArgsHex: string = "",
  gas: bigint = INNER_GAS,
  pc: number = 0,
): string {
  // Strip metadata prefix: inner programs are passed as raw SPI to the anan-as
  // interpreter running inside PVM, which expects raw SPI format.
  const programBytes = stripMetadata(fs.readFileSync(innerJamPath));
  const innerArgs = innerArgsHex
    ? Buffer.from(innerArgsHex, "hex")
    : Buffer.alloc(0);

  const header = Buffer.alloc(20);
  let offset = 0;
  header.writeBigUInt64LE(gas, offset);
  offset += 8;
  header.writeUInt32LE(pc, offset);
  offset += 4;
  header.writeUInt32LE(programBytes.length, offset);
  offset += 4;
  header.writeUInt32LE(innerArgs.length, offset);

  return Buffer.concat([header, programBytes, innerArgs]).toString("hex");
}

/**
 * Run the compiler.jam with the given args through the outer anan-as CLI.
 * Returns the parsed inner result.
 */
function runCompilerJam(argsHex: string): InnerResult {
  const cmd = `node ${ANAN_AS_CLI} run --spi --no-logs --gas=${OUTER_GAS} ${COMPILER_JAM} 0x${argsHex}`;

  let stdout: string;
  try {
    stdout = execSync(cmd, {
      cwd: PROJECT_ROOT,
      encoding: "utf8",
      stdio: ["pipe", "pipe", "pipe"],
      maxBuffer: 10 * 1024 * 1024,
    });
  } catch (error: any) {
    const errStdout = error.stdout?.toString() ?? "";
    const errStderr = error.stderr?.toString() ?? "";
    // Try to parse even from non-zero exit
    if (errStdout) {
      stdout = errStdout;
    } else {
      throw new Error(
        `Outer execution failed: ${error.message.split("\n")[0]}\nstderr: ${errStderr.substring(0, 500)}`,
      );
    }
  }

  // Check if the outer interpreter itself panicked
  const statusMatch = stdout.match(/Status:\s*(\d+)/);
  const outerStatus = statusMatch ? parseInt(statusMatch[1], 10) : -1;
  if (outerStatus !== 0) {
    const pcMatch = stdout.match(/Program counter:\s*(\d+)/);
    const outerPc = pcMatch ? pcMatch[1] : "unknown";
    throw new Error(
      `Outer interpreter panicked (status=${outerStatus}, pc=${outerPc}). ` +
      `Full output: ${stdout.substring(0, 800)}`,
    );
  }

  // Parse the Result: [0x...] from the output (may be empty: [0x])
  const resultMatch = stdout.match(/Result:\s*\[0x([0-9a-fA-F]*)\]/);
  if (!resultMatch) {
    throw new Error(`Could not parse result from output: ${stdout.substring(0, 500)}`);
  }

  const resultHex = resultMatch[1];
  const resultBuffer = resultHex.length > 0
    ? Buffer.from(resultHex, "hex")
    : Buffer.alloc(0);

  // Minimum: status(1) + exitCode(4) + gas(8) + pc(4) = 17 bytes
  if (resultBuffer.length < 17) {
    throw new Error(
      `Result too short (${resultBuffer.length} bytes, need >= 17): 0x${resultHex}`,
    );
  }

  return {
    status: resultBuffer.readUInt8(0),
    exitCode: resultBuffer.readUInt32LE(1),
    gasLeft: resultBuffer.readBigUInt64LE(5),
    pc: resultBuffer.readUInt32LE(13),
    resultBytes: resultBuffer.subarray(17),
  };
}

// PVM-in-PVM tests are slow (~85s each) because the outer anan-as interpreter
// must execute ~525M PVM instructions to run the inner program through the
// PVM-compiled interpreter. These tests are skipped by default in CI.
const PVM_IN_PVM_TIMEOUT = 180_000;

describe("pvm-in-pvm (issue #23)", () => {
  test("trap.jam → inner program panics", () => {
    const trapJam = path.join(JAM_DIR, "trap.jam");
    const argsHex = buildCompilerArgs(trapJam);
    const result = runCompilerJam(argsHex);

    expect(result.status).toBe(STATUS_PANIC);
  }, PVM_IN_PVM_TIMEOUT);

  test("add.jam with 5+7 → inner result is 12", () => {
    const addJam = path.join(JAM_DIR, "add.jam");
    // 5 and 7 as little-endian u32
    const argsHex = buildCompilerArgs(addJam, "0500000007000000");
    const result = runCompilerJam(argsHex);

    expect(result.status).toBe(STATUS_HALT);

    // Inner result should contain the sum as LE u32
    expect(result.resultBytes.length).toBeGreaterThanOrEqual(4);
    const sum = result.resultBytes.readUInt32LE(0);
    expect(sum).toBe(12);
  }, PVM_IN_PVM_TIMEOUT);
});
