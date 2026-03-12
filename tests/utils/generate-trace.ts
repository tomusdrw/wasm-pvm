#!/usr/bin/env bun
/**
 * Generate a trace file from running a JAM/SPI program.
 *
 * Usage:
 *   bun tests/utils/generate-trace.ts <jam-file> [args-hex] [--gas <n>] [--pc <n>]
 *
 * Output goes to stdout in the anan-as trace format.
 * Redirect to a file: bun tests/utils/generate-trace.ts prog.jam args > trace.log
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
} from "../../vendor/anan-as/build/release.js";
import { LOG_HOST_CALL_INDEX, printLogHostCall, WHAT } from "../../vendor/anan-as/bin/src/log-host-call.js";
import { ARGS_SEGMENT_START, STATUS } from "../../vendor/anan-as/bin/src/trace-parse.js";

function hexEncode(data: Uint8Array | number[]): string {
  return "0x" + Buffer.from(data).toString("hex");
}

function formatRegisters(regs: bigint[]): string {
  return regs
    .map((v, i) => (v !== 0n ? `r${i}=0x${v.toString(16)}` : null))
    .filter(Boolean)
    .join(" ");
}

function main() {
  const argv = process.argv.slice(2);
  const positionals: string[] = [];
  let gas = 10_000_000n;
  let pc = 0;

  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === "--gas" && i + 1 < argv.length) {
      gas = BigInt(argv[++i]);
    } else if (argv[i] === "--pc" && i + 1 < argv.length) {
      pc = Number(argv[++i]);
    } else if (!argv[i].startsWith("--")) {
      positionals.push(argv[i]);
    }
  }

  if (positionals.length === 0) {
    console.error("Usage: bun generate-trace.ts <jam-file> [args-hex] [--gas <n>] [--pc <n>]");
    process.exit(1);
  }

  const jamFile = positionals[0];
  const argsHex = positionals[1] ?? "";

  // Parse args
  const spiArgs = argsHex
    ? Array.from(Buffer.from(argsHex.replace(/^0x/, ""), "hex"))
    : [];

  // Load program
  const programCode = Array.from(readFileSync(jamFile));

  // Print program line
  console.log(`program ${hexEncode(new Uint8Array(programCode))}`);

  // Prepare and start
  const program = prepareProgram(
    InputKind.SPI,
    HasMetadata.Yes,
    programCode,
    [],
    [],
    [],
    spiArgs,
    128,
    true,
  );
  const id = pvmStart(program);

  // Print initial memwrite for SPI args
  if (spiArgs.length > 0) {
    console.log(`memwrite 0x${ARGS_SEGMENT_START.toString(16)} len=${spiArgs.length} <- ${hexEncode(new Uint8Array(spiArgs))}`);
  }

  // Print start line
  // For SPI, r7 = args_ptr, r8 = args_len at start
  console.log(`start pc=${pc} gas=${gas} r7=0x${ARGS_SEGMENT_START.toString(16)} r8=0x${spiArgs.length.toString(16)}`);

  for (;;) {
    const pause = pvmResume(id, gas, pc, false);
    if (!pause) {
      throw new Error("pvmResume returned null");
    }

    if (pause.status === STATUS.HOST) {
      const ecalliIdx = pause.exitCode;

      // Print ecalli line
      console.log(`\necalli=${ecalliIdx} pc=${pause.pc} gas=${pause.gas} ${formatRegisters(pause.registers)}`);

      if (ecalliIdx === LOG_HOST_CALL_INDEX) {
        // JIP-1 log: read target and message from memory
        const targetPtr = Number(pause.registers[8] & 0xFFFFFFFFn);
        const targetLen = Number(pause.registers[9] & 0xFFFFFFFFn);
        const msgPtr = Number(pause.registers[10] & 0xFFFFFFFFn);
        const msgLen = Number(pause.registers[11] & 0xFFFFFFFFn);

        if (targetLen > 0) {
          const targetData = pvmReadMemory(id, targetPtr, targetLen);
          if (targetData) {
            console.log(`  memread 0x${targetPtr.toString(16)} len=${targetLen} -> ${hexEncode(targetData)}`);
          }
        }
        if (msgLen > 0) {
          const msgData = pvmReadMemory(id, msgPtr, msgLen);
          if (msgData) {
            console.log(`  memread 0x${msgPtr.toString(16)} len=${msgLen} -> ${hexEncode(msgData)}`);
          }
        }

        // Set r7 = WHAT (standard response)
        console.log(`  setreg r07 <- 0x${WHAT.toString(16)}`);

        const regs = pause.registers;
        regs[7] = WHAT;
        pvmSetRegisters(id, regs);
        // Deduct a nominal gas cost for ecalli handling (matches anan-as convention)
        const ECALLI_GAS_COST = 10n;
        gas = pause.gas >= ECALLI_GAS_COST ? pause.gas - ECALLI_GAS_COST : 0n;
        pc = pause.nextPc;
      } else {
        // Unknown ecalli: fail fast with identifying context
        throw new Error(
          `Unsupported ecalli encountered: index=${ecalliIdx}, pc=${pause.pc}, gas=${pause.gas}, nextPc=${pause.nextPc}`
        );
      }
    } else {
      // Termination
      let type: string;
      if (pause.status === STATUS.HALT) type = "HALT";
      else if (pause.status === STATUS.PANIC) type = `PANIC`;
      else if (pause.status === STATUS.OOG) type = "OOG";
      else type = `FAULT`;

      if (pause.status === STATUS.PANIC && pause.exitCode !== 0) {
        type = `PANIC=${pause.exitCode}`;
      }

      console.log(`\n------`);
      console.log(`${type} pc=${pause.pc} gas=${pause.gas} ${formatRegisters(pause.registers)}`);
      break;
    }
  }

  pvmDestroy(id);
}

main();