import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {
  AS_ASSEMBLY_DIR,
  CLI_BINARY,
  PROJECT_ROOT,
  TESTS_DIR,
  WASM_DIR,
} from "./paths";

export function isStale(source: string, target: string): boolean {
  if (!fs.existsSync(target)) return true;
  const srcStat = fs.statSync(source);
  const tgtStat = fs.statSync(target);
  return srcStat.mtimeMs > tgtStat.mtimeMs;
}

export function compileAS(
  sourceName: string,
  outputName: string,
  runtime: string = "stub"
): string {
  const sourceFile = path.join(AS_ASSEMBLY_DIR, sourceName);
  const wasmFile = path.join(WASM_DIR, `${outputName}.wasm`);

  if (!isStale(sourceFile, wasmFile)) {
    return wasmFile;
  }

  const ascBin = path.join(TESTS_DIR, "node_modules/.bin/asc");
  execFileSync(ascBin, [
    sourceFile,
    "-o",
    wasmFile,
    "--runtime",
    runtime,
    "--noAssert",
    "--optimizeLevel",
    "3",
    "--shrinkLevel",
    "2",
    "--converge",
    "--use",
    "abort=",
  ], {
    cwd: TESTS_DIR,
    encoding: "utf8",
    stdio: ["pipe", "pipe", "pipe"],
  });
  return wasmFile;
}

export function compileToJAM(inputPath: string, outputName: string, importsPath?: string, adapterPath?: string): string {
  const jamFile = path.join(
    TESTS_DIR,
    "build",
    "jam",
    `${outputName}.jam`
  );

  const args = [
    "compile",
    inputPath,
    "-o",
    jamFile,
  ];

  if (adapterPath) {
    args.push("--adapter", adapterPath);
  }
  if (importsPath) {
    args.push("--imports", importsPath);
  }

  execFileSync(CLI_BINARY, args, {
    cwd: PROJECT_ROOT,
    encoding: "utf8",
    stdio: ["pipe", "pipe", "pipe"],
  });
  return jamFile;
}
