#!/usr/bin/env npx tsx
/**
 * Compile all .jam.wat examples into JAM bytecode files under ./dist
 * Usage: npx tsx scripts/build-examples.ts
 */

import { execSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const projectRoot = path.join(__dirname, '..');
const examplesDir = path.join(projectRoot, 'examples-wat');
const distDir = path.join(projectRoot, 'dist');

function compileExample(inputPath: string, outputPath: string) {
  execSync(
    `cargo run -p wasm-pvm-cli --quiet -- compile "${inputPath}" -o "${outputPath}"`,
    { cwd: projectRoot, stdio: 'inherit' }
  );
}

function main() {
  fs.mkdirSync(distDir, { recursive: true });

  const entries = fs
    .readdirSync(examplesDir, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith('.jam.wat'));

  if (entries.length === 0) {
    console.log('No .jam.wat examples found in examples-wat/');
    return;
  }

  console.log(`Compiling ${entries.length} example(s) to dist/`);

  for (const entry of entries) {
    const inputPath = path.join(examplesDir, entry.name);
    const outputName = entry.name.replace(/\.jam\.wat$/, '.jam');
    const outputPath = path.join(distDir, outputName);

    console.log(`- ${entry.name} -> dist/${outputName}`);
    compileExample(inputPath, outputPath);
  }

  console.log('\nDone. Outputs are in dist/');
}

main();
