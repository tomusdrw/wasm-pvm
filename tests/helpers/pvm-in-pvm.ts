import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import crypto from "node:crypto";
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
export function buildCompilerArgs(
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
export class PvmInPvmTimeout extends Error {
  constructor(ms: number) {
    super(`PVM-in-PVM execution timed out after ${ms}ms`);
    this.name = "PvmInPvmTimeout";
  }
}

export function runCompilerJam(argsHex: string, timeoutMs?: number): InnerResult {
  // Write args to a temp binary file to avoid E2BIG (arg list too long) on Linux
  // when the inner JAM program is large. The anan-as CLI accepts a file path as args.
  const argsBuf = Buffer.from(argsHex, "hex");
  const debug = process.env.PVM_IN_PVM_DEBUG === "1";
  const keepArgs = process.env.PVM_IN_PVM_KEEP_ARGS === "1";
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "pvm-in-pvm-"));
  const tmpFile = path.join(
    tmpDir,
    `args-${process.pid}-${Date.now()}-${crypto.randomUUID()}.bin`,
  );
  fs.writeFileSync(tmpFile, argsBuf);
  if (debug) {
    const sha = crypto.createHash("sha256").update(argsBuf).digest("hex");
    console.log(
      `[pvm-in-pvm] args tmp=${tmpFile} bytes=${argsBuf.length} sha256=${sha}`,
    );
  }

  const cmd = `node ${ANAN_AS_CLI} run --spi --no-logs --gas=${OUTER_GAS} ${COMPILER_JAM} ${tmpFile}`;

  let stdout: string;
  try {
    stdout = execSync(cmd, {
      cwd: PROJECT_ROOT,
      encoding: "utf8",
      stdio: ["pipe", "pipe", "pipe"],
      maxBuffer: 10 * 1024 * 1024,
      timeout: timeoutMs,
    });
  } catch (error: any) {
    if (!keepArgs) {
      fs.rmSync(tmpDir, { recursive: true, force: true });
    }
    if (error.killed || error.signal === "SIGTERM") {
      throw new PvmInPvmTimeout(timeoutMs ?? 0);
    }
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
  if (!keepArgs) {
    fs.rmSync(tmpDir, { recursive: true, force: true });
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
    // Include full outer interpreter output for debugging CI failures.
    const gasMatch = stdout.match(/Gas remaining:\s*(\d+)/);
    const pcMatch = stdout.match(/Program counter:\s*(\d+)/);
    const regsMatch = stdout.match(/Registers:\s*\[([^\]]*)\]/);
    if (debug) {
      const jamPath = COMPILER_JAM;
      const jamBytes = fs.readFileSync(jamPath);
      const jamSha = crypto.createHash("sha256").update(jamBytes).digest("hex");
      console.log(
        `[pvm-in-pvm] compiler jam=${jamPath} bytes=${jamBytes.length} sha256=${jamSha}`,
      );
      if (keepArgs) {
        console.log(
          `[pvm-in-pvm] kept args file: ${tmpFile} (dir ${tmpDir})`,
        );
      }
    }
    throw new Error(
      `Result too short (${resultBuffer.length} bytes, need >= 17): 0x${resultHex}\n` +
        `Outer status=${outerStatus}, gas=${gasMatch?.[1] ?? "?"}, pc=${pcMatch?.[1] ?? "?"}\n` +
        `Registers: ${regsMatch?.[1] ?? "?"}\n` +
        `Full output (first 1000 chars): ${stdout.substring(0, 1000)}`,
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
  timeoutMs?: number,
): number {
  const argsHex = buildCompilerArgs(
    jamFile,
    args,
    INNER_GAS,
    pc ?? 0,
  );
  const result = runCompilerJam(argsHex, timeoutMs);

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
