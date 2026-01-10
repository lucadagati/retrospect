#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Compile WASM modules for testing

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Compiling WASM Modules ===${NC}"
echo ""

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

WASM_DIR="$PROJECT_ROOT/zephyr-app/examples/simple_wasm"
OUTPUT_DIR="$PROJECT_ROOT/wasm-binaries"
mkdir -p "$OUTPUT_DIR"

# Check for WABT
if ! command -v wat2wasm &> /dev/null; then
    echo -e "${YELLOW}WABT not found. Installing...${NC}"
    echo ""
    echo "Please install WABT:"
    echo "  sudo apt-get install wabt"
    echo ""
    echo "Or download from: https://github.com/WebAssembly/wabt"
    exit 1
fi

echo -e "${GREEN}✓${NC} WABT found"
echo ""

# Compile WAT to WASM
WAT_FILE="$WASM_DIR/test_simple.wat"
WASM_FILE="$OUTPUT_DIR/test_simple.wasm"

if [ ! -f "$WAT_FILE" ]; then
    echo -e "${RED}✗${NC} WAT file not found: $WAT_FILE"
    exit 1
fi

echo -e "${YELLOW}Compiling: $WAT_FILE${NC}"
wat2wasm "$WAT_FILE" -o "$WASM_FILE"

if [ -f "$WASM_FILE" ]; then
    SIZE=$(stat -c%s "$WASM_FILE")
    echo -e "${GREEN}✓${NC} Compiled: $WASM_FILE (${SIZE} bytes)"
else
    echo -e "${RED}✗${NC} Compilation failed"
    exit 1
fi

echo ""
echo -e "${GREEN}=== Compilation Complete ===${NC}"
echo ""
echo "WASM binary: $WASM_FILE"
echo ""
echo "To use in WAMR:"
echo "  1. Read WASM file into memory"
echo "  2. Call wamr_load_module(wasm_bytes, wasm_size)"
echo "  3. Call wamr_instantiate(module_id, stack_size, heap_size)"
echo "  4. Call wamr_call_function(instance_id, \"add\", 2, args)"

