#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

echo "=== Wasmbed Firmware Build Script (Simplified) ==="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
TARGET="thumbv7m-none-eabi"
FIRMWARE_DIR="$(dirname "$0")"
OUTPUT_DIR="$FIRMWARE_DIR/build"

echo -e "${YELLOW}Building ARM Cortex-M firmware...${NC}"

# Check if Rust toolchain is installed
echo "1. Checking Rust toolchain..."
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Cargo not found. Please install Rust.${NC}"
    exit 1
fi

# Check if ARM target is installed
if ! rustup target list --installed | grep -q "$TARGET"; then
    echo -e "${YELLOW}Installing ARM Cortex-M target...${NC}"
    rustup target add "$TARGET"
fi
echo -e "${GREEN}✓ Rust toolchain ready${NC}"

# Create output directory
echo "2. Creating output directory..."
mkdir -p "$OUTPUT_DIR"
echo -e "${GREEN}✓ Output directory created${NC}"

# Compile firmware
echo "3. Compiling firmware..."
cd "$FIRMWARE_DIR"

# Set environment variables for embedded build
# export RUSTFLAGS="-C link-arg=-T$FIRMWARE_DIR/memory.x"

# Build firmware
if cargo build --target "$TARGET" --release; then
    echo -e "${GREEN}✓ Firmware compiled successfully${NC}"
else
    echo -e "${RED}✗ Firmware compilation failed${NC}"
    exit 1
fi

# Convert to binary
echo "4. Converting to binary..."
if [ -f "../target/$TARGET/release/wasmbed-firmware" ]; then
    arm-none-eabi-objcopy -O binary "../target/$TARGET/release/wasmbed-firmware" "$OUTPUT_DIR/wasmbed-firmware-mps2-an385.bin"
    echo -e "${GREEN}✓ Binary created${NC}"
else
    echo -e "${RED}✗ Binary conversion failed${NC}"
    exit 1
fi

# Create a simple device tree placeholder
echo "5. Creating device tree placeholder..."
cat > "$OUTPUT_DIR/mps2-an385.dtb" << 'EOF'
# Placeholder device tree
# In a real implementation, this would be compiled from mps2-an385.dts
EOF
echo -e "${GREEN}✓ Device tree placeholder created${NC}"

# Create firmware info
echo "6. Creating firmware info..."
cat > "$OUTPUT_DIR/firmware-info.txt" << EOF
Wasmbed Firmware Build Information
==================================

Build Date: $(date)
Target: $TARGET
Firmware Size: $(stat -c%s "$OUTPUT_DIR/wasmbed-firmware-mps2-an385.bin") bytes
Device Tree: Placeholder

Features:
- ARM Cortex-M3 support
- UART communication
- Ethernet networking
- TLS client
- WASM runtime
- Hardware abstraction

Memory Layout:
- Flash: 1MB (0x00000000 - 0x00100000)
- RAM: 256KB (0x20000000 - 0x20040000)
- Stack: 8KB
- Heap: 248KB

Network Configuration:
- IP: 192.168.1.101
- Gateway: 192.168.1.1
- Netmask: 255.255.255.0
- MAC: 02:00:00:00:00:01

Status: COMPILED SUCCESSFULLY
EOF

echo -e "${GREEN}✓ Firmware info created${NC}"

# Summary
echo ""
echo -e "${GREEN}=== BUILD COMPLETE ===${NC}"
echo "Output files:"
echo "  - Firmware: $OUTPUT_DIR/wasmbed-firmware-mps2-an385.bin"
echo "  - Device Tree: $OUTPUT_DIR/mps2-an385.dtb (placeholder)"
echo "  - Info: $OUTPUT_DIR/firmware-info.txt"
echo ""
echo "To use with QEMU:"
echo "  qemu-system-arm -machine mps2-an385 -cpu cortex-m3 -m 16M \\"
echo "    -kernel $OUTPUT_DIR/wasmbed-firmware-mps2-an385.bin \\"
echo "    -serial tcp:127.0.0.1:30450:server,nowait \\"
echo "    -netdev user,id=net0,hostfwd=tcp:30451-:8080 \\"
echo "    -device lan9118,netdev=net0 \\"
echo "    -nographic"
echo ""
echo -e "${GREEN}Firmware build completed successfully!${NC}"
