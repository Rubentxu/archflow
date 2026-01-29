#!/bin/bash
# Build script for ArchFlow SDK TypeScript types
# This script regenerates TypeScript bindings from Rust and copies them to the packages

set -e

echo "🏗️  Building ArchFlow SDK TypeScript types..."

# Build the Rust crate with tests to generate bindings
echo "📦 Building Rust crate and generating TypeScript bindings..."
cargo build --package archflow-sdk --features wasm

# Run tests to ensure everything works
echo "🧪 Running tests..."
cargo test --package archflow-sdk --features wasm

echo "✅ TypeScript types generated successfully!"
echo ""
echo "Generated files:"
ls -1 packages/archflow-sdk-types/src/generated/*.ts | xargs -n1 basename
echo ""
echo "📍 Location: packages/archflow-sdk-types/src/generated/"
