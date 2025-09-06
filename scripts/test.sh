#!/bin/bash
# Script di convenienza per test
set -euo pipefail

echo "🧪 Wasmbed - Test Completo"
echo "=========================="
echo "Esecuzione test completi..."
./scripts/testing/run-all-tests.sh

echo "✅ Test completati!"
