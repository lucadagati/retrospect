#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

echo "=== Wasmbed Firmware Build Script ==="

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

# Check if ARM GCC toolchain is installed
echo "2. Checking ARM GCC toolchain..."
if ! command -v arm-none-eabi-gcc &> /dev/null; then
    echo -e "${YELLOW}Installing ARM GCC toolchain...${NC}"
    if command -v apt &> /dev/null; then
        sudo apt update
        sudo apt install -y gcc-arm-none-eabi
    elif command -v brew &> /dev/null; then
        brew install arm-none-eabi-gcc
    else
        echo -e "${RED}Error: Cannot install ARM GCC toolchain. Please install manually.${NC}"
        exit 1
    fi
fi
echo -e "${GREEN}✓ ARM GCC toolchain ready${NC}"

# Check if device tree compiler is installed
echo "3. Checking device tree compiler..."
if ! command -v dtc &> /dev/null; then
    echo -e "${YELLOW}Installing device tree compiler...${NC}"
    if command -v apt &> /dev/null; then
        sudo apt install -y device-tree-compiler
    elif command -v brew &> /dev/null; then
        brew install dtc
    else
        echo -e "${RED}Error: Cannot install device tree compiler. Please install manually.${NC}"
        exit 1
    fi
fi
echo -e "${GREEN}✓ Device tree compiler ready${NC}"

# Create output directory
echo "4. Creating output directory..."
mkdir -p "$OUTPUT_DIR"
echo -e "${GREEN}✓ Output directory created${NC}"

# Compile device tree
echo "5. Compiling device tree..."
if [ -f "$FIRMWARE_DIR/mps2-an385.dts" ]; then
    dtc -I dts -O dtb -o "$OUTPUT_DIR/mps2-an385.dtb" "$FIRMWARE_DIR/mps2-an385.dts"
    echo -e "${GREEN}✓ Device tree compiled${NC}"
else
    echo -e "${YELLOW}⚠ Device tree source not found, skipping${NC}"
fi

# Compile firmware
echo "6. Compiling firmware..."
cd "$FIRMWARE_DIR"

# Set environment variables for embedded build
export RUSTFLAGS="-C link-arg=-Tmemory.x -C link-arg=-nostartfiles"

# Build firmware
if cargo build --target "$TARGET" --release; then
    echo -e "${GREEN}✓ Firmware compiled successfully${NC}"
else
    echo -e "${RED}✗ Firmware compilation failed${NC}"
    exit 1
fi

# Convert to binary
echo "7. Converting to binary..."
if [ -f "target/$TARGET/release/libwasmbed_firmware.a" ]; then
    arm-none-eabi-objcopy -O binary "target/$TARGET/release/libwasmbed_firmware.a" "$OUTPUT_DIR/wasmbed-firmware-mps2-an385.bin"
    echo -e "${GREEN}✓ Binary created${NC}"
else
    echo -e "${RED}✗ Binary conversion failed${NC}"
    exit 1
fi

# Create firmware info
echo "8. Creating firmware info..."
cat > "$OUTPUT_DIR/firmware-info.txt" << EOF
Wasmbed Firmware Build Information
==================================

Build Date: $(date)
Target: $TARGET
Firmware Size: $(stat -c%s "$OUTPUT_DIR/wasmbed-firmware-mps2-an385.bin") bytes
Device Tree: $(stat -c%s "$OUTPUT_DIR/mps2-an385.dtb") bytes

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
EOF

echo -e "${GREEN}✓ Firmware info created${NC}"

# Summary
echo ""
echo -e "${GREEN}=== BUILD COMPLETE ===${NC}"
echo "Output files:"
echo "  - Firmware: $OUTPUT_DIR/wasmbed-firmware-mps2-an385.bin"
echo "  - Device Tree: $OUTPUT_DIR/mps2-an385.dtb"
echo "  - Info: $OUTPUT_DIR/firmware-info.txt"
echo ""
echo "To use with QEMU:"
echo "  qemu-system-arm -machine mps2-an385 -cpu cortex-m3 -m 16M \\"
echo "    -kernel $OUTPUT_DIR/wasmbed-firmware-mps2-an385.bin \\"
echo "    -dtb $OUTPUT_DIR/mps2-an385.dtb \\"
echo "    -serial tcp:127.0.0.1:30450:server,nowait \\"
echo "    -netdev user,id=net0,hostfwd=tcp:30451-:8080 \\"
echo "    -device lan9118,netdev=net0 \\"
echo "    -nographic"
echo ""
echo -e "${GREEN}Firmware build completed successfully!${NC}"

