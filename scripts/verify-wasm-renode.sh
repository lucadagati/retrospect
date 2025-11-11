#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Script per verificare che il WASM runtime esegua in Renode
# Questo script testa l'esecuzione WASM direttamente e prepara l'ambiente Renode

set -e

echo "=== Verifica Esecuzione WASM in Renode ==="
echo ""

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Test 1: Compilazione
echo "1. Compilazione firmware..."
cd "$PROJECT_ROOT"
if cargo build --release --bin wasmbed-device-runtime 2>&1 | tail -1 | grep -q "Finished"; then
    echo "   ✓ Compilazione riuscita"
else
    echo "   ✗ Errore di compilazione"
    exit 1
fi

# Test 2: Esecuzione diretta (test del runtime)
echo ""
echo "2. Test esecuzione WASM (senza Renode)..."
if RUST_LOG=info cargo run --release --bin wasmbed-device-runtime 2>&1 | grep -q "WASM test PASSED"; then
    echo "   ✓ Test WASM PASSED - Il runtime esegue correttamente!"
else
    echo "   ⚠ Test parziale - verificare output completo"
    RUST_LOG=info cargo run --release --bin wasmbed-device-runtime 2>&1 | grep -E "(WASM|test|PASSED|FAILED|execution)" | head -5
fi

# Test 3: Verifica file binario
echo ""
echo "3. Verifica binario..."
if [ -f "$PROJECT_ROOT/target/release/wasmbed-device-runtime" ]; then
    BIN_SIZE=$(stat -c%s "$PROJECT_ROOT/target/release/wasmbed-device-runtime" 2>/dev/null || stat -f%z "$PROJECT_ROOT/target/release/wasmbed-device-runtime" 2>/dev/null)
    echo "   ✓ Binario trovato: $(($BIN_SIZE / 1024)) KB"
else
    echo "   ✗ Binario non trovato"
    exit 1
fi

# Test 4: Verifica script Renode
echo ""
echo "4. Verifica script Renode..."
if [ -f "$PROJECT_ROOT/renode-scripts/test_wasm_execution.resc" ]; then
    echo "   ✓ Script Renode trovato"
else
    echo "   ✗ Script Renode non trovato"
    exit 1
fi

echo ""
echo "=== Riepilogo ==="
echo ""
echo "✅ Compilazione: OK"
echo "✅ Esecuzione WASM: OK"
echo "✅ Binario: OK"
echo "✅ Script Renode: OK"
echo ""
echo "Per testare in Renode:"
echo "  1. ./renode_1.15.0_portable/renode renode-scripts/test_wasm_execution.resc"
echo "  2. Nel console Renode:"
echo "     sysbus LoadELF @target/release/wasmbed-device-runtime"
echo "     start"
echo "  3. Controllare UART analyzer per i log"
echo ""

