#!/bin/bash
# Run unit tests for Wasmbed platform
# This script runs all unit tests for Rust components

set -e

echo "🧪 Running unit tests..."

# Run cargo tests for all crates
echo "📋 Testing wasmbed-protocol..."
cargo test -p wasmbed-protocol

echo "📋 Testing wasmbed-types..."
cargo test -p wasmbed-types

echo "📋 Testing wasmbed-k8s-resource..."
cargo test -p wasmbed-k8s-resource

echo "📋 Testing wasmbed-k8s-resource-tool..."
cargo test -p wasmbed-k8s-resource-tool

echo "📋 Testing wasmbed-cert-tool..."
cargo test -p wasmbed-cert-tool

echo "📋 Testing wasmbed-gateway..."
cargo test -p wasmbed-gateway

echo "📋 Testing wasmbed-k8s-controller..."
cargo test -p wasmbed-k8s-controller

echo "📋 Testing wasmbed-protocol-server..."
cargo test -p wasmbed-protocol-server

echo "📋 Testing wasmbed-gateway-test-client..."
cargo test -p wasmbed-gateway-test-client

echo ""
echo "✅ All unit tests passed!"
echo ""
echo "Next steps:"
echo "  ./wasmbed.sh test-integration         # Run integration tests"
echo "  ./wasmbed.sh test-end-to-end          # Run end-to-end tests"

