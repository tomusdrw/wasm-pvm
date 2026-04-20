import { execSync } from "node:child_process";
import { ANAN_AS_CLI, PROJECT_ROOT } from "./paths";

export interface RunResult {
  exitValue: number;
  exitCode: number;
  stdout: string;
  stderr: string;
}

const DEFAULT_GAS = 100_000_000;

function buildRunCmd(
  jamFile: string,
  args: string,
  pc?: number,
  logs = false,
  gas: number = DEFAULT_GAS,
): string {
  let cmd = `node ${ANAN_AS_CLI} run --spi`;

  if (!logs) {
    cmd += ` --no-logs`;
  }

  if (pc !== undefined) {
    cmd += ` --pc=${pc}`;
  }

  cmd += ` --gas=${gas}`;
  cmd += ` ${jamFile} 0x${args}`;
  return cmd;
}

function execAnanAs(cmd: string): string {
  try {
    return execSync(cmd, {
      cwd: PROJECT_ROOT,
      encoding: "utf8",
      stdio: ["pipe", "pipe", "pipe"],
    });
  } catch (error: any) {
    if (error.stdout) console.log(error.stdout.toString());
    if (error.stderr) console.error(error.stderr.toString());
    throw new Error(`Execution failed: ${error.message.split("\n")[0]}`, {
      cause: error,
    });
  }
}

function parseExitValue(output: string): number {
  const resultMatch = output.match(/Result:\s*\[0x([0-9a-fA-F]+)\]/);

  if (resultMatch) {
    let hexResult = resultMatch[1];

    // Ensure even length so byte-splitting works correctly (e.g. "1" -> "01").
    if (hexResult.length % 2 !== 0) {
      hexResult = "0" + hexResult;
    }

    if (hexResult.length < 8) {
      hexResult = hexResult.padEnd(8, "0");
    } else if (hexResult.length > 8) {
      hexResult = hexResult.slice(0, 8);
    }

    const bytes = hexResult.match(/.{2}/g) || [];
    return parseInt(bytes.reverse().join(""), 16);
  }

  throw new Error(`Could not parse result from output: ${output}`);
}

/**
 * Parse the full raw result bytes from anan-as `Result: [0x...]` output.
 * Unlike `parseExitValue`, returns the complete byte string without truncation.
 */
function parseResultBytes(output: string): Uint8Array {
  const resultMatch = output.match(/Result:\s*\[0x([0-9a-fA-F]*)\]/);
  if (!resultMatch) {
    throw new Error(`Could not parse result from output: ${output}`);
  }
  let hex = resultMatch[1];
  if (hex.length % 2 !== 0) {
    hex = "0" + hex;
  }
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
  }
  return bytes;
}

/**
 * Run a JAM file and return the raw result bytes (no truncation).
 *
 * Unlike `runJam` which collapses to a u32, this preserves the full output.
 * Use for fixtures that return more than 4 bytes (e.g. hash functions).
 *
 * @param gas optional gas override. Defaults to 100_000_000 (matches `runJam`).
 */
export function runJamBytes(
  jamFile: string,
  args: string,
  pc?: number,
  gas: number = DEFAULT_GAS,
): Uint8Array {
  const cmd = buildRunCmd(jamFile, args, pc, false, gas);
  const output = execAnanAs(cmd);
  return parseResultBytes(output);
}

export function runJam(jamFile: string, args: string, pc?: number): number {
  const cmd = buildRunCmd(jamFile, args, pc);
  const output = execAnanAs(cmd);
  return parseExitValue(output);
}

export function runJamWithOutput(jamFile: string, args: string, pc?: number): RunResult {
  const cmd = buildRunCmd(jamFile, args, pc, true);

  try {
    const stdout = execSync(cmd, {
      cwd: PROJECT_ROOT,
      encoding: "utf8",
      stdio: ["pipe", "pipe", "pipe"],
    });

    let exitValue: number;
    try {
      exitValue = parseExitValue(stdout);
    } catch {
      exitValue = -1;
    }

    return {
      exitValue,
      exitCode: 0,
      stdout,
      stderr: "",
    };
  } catch (error: any) {
    const stdout = error.stdout?.toString() ?? "";
    const stderr = error.stderr?.toString() ?? "";

    // Try to parse result even from failed execution (non-zero exit).
    try {
      return {
        exitValue: parseExitValue(stdout),
        exitCode: error.status ?? 1,
        stdout,
        stderr,
      };
    } catch {
      // If we can't parse the result, return -1 as exitValue but still
      // provide stdout/stderr so callers can inspect the output.
      return {
        exitValue: -1,
        exitCode: error.status ?? 1,
        stdout,
        stderr,
      };
    }
  }
}
