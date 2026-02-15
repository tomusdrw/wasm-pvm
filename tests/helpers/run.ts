import { execSync } from "node:child_process";
import { ANAN_AS_CLI, PROJECT_ROOT } from "./paths";

export interface RunResult {
  exitValue: number;
  exitCode: number;
  stdout: string;
  stderr: string;
}

function buildRunCmd(jamFile: string, args: string, pc?: number, logs = false): string {
  let cmd = `node ${ANAN_AS_CLI} run --spi --no-metadata`;

  if (!logs) {
    cmd += ` --no-logs`;
  }

  if (pc !== undefined) {
    cmd += ` --pc=${pc}`;
  }

  cmd += ` --gas=100000000`;
  cmd += ` ${jamFile} 0x${args}`;
  return cmd;
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

export function runJam(jamFile: string, args: string, pc?: number): number {
  const cmd = buildRunCmd(jamFile, args, pc);

  try {
    const output = execSync(cmd, {
      cwd: PROJECT_ROOT,
      encoding: "utf8",
      stdio: ["pipe", "pipe", "pipe"],
    });

    return parseExitValue(output);
  } catch (error: any) {
    if (error.stdout) console.log(error.stdout.toString());
    if (error.stderr) console.error(error.stderr.toString());
    throw new Error(`Execution failed: ${error.message.split("\n")[0]}`, {
      cause: error,
    });
  }
}

export function runJamWithOutput(jamFile: string, args: string, pc?: number): RunResult {
  const cmd = buildRunCmd(jamFile, args, pc, true);

  try {
    const stdout = execSync(cmd, {
      cwd: PROJECT_ROOT,
      encoding: "utf8",
      stdio: ["pipe", "pipe", "pipe"],
    });

    return {
      exitValue: parseExitValue(stdout),
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
      throw new Error(`Execution failed: ${error.message.split("\n")[0]}`, {
        cause: error,
      });
    }
  }
}
