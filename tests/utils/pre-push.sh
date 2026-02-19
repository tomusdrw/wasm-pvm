#!/bin/bash
# Pre-push hook for wasm-pvm
# Install with: cp tests/utils/pre-push.sh .git/hooks/pre-push

set -e

echo "Running pre-push checks (same as CI)..."

# Run formatting check
echo "Checking code formatting..."
cargo fmt --check
echo "Formatting OK"

# Run clippy
echo "Running Clippy..."
cargo clippy -- -D warnings
echo "Clippy OK"

# Run unit tests (all packages)
echo "Running unit tests..."
cargo test --package wasm-pvm --features test-harness
cargo test --package wasm-pvm-cli
echo "Unit tests OK"

# Build the project
echo "Building project..."
cargo build --release
echo "Build OK"

# Build anan-as if needed
echo "Building anan-as..."
cd vendor/anan-as
if [ ! -d "node_modules" ]; then
    npm ci
fi
npm run build
cd ../..
echo "anan-as OK"

# Run full integration test suite (clean build to match CI)
echo "Running full integration test suite..."
cd tests
if [ ! -d "node_modules" ]; then
    bun install
fi
rm -rf build
bun run test
cd ..
echo "Integration tests OK"

echo ""
echo "All pre-push checks passed! You can now push safely."
