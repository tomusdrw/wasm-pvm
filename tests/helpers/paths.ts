import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export const TESTS_DIR = path.join(__dirname, "..");
export const PROJECT_ROOT = path.join(TESTS_DIR, "..");
export const FIXTURES_DIR = path.join(TESTS_DIR, "fixtures");
export const AS_ASSEMBLY_DIR = path.join(FIXTURES_DIR, "assembly");
export const WAT_DIR = path.join(FIXTURES_DIR, "wat");
export const BUILD_DIR = path.join(TESTS_DIR, "build");
export const WASM_DIR = path.join(BUILD_DIR, "wasm");
export const JAM_DIR = path.join(BUILD_DIR, "jam");
export const ANAN_AS_CLI = path.join(
  PROJECT_ROOT,
  "vendor/anan-as/dist/bin/index.js"
);
export const CLI_BINARY = path.join(
  PROJECT_ROOT,
  "target/release/wasm-pvm"
);
