#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Script per testare l'esecuzione WASM in Renode

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RENODE_BINARY="$PROJECT_ROOT/renode_1.15.0_portable/renode"
FIRMWARE_PATH="$PROJECT_ROOT/target/release/wasmbed-device-runtime"
RENODE_SCRIPT="$PROJECT_ROOT/renode-scripts/test_wasm_execution.resc"

echo "=== Test WASM Runtime in Renode ==="
echo ""

# Verifica binario
echo "1. Verifica binario firmware..."
if [ ! -f "$FIRMWARE_PATH" ]; then
    echo "   ❌ Binario non trovato"
    echo "   Compilazione in corso..."
    cd "$PROJECT_ROOT"
    cargo build --release --bin wasmbed-device-runtime --features std
    if [ ! -f "$FIRMWARE_PATH" ]; then
        echo "   ❌ Compilazione fallita"
        exit 1
    fi
fi
echo "   ✅ Binario trovato: $(ls -lh "$FIRMWARE_PATH" | awk '{print $5}')"
echo ""

# Verifica Renode
echo "2. Verifica Renode..."
if [ ! -f "$RENODE_BINARY" ]; then
    echo "   ❌ Renode non trovato a: $RENODE_BINARY"
    exit 1
fi
echo "   ✅ Renode trovato"
echo ""

# Verifica script
echo "3. Verifica script Renode..."
if [ ! -f "$RENODE_SCRIPT" ]; then
    echo "   ❌ Script Renode non trovato: $RENODE_SCRIPT"
    exit 1
fi
echo "   ✅ Script trovato"
echo ""

echo "=== Avvio Renode ==="
echo ""
echo "Il firmware verrà caricato automaticamente."
echo "Controlla la finestra Renode e l'UART analyzer per i log."
echo ""
echo "Log attesi:"
echo "  - Device runtime initialized - WASM execution enabled"
echo "  - Testing WASM execution..."
echo "  - WASM module loaded: 1 functions"
echo "  - WASM execution completed successfully"
echo "  - WASM test PASSED: Memory contains correct value (42)"
echo ""
echo "Premi Ctrl+C per uscire"
echo ""

# Avvia Renode
cd "$PROJECT_ROOT"
"$RENODE_BINARY" "$RENODE_SCRIPT"

