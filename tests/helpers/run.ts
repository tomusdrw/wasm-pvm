import { execSync } from "node:child_process";
import { ANAN_AS_CLI, PROJECT_ROOT } from "./paths";

export function runJam(jamFile: string, args: string, pc?: number): number {
  let cmd = `node ${ANAN_AS_CLI} run --spi --no-metadata --no-logs`;

  if (pc !== undefined) {
    cmd += ` --pc=${pc}`;
  }

  cmd += ` --gas=100000000`;
  cmd += ` ${jamFile} 0x${args}`;

  try {
    const output = execSync(cmd, {
      cwd: PROJECT_ROOT,
      encoding: "utf8",
      stdio: ["pipe", "pipe", "pipe"],
    });

    const resultMatch = output.match(/Result:\s*\[0x([0-9a-fA-F]+)\]/);

    if (resultMatch) {
      let hexResult = resultMatch[1];

      if (hexResult.length < 8) {
        hexResult = hexResult.padEnd(8, "0");
      } else if (hexResult.length > 8) {
        hexResult = hexResult.slice(0, 8);
      }

      const bytes = hexResult.match(/.{2}/g) || [];
      const value = parseInt(bytes.reverse().join(""), 16);
      return value;
    }

    throw new Error(`Could not parse result from output: ${output}`);
  } catch (error: any) {
    if (error.stdout) console.log(error.stdout.toString());
    if (error.stderr) console.error(error.stderr.toString());
    throw new Error(`Execution failed: ${error.message.split("\n")[0]}`, {
      cause: error,
    });
  }
}
