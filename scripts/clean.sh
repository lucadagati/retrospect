#!/bin/bash
# Convenience script for cleanup
set -euo pipefail

echo "Wasmbed - System Cleanup"
echo "========================"
echo "Cleaning platform..."
./scripts/setup/04-cleanup-platform.sh

echo "Cleanup completed!"