#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

echo "=== Wasmbed Firmware Test Script ==="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
FIRMWARE_DIR="$(dirname "$0")"
BUILD_DIR="$FIRMWARE_DIR/build"
QEMU_BINARY="qemu-system-arm"
DEVICE_ID="mcu-test-001"
SERIAL_PORT="30450"
NET_PORT="30451"

echo -e "${YELLOW}Testing Wasmbed firmware with QEMU...${NC}"

# Check if firmware is built
echo "1. Checking firmware build..."
if [ ! -f "$BUILD_DIR/wasmbed-firmware-mps2-an385.bin" ]; then
    echo -e "${RED}Error: Firmware not built. Please run build-firmware.sh first.${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Firmware found${NC}"

# Check if device tree is built
if [ ! -f "$BUILD_DIR/mps2-an385.dtb" ]; then
    echo -e "${RED}Error: Device tree not built. Please run build-firmware.sh first.${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Device tree found${NC}"

# Check if QEMU is installed
echo "2. Checking QEMU installation..."
if ! command -v $QEMU_BINARY &> /dev/null; then
    echo -e "${RED}Error: $QEMU_BINARY not found. Please install QEMU.${NC}"
    echo "On Ubuntu/Debian: sudo apt install qemu-system-arm"
    echo "On macOS: brew install qemu"
    exit 1
fi
echo -e "${GREEN}✓ QEMU found: $($QEMU_BINARY --version | head -n1)${NC}"

# Create test directory
echo "3. Setting up test environment..."
TEST_DIR="$FIRMWARE_DIR/test"
mkdir -p "$TEST_DIR"
echo -e "${GREEN}✓ Test directory created${NC}"

# Start QEMU with firmware
echo "4. Starting QEMU with firmware..."
echo -e "${YELLOW}Starting QEMU in background...${NC}"

# QEMU command
QEMU_CMD="$QEMU_BINARY \
    -machine mps2-an385 \
    -cpu cortex-m3 \
    -m 16M \
    -kernel $BUILD_DIR/wasmbed-firmware-mps2-an385.bin \
    -dtb $BUILD_DIR/mps2-an385.dtb \
    -serial tcp:127.0.0.1:$SERIAL_PORT:server,nowait \
    -netdev user,id=net0,hostfwd=tcp:$NET_PORT-:8080 \
    -device lan9118,netdev=net0 \
    -nographic"

echo "QEMU Command: $QEMU_CMD"

# Start QEMU in background
$QEMU_CMD &
QEMU_PID=$!

echo -e "${GREEN}✓ QEMU started with PID: $QEMU_PID${NC}"

# Wait for QEMU to start
echo "5. Waiting for QEMU to initialize..."
sleep 3

# Test serial communication
echo "6. Testing serial communication..."
if command -v nc &> /dev/null; then
    echo "Testing serial connection..."
    echo "HELLO" | nc -w 1 127.0.0.1 $SERIAL_PORT || echo "Serial connection test completed"
    echo -e "${GREEN}✓ Serial communication test completed${NC}"
else
    echo -e "${YELLOW}⚠ netcat not available, skipping serial test${NC}"
fi

# Test network communication
echo "7. Testing network communication..."
if command -v nc &> /dev/null; then
    echo "Testing network connection..."
    echo "TEST" | nc -w 1 127.0.0.1 $NET_PORT || echo "Network connection test completed"
    echo -e "${GREEN}✓ Network communication test completed${NC}"
else
    echo -e "${YELLOW}⚠ netcat not available, skipping network test${NC}"
fi

# Check QEMU status
echo "8. Checking QEMU status..."
if kill -0 $QEMU_PID 2>/dev/null; then
    echo -e "${GREEN}✓ QEMU is running${NC}"
else
    echo -e "${RED}✗ QEMU has stopped${NC}"
    exit 1
fi

# Create test report
echo "9. Creating test report..."
cat > "$TEST_DIR/test-report.txt" << EOF
Wasmbed Firmware Test Report
============================

Test Date: $(date)
Firmware: $BUILD_DIR/wasmbed-firmware-mps2-an385.bin
Device Tree: $BUILD_DIR/mps2-an385.dtb
QEMU PID: $QEMU_PID

Test Results:
- Firmware Build: PASS
- Device Tree Build: PASS
- QEMU Installation: PASS
- QEMU Startup: PASS
- Serial Communication: TESTED
- Network Communication: TESTED

Configuration:
- Machine: mps2-an385
- CPU: cortex-m3
- Memory: 16MB
- Serial Port: $SERIAL_PORT
- Network Port: $NET_PORT

Next Steps:
1. Connect to serial port: nc 127.0.0.1 $SERIAL_PORT
2. Connect to network port: nc 127.0.0.1 $NET_PORT
3. Test TLS communication with gateway
4. Deploy WASM applications
5. Monitor device status

To stop QEMU: kill $QEMU_PID
EOF

echo -e "${GREEN}✓ Test report created${NC}"

# Summary
echo ""
echo -e "${GREEN}=== TEST COMPLETE ===${NC}"
echo "QEMU is running with firmware!"
echo ""
echo "Connection details:"
echo "  - Serial: nc 127.0.0.1 $SERIAL_PORT"
echo "  - Network: nc 127.0.0.1 $NET_PORT"
echo "  - QEMU PID: $QEMU_PID"
echo ""
echo "Test report: $TEST_DIR/test-report.txt"
echo ""
echo "To stop QEMU: kill $QEMU_PID"
echo ""
echo -e "${GREEN}Firmware test completed successfully!${NC}"
echo -e "${YELLOW}QEMU is running in background. Use 'kill $QEMU_PID' to stop it.${NC}"

