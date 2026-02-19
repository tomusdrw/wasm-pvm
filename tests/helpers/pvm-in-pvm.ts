import fs from "node:fs";
import path from "node:path";
import { execSync } from "node:child_process";
import { JAM_DIR, PROJECT_ROOT, ANAN_AS_CLI } from "./paths";

const COMPILER_JAM = path.join(JAM_DIR, "anan-as-compiler.jam");

/** High gas budget for the outer interpreter (10 billion). */
const OUTER_GAS = "10000000000";

/** Gas budget for the inner program. */
const INNER_GAS = BigInt(100_000_000);

export interface InnerResult {
  status: number;
  exitCode: number;
  gasLeft: bigint;
  pc: number;
  resultBytes: Buffer;
}

/**
 * Build the args buffer for the anan-as compiler.jam.
 * Format: gas(8LE) + pc(4LE) + program_len(4LE) + inner_args_len(4LE) + program + inner_args
 *
 * The inner program is passed as the full SPI blob (including metadata prefix).
 * The anan-as interpreter handles metadata stripping internally.
 */
function buildCompilerArgs(
  innerJamPath: string,
  innerArgsHex: string = "",
  gas: bigint = INNER_GAS,
  pc: number = 0,
): string {
  const programBytes = fs.readFileSync(innerJamPath);
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
export function runCompilerJam(argsHex: string): InnerResult {
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
    throw new Error(
      `Could not parse result from output: ${stdout.substring(0, 500)}`,
    );
  }

  const resultHex = resultMatch[1];
  const resultBuffer =
    resultHex.length > 0 ? Buffer.from(resultHex, "hex") : Buffer.alloc(0);

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

/**
 * Run a JAM file through PVM-in-PVM and return the result as a u32.
 * This mirrors the `runJam` interface for easy comparison.
 */
export function runJamPvmInPvm(
  jamFile: string,
  args: string,
  pc?: number,
): number {
  const argsHex = buildCompilerArgs(
    jamFile,
    args,
    INNER_GAS,
    pc ?? 0,
  );
  const result = runCompilerJam(argsHex);

  if (result.status !== 0) {
    throw new Error(
      `Inner program panicked (status=${result.status}, exitCode=${result.exitCode}, gasLeft=${result.gasLeft}, pc=${result.pc})`,
    );
  }

  if (result.resultBytes.length < 4) {
    throw new Error(
      `Inner result too short: ${result.resultBytes.length} bytes (need >= 4)`,
    );
  }

  // Parse result the same way runJam does: first 4 bytes as LE u32
  return result.resultBytes.readUInt32LE(0);
}
