#!/usr/bin/env bun
/**
 * Pretest build orchestrator.
 * 1. Builds the CLI binary (cargo build)
 * 2. Compiles AS sources -> WASM
 * 3. Compiles WAT/WASM -> JAM
 */

import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {
  PROJECT_ROOT,
  WAT_DIR,
  WASM_DIR,
  JAM_DIR,
  CLI_BINARY,
} from "./helpers/paths";
import { compileAS, compileToJAM } from "./helpers/compile";
import { getAllSuites } from "./data/test-cases";

const CONCURRENCY = 8;

async function runParallel<T>(
  items: T[],
  fn: (item: T) => void,
  concurrency: number
): Promise<void> {
  let index = 0;
  const workers = Array.from({ length: concurrency }, async () => {
    while (index < items.length) {
      const i = index++;
      fn(items[i]);
    }
  });
  await Promise.all(workers);
}

function ensureDirs() {
  fs.mkdirSync(WASM_DIR, { recursive: true });
  fs.mkdirSync(JAM_DIR, { recursive: true });
}

function buildCliBinary() {
  console.log("Building CLI binary...");
  execSync("cargo build -p wasm-pvm-cli --release", {
    cwd: PROJECT_ROOT,
    stdio: "inherit",
  });
  console.log("CLI binary built.");
}

interface ASBuildTarget {
  sourceName: string;
  outputName: string;
  runtime: string;
}

interface JAMBuildTarget {
  inputPath: string;
  outputName: string;
}

function collectBuildTargets(): {
  asTargets: ASBuildTarget[];
  jamTargets: JAMBuildTarget[];
} {
  const suites = getAllSuites();
  const asTargets: ASBuildTarget[] = [];
  const jamTargets: JAMBuildTarget[] = [];
  const seenAS = new Set<string>();
  const seenJAM = new Set<string>();

  for (const suite of suites) {
    if (suite.source.type === "as") {
      const key = `${suite.source.file}:${suite.source.runtime || "stub"}:${suite.name}`;
      if (!seenAS.has(key)) {
        seenAS.add(key);
        // The output name for AS is the suite name without the "as-" prefix
        const baseName = suite.name.slice(3);
        asTargets.push({
          sourceName: suite.source.file,
          outputName: baseName,
          runtime: suite.source.runtime || "stub",
        });
      }
    } else if (suite.source.type === "wat") {
      if (!seenJAM.has(suite.name)) {
        seenJAM.add(suite.name);
        const watFile = path.join(WAT_DIR, suite.source.file);
        jamTargets.push({
          inputPath: watFile,
          outputName: suite.name,
        });
      }
    }
  }

  return { asTargets, jamTargets };
}

async function main() {
  console.log("=== Build Orchestrator ===\n");

  ensureDirs();
  buildCliBinary();

  const { asTargets, jamTargets } = collectBuildTargets();

  // Phase 1: Compile AS -> WASM
  console.log(`\nCompiling ${asTargets.length} AS sources -> WASM...`);
  let asCompiled = 0;
  let asSkipped = 0;

  await runParallel(
    asTargets,
    (target) => {
      try {
        const wasmFile = path.join(WASM_DIR, `${target.outputName}.wasm`);
        const existed = fs.existsSync(wasmFile);
        compileAS(target.sourceName, target.outputName, target.runtime);
        if (existed) {
          asSkipped++;
        } else {
          asCompiled++;
        }
      } catch (err: any) {
        console.error(
          `  FAIL: ${target.sourceName} (${target.runtime}): ${err.message}`
        );
      }
    },
    CONCURRENCY
  );
  console.log(
    `  AS: ${asCompiled} compiled, ${asSkipped} up-to-date`
  );

  // Phase 2: Compile WAT -> JAM and WASM -> JAM
  // First, collect all JAM targets (WAT files + AS WASM files)
  const allJamTargets: JAMBuildTarget[] = [...jamTargets];

  // Add AS WASM -> JAM targets
  for (const asTarget of asTargets) {
    const wasmFile = path.join(WASM_DIR, `${asTarget.outputName}.wasm`);
    if (fs.existsSync(wasmFile)) {
      allJamTargets.push({
        inputPath: wasmFile,
        outputName: `as-${asTarget.outputName}`,
      });
    }
  }

  console.log(`\nCompiling ${allJamTargets.length} files -> JAM...`);
  let jamCompiled = 0;

  await runParallel(
    allJamTargets,
    (target) => {
      try {
        compileToJAM(target.inputPath, target.outputName);
        jamCompiled++;
      } catch (err: any) {
        console.error(
          `  FAIL: ${target.outputName}: ${err.message}`
        );
      }
    },
    CONCURRENCY
  );
  console.log(
    `  JAM: ${jamCompiled} compiled`
  );

  console.log("\nBuild complete.");
}

main().catch((err) => {
  console.error("Build failed:", err);
  process.exit(1);
});
