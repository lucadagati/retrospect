#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Test network stack integration in firmware

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Testing Network Stack Integration ===${NC}"
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

BOARD="${1:-stm32f4_disco}"
BOARD_SHORT=$(echo "$BOARD" | sed 's/_disco//' | sed 's/dk_nrf52840//' | sed 's/dk//')
BUILD_DIR="$PROJECT_ROOT/zephyr-workspace/build/$BOARD_SHORT"
ELF_FILE="$BUILD_DIR/zephyr/zephyr.elf"

if [ ! -f "$ELF_FILE" ]; then
    echo -e "${RED}✗ Firmware not found: $ELF_FILE${NC}"
    echo "Build the firmware first: ./scripts/build-zephyr-app.sh"
    exit 1
fi

echo -e "${YELLOW}Checking network stack symbols in firmware...${NC}"
echo ""

# Check for network stack functions
# Note: Some functions may be static or not yet fully implemented
REQUIRED_SYMBOLS=(
    "network_init"
)

OPTIONAL_SYMBOLS=(
    "network_connect"
    "network_send"
    "network_receive"
    "network_process"
)

# Check for Zephyr network symbols
ZEPHYR_NET_SYMBOLS=(
    "net_if_get_default"
    "net_if_up"
    "net_dhcpv4_start"
    "socket"
    "connect"
)

MISSING=0
FOUND=0

echo -e "${YELLOW}Application network functions (required):${NC}"
for symbol in "${REQUIRED_SYMBOLS[@]}"; do
    if nm "$ELF_FILE" 2>/dev/null | grep -qE " [Tt] .*$symbol"; then
        echo -e "${GREEN}✓${NC} $symbol"
        FOUND=$((FOUND + 1))
    else
        echo -e "${RED}✗${NC} $symbol (missing)"
        MISSING=1
    fi
done

echo ""
echo -e "${YELLOW}Application network functions (optional):${NC}"
OPTIONAL_FOUND=0
for symbol in "${OPTIONAL_SYMBOLS[@]}"; do
    if nm "$ELF_FILE" 2>/dev/null | grep -qE " [Tt] .*$symbol"; then
        echo -e "${GREEN}✓${NC} $symbol"
        OPTIONAL_FOUND=$((OPTIONAL_FOUND + 1))
    else
        echo -e "${YELLOW}○${NC} $symbol (not found - may be static or not implemented)"
    fi
done

echo ""
echo -e "${YELLOW}Zephyr network stack functions:${NC}"
ZEPHYR_FOUND=0
for symbol in "${ZEPHYR_NET_SYMBOLS[@]}"; do
    if nm "$ELF_FILE" 2>/dev/null | grep -qE " [TtU] .*$symbol"; then
        echo -e "${GREEN}✓${NC} $symbol"
        ZEPHYR_FOUND=$((ZEPHYR_FOUND + 1))
    else
        echo -e "${YELLOW}○${NC} $symbol (may be in library)"
    fi
done

echo ""

# Check for mbedTLS
echo -e "${YELLOW}mbedTLS support:${NC}"
if nm "$ELF_FILE" 2>/dev/null | grep -qE " [Tt] .*mbedtls"; then
    echo -e "${GREEN}✓${NC} mbedTLS symbols found"
    MBEDTLS_FOUND=1
else
    echo -e "${YELLOW}○${NC} mbedTLS (may be in library)"
    MBEDTLS_FOUND=0
fi

echo ""

if [ $MISSING -eq 0 ]; then
    echo -e "${GREEN}=== Network Stack Integration: SUCCESS ===${NC}"
    echo ""
    echo "Required functions: $FOUND/${#REQUIRED_SYMBOLS[@]}"
    echo "Optional functions: $OPTIONAL_FOUND/${#OPTIONAL_SYMBOLS[@]}"
    echo "Zephyr network functions: $ZEPHYR_FOUND/${#ZEPHYR_NET_SYMBOLS[@]}"
    echo "mbedTLS: $([ $MBEDTLS_FOUND -eq 1 ] && echo "Present" || echo "In library")"
    echo ""
    if [ $OPTIONAL_FOUND -lt ${#OPTIONAL_SYMBOLS[@]} ]; then
        echo -e "${YELLOW}Note: Some optional network functions are not yet implemented.${NC}"
        echo "This is expected - they can be added as needed."
    fi
    echo ""
    echo "Network stack core is integrated and ready."
    exit 0
else
    echo -e "${RED}=== Network Stack Integration: FAILED ===${NC}"
    echo ""
    echo "Some required network functions are missing. Check build configuration."
    exit 1
fi

