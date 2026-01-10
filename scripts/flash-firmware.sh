#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Flash firmware to STM32F4 Discovery board

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Flashing Wasmbed Firmware ===${NC}"
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Source environment
source .venv/bin/activate 2>/dev/null || true
source .env.zephyr

BOARD="${1:-stm32f4_disco}"
BUILD_DIR="$PROJECT_ROOT/zephyr-workspace/build/$BOARD"

if [ ! -d "$BUILD_DIR" ]; then
    echo -e "${RED}✗ Build directory not found: $BUILD_DIR${NC}"
    echo "Build the firmware first: ./scripts/build-zephyr-app.sh"
    exit 1
fi

echo -e "${YELLOW}Board: $BOARD${NC}"
echo -e "${YELLOW}Build directory: $BUILD_DIR${NC}"
echo ""

# Check if board is connected
echo -e "${YELLOW}Checking for connected board...${NC}"
if ! west flash --build-dir "$BUILD_DIR" --dry-run &>/dev/null; then
    echo -e "${RED}✗ Board not detected or west flash not configured${NC}"
    echo ""
    echo "Manual flash options:"
    echo "  1. Use ST-Link: st-flash write $BUILD_DIR/zephyr/zephyr.bin 0x8000000"
    echo "  2. Use OpenOCD: openocd -f ... -c 'program $BUILD_DIR/zephyr/zephyr.bin reset exit'"
    exit 1
fi

echo -e "${GREEN}Board detected. Flashing firmware...${NC}"
echo ""

# Flash firmware
cd "$PROJECT_ROOT/zephyr-workspace"
west flash --build-dir "build/$BOARD"

echo ""
echo -e "${GREEN}=== Flash Complete ===${NC}"
echo ""
echo "Firmware flashed to $BOARD"
echo "Monitor output: west attach --build-dir $BUILD_DIR"

