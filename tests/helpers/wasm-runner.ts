/**
 * Native WASM runner for differential testing.
 *
 * Runs WASM modules using Bun's built-in WebAssembly engine and returns
 * results in the same format as `runJam()` for comparison.
 */

import fs from "node:fs";
import path from "node:path";
import { WAT_DIR, WASM_DIR } from "./paths";

/** Args are written to linear memory at this offset. */
const ARGS_OFFSET = 0x1000;

/**
 * Convert a `.jam.wat` file to a WASM binary using the `wabt` npm package.
 *
 * Results are cached in-memory for the lifetime of the test run.
 */
const watCache = new Map<string, Uint8Array>();

async function watToWasm(watPath: string): Promise<Uint8Array> {
  const cached = watCache.get(watPath);
  if (cached) return cached;

  let watSource = fs.readFileSync(watPath, "utf8");

  // Ensure memory is exported so we can read/write args and results.
  // If the module declares `(memory N)` without an export, add one.
  if (
    !watSource.includes('(export "memory"') &&
    !watSource.includes("(export 'memory'")
  ) {
    // Replace `(memory N)` with `(memory (export "memory") N)`
    watSource = watSource.replace(
      /\(memory\s+(\d+)\)/,
      '(memory (export "memory") $1)',
    );
  }

  const wabt = await import("wabt");
  const wabtModule = await wabt.default();
  const parsed = wabtModule.parseWat(watPath, watSource, {
    multi_value: true,
    mutable_globals: true,
    bulk_memory: true,
    sign_extension: true,
  });
  parsed.validate();
  const { buffer } = parsed.toBinary({});
  const bytes = new Uint8Array(buffer);
  watCache.set(watPath, bytes);
  return bytes;
}

/**
 * Load a WASM binary for a given suite name.
 *
 * - WAT fixtures: converted from `tests/fixtures/wat/<name>.jam.wat`
 * - AS fixtures: loaded from `tests/build/wasm/<name>.wasm` (stripping `as-` prefix)
 */
async function loadWasmBinary(suiteName: string): Promise<Uint8Array | null> {
  // Try WAT fixture first
  const watPath = path.join(WAT_DIR, `${suiteName}.jam.wat`);
  if (fs.existsSync(watPath)) {
    return watToWasm(watPath);
  }

  // Try pre-built WASM (AS fixtures are prefixed with "as-")
  const wasmName = suiteName.startsWith("as-")
    ? suiteName.slice(3)
    : suiteName;
  const wasmPath = path.join(WASM_DIR, `${wasmName}.wasm`);
  if (fs.existsSync(wasmPath)) {
    return new Uint8Array(fs.readFileSync(wasmPath));
  }

  return null;
}

/**
 * Check whether a WASM binary has imports that require host functions.
 *
 * Modules with imports (like `host_call`, `pvm_ptr`) cannot be run natively
 * without providing matching stubs. We skip these in differential testing.
 */
function hasNonMemoryImports(wasmBinary: Uint8Array): boolean {
  try {
    const module = new WebAssembly.Module(wasmBinary as BufferSource);
    const imports = WebAssembly.Module.imports(module);
    // Allow memory imports; skip if there are function imports
    return imports.some((imp) => imp.kind === "function");
  } catch {
    return true;
  }
}

export interface WasmRunResult {
  /** The parsed return value (little-endian u32), or null if execution trapped. */
  value: number | null;
  /** True if the module trapped (unreachable, div-by-zero, etc.). */
  trapped: boolean;
  /** Error message if trapped. */
  error?: string;
}

/**
 * Run a WASM module natively using Bun's WebAssembly engine.
 *
 * @param wasmBinary - The WASM binary to instantiate
 * @param argsHex - Hex string of arguments (e.g. "0500000007000000")
 * @returns The result value or trap information
 */
export async function runWasmNative(
  wasmBinary: Uint8Array,
  argsHex: string,
): Promise<WasmRunResult> {
  const argsBytes = hexToBytes(argsHex);

  try {
    const module = new WebAssembly.Module(wasmBinary as BufferSource);
    const memory = new WebAssembly.Memory({ initial: 2 });
    const importObject: WebAssembly.Imports = {};

    // Provide memory if the module imports it
    const moduleImports = WebAssembly.Module.imports(module);
    for (const imp of moduleImports) {
      if (imp.kind === "memory") {
        if (!importObject[imp.module]) importObject[imp.module] = {};
        (importObject[imp.module] as Record<string, unknown>)[imp.name] =
          memory;
      }
    }

    const instance = new WebAssembly.Instance(module, importObject);
    const mainFn = instance.exports.main as Function;
    if (!mainFn) {
      return { value: null, trapped: true, error: "No 'main' export found" };
    }

    // Use the module's own memory if it exports one, otherwise use the imported one
    const mem =
      (instance.exports.memory as WebAssembly.Memory) ?? memory;
    const memView = new Uint8Array(mem.buffer);

    // Write args to linear memory
    memView.set(argsBytes, ARGS_OFFSET);

    // Call main(args_ptr, args_len)
    const result = mainFn(ARGS_OFFSET, argsBytes.length);

    // Multi-value return: result is [result_ptr, result_len]
    let resultPtr: number;
    let resultLen: number;

    if (Array.isArray(result)) {
      [resultPtr, resultLen] = result;
    } else if (typeof result === "number") {
      // Single-value return (shouldn't happen with our fixtures, but handle it)
      return { value: result, trapped: false };
    } else {
      return {
        value: null,
        trapped: true,
        error: `Unexpected return type: ${typeof result}`,
      };
    }

    // Read result from linear memory (re-acquire view in case memory grew)
    const resultView = new Uint8Array(mem.buffer);
    const resultBytes = resultView.slice(resultPtr, resultPtr + resultLen);

    // Parse as little-endian u32 (same as runJam's parseExitValue)
    const value = parseLittleEndianU32(resultBytes);
    return { value, trapped: false };
  } catch (err: any) {
    const msg = err?.message ?? String(err);
    // WebAssembly traps manifest as RuntimeError
    if (
      err instanceof WebAssembly.RuntimeError ||
      msg.includes("unreachable") ||
      msg.includes("integer divide by zero") ||
      msg.includes("integer overflow")
    ) {
      return { value: null, trapped: true, error: msg };
    }
    return { value: null, trapped: true, error: msg };
  }
}

/**
 * High-level differential test runner: load WASM for a suite and run a test case.
 *
 * @returns The native WASM result, or null if the suite cannot be run natively
 *          (e.g. has function imports).
 */
export async function runWasmForSuite(
  suiteName: string,
  argsHex: string,
): Promise<WasmRunResult | null> {
  const binary = await loadWasmBinary(suiteName);
  if (!binary) return null;
  if (hasNonMemoryImports(binary)) return null;

  return runWasmNative(binary, argsHex);
}

// ---------------------------------------------------------------------------
// Utility functions
// ---------------------------------------------------------------------------

function hexToBytes(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2);
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16);
  }
  return bytes;
}

function parseLittleEndianU32(bytes: Uint8Array): number {
  if (bytes.length === 0) return 0;
  // Pad to 4 bytes
  const padded = new Uint8Array(4);
  padded.set(bytes.slice(0, 4));
  // Use >>> 0 on the whole expression to get unsigned u32
  return (
    (padded[0] |
      (padded[1] << 8) |
      (padded[2] << 16) |
      (padded[3] << 24)) >>>
    0
  );
}
