#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Setup WAMR Integration for Zephyr
# Clones and builds WAMR for embedded use

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== WAMR Integration Setup ===${NC}"
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

# WAMR directory
WAMR_DIR="$PROJECT_ROOT/wamr"

# Clone WAMR if not exists
if [ ! -d "$WAMR_DIR" ]; then
    echo -e "${YELLOW}⚠ Cloning WAMR repository...${NC}"
    git clone https://github.com/bytecodealliance/wasm-micro-runtime.git "$WAMR_DIR"
    echo -e "${GREEN}✓ WAMR cloned${NC}"
else
    echo -e "${GREEN}✓ WAMR repository found${NC}"
    cd "$WAMR_DIR"
    echo "Updating WAMR..."
    git pull
fi

# Build WAMR for embedded (interpreter mode)
cd "$WAMR_DIR"
BUILD_DIR="$WAMR_DIR/build-embedded"

if [ ! -d "$BUILD_DIR" ]; then
    echo -e "${YELLOW}⚠ Building WAMR for embedded...${NC}"
    mkdir -p "$BUILD_DIR"
    cd "$BUILD_DIR"
    
    # Configure for embedded (interpreter only, no AOT)
    cmake .. \
        -DWAMR_BUILD_INTERP=1 \
        -DWAMR_BUILD_AOT=0 \
        -DWAMR_BUILD_JIT=0 \
        -DWAMR_BUILD_FAST_JIT=0 \
        -DWAMR_BUILD_LIBC_BUILTIN=1 \
        -DWAMR_BUILD_LIBC_WASI=0 \
        -DWAMR_BUILD_MULTI_MODULE=1 \
        -DWAMR_BUILD_FAST_INTERP=1
    
    make -j$(nproc)
    echo -e "${GREEN}✓ WAMR built for embedded${NC}"
else
    echo -e "${GREEN}✓ WAMR already built${NC}"
fi

# Note: WAMR is now integrated directly via CMakeLists.txt in zephyr-app
# No need for separate library symlink

echo ""
echo -e "${GREEN}=== WAMR Integration Setup Complete ===${NC}"
echo ""
echo "WAMR library location: $BUILD_DIR"
echo "Include directory: $WAMR_DIR/core/iwasm/include"
echo ""
echo "Next steps:"
echo "  1. Update zephyr-app/CMakeLists.txt to link WAMR"
echo "  2. Update wamr_integration.c with WAMR API calls"

