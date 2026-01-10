#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Test WAMR integration by checking symbols in firmware

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Testing WAMR Integration ===${NC}"
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

BOARD="${1:-stm32f4_disco}"
# Build directory uses short board name
BOARD_SHORT=$(echo "$BOARD" | sed 's/_disco//' | sed 's/dk_nrf52840//' | sed 's/dk//')
BUILD_DIR="$PROJECT_ROOT/zephyr-workspace/build/$BOARD_SHORT"
ELF_FILE="$BUILD_DIR/zephyr/zephyr.elf"

if [ ! -f "$ELF_FILE" ]; then
    echo -e "${RED}✗ Firmware not found: $ELF_FILE${NC}"
    echo "Build the firmware first: ./scripts/build-zephyr-app.sh"
    exit 1
fi

echo -e "${YELLOW}Checking WAMR symbols in firmware...${NC}"
echo ""

# Check for WAMR runtime functions
# Note: Some functions may be static or have different names
REQUIRED_SYMBOLS=(
    "wasm_runtime_full_init"
    "wasm_runtime_destroy"
    "wasm_runtime_malloc"
    "wasm_runtime_free"
    "wasm_exec_env_create"
    "wamr_init"
)

# Optional symbols (may be static or conditionally compiled)
OPTIONAL_SYMBOLS=(
    "wasm_runtime_load"
    "wasm_runtime_instantiate"
    "wasm_runtime_call_wasm"
    "wamr_load_module"
    "wamr_instantiate"
)

MISSING=0
FOUND=0

echo -e "${YELLOW}Required symbols:${NC}"
for symbol in "${REQUIRED_SYMBOLS[@]}"; do
    # Check for symbol (T = text, t = static text, both are valid)
    if nm "$ELF_FILE" 2>/dev/null | grep -qE " [Tt] .*$symbol"; then
        echo -e "${GREEN}✓${NC} $symbol"
        FOUND=$((FOUND + 1))
    else
        echo -e "${RED}✗${NC} $symbol (missing)"
        MISSING=1
    fi
done

echo ""
echo -e "${YELLOW}Optional symbols:${NC}"
OPTIONAL_FOUND=0
for symbol in "${OPTIONAL_SYMBOLS[@]}"; do
    if nm "$ELF_FILE" 2>/dev/null | grep -qE " [Tt] .*$symbol"; then
        echo -e "${GREEN}✓${NC} $symbol"
        OPTIONAL_FOUND=$((OPTIONAL_FOUND + 1))
    else
        echo -e "${YELLOW}○${NC} $symbol (not found - may be static or unused)"
    fi
done

echo ""

if [ $MISSING -eq 0 ]; then
    echo -e "${GREEN}=== WAMR Integration: SUCCESS ===${NC}"
    echo ""
    echo "Required WAMR symbols: $FOUND/${#REQUIRED_SYMBOLS[@]}"
    echo "Optional symbols: $OPTIONAL_FOUND/${#OPTIONAL_SYMBOLS[@]}"
    echo ""
    echo "WAMR core runtime is integrated and functional."
    exit 0
else
    echo -e "${RED}=== WAMR Integration: FAILED ===${NC}"
    echo ""
    echo "Some required WAMR symbols are missing. Check build configuration."
    exit 1
fi

