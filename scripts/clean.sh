#!/bin/bash
# Script di convenienza per pulizia
set -euo pipefail

echo "🧹 Wasmbed - Pulizia Sistema"
echo "==========================="
echo "Pulizia piattaforma..."
./scripts/setup/04-cleanup-platform.sh

echo "✅ Pulizia completata!"
