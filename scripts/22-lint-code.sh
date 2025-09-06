#!/bin/bash
# Run code linting for Wasmbed platform
# This script runs various linting tools on the codebase

set -e

echo "🔍 Running code linting..."

# Check if cargo is available
if ! command -v cargo >/dev/null 2>&1; then
    echo "❌ Cargo not found"
    echo "Please install Rust and Cargo"
    exit 1
fi

echo "✅ Cargo is available"

# Run clippy linting
echo "📋 Running Clippy linting..."
cargo clippy --all-targets --all-features -- -D warnings

if [ $? -eq 0 ]; then
    echo "✅ Clippy linting passed"
else
    echo "❌ Clippy linting failed"
    exit 1
fi

# Run rustfmt formatting check
echo "📋 Running rustfmt formatting check..."
cargo fmt --all -- --check

if [ $? -eq 0 ]; then
    echo "✅ Code formatting is correct"
else
    echo "❌ Code formatting issues found"
    echo "Run: cargo fmt --all"
    exit 1
fi

# Run cargo check
echo "📋 Running cargo check..."
cargo check --all-targets --all-features

if [ $? -eq 0 ]; then
    echo "✅ Cargo check passed"
else
    echo "❌ Cargo check failed"
    exit 1
fi

# Check for unused dependencies
echo "📋 Checking for unused dependencies..."
cargo check --all-targets --all-features --message-format=short | grep -E "unused|cannot find" || true

# Check documentation
echo "📋 Checking documentation..."
cargo doc --no-deps --all-features

if [ $? -eq 0 ]; then
    echo "✅ Documentation generated successfully"
else
    echo "❌ Documentation generation failed"
    exit 1
fi

echo ""
echo "🎉 All linting checks passed!"
echo ""
echo "📊 Linting Summary:"
echo "==================="
echo "✅ Clippy: No warnings or errors"
echo "✅ Formatting: Code is properly formatted"
echo "✅ Compilation: All code compiles successfully"
echo "✅ Documentation: Generated successfully"
echo ""
echo "Next steps:"
echo "  ./wasmbed.sh test-unit                # Run unit tests"
echo "  ./wasmbed.sh test-integration          # Run integration tests"
echo "  ./wasmbed.sh build                     # Build all components"

