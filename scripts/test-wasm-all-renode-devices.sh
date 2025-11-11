#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Script per testare il runtime WASM su tutti i dispositivi Renode configurati

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RENODE_BINARY="$PROJECT_ROOT/renode_1.15.0_portable/renode"
ARM_TARGET="thumbv7em-none-eabihf"
FIRMWARE_PATH="$PROJECT_ROOT/target/$ARM_TARGET/release/wasmbed-device-runtime"

# Colori
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "=== Test WASM Runtime su Tutti i Dispositivi Renode ==="
echo ""

# Verifica toolchain
echo "1. Verifica toolchain ARM..."
if ! rustup target list --installed | grep -q "$ARM_TARGET"; then
    echo -e "${YELLOW}Installazione toolchain...${NC}"
    rustup target add "$ARM_TARGET"
fi
echo -e "${GREEN}✓ Toolchain ARM installato${NC}"
echo ""

# Compila firmware
echo "2. Compilazione firmware ARM..."
if [ ! -f "$FIRMWARE_PATH" ]; then
    echo "Compilazione in corso..."
    "$SCRIPT_DIR/build-wasm-runtime-arm.sh"
    if [ ! -f "$FIRMWARE_PATH" ]; then
        echo -e "${RED}✗ Compilazione fallita${NC}"
        exit 1
    fi
fi
echo -e "${GREEN}✓ Firmware ARM disponibile${NC}"
echo ""

# Verifica Renode
echo "3. Verifica Renode..."
if [ ! -f "$RENODE_BINARY" ]; then
    echo -e "${RED}✗ Renode non trovato${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Renode disponibile${NC}"
echo ""

# Lista dispositivi da testare
DEVICES=(
    "arduino_nano:renode-scripts/test_wasm_arduino_nano.resc:Arduino Nano 33 BLE"
    "nrf52840:renode-scripts/test_wasm_nrf52840.resc:nRF52840 DK"
    "stm32f4:renode-scripts/test_wasm_stm32f4.resc:STM32F4 Discovery"
    "wasm_execution:renode-scripts/test_wasm_execution.resc:WASM Execution Test"
)

echo "4. Test su dispositivi:"
echo ""
for device_info in "${DEVICES[@]}"; do
    IFS=':' read -r device_id script_path device_name <<< "$device_info"
    script_full_path="$PROJECT_ROOT/$script_path"
    
    echo "   Testing: $device_name"
    if [ -f "$script_full_path" ]; then
        echo -e "   ${GREEN}✓ Script disponibile${NC}"
    else
        echo -e "   ${RED}✗ Script non trovato: $script_path${NC}"
    fi
done

echo ""
echo "=== Istruzioni per Test Manuale ==="
echo ""
echo "Per testare ogni dispositivo, esegui:"
echo ""
for device_info in "${DEVICES[@]}"; do
    IFS=':' read -r device_id script_path device_name <<< "$device_info"
    echo "  $device_name:"
    echo "    ./renode_1.15.0_portable/renode $script_path"
    echo ""
done

echo "=== Test Automatico (opzionale) ==="
echo ""
echo "Per eseguire test automatici, usa:"
echo "  ./renode_1.15.0_portable/renode --console --execute \"mach create 'test'; sysbus LoadELF @$FIRMWARE_PATH; start; sleep 3; quit\""
echo ""

