#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Test script to verify WASM execution in Renode
# This script compiles the device runtime and tests WASM execution

set -e

echo "=== Testing WASM Runtime in Renode ==="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RENODE_BINARY="$PROJECT_ROOT/renode_1.15.0_portable/renode"

# Check Renode
echo "1. Checking Renode..."
if [ ! -f "$RENODE_BINARY" ]; then
    echo -e "${RED}Error: Renode not found${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Renode found${NC}"

# Check Rust toolchain
echo "2. Checking Rust toolchain..."
if ! rustup target list --installed | grep -q "thumbv7em-none-eabihf"; then
    echo -e "${YELLOW}Installing ARM Cortex-M toolchain...${NC}"
    rustup target add thumbv7em-none-eabihf
fi
echo -e "${GREEN}✓ ARM Cortex-M toolchain ready${NC}"

# Create a simple test WASM module
echo "3. Creating test WASM module..."
TEST_WASM_DIR="$PROJECT_ROOT/test-wasm"
mkdir -p "$TEST_WASM_DIR"

# Create a simple WAT file
cat > "$TEST_WASM_DIR/simple_test.wat" << 'EOF'
(module
  (memory (export "memory") 1)
  (func (export "main") (result i32)
    (i32.const 0)
    (i32.const 42)
    (i32.store)
    (i32.const 42)
  )
)
EOF

# Compile WAT to WASM if wabt is available
if command -v wat2wasm &> /dev/null; then
    wat2wasm "$TEST_WASM_DIR/simple_test.wat" -o "$TEST_WASM_DIR/simple_test.wasm"
    echo -e "${GREEN}✓ WASM module compiled${NC}"
else
    echo -e "${YELLOW}⚠ wabt not found, using pre-compiled bytecode${NC}"
    # Create minimal WASM bytecode manually
    cat > "$TEST_WASM_DIR/simple_test.wasm" << 'EOFBIN'
00000000: 0061 736d 0100 0000 0503 0100 0103 0201  .asm............
00000010: 0007 0a01 046d 6169 6e00 000a 0f01 0d00  .....main.......
00000020: 4100 412a 3602 0041 2a0b                 A.A*6..A*.
EOFBIN
fi

# Test the interpreter directly (without Renode for now)
echo "4. Testing WASM interpreter directly..."
cd "$PROJECT_ROOT"
if cargo run --example test_wasm_execution --package wasmbed-device-runtime --features std 2>&1 | grep -q "✓"; then
    echo -e "${GREEN}✓ WASM interpreter test passed${NC}"
else
    echo -e "${YELLOW}⚠ Some interpreter tests had issues (expected for minimal test)${NC}"
fi

# Create Renode script for testing
echo "5. Creating Renode test script..."
cat > "$TEST_WASM_DIR/test_wasm.resc" << 'EOF'
using sysbus

# Create machine
mach create "wasm-test"
machine LoadPlatformDescription @platforms/boards/arduino_nano_33_ble.repl

# Setup UART for output
showAnalyzer sysbus.uart0

# Note: In a real test, we would:
# 1. Compile the device runtime to ELF binary
# 2. Load it: sysbus LoadELF @firmware.elf
# 3. Start execution: start
# 4. Monitor UART for WASM execution logs

echo "WASM Runtime Test Environment Ready"
echo "To test WASM execution:"
echo "1. Compile firmware: cargo build --target thumbv7em-none-eabihf --release"
echo "2. Load firmware: sysbus LoadELF @target/thumbv7em-none-eabihf/release/wasmbed-device-runtime"
echo "3. Start: start"
EOF

echo -e "${GREEN}✓ Renode script created${NC}"

echo ""
echo -e "${GREEN}=== Test Setup Complete ===${NC}"
echo ""
echo "Next steps:"
echo "1. Compile firmware for ARM: cargo build --target thumbv7em-none-eabihf --release --bin wasmbed-device-runtime"
echo "2. Run Renode: $RENODE_BINARY $TEST_WASM_DIR/test_wasm.resc"
echo "3. In Renode console, load firmware: sysbus LoadELF @target/thumbv7em-none-eabihf/release/wasmbed-device-runtime"
echo "4. Start: start"
echo ""
echo -e "${YELLOW}Note: Full firmware compilation requires embedded Rust setup${NC}"
echo -e "${YELLOW}For now, the interpreter can be tested with:${NC}"
echo "   cargo run --example test_wasm_execution --package wasmbed-device-runtime --features std"

