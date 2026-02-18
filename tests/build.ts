#!/usr/bin/env bun
/**
 * Pretest build orchestrator.
 * 1. Builds the CLI binary (cargo build)
 * 2. Compiles AS sources -> WASM
 * 3. Compiles WAT/WASM -> JAM
 *
 * Build targets are discovered from the filesystem:
 * - WAT files: tests/fixtures/wat/*.jam.wat
 * - AS files: tests/fixtures/assembly/*.ts
 *
 * For AS sources that need multiple runtime variants (e.g. alloc-test),
 * add entries to AS_RUNTIME_VARIANTS below.
 */

import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import {
  PROJECT_ROOT,
  WAT_DIR,
  WASM_DIR,
  JAM_DIR,
  AS_ASSEMBLY_DIR,
  IMPORTS_DIR,
} from "./helpers/paths";
import { compileAS, compileToJAM } from "./helpers/compile";

const CONCURRENCY = 8;

/**
 * AS sources that need to be compiled with non-default runtimes.
 * Each entry maps a source file to additional (outputSuffix, runtime) pairs.
 * Every AS source is always compiled with "stub" runtime by default.
 */
const AS_RUNTIME_VARIANTS: Record<string, { suffix: string; runtime: string }[]> = {
  "alloc-test.ts": [
    { suffix: "-stub", runtime: "stub" },
    { suffix: "-minimal", runtime: "minimal" },
    { suffix: "-incremental", runtime: "incremental" },
  ],
};

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
  importsPath?: string;
  adapterPath?: string;
}

function collectBuildTargets(): {
  asTargets: ASBuildTarget[];
  jamTargets: JAMBuildTarget[];
} {
  const asTargets: ASBuildTarget[] = [];
  const jamTargets: JAMBuildTarget[] = [];

  // Discover WAT files from filesystem
  const watFiles = fs.readdirSync(WAT_DIR).filter((f) => f.endsWith(".jam.wat"));
  for (const watFile of watFiles) {
    const outputName = watFile.replace(/\.jam\.wat$/, "");
    jamTargets.push({
      inputPath: path.join(WAT_DIR, watFile),
      outputName,
    });
  }

  // Discover AS files from filesystem
  const asFiles = fs.readdirSync(AS_ASSEMBLY_DIR).filter((f) => f.endsWith(".ts"));
  for (const asFile of asFiles) {
    const baseName = asFile.replace(/\.ts$/, "");

    // Default: compile with "stub" runtime
    asTargets.push({
      sourceName: asFile,
      outputName: baseName,
      runtime: "stub",
    });

    // Additional runtime variants if configured
    const variants = AS_RUNTIME_VARIANTS[asFile];
    if (variants) {
      for (const variant of variants) {
        asTargets.push({
          sourceName: asFile,
          outputName: `${baseName}${variant.suffix}`,
          runtime: variant.runtime,
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
      // Check for a matching adapter WAT file and/or import map file.
      const adapterFile = path.join(IMPORTS_DIR, `${asTarget.outputName}.adapter.wat`);
      const adapterPath = fs.existsSync(adapterFile) ? adapterFile : undefined;
      const importsFile = path.join(IMPORTS_DIR, `${asTarget.outputName}.imports`);
      const importsPath = fs.existsSync(importsFile) ? importsFile : undefined;
      allJamTargets.push({
        inputPath: wasmFile,
        outputName: `as-${asTarget.outputName}`,
        importsPath,
        adapterPath,
      });
    }
  }

  console.log(`\nCompiling ${allJamTargets.length} files -> JAM...`);
  let jamCompiled = 0;

  await runParallel(
    allJamTargets,
    (target) => {
      try {
        compileToJAM(target.inputPath, target.outputName, target.importsPath, target.adapterPath);
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

  // Phase 3: Compile anan-as compiler WASM -> JAM (for PVM-in-PVM tests)
  const ananAsCompilerWasm = path.join(PROJECT_ROOT, "vendor/anan-as/dist/build/compiler.wasm");
  const ananAsCompilerImports = path.join(IMPORTS_DIR, "anan-as-compiler.imports");
  const ananAsCompilerAdapter = path.join(IMPORTS_DIR, "anan-as-compiler.adapter.wat");
  if (fs.existsSync(ananAsCompilerWasm)) {
    console.log("\nCompiling anan-as compiler WASM -> JAM...");
    const imports = fs.existsSync(ananAsCompilerImports) ? ananAsCompilerImports : undefined;
    const adapter = fs.existsSync(ananAsCompilerAdapter) ? ananAsCompilerAdapter : undefined;
    try {
      compileToJAM(ananAsCompilerWasm, "anan-as-compiler", imports, adapter);
      console.log("  anan-as-compiler.jam compiled.");
    } catch (err: any) {
      console.error(`  FAIL: anan-as-compiler: ${err.message}`);
    }
  } else {
    console.log("\nSkipping anan-as compiler (WASM not found at vendor/anan-as/dist/build/compiler.wasm).");
  }

  console.log("\nBuild complete.");
}

main().catch((err) => {
  console.error("Build failed:", err);
  process.exit(1);
});
