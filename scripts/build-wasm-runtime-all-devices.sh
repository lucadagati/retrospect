#!/bin/bash
# Build WASM runtime for all supported devices

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

echo "=== Building WASM Runtime for all devices ==="
echo ""

# Build for STM32F4 Discovery
echo "1. Building for STM32F4 Discovery..."
LINKER_SCRIPT=memory_stm32f4.x cargo build \
    --target thumbv7em-none-eabihf \
    --package wasmbed-device-runtime \
    --no-default-features \
    --release \
    --target-dir target/stm32f4

# Copy to standard location for Renode scripts
mkdir -p target/thumbv7em-none-eabihf/release
cp target/stm32f4/thumbv7em-none-eabihf/release/wasmbed-device-runtime \
   target/thumbv7em-none-eabihf/release/wasmbed-device-runtime-stm32f4

echo "✅ STM32F4 build complete"
echo ""

# Build for nRF52840 (Arduino Nano 33 BLE and nRF52840 DK)
echo "2. Building for nRF52840 (Arduino Nano 33 BLE / nRF52840 DK)..."
LINKER_SCRIPT=memory_nrf52840.x cargo build \
    --target thumbv7em-none-eabihf \
    --package wasmbed-device-runtime \
    --no-default-features \
    --release \
    --target-dir target/nrf52840

# Copy to standard location for Renode scripts
cp target/nrf52840/thumbv7em-none-eabihf/release/wasmbed-device-runtime \
   target/thumbv7em-none-eabihf/release/wasmbed-device-runtime-nrf52840

echo "✅ nRF52840 build complete"
echo ""

echo "=== Build Summary ==="
echo "STM32F4:     target/thumbv7em-none-eabihf/release/wasmbed-device-runtime-stm32f4"
echo "nRF52840:    target/thumbv7em-none-eabihf/release/wasmbed-device-runtime-nrf52840"
echo ""
echo "To test in Renode:"
echo "  STM32F4:     ./renode-scripts/test_wasm_stm32f4.resc"
echo "  Arduino:     ./renode-scripts/test_wasm_arduino_nano.resc (use nrf52840 binary)"
echo "  nRF52840 DK: ./renode-scripts/test_wasm_nrf52840.resc (use nrf52840 binary)"

