#!/usr/bin/env bun
import { runJam } from "../helpers/run";

const jamFile = process.argv[2];
const argsArg = process.argv.find(a => a.startsWith("--args="));
const args = argsArg ? argsArg.split("=")[1] : (process.argv[3] || "");

if (!jamFile) {
  console.error("Usage: bun utils/run-jam.ts <file.jam> --args=<hex>");
  process.exit(1);
}

try {
  console.log(`Running ${jamFile} with args ${args || "(none)"}...`);
  const result = runJam(jamFile, args);
  console.log(`Result: ${result} (0x${result.toString(16)})`);
} catch (e: any) {
  console.error(e.message);
  process.exit(1);
}
