// Run anan-as compiler.wasm NATIVELY (Bun WebAssembly) with the pip protocol.
// main(args_ptr: i32, args_len: i32) -> i64 packed ptr|len — same unified ABI,
// but natively args land in linear memory; main reads them via pointers.
import { readFileSync } from "node:fs";
import * as path from "node:path";
// Resolve paths relative to this script so the repro runs on any checkout.
const HERE = import.meta.dir;
const REPO = path.resolve(HERE, "../..");
const wasmBytes = readFileSync(path.join(REPO, "vendor/anan-as/dist/build/compiler.wasm"));

function buildArgs(innerJam: string, innerArgsHex: string): Buffer {
  const programBytes = readFileSync(innerJam);
  const innerArgs = Buffer.from(innerArgsHex, "hex");
  const header = Buffer.alloc(20);
  header.writeBigUInt64LE(100_000_000n, 0);
  header.writeUInt32LE(0, 8);
  header.writeUInt32LE(programBytes.length, 12);
  header.writeUInt32LE(innerArgs.length, 16);
  return Buffer.concat([header, programBytes, innerArgs]);
}

for (const f of [path.join(HERE, "repro-17.jam"), path.join(HERE, "repro-18.jam")]) {
  const module = new WebAssembly.Module(wasmBytes);
  let memory: WebAssembly.Memory;
  const imports: any = {
    env: {
      abort: (msg: number, file: number, line: number, col: number) => { throw new Error(`AS abort at ${line}:${col}`); },
      "console.log": (_p: number) => {},
      host_call_6b: (_e: bigint, _r7: bigint, _r8: bigint, _r9: bigint, _r10: bigint, _r11: bigint, _r12: bigint): bigint => 0n,
      host_call_r8: (): bigint => 0n,
    },
  };
  const inst = new WebAssembly.Instance(module, imports);
  memory = inst.exports.memory as WebAssembly.Memory;
  const main = inst.exports.main as (p: number, l: number) => bigint;
  const args = buildArgs(f, "0500");
  // place args somewhere in memory — grow and copy at a high offset
  const need = args.length + 65536;
  const cur = memory.buffer.byteLength;
  if (cur < need + 1048576) memory.grow(Math.ceil((need + 1048576 - cur) / 65536));
  const argsPtr = memory.buffer.byteLength - args.length - 4096;
  new Uint8Array(memory.buffer).set(args, argsPtr);
  try {
    const packed = main(argsPtr, args.length);
    const ptr = Number(packed & 0xFFFFFFFFn);
    const len = Number(packed >> 32n);
    const out = new Uint8Array(memory.buffer).slice(ptr, ptr + Math.min(len, 64));
    console.log(f, "len=", len, "bytes=", Buffer.from(out).toString("hex"));
  } catch (e) {
    console.log(f, "ERROR:", String(e).slice(0, 120));
  }
}
