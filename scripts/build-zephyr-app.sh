#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Build Zephyr application with WAMR and network stack

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Building Wasmbed Zephyr Application ===${NC}"
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# Check environment
if [ ! -f ".env.zephyr" ]; then
    echo -e "${RED}✗ .env.zephyr not found. Run setup-zephyr-workspace.sh first${NC}"
    exit 1
fi

# Source environment
source .venv/bin/activate 2>/dev/null || true
source .env.zephyr
export ZEPHYR_SDK_INSTALL_DIR="$PROJECT_ROOT/zephyr-sdk-0.16.5"

# Board selection
BOARD="${1:-stm32f4_disco}"
BUILD_DIR="$PROJECT_ROOT/zephyr-workspace/build/$BOARD"

echo -e "${YELLOW}Board: $BOARD${NC}"
echo -e "${YELLOW}Build directory: $BUILD_DIR${NC}"
echo ""

# Build
cd "$PROJECT_ROOT/zephyr-workspace"
west build -b "$BOARD" ../zephyr-app --build-dir "build/$BOARD" -G "Unix Makefiles"

echo ""
echo -e "${GREEN}=== Build Complete ===${NC}"
echo ""
echo "Firmware location:"
echo "  ELF: $BUILD_DIR/zephyr/zephyr.elf"
echo "  BIN: $BUILD_DIR/zephyr/zephyr.bin"
echo ""
echo "To flash:"
echo "  west flash --build-dir $BUILD_DIR"

