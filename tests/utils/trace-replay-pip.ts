#!/usr/bin/env bun
/**
 * PVM-in-PVM Trace Replay
 *
 * Replays a trace file through the PVM-in-PVM pipeline:
 *   1. Parse trace → extract inner program, start state, ecalli entries
 *   2. Build compiler args (gas, pc, inner program, inner args)
 *   3. Run anan-as-compiler-replay.jam on the outer PVM
 *   4. Handle outer ecalli 0 (forward) and ecalli 1 (get r8) using trace data
 *   5. Compare final result with trace's expected termination
 *
 * Usage:
 *   bun tests/utils/trace-replay-pip.ts <trace-file> [--no-verify] [--logs]
 */

import { readFileSync } from "node:fs";
import * as path from "node:path";
import {
  HasMetadata,
  InputKind,
  prepareProgram,
  pvmDestroy,
  pvmReadMemory,
  pvmResume,
  pvmSetRegisters,
  pvmStart,
  pvmWriteMemory,
} from "../../vendor/anan-as/build/release.js";
import {
  type EcalliEntry,
  extractSpiArgs,
  isSpiTrace,
  parseTrace,
  STATUS,
  statusToTermination,
} from "../../vendor/anan-as/bin/src/trace-parse.js";

const PROJECT_ROOT = path.resolve(import.meta.dir, "../..");
const COMPILER_REPLAY_JAM = path.join(PROJECT_ROOT, "tests/build/jam/anan-as-compiler-replay.jam");

// Outer PVM gas: generous budget for the compiler
const OUTER_GAS = 100_000_000_000n;

// Inner status codes (must match assembly/interpreter.ts Status enum)
const INNER_STATUS = {
  HALT: 0,
  PANIC: 1,
  FAULT: 2,
  HOST: 3,
  OOG: 4,
} as const;

function main() {
  const args = process.argv.slice(2);
  const flags = new Set(args.filter((a) => a.startsWith("--")));
  const positionals = args.filter((a) => !a.startsWith("--"));

  if (positionals.length === 0) {
    console.error("Usage: bun trace-replay-pip.ts <trace-file> [--no-verify] [--logs]");
    process.exit(1);
  }

  const traceFile = positionals[0];
  const verify = !flags.has("--no-verify");
  const logs = flags.has("--logs");

  // 1. Parse trace
  const traceInput = readFileSync(traceFile, "utf8");
  const trace = parseTrace(traceInput);
  const { program: innerProgram, initialMemWrites, start, ecalliEntries, termination } = trace;

  const useSpi = isSpiTrace(start, initialMemWrites);
  if (!useSpi) {
    console.error("Error: only SPI traces are supported for PVM-in-PVM replay.");
    process.exit(1);
  }

  const innerArgs = extractSpiArgs(start, initialMemWrites);
  console.log(
    `Trace: ${ecalliEntries.length} ecalli, termination=${termination.type}, ` +
      `program=${innerProgram.length}B, args=${innerArgs.length}B`,
  );

  // 2. Build compiler args: [8:gas][4:pc][4:prog_len][4:args_len][prog][args]
  const innerGas = start.gas;
  const innerPc = start.pc;
  const headerSize = 20;
  const totalSize = headerSize + innerProgram.length + innerArgs.length;
  const compilerArgs = new Uint8Array(totalSize);
  const view = new DataView(compilerArgs.buffer);

  view.setBigUint64(0, innerGas, true);
  view.setUint32(8, innerPc, true);
  view.setUint32(12, innerProgram.length, true);
  view.setUint32(16, innerArgs.length, true);
  compilerArgs.set(innerProgram, headerSize);
  compilerArgs.set(innerArgs, headerSize + innerProgram.length);

  // 3. Load and prepare outer program (compiler-replay.jam)
  const compilerJam = readFileSync(COMPILER_REPLAY_JAM);
  const outerProgram = prepareProgram(
    InputKind.SPI,
    HasMetadata.Yes,
    Array.from(compilerJam),
    [],
    [],
    [],
    Array.from(compilerArgs),
    128, // preallocateMemoryPages
    true, // useBlockGas
  );

  const outerId = pvmStart(outerProgram);
  let outerGas = OUTER_GAS;
  let outerPc = 0;

  // Track ecalli replay state
  const pendingEcalli = [...ecalliEntries];
  let lastR8 = 0n;
  let ecalliCount = 0;

  try {
    // 4. Run outer PVM with ecalli handling
    for (;;) {
      const pause = pvmResume(outerId, outerGas, outerPc, logs);
      if (!pause) {
        throw new Error("pvmResume returned null");
      }

      if (pause.status !== STATUS.HOST) {
        // Outer PVM terminated
        break;
      }

      const outerEcalli = pause.exitCode;

      if (outerEcalli === 0) {
        // Ecalli 0: forward inner ecalli
        const scratchPvmAddr = Number(pause.registers[7] & 0xFFFFFFFFn);
        const innerEcalliIdx = Number(pause.registers[8] & 0xFFFFFFFFn);

        const entry = pendingEcalli.shift();
        if (!entry) {
          throw new Error(`Unexpected inner ecalli ${innerEcalliIdx} (no more trace entries)`);
        }

        if (verify && entry.index !== innerEcalliIdx) {
          throw new Error(
            `Ecalli index mismatch: trace expects ${entry.index}, inner program sent ${innerEcalliIdx}`,
          );
        }

        ecalliCount++;

        // Build scratch buffer response
        const response = buildScratchResponse(entry);
        lastR8 = getNewR8(entry);

        // Write response to outer PVM memory
        const written = pvmWriteMemory(outerId, scratchPvmAddr, response);
        if (!written) {
          throw new Error(`Failed to write scratch buffer at PVM addr 0x${scratchPvmAddr.toString(16)}`);
        }

        // Resume outer PVM (r7 return value doesn't matter, adapter reads from scratch)
        const regs = pause.registers;
        pvmSetRegisters(outerId, regs);
        outerGas = pause.gas;
        outerPc = pause.nextPc;
      } else if (outerEcalli === 1) {
        // Ecalli 1: get last r8
        const regs = pause.registers;
        regs[7] = lastR8;
        pvmSetRegisters(outerId, regs);
        outerGas = pause.gas;
        outerPc = pause.nextPc;
      } else {
        console.error(`Unknown outer ecalli ${outerEcalli}, stopping.`);
        break;
      }
    }

    // 5. Parse inner result from outer PVM output
    const outerResult = pvmDestroy(outerId);
    if (!outerResult || outerResult.status !== INNER_STATUS.HALT) {
      const status = outerResult?.status ?? -1;
      console.error(`Outer PVM did not HALT (status=${status}). PVM-in-PVM execution failed.`);
      process.exit(1);
    }

    // Outer result is the inner program's packed output
    const resultBytes = new Uint8Array(outerResult.result ?? []);
    if (resultBytes.length < 5) {
      console.error(`Outer result too short (${resultBytes.length} bytes).`);
      process.exit(1);
    }

    const innerStatus = resultBytes[0];
    const resultView = new DataView(resultBytes.buffer);
    const innerExitCode = resultView.getUint32(1, true);

    const innerType = statusToTermination(innerStatus);

    // HALT requires full 17-byte header; reject truncated payloads
    if (innerStatus === INNER_STATUS.HALT && resultBytes.length < 17) {
      console.error(`HALT result too short (${resultBytes.length} bytes, need >= 17).`);
      process.exit(1);
    }

    let innerGasLeft = 0n;
    let innerPcFinal = 0;
    if (innerStatus === INNER_STATUS.HALT && resultBytes.length >= 17) {
      innerGasLeft = resultView.getBigUint64(5, true);
      innerPcFinal = resultView.getUint32(13, true);
    }

    console.log(`\nPVM-in-PVM replay complete: ${ecalliCount} ecalli handled`);
    console.log(`Inner status: ${innerType} (exit code: ${innerExitCode})`);
    console.log(`Inner PC: ${innerPcFinal}, Gas remaining: ${innerGasLeft}`);

    // 6. Verify against trace
    if (verify) {
      let ok = true;

      if (innerType !== termination.type) {
        console.error(`MISMATCH: termination type: expected ${termination.type}, got ${innerType}`);
        ok = false;
      }

      if (termination.panicArg !== undefined && innerExitCode !== termination.panicArg) {
        console.error(`MISMATCH: panic arg: expected ${termination.panicArg}, got ${innerExitCode}`);
        ok = false;
      }

      if (pendingEcalli.length > 0) {
        console.error(`MISMATCH: ${pendingEcalli.length} ecalli entries were not consumed`);
        ok = false;
      }

      // Note: gas and PC may differ slightly in PVM-in-PVM due to overhead,
      // so we only check termination type and exit code by default.

      if (ok) {
        console.log("Verification: PASSED");
      } else {
        console.log("Verification: FAILED");
        process.exit(1);
      }
    }
  } catch (error) {
    pvmDestroy(outerId);
    throw error;
  }
}

/**
 * Build the scratch buffer response for an ecalli entry.
 *
 * Format: [8:new_r7][8:new_r8][4:num_memwrites][8:new_gas][per memwrite: 4:addr, 4:len, data]
 *
 * new_gas: 0 = no change, nonzero = set inner gas to this value.
 * Currently the adapter ignores new_gas (TODO #174).
 */
function buildScratchResponse(entry: EcalliEntry): Uint8Array {
  const newR7 = getNewR7(entry);
  const newR8 = getNewR8(entry);
  const newGas = getSetGas(entry);

  // Calculate total size
  let memwriteDataSize = 0;
  for (const mw of entry.memWrites) {
    memwriteDataSize += 8 + mw.data.length; // 4:addr + 4:len + data
  }
  const totalSize = 8 + 8 + 4 + 8 + memwriteDataSize;

  // Scratch buffer is a single WASM page (64KB). Reject oversize responses.
  const SCRATCH_PAGE_SIZE = 65536;
  if (totalSize > SCRATCH_PAGE_SIZE) {
    throw new Error(
      `Scratch response too large (${totalSize} bytes, max ${SCRATCH_PAGE_SIZE}). ` +
        `Trace entry has ${entry.memWrites.length} memwrites totaling ${memwriteDataSize} bytes.`,
    );
  }

  const buf = new Uint8Array(totalSize);
  const view = new DataView(buf.buffer);

  // new_r7 (8 bytes LE)
  view.setBigUint64(0, newR7, true);
  // new_r8 (8 bytes LE)
  view.setBigUint64(8, newR8, true);
  // num_memwrites (4 bytes LE)
  view.setUint32(16, entry.memWrites.length, true);
  // new_gas (8 bytes LE, 0 = no change)
  view.setBigUint64(20, newGas, true);

  // Memwrite entries
  let offset = 28;
  for (const mw of entry.memWrites) {
    view.setUint32(offset, mw.address, true);
    view.setUint32(offset + 4, mw.data.length, true);
    buf.set(mw.data, offset + 8);
    offset += 8 + mw.data.length;
  }

  return buf;
}

/** Extract new r7 value from trace entry's setRegs */
function getNewR7(entry: EcalliEntry): bigint {
  for (const sr of entry.setRegs) {
    if (sr.index === 7) return sr.value;
  }
  return 0n;
}

/** Extract new r8 value from trace entry's setRegs */
function getNewR8(entry: EcalliEntry): bigint {
  for (const sr of entry.setRegs) {
    if (sr.index === 8) return sr.value;
  }
  return 0n;
}

/** Extract setGas value (0 = no change) */
function getSetGas(entry: EcalliEntry): bigint {
  return entry.setGas ?? 0n;
}

main();
