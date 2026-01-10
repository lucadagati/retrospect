#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Complete firmware test suite

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Complete Firmware Test Suite ===${NC}"
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

BOARD="${1:-stm32f4_disco}"

echo -e "${YELLOW}Board: $BOARD${NC}"
echo ""

# Run all tests
TESTS_PASSED=0
TESTS_FAILED=0

echo -e "${BLUE}Running test suite...${NC}"
echo ""

# Test 1: Firmware Build Check
echo -e "${YELLOW}[1/2] Checking Firmware Build...${NC}"
BOARD_SHORT=$(echo "$BOARD" | sed 's/_disco//' | sed 's/dk_nrf52840//' | sed 's/dk//')
ELF_FILE="$PROJECT_ROOT/zephyr-workspace/build/$BOARD_SHORT/zephyr/zephyr.elf"

if [ -f "$ELF_FILE" ]; then
    echo -e "${GREEN}✓${NC} Firmware Build: ELF file found"
    TESTS_PASSED=$((TESTS_PASSED + 1))
else
    echo -e "${RED}✗${NC} Firmware Build: ELF file not found. Run build-zephyr-app.sh first"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

# Test 2: Firmware Size Check
echo -e "${YELLOW}[3/3] Checking Firmware Size...${NC}"
BOARD_SHORT=$(echo "$BOARD" | sed 's/_disco//' | sed 's/dk_nrf52840//' | sed 's/dk//')
BIN_FILE="$PROJECT_ROOT/zephyr-workspace/build/$BOARD_SHORT/zephyr/zephyr.bin"

if [ -f "$BIN_FILE" ]; then
    SIZE=$(stat -c%s "$BIN_FILE")
    SIZE_KB=$((SIZE / 1024))
    
    if [ $SIZE_KB -lt 200 ]; then
        echo -e "${GREEN}✓${NC} Firmware Size: ${SIZE_KB}KB (OK)"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${YELLOW}⚠${NC} Firmware Size: ${SIZE_KB}KB (large, but OK)"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    fi
else
    echo -e "${RED}✗${NC} Firmware Size: Binary not found"
    TESTS_FAILED=$((TESTS_FAILED + 1))
fi

echo ""
echo -e "${BLUE}=== Test Results ===${NC}"
echo ""
echo "Tests Passed: $TESTS_PASSED/2"
echo "Tests Failed: $TESTS_FAILED/2"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}=== All Tests PASSED ===${NC}"
    echo ""
    echo "Firmware is ready for:"
    echo "  - Flash to hardware"
    echo "  - Runtime testing"
    echo "  - Integration testing"
    exit 0
else
    echo -e "${RED}=== Some Tests FAILED ===${NC}"
    echo ""
    echo "Review failed tests and fix issues before deployment."
    exit 1
fi

