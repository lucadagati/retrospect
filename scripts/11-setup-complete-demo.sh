#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

# Wasmbed Platform - Complete Demo Setup
# This script creates real Renode-emulated devices, gateways, and WASM applications

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    local status=$1
    local message=$2
    case $status in
        "SUCCESS")
            echo -e "${GREEN}✓ $message${NC}"
            ;;
        "ERROR")
            echo -e "${RED}✗ $message${NC}"
            ;;
        "WARNING")
            echo -e "${YELLOW}⚠ $message${NC}"
            ;;
        "INFO")
            echo -e "${BLUE}ℹ $message${NC}"
            ;;
    esac
}

print_header() {
    echo ""
    echo "========================================"
    echo "  $1"
    echo "========================================"
}

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RENODE_BINARY="$PROJECT_ROOT/renode_1.15.0_portable/renode"
WASM_DIR="$PROJECT_ROOT/wasm-apps"

print_header "Wasmbed Complete Demo Setup"

# Check if platform is deployed
print_status "INFO" "Checking if Wasmbed platform is deployed..."
if ! kubectl get namespace wasmbed &>/dev/null; then
    print_status "ERROR" "Wasmbed platform is not deployed. Run './scripts/06-master-control.sh deploy' first"
    exit 1
fi
print_status "SUCCESS" "Platform is deployed"

# Create WASM applications directory
mkdir -p "$WASM_DIR"
cd "$WASM_DIR"

print_header "Creating WASM Applications"

# 1. LED Blinker Application
print_status "INFO" "Creating LED blinker WASM application..."
cat > led_blinker.wat <<'EOF'
(module
  ;; Import host function to control LED
  (import "env" "led_set" (func $led_set (param i32 i32)))
  (import "env" "delay_ms" (func $delay_ms (param i32)))
  (import "env" "log" (func $log (param i32 i32)))
  
  ;; Memory for string data
  (memory (export "memory") 1)
  (data (i32.const 0) "LED Blinker started\n")
  (data (i32.const 20) "LED ON\n")
  (data (i32.const 27) "LED OFF\n")
  
  ;; Main function - blink LED forever
  (func (export "main") (result i32)
    (local $counter i32)
    
    ;; Log startup message
    (call $log (i32.const 0) (i32.const 20))
    
    ;; Blink loop
    (loop $blink_loop
      ;; Turn LED ON (pin 0, value 1)
      (call $led_set (i32.const 0) (i32.const 1))
      (call $log (i32.const 20) (i32.const 7))
      (call $delay_ms (i32.const 500))
      
      ;; Turn LED OFF (pin 0, value 0)
      (call $led_set (i32.const 0) (i32.const 0))
      (call $log (i32.const 27) (i32.const 8))
      (call $delay_ms (i32.const 500))
      
      ;; Increment counter
      (local.set $counter (i32.add (local.get $counter) (i32.const 1)))
      
      ;; Continue blinking (infinite loop for demo)
      (br_if $blink_loop (i32.lt_u (local.get $counter) (i32.const 100)))
    )
    
    (i32.const 0)
  )
)
EOF

# 2. Temperature Sensor Reader
print_status "INFO" "Creating temperature sensor WASM application..."
cat > temp_sensor.wat <<'EOF'
(module
  ;; Import host functions
  (import "env" "adc_read" (func $adc_read (param i32) (result i32)))
  (import "env" "uart_write" (func $uart_write (param i32 i32)))
  (import "env" "delay_ms" (func $delay_ms (param i32)))
  
  ;; Memory for data
  (memory (export "memory") 1)
  (data (i32.const 0) "Temperature Monitor\n")
  (data (i32.const 20) "Reading: ")
  
  ;; Convert temperature value to ASCII string
  (func $int_to_string (param $value i32) (param $buffer i32)
    (local $digit i32)
    
    ;; Simple conversion (2 digits)
    (local.set $digit (i32.div_u (local.get $value) (i32.const 10)))
    (i32.store8 (local.get $buffer) (i32.add (local.get $digit) (i32.const 48)))
    
    (local.set $digit (i32.rem_u (local.get $value) (i32.const 10)))
    (i32.store8 (i32.add (local.get $buffer) (i32.const 1)) 
                 (i32.add (local.get $digit) (i32.const 48)))
  )
  
  ;; Main function
  (func (export "main") (result i32)
    (local $temp i32)
    (local $counter i32)
    
    ;; Startup message
    (call $uart_write (i32.const 0) (i32.const 20))
    
    ;; Read temperature in loop
    (loop $read_loop
      ;; Read ADC channel 0 (temperature sensor)
      (local.set $temp (call $adc_read (i32.const 0)))
      
      ;; Convert to Celsius (simplified: raw_value / 10)
      (local.set $temp (i32.div_u (local.get $temp) (i32.const 10)))
      
      ;; Write "Reading: "
      (call $uart_write (i32.const 20) (i32.const 9))
      
      ;; Convert temp to string and write
      (call $int_to_string (local.get $temp) (i32.const 100))
      (call $uart_write (i32.const 100) (i32.const 2))
      
      ;; Wait 2 seconds
      (call $delay_ms (i32.const 2000))
      
      ;; Continue for 30 readings
      (local.set $counter (i32.add (local.get $counter) (i32.const 1)))
      (br_if $read_loop (i32.lt_u (local.get $counter) (i32.const 30)))
    )
    
    (i32.const 0)
  )
)
EOF

# 3. Button Counter Application
print_status "INFO" "Creating button counter WASM application..."
cat > button_counter.wat <<'EOF'
(module
  ;; Import host functions
  (import "env" "gpio_read" (func $gpio_read (param i32) (result i32)))
  (import "env" "gpio_write" (func $gpio_write (param i32 i32)))
  (import "env" "delay_ms" (func $delay_ms (param i32)))
  (import "env" "log" (func $log (param i32 i32)))
  
  ;; Memory
  (memory (export "memory") 1)
  (data (i32.const 0) "Button Counter App\n")
  (data (i32.const 20) "Button pressed! Count: ")
  
  ;; Global counter
  (global $press_count (mut i32) (i32.const 0))
  
  ;; Main function
  (func (export "main") (result i32)
    (local $button_state i32)
    (local $prev_state i32)
    (local $counter i32)
    
    ;; Startup
    (call $log (i32.const 0) (i32.const 19))
    
    ;; Monitor button (GPIO pin 2)
    (loop $monitor_loop
      (local.set $button_state (call $gpio_read (i32.const 2)))
      
      ;; Detect button press (transition from 0 to 1)
      (if (i32.and
            (i32.eq (local.get $button_state) (i32.const 1))
            (i32.eq (local.get $prev_state) (i32.const 0)))
        (then
          ;; Button pressed!
          (global.set $press_count 
            (i32.add (global.get $press_count) (i32.const 1)))
          
          ;; Log press count
          (call $log (i32.const 20) (i32.const 23))
          
          ;; Toggle LED (pin 0)
          (call $gpio_write (i32.const 0) (global.get $press_count))
        )
      )
      
      (local.set $prev_state (local.get $button_state))
      (call $delay_ms (i32.const 50))
      
      ;; Run for a while
      (local.set $counter (i32.add (local.get $counter) (i32.const 1)))
      (br_if $monitor_loop (i32.lt_u (local.get $counter) (i32.const 2000)))
    )
    
    (i32.const 0)
  )
)
EOF

# Compile WAT to WASM
print_status "INFO" "Compiling WAT to WASM..."
if command -v wat2wasm &>/dev/null; then
    wat2wasm led_blinker.wat -o led_blinker.wasm
    wat2wasm temp_sensor.wat -o temp_sensor.wasm
    wat2wasm button_counter.wat -o button_counter.wasm
    print_status "SUCCESS" "WASM applications compiled"
else
    print_status "WARNING" "wat2wasm not found. Install wabt toolkit: https://github.com/WebAssembly/wabt"
    print_status "INFO" "Creating placeholder WASM files for now..."
    # Create minimal valid WASM files
    echo -ne '\x00asm\x01\x00\x00\x00' > led_blinker.wasm
    echo -ne '\x00asm\x01\x00\x00\x00' > temp_sensor.wasm
    echo -ne '\x00asm\x01\x00\x00\x00' > button_counter.wasm
fi

cd "$PROJECT_ROOT"

print_header "Creating Kubernetes Resources"

# Create Gateway resource
print_status "INFO" "Creating gateway resource..."
cat > /tmp/demo-gateway.yaml <<EOF
apiVersion: wasmbed.io/v1
kind: Gateway
metadata:
  name: demo-gateway-1
  namespace: wasmbed
spec:
  endpoint: "http://localhost:8080"
  port: 8080
  tlsPort: 8081
  maxDevices: 50
  region: "local"
  capabilities:
    - "wasm-runtime"
    - "gpio"
    - "uart"
    - "adc"
  config:
    heartbeatInterval: "30s"
    connectionTimeout: "10m"
    enrollmentTimeout: "5m"
  resources:
    cpu: "2"
    memory: "4Gi"
    storage: "10Gi"
EOF

kubectl apply -f /tmp/demo-gateway.yaml
print_status "SUCCESS" "Gateway created"

# Create Renode-emulated devices
print_status "INFO" "Creating Renode-emulated devices..."

# Device 1: Arduino Nano 33 BLE (LED Blinker)
cat > /tmp/device-arduino-nano-1.yaml <<EOF
apiVersion: wasmbed.github.io/v0
kind: Device
metadata:
  name: arduino-nano-ble-001
  namespace: wasmbed
spec:
  architecture: "ARM_CORTEX_M"
  deviceType: "MCU"
  mcuType: "Mps2An385"
  publicKey: "-----BEGIN PUBLIC KEY-----\nMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEDemo1111111111111111111111\n-----END PUBLIC KEY-----"
EOF

kubectl apply -f /tmp/device-arduino-nano-1.yaml

# Device 2: STM32F4 Discovery (Temperature Sensor)
cat > /tmp/device-stm32f4-1.yaml <<EOF
apiVersion: wasmbed.github.io/v0
kind: Device
metadata:
  name: stm32f4-discovery-001
  namespace: wasmbed
spec:
  architecture: "ARM_CORTEX_M"
  deviceType: "MCU"
  mcuType: "Stm32Vldiscovery"
  publicKey: "-----BEGIN PUBLIC KEY-----\nMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEDemo2222222222222222222222\n-----END PUBLIC KEY-----"
EOF

kubectl apply -f /tmp/device-stm32f4-1.yaml

# Device 3: nRF52840 DK (Button Counter)
cat > /tmp/device-nrf52840-1.yaml <<EOF
apiVersion: wasmbed.github.io/v0
kind: Device
metadata:
  name: nrf52840-dk-001
  namespace: wasmbed
spec:
  architecture: "ARM_CORTEX_M"
  deviceType: "MCU"
  mcuType: "OlimexStm32H405"
  publicKey: "-----BEGIN PUBLIC KEY-----\nMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEDemo3333333333333333333333\n-----END PUBLIC KEY-----"
EOF

kubectl apply -f /tmp/device-nrf52840-1.yaml

print_status "SUCCESS" "3 Renode-emulated devices created"

# Create WASM Applications
print_status "INFO" "Creating WASM application deployments..."

# Encode WASM files to base64
LED_WASM_B64=$(base64 -w 0 "$WASM_DIR/led_blinker.wasm")
TEMP_WASM_B64=$(base64 -w 0 "$WASM_DIR/temp_sensor.wasm")
BUTTON_WASM_B64=$(base64 -w 0 "$WASM_DIR/button_counter.wasm")

# Application 1: LED Blinker
cat > /tmp/app-led-blinker.yaml <<EOF
apiVersion: wasmbed.github.io/v1alpha1
kind: Application
metadata:
  name: led-blinker-app
  namespace: wasmbed
spec:
  name: "LED Blinker"
  description: "Blinks an LED on GPIO pin 0"
  wasmBytes: "$LED_WASM_B64"
  targetDevices:
    deviceNames:
      - "arduino-nano-ble-001"
EOF

kubectl apply -f /tmp/app-led-blinker.yaml

# Application 2: Temperature Sensor
cat > /tmp/app-temp-sensor.yaml <<EOF
apiVersion: wasmbed.github.io/v1alpha1
kind: Application
metadata:
  name: temp-sensor-app
  namespace: wasmbed
spec:
  name: "Temperature Monitor"
  description: "Reads temperature from ADC and logs via UART"
  wasmBytes: "$TEMP_WASM_B64"
  targetDevices:
    deviceNames:
      - "stm32f4-discovery-001"
EOF

kubectl apply -f /tmp/app-temp-sensor.yaml

# Application 3: Button Counter
cat > /tmp/app-button-counter.yaml <<EOF
apiVersion: wasmbed.github.io/v1alpha1
kind: Application
metadata:
  name: button-counter-app
  namespace: wasmbed
spec:
  name: "Button Counter"
  description: "Counts button presses and toggles LED"
  wasmBytes: "$BUTTON_WASM_B64"
  targetDevices:
    deviceNames:
      - "nrf52840-dk-001"
EOF

kubectl apply -f /tmp/app-button-counter.yaml

print_status "SUCCESS" "3 WASM applications created"

print_header "Creating Renode Startup Scripts"

# Create Renode scripts directory
mkdir -p "$PROJECT_ROOT/renode-scripts"

# Arduino Nano 33 BLE script
cat > "$PROJECT_ROOT/renode-scripts/arduino_nano_ble.resc" <<'EOF'
using sysbus

# Load Arduino Nano 33 BLE platform
mach create "arduino-nano-ble-001"
machine LoadPlatformDescription @platforms/boards/arduino_nano_33_ble.repl

# Setup serial connection
showAnalyzer sysbus.uart0

# Setup GPIO for LED
sysbus.gpio0 LED @ sysbus.gpio0 0

# Connect to Wasmbed gateway via UART
connector Connect sysbus.uart0 127.0.0.1:10001

# Start emulation
start
EOF

# STM32F4 Discovery script
cat > "$PROJECT_ROOT/renode-scripts/stm32f4_discovery.resc" <<'EOF'
using sysbus

# Load STM32F4 Discovery platform
mach create "stm32f4-discovery-001"
machine LoadPlatformDescription @platforms/boards/stm32f4_discovery.repl

# Setup serial connection
showAnalyzer sysbus.usart2

# Setup ADC for temperature sensor
sysbus.adc1 TemperatureSensor @ sysbus.adc1 0

# Connect to Wasmbed gateway
connector Connect sysbus.usart2 127.0.0.1:10002

# Start emulation
start
EOF

# nRF52840 DK script
cat > "$PROJECT_ROOT/renode-scripts/nrf52840_dk.resc" <<'EOF'
using sysbus

# Load nRF52840 DK platform
mach create "nrf52840-dk-001"
machine LoadPlatformDescription @platforms/boards/nrf52840dk_nrf52840.repl

# Setup serial connection
showAnalyzer sysbus.uart0

# Setup GPIO for button and LED
sysbus.gpio0 Button @ sysbus.gpio0 2
sysbus.gpio0 LED @ sysbus.gpio0 0

# Connect to Wasmbed gateway
connector Connect sysbus.uart0 127.0.0.1:10003

# Start emulation
start
EOF

print_status "SUCCESS" "Renode scripts created"

print_header "Demo Setup Complete!"

echo ""
print_status "INFO" "Created Resources:"
echo "  Gateways: 1 (demo-gateway-1)"
echo "  Devices: 3 (Arduino Nano BLE, STM32F4 Discovery, nRF52840 DK)"
echo "  Applications: 3 (LED Blinker, Temperature Monitor, Button Counter)"
echo ""

print_status "INFO" "WASM Applications located at:"
echo "  $WASM_DIR/"
echo ""

print_status "INFO" "Renode Scripts located at:"
echo "  $PROJECT_ROOT/renode-scripts/"
echo ""

print_status "INFO" "To start Renode emulated devices:"
echo "  # Terminal 1 - Arduino Nano BLE:"
echo "  $RENODE_BINARY renode-scripts/arduino_nano_ble.resc"
echo ""
echo "  # Terminal 2 - STM32F4 Discovery:"
echo "  $RENODE_BINARY renode-scripts/stm32f4_discovery.resc"
echo ""
echo "  # Terminal 3 - nRF52840 DK:"
echo "  $RENODE_BINARY renode-scripts/nrf52840_dk.resc"
echo ""

print_status "INFO" "Check deployment status:"
echo "  kubectl get devices -n wasmbed"
echo "  kubectl get applications -n wasmbed"
echo "  kubectl get gateways -n wasmbed"
echo ""

print_status "INFO" "View in Dashboard:"
echo "  http://localhost:3000"
echo ""

print_status "SUCCESS" "Demo environment ready! Start Renode instances to see devices connect."
