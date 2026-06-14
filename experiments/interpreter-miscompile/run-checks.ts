#!/usr/bin/env bun
/**
 * Self-contained 3-way checker for the anan-as interpreter miscompile (#261).
 *
 * Runs the triggering inner PVM program three ways and prints a verdict:
 *   A. through the wasm-pvm-compiled interpreter (PVM-in-PVM CLI path)  -> BUG: empty
 *   B. directly on the native anan-as engine (release.js)              -> correct
 *   C. through the interpreter SOURCE run natively (compiler.wasm)      -> correct
 *
 * A == wrong while B == C == correct proves wasm-pvm miscompiles the
 * interpreter (the inner program is valid; only its PVM-compiled interpreter
 * mis-runs it).
 *
 * Usage:
 *   bun run-checks.ts <interpreter.jam> <trigger.jam>
 *
 * The expected inner result and inner args are fixed for the committed
 * `trigger.jam` (a compiled `as-tests-globals`, args=00000000, result=17).
 */
import { execSync } from "node:child_process";
import { readFileSync, writeFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import * as path from "node:path";

const REPO = path.resolve(import.meta.dir, "../..");
const RELEASE_JS = path.join(REPO, "vendor/anan-as/build/release.js");
const COMPILER_WASM = path.join(REPO, "vendor/anan-as/dist/build/compiler.wasm");
const ANAN_CLI = path.join(REPO, "vendor/anan-as/dist/bin/index.js");

const interpJam = process.argv[2];
const triggerJam = process.argv[3];
if (!interpJam || !triggerJam) {
  console.error("usage: bun run-checks.ts <interpreter.jam> <trigger.jam>");
  process.exit(2);
}

// Fixed for the committed trigger.jam.
const INNER_ARGS = Buffer.from("00000000", "hex"); // as-tests-globals input
const EXPECTED_HEX = "11000000"; // 17 as LE u32

/** PVM-in-PVM args blob: gas(8) pc(4) programLen(4) innerArgsLen(4) program innerArgs. */
function buildArgs(program: Buffer): Buffer {
  const h = Buffer.alloc(20);
  h.writeBigUInt64LE(100_000_000n, 0);
  h.writeUInt32LE(0, 8);
  h.writeUInt32LE(program.length, 12);
  h.writeUInt32LE(INNER_ARGS.length, 16);
  return Buffer.concat([h, program, INNER_ARGS]);
}

// ── A. through the wasm-pvm-compiled interpreter (CLI; mirrors runCompilerJam) ──
function throughCompiledInterpreter(): { ok: boolean; detail: string } {
  const program = readFileSync(triggerJam);
  const dir = mkdtempSync(path.join(tmpdir(), "interp-miscompile-"));
  const argsFile = path.join(dir, "args.bin");
  writeFileSync(argsFile, buildArgs(program));
  try {
    const out = execSync(
      `node ${ANAN_CLI} run --spi --no-logs --gas=10000000000 ${interpJam} ${argsFile}`,
      { cwd: REPO, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 },
    );
    const m = out.match(/Result:\s*\[0x([0-9a-fA-F]*)\]/);
    const hex = m ? m[1] : "";
    // PVM-in-PVM HALT result is [status:1][exitCode:4][gas:8][pc:4][data...]; the
    // inner u32 result lives at byte offset 17.
    const innerHex = hex.length >= 42 ? hex.slice(34, 42) : "";
    const ok = innerHex.toLowerCase() === EXPECTED_HEX;
    return { ok, detail: hex.length ? `result=0x${hex}` : "result=0x (empty)" };
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
}

// ── B. directly on the native anan-as engine ──
async function nativeEngineDirect(): Promise<{ ok: boolean; detail: string }> {
  const anan = await import(RELEASE_JS);
  const bytes = Array.from(readFileSync(triggerJam));
  const prog = anan.prepareProgram(
    anan.InputKind.SPI,
    anan.HasMetadata.Yes,
    bytes,
    [],
    [],
    [],
    Array.from(INNER_ARGS),
    128,
    false,
  );
  const res = anan.runProgram(prog, 100_000_000n, 0, false);
  const hex = res.result ? Buffer.from(res.result).toString("hex") : "";
  return { ok: hex.toLowerCase() === EXPECTED_HEX, detail: `result=0x${hex}` };
}

// ── C. through the interpreter SOURCE run natively (compiler.wasm) ──
function interpreterSourceNative(): { ok: boolean; detail: string } {
  const wasmBytes = readFileSync(COMPILER_WASM);
  const mod = new WebAssembly.Module(wasmBytes);
  const imports: any = {
    env: {
      abort: (_m: number, _f: number, l: number, c: number) => {
        throw new Error(`AS abort ${l}:${c}`);
      },
      "console.log": () => {},
      host_call_6b: () => 0n,
      host_call_r8: () => 0n,
    },
  };
  const inst = new WebAssembly.Instance(mod, imports);
  const memory = inst.exports.memory as WebAssembly.Memory;
  const main = inst.exports.main as (p: number, l: number) => bigint;
  const args = buildArgs(readFileSync(triggerJam));
  const need = args.length + 2_000_000;
  if (memory.buffer.byteLength < need) {
    memory.grow(Math.ceil((need - memory.buffer.byteLength) / 65536));
  }
  const ptr = memory.buffer.byteLength - args.length - 8192;
  new Uint8Array(memory.buffer).set(args, ptr);
  const packed = main(ptr, args.length);
  const rp = Number(packed & 0xffffffffn);
  const rl = Number(packed >> 32n);
  const buf = Buffer.from(new Uint8Array(memory.buffer).slice(rp, rp + rl));
  const innerHex = rl >= 21 ? buf.subarray(17, 21).toString("hex") : "";
  return { ok: innerHex.toLowerCase() === EXPECTED_HEX, detail: `len=${rl} result=0x${buf.toString("hex")}` };
}

const a = throughCompiledInterpreter();
const b = await nativeEngineDirect();
const c = interpreterSourceNative();

console.log("A. wasm-pvm-compiled interpreter :", a.ok ? "OK" : "WRONG", "-", a.detail);
console.log("B. native anan-as engine (direct):", b.ok ? "OK" : "WRONG", "-", b.detail);
console.log("C. interpreter source (native)   :", c.ok ? "OK" : "WRONG", "-", c.detail);
console.log("");

if (!a.ok && b.ok && c.ok) {
  console.log("REPRODUCED (#261): the inner program is valid (B and C agree),");
  console.log("but the wasm-pvm-compiled interpreter mis-runs it (A wrong).");
  process.exit(1);
}
if (a.ok && b.ok && c.ok) {
  console.log("NOT reproduced: the compiled interpreter ran trigger.jam correctly.");
  console.log("The core-lowering miscompile (#261) appears fixed for this trigger.");
  process.exit(0);
}
console.log("INCONCLUSIVE: native references disagree — check the toolchain build.");
process.exit(3);
