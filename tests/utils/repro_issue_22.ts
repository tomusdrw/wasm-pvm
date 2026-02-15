#!/usr/bin/env node

// Reproduction script for issue #22 - inner interpreter PANIC
// This is a specialized debugging script, not a test file.
// It manually constructs the anan-as CLI arguments to reproduce
// the inner interpreter execution issue.

import fs from 'node:fs';
import path from 'node:path';
import { execSync } from 'node:child_process';

const PROJECT_ROOT = path.resolve(__dirname, '../../');
const ANAN_AS_WASM = path.join(PROJECT_ROOT, 'vendor/anan-as/dist/build/compiler.wasm');
const ADD_WAT = path.join(PROJECT_ROOT, 'tests/fixtures/wat/add.jam.wat');
const DIST_DIR = path.join(PROJECT_ROOT, 'tests/dist');
const ANAN_AS_JAM = path.join(DIST_DIR, 'anan-as.jam');
const ADD_JAM = path.join(DIST_DIR, 'add.jam');

function ensureDist() {
    if (!fs.existsSync(DIST_DIR)) {
        fs.mkdirSync(DIST_DIR, { recursive: true });
    }
}

function compile(input: string, output: string) {
    console.log(`Compiling ${input} -> ${output}`);
    execSync(`cargo run -p wasm-pvm-cli --quiet -- compile "${input}" -o "${output}"`, {
        cwd: PROJECT_ROOT,
        stdio: 'inherit'
    });
}

function runJam(jamFile: string, argsHex: string) {
    console.log(`Running ${jamFile} with args length ${argsHex.length / 2}`);
    const ananAsCli = path.join(PROJECT_ROOT, 'vendor/anan-as/dist/bin/index.js');
    const logFile = path.join(PROJECT_ROOT, 'repro.log');
    try {
        const cmd = `node "${ananAsCli}" run --spi --no-metadata --gas=100000000 "${jamFile}" 0x${argsHex}`;
        // Redirect output to file to avoid buffer overflow and allow analysis
        execSync(`${cmd} > "${logFile}" 2>&1`, {
            cwd: PROJECT_ROOT,
        });
        console.log("Execution finished successfully");
        console.log(fs.readFileSync(logFile, 'utf8').slice(-1000));
    } catch (e) {
        console.error("Execution failed");
        if (fs.existsSync(logFile)) {
            console.log(fs.readFileSync(logFile, 'utf8').slice(-1000));
        } else {
            console.log("Log file not found");
        }
        process.exit(1);
    }
}

function main() {
    ensureDist();

    if (!fs.existsSync(ANAN_AS_WASM)) {
        console.error(`Error: ${ANAN_AS_WASM} not found. Run 'npm run build' in vendor/anan-as.`);
        process.exit(1);
    }

    // 1. Compile anan-as to JAM
    compile(ANAN_AS_WASM, ANAN_AS_JAM);

    // 2. Compile add.wat to JAM
    compile(ADD_WAT, ADD_JAM);

    // 3. Prepare arguments
    // Input format:
    // 8 (gas) + 4 (pc) + 4 (spi-program-len) + 4 (inner-args-len) + ? (spi-program) + ? (inner-args)
    
    const addJamBytes = fs.readFileSync(ADD_JAM);
    
    // Arguments for add.jam: 5 and 7 (little endian u32s)
    // 05 00 00 00 | 07 00 00 00
    const innerArgs = Buffer.from([0x05, 0x00, 0x00, 0x00, 0x07, 0x00, 0x00, 0x00]);

    const gas = BigInt(100_000_000);
    const pc = 0;
    
    const buffer = Buffer.alloc(20 + addJamBytes.length + innerArgs.length);
    let offset = 0;
    
    // Gas (u64 le)
    buffer.writeBigUInt64LE(gas, offset);
    offset += 8;
    
    // PC (u32 le)
    buffer.writeUInt32LE(pc, offset);
    offset += 4;
    
    // Program len (u32 le)
    buffer.writeUInt32LE(addJamBytes.length, offset);
    offset += 4;
    
    // Inner args len (u32 le)
    buffer.writeUInt32LE(innerArgs.length, offset);
    offset += 4;
    
    // Program
    addJamBytes.copy(buffer, offset);
    offset += addJamBytes.length;
    
    // Inner args
    innerArgs.copy(buffer, offset);
    offset += innerArgs.length;
    
    const argsHex = buffer.toString('hex');
    
    // 4. Run
    runJam(ANAN_AS_JAM, argsHex);
}

main();
