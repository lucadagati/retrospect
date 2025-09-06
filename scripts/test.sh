#!/bin/bash
# Convenience script for testing
set -euo pipefail

echo "Wasmbed - Complete Testing"
echo "=========================="
echo "Running complete tests..."
./scripts/testing/run-all-tests.sh

echo "Testing completed!"