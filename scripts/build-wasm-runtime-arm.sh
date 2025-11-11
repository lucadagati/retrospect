#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Script per compilare il runtime WASM per ARM Cortex-M
# Supporta tutti i dispositivi Renode configurati

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "=== Compilazione WASM Runtime per ARM Cortex-M ==="
echo ""

# Colori
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Target ARM Cortex-M
# thumbv7em-none-eabihf: Cortex-M4/M7 con FPU hardware (STM32F4, nRF52840)
# thumbv7m-none-eabi: Cortex-M3/M4 senza FPU (STM32F1, alcuni STM32F4)
ARM_TARGET="thumbv7em-none-eabihf"

echo "1. Verifica toolchain ARM..."
if ! rustup target list --installed | grep -q "$ARM_TARGET"; then
    echo -e "${YELLOW}Installazione toolchain ARM...${NC}"
    rustup target add "$ARM_TARGET"
fi
echo -e "${GREEN}✓ Toolchain ARM installato: $ARM_TARGET${NC}"
echo ""

echo "2. Compilazione runtime WASM per ARM (no_std)..."
cd "$PROJECT_ROOT"

# Compila senza feature std per no_std puro
if cargo build --target "$ARM_TARGET" --release --package wasmbed-device-runtime --no-default-features 2>&1 | tee /tmp/wasm-arm-build.log; then
    echo -e "${GREEN}✓ Compilazione riuscita${NC}"
    BINARY_PATH="target/$ARM_TARGET/release/wasmbed-device-runtime"
    if [ -f "$BINARY_PATH" ]; then
        SIZE=$(stat -c%s "$BINARY_PATH" 2>/dev/null || stat -f%z "$BINARY_PATH" 2>/dev/null)
        echo -e "${GREEN}✓ Binario creato: $BINARY_PATH ($(($SIZE / 1024)) KB)${NC}"
    else
        echo -e "${RED}✗ Binario non trovato${NC}"
        exit 1
    fi
else
    echo -e "${RED}✗ Compilazione fallita${NC}"
    echo "Errori:"
    tail -20 /tmp/wasm-arm-build.log
    exit 1
fi

echo ""
echo "3. Verifica binario ELF..."
if file "$BINARY_PATH" | grep -q "ARM"; then
    echo -e "${GREEN}✓ Binario ARM valido${NC}"
    file "$BINARY_PATH"
else
    echo -e "${YELLOW}⚠ Verifica tipo binario:${NC}"
    file "$BINARY_PATH"
fi

echo ""
echo "=== Compilazione completata ==="
echo ""
echo "Binario ARM: $BINARY_PATH"
echo ""
echo "Per testare in Renode:"
echo "  ./renode_1.15.0_portable/renode renode-scripts/test_wasm_arduino_nano.resc"
echo "  ./renode_1.15.0_portable/renode renode-scripts/test_wasm_nrf52840.resc"
echo "  ./renode_1.15.0_portable/renode renode-scripts/test_wasm_stm32f4.resc"

