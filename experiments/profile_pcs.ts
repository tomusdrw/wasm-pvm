#!/usr/bin/env bun
/**
 * Per-PC execution profiler: single-steps a JAM program and histograms
 * program-counter values. Gas is uniform per instruction, so the PC histogram
 * is exactly the per-instruction gas profile.
 *
 * Usage: bun experiments/profile_pcs.ts <file.jam> <args-hex> [maxSteps]
 * Output: lines of "<pc> <count>", plus a "# total_steps N" header.
 */
import { readFileSync } from "node:fs";
import * as path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ananAs = await import(path.join(__dirname, "../vendor/anan-as/build/release.js"));

const jamFile = process.argv[2];
const argsHex = process.argv[3] ?? "";
const maxSteps = Number(process.argv[4] ?? 50_000_000);

const args = argsHex
  ? Array.from(Buffer.from(argsHex.replace(/^0x/, ""), "hex"))
  : [];
const program = Array.from(readFileSync(jamFile));

ananAs.resetJAM(program, 0, 10_000_000_000n, args, true, false);

const counts = new Map<number, number>();
let steps = 0;
while (steps < maxSteps) {
  const pc = ananAs.getProgramCounter();
  counts.set(pc, (counts.get(pc) ?? 0) + 1);
  steps++;
  if (!ananAs.nextStep()) break;
}

const status = ananAs.getStatus();
console.log(`# total_steps ${steps}`);
console.log(`# status ${status}`);
const sorted = [...counts.entries()].sort((a, b) => a[0] - b[0]);
for (const [pc, c] of sorted) console.log(`${pc} ${c}`);
