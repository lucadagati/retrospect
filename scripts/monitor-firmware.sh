#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Monitor firmware serial output

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Monitoring Firmware Output ===${NC}"
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

BOARD="${1:-stm32f4_disco}"
BOARD_SHORT=$(echo "$BOARD" | sed 's/_disco//' | sed 's/dk_nrf52840//' | sed 's/dk//')
BUILD_DIR="$PROJECT_ROOT/zephyr-workspace/build/$BOARD_SHORT"

# Source environment
source .venv/bin/activate 2>/dev/null || true
source .env.zephyr 2>/dev/null || true

echo -e "${YELLOW}Board: $BOARD${NC}"
echo -e "${YELLOW}Build directory: $BUILD_DIR${NC}"
echo ""

# Check if west is available
if ! command -v west &> /dev/null; then
    echo -e "${RED}✗${NC} west not found. Install Zephyr SDK first."
    exit 1
fi

echo -e "${GREEN}Starting serial monitor...${NC}"
echo ""
echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
echo ""

# Use west attach for monitoring
cd "$PROJECT_ROOT/zephyr-workspace"
west attach --build-dir "build/$BOARD_SHORT" 2>&1 || {
    echo ""
    echo -e "${YELLOW}Alternative: Use screen or minicom${NC}"
    echo ""
    echo "Find serial port:"
    echo "  ls /dev/ttyACM* /dev/ttyUSB*"
    echo ""
    echo "Connect with screen:"
    echo "  screen /dev/ttyACM0 115200"
    echo ""
    echo "Or with minicom:"
    echo "  minicom -D /dev/ttyACM0 -b 115200"
}

