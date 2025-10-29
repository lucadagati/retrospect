# Wasmbed Platform - Complete Demo Guide

This guide walks you through a complete demonstration of the Wasmbed platform with real Renode-emulated IoT devices running WASM applications.

## Overview

The demo creates:
- **1 Gateway** managing device connections and load balancing
- **3 Renode-emulated devices**:
  - Arduino Nano 33 BLE (nRF52840)
  - STM32F4 Discovery (STM32F407)
  - nRF52840 Development Kit
- **3 WASM Applications**:
  - LED Blinker (blinks LED on GPIO)
  - Temperature Monitor (reads ADC sensor)
  - Button Counter (counts button presses)

## Prerequisites

1. **Wasmbed Platform Deployed**
   ```bash
   ./scripts/06-master-control.sh deploy
   ```

2. **Optional: WABT Toolkit** (for WASM compilation)
   ```bash
   # Ubuntu/Debian
   sudo apt-get install wabt
   
   # Or build from source
   git clone --recursive https://github.com/WebAssembly/wabt
   cd wabt
   mkdir build && cd build
   cmake ..
   make
   sudo make install
   ```

## Quick Start

### 1. Setup Demo Environment

Run the automated setup script:

```bash
./scripts/08-setup-complete-demo.sh
```

This script will:
- Create 3 WASM application source files (.wat format)
- Compile them to WASM bytecode
- Create Kubernetes resources (Gateway, Devices, Applications)
- Generate Renode startup scripts

### 2. Verify Resources Created

```bash
# Check gateway
kubectl get gateways -n wasmbed

# Check devices
kubectl get devices -n wasmbed

# Check applications
kubectl get applications -n wasmbed
```

Expected output:
```
NAME              TYPE          ARCHITECTURE   STATUS
arduino-nano-ble-001   ARM_CORTEX_M   arm           Pending
stm32f4-discovery-001  ARM_CORTEX_M   arm           Pending
nrf52840-dk-001        ARM_CORTEX_M   arm           Pending
```

### 3. Start Renode Emulated Devices

Open **3 separate terminal windows** and start each device:

**Terminal 1 - Arduino Nano 33 BLE (LED Blinker):**
```bash
cd /home/lucadag/18_10_23_retrospect/retrospect
./renode_1.15.0_portable/renode renode-scripts/arduino_nano_ble.resc
```

**Terminal 2 - STM32F4 Discovery (Temperature Sensor):**
```bash
cd /home/lucadag/18_10_23_retrospect/retrospect
./renode_1.15.0_portable/renode renode-scripts/stm32f4_discovery.resc
```

**Terminal 3 - nRF52840 DK (Button Counter):**
```bash
cd /home/lucadag/18_10_23_retrospect/retrospect
./renode_1.15.0_portable/renode renode-scripts/nrf52840_dk.resc
```

### 4. Monitor Device Connection

The devices will connect to the gateway via UART serial ports (10001, 10002, 10003).

Check device enrollment:
```bash
kubectl get devices -n wasmbed -w
```

Wait until `STATUS` changes from `Pending` to `Enrolled`.

### 5. View in Dashboard

Open the Wasmbed dashboard:
```
http://localhost:3000
```

You should see:
- 3 connected devices
- 3 deployed applications
- Real-time metrics from emulated devices

### 6. Interact with Devices

In each Renode window, you can:

**Arduino Nano BLE (LED Blinker):**
- Watch the LED blink in the Renode GPIO monitor
- See logs in the UART analyzer

**STM32F4 Discovery (Temperature):**
- View temperature readings in the UART analyzer
- Readings come from the emulated ADC sensor

**nRF52840 DK (Button Counter):**
- Use Renode's GPIO to simulate button presses:
  ```
  (monitor) sysbus.gpio0.button PressAndRelease
  ```
- Watch the counter increment and LED toggle

## Architecture Details

### WASM Application Flow

1. **Compilation**: WAT (WebAssembly Text) → WASM bytecode
2. **Deployment**: Application controller sends WASM to gateway
3. **Distribution**: Gateway forwards WASM to target devices
4. **Execution**: Device runtime loads and executes WASM module
5. **Host Functions**: WASM calls imported functions (GPIO, ADC, UART)

### Device-Gateway Communication

```
┌─────────────────┐
│  Renode Device  │
│  (ARM Cortex-M) │
└────────┬────────┘
         │ UART Serial
         │ (localhost:10001-10003)
         ▼
┌─────────────────┐
│     Gateway     │
│  (Port 8080)    │
└────────┬────────┘
         │ WebSocket/HTTP
         ▼
┌─────────────────┐
│   Controllers   │
│   (Kubernetes)  │
└─────────────────┘
```

### Renode Platform Descriptions

Each device uses a `.repl` (Renode Platform) file that describes:
- CPU type and memory layout
- Peripheral addresses (GPIO, UART, ADC, etc.)
- Flash and RAM sizes
- Interrupt mappings

Located in: `renode_1.15.0_portable/platforms/boards/`

## WASM Application Details

### 1. LED Blinker (`led_blinker.wasm`)

**Source:** `wasm-apps/led_blinker.wat`

**Features:**
- Blinks GPIO pin 0 every 500ms
- Uses host functions: `led_set()`, `delay_ms()`, `log()`
- Runs for 100 iterations (50 seconds)

**Host Function Imports:**
```wat
(import "env" "led_set" (func $led_set (param i32 i32)))
(import "env" "delay_ms" (func $delay_ms (param i32)))
(import "env" "log" (func $log (param i32 i32)))
```

### 2. Temperature Monitor (`temp_sensor.wasm`)

**Source:** `wasm-apps/temp_sensor.wat`

**Features:**
- Reads ADC channel 0 every 2 seconds
- Converts raw ADC value to Celsius
- Outputs readings via UART
- Runs for 30 readings (1 minute)

**Host Function Imports:**
```wat
(import "env" "adc_read" (func $adc_read (param i32) (result i32)))
(import "env" "uart_write" (func $uart_write (param i32 i32)))
(import "env" "delay_ms" (func $delay_ms (param i32)))
```

### 3. Button Counter (`button_counter.wasm`)

**Source:** `wasm-apps/button_counter.wat`

**Features:**
- Monitors GPIO pin 2 for button presses
- Increments counter on each press
- Toggles LED on GPIO pin 0
- Debouncing with 50ms delay

**Host Function Imports:**
```wat
(import "env" "gpio_read" (func $gpio_read (param i32) (result i32)))
(import "env" "gpio_write" (func $gpio_write (param i32 i32)))
(import "env" "delay_ms" (func $delay_ms (param i32)))
(import "env" "log" (func $log (param i32 i32)))
```

## Troubleshooting

### Devices Not Connecting

**Check gateway is running:**
```bash
kubectl get pods -n wasmbed | grep gateway
```

**Check Renode serial ports are listening:**
```bash
netstat -an | grep "10001\|10002\|10003"
```

**View gateway logs:**
```bash
kubectl logs -n wasmbed -l app=wasmbed-gateway --tail=50
```

### WASM Applications Not Deploying

**Check application controller:**
```bash
kubectl get pods -n wasmbed | grep application-controller
kubectl logs -n wasmbed -l app=wasmbed-application-controller
```

**Verify WASM files exist:**
```bash
ls -lh wasm-apps/*.wasm
```

**Check application status:**
```bash
kubectl describe application led-blinker-app -n wasmbed
```

### Renode Issues

**Renode won't start:**
```bash
# Check Renode binary exists
ls -lh renode_1.15.0_portable/renode

# Make it executable if needed
chmod +x renode_1.15.0_portable/renode

# Check Mono is installed (required for Renode)
mono --version
```

**Platform file not found:**
```bash
# List available platforms
ls renode_1.15.0_portable/platforms/boards/*.repl

# Verify platform exists
grep -r "arduino_nano_33_ble" renode_1.15.0_portable/platforms/
```

## Advanced Usage

### Create Custom WASM Applications

1. **Write WAT source:**
   ```bash
   cat > wasm-apps/my_app.wat <<EOF
   (module
     (import "env" "log" (func $log (param i32 i32)))
     (memory (export "memory") 1)
     (data (i32.const 0) "Hello from WASM!")
     
     (func (export "main") (result i32)
       (call $log (i32.const 0) (i32.const 16))
       (i32.const 0)
     )
   )
   EOF
   ```

2. **Compile to WASM:**
   ```bash
   wat2wasm wasm-apps/my_app.wat -o wasm-apps/my_app.wasm
   ```

3. **Create Kubernetes Application:**
   ```bash
   kubectl apply -f - <<EOF
   apiVersion: wasmbed.github.io/v1
   kind: Application
   metadata:
     name: my-custom-app
     namespace: wasmbed
   spec:
     name: "My Custom App"
     version: "1.0.0"
     targetDevices: ["arduino-nano-ble-001"]
     wasmModule:
       source: "file://$(pwd)/wasm-apps/my_app.wasm"
     requiredCapabilities: []
     scheduling:
       startImmediately: true
   EOF
   ```

### Add More Devices

1. **Create device resource:**
   ```bash
   kubectl apply -f - <<EOF
   apiVersion: wasmbed.github.io/v1
   kind: Device
   metadata:
     name: stm32f0-discovery-001
     namespace: wasmbed
   spec:
     deviceId: "stm32f0-discovery-001"
     deviceType: "ARM_CORTEX_M"
     architecture: "arm"
     mcuType: "STM32F051"
     emulator:
       type: "renode"
       platform: "stm32f0_discovery"
       enabled: true
     gateway: "demo-gateway-1"
   EOF
   ```

2. **Create Renode script:**
   ```bash
   cat > renode-scripts/stm32f0_discovery.resc <<EOF
   using sysbus
   mach create "stm32f0-discovery-001"
   machine LoadPlatformDescription @platforms/boards/stm32f0_discovery.repl
   showAnalyzer sysbus.usart1
   connector Connect sysbus.usart1 127.0.0.1:10004
   start
   EOF
   ```

3. **Start Renode:**
   ```bash
   ./renode_1.15.0_portable/renode renode-scripts/stm32f0_discovery.resc
   ```

### Monitor Metrics

**View device metrics:**
```bash
kubectl get --raw "/apis/custom.metrics.k8s.io/v1beta1/namespaces/wasmbed/devices/*/cpu_usage"
```

**Gateway metrics:**
```bash
curl http://localhost:9090/metrics
```

## Cleanup

To remove all demo resources:

```bash
# Delete applications
kubectl delete applications -n wasmbed --all

# Delete devices
kubectl delete devices -n wasmbed --all

# Delete gateways
kubectl delete gateways -n wasmbed --all

# Stop Renode instances (Ctrl+C in each terminal)

# Remove WASM files
rm -rf wasm-apps/

# Remove Renode scripts
rm -rf renode-scripts/
```

## Next Steps

- **Scale Testing**: Add more devices and measure gateway performance
- **Custom Platforms**: Create `.repl` files for your own hardware
- **Complex Applications**: Build WASM apps with multiple modules
- **Monitoring**: Integrate Prometheus/Grafana for metrics visualization
- **CI/CD**: Automate WASM builds and deployments

## References

- [Renode Documentation](https://renode.readthedocs.io/)
- [WebAssembly Specification](https://webassembly.github.io/spec/)
- [WABT Tools](https://github.com/WebAssembly/wabt)
- [Kubernetes Custom Resources](https://kubernetes.io/docs/concepts/extend-kubernetes/api-extension/custom-resources/)
