#!/bin/bash
# Pre-push hook for wasm-pvm
# Install with: cp scripts/pre-push.sh .git/hooks/pre-push

set -e

echo "🔍 Running pre-push checks (same as CI)..."

# Run formatting check
echo "📝 Checking code formatting..."
cargo fmt --check
echo "✅ Formatting OK"

# Run clippy
echo "🔍 Running Clippy..."
cargo clippy -- -D warnings
echo "✅ Clippy OK"

# Run unit tests (all packages)
echo "🧪 Running unit tests..."
cargo test --package wasm-pvm --features test-harness
cargo test --package wasm-pvm-cli
echo "✅ Unit tests OK"

# Build the project
echo "🔨 Building project..."
cargo build --release
echo "✅ Build OK"

# Build anan-as if needed
echo "🔧 Building anan-as..."
cd vendor/anan-as
if [ ! -d "node_modules" ]; then
    npm ci
fi
npm run build
cd ../..
echo "✅ anan-as OK"

# Build examples-as if needed
echo "🔧 Building examples-as..."
cd examples-as
if [ ! -d "node_modules" ]; then
    npm ci
fi
npm run build
cd ..
echo "✅ examples-as OK"

# Run full integration test suite
echo "🧪 Running full integration test suite (test-all.ts)..."
npx tsx scripts/test-all.ts
echo "✅ Integration tests OK"

echo ""
echo "🎉 All pre-push checks passed! You can now push safely."
