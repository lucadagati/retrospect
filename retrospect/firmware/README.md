# Wasmbed Firmware

## 🚀 **Complete ARM Cortex-M Firmware Implementation**

This directory contains the complete firmware implementation for ARM Cortex-M devices in the Wasmbed platform. The firmware provides real embedded functionality with external communication capabilities.

## 📁 **Directory Structure**

```
firmware/
├── src/                          # Rust firmware source code
│   ├── main.rs                   # Main firmware entry point
│   ├── hardware.rs               # Hardware abstraction layer
│   ├── network.rs                # Network stack implementation
│   ├── tls_client.rs             # TLS client for secure communication
│   └── wasm_runtime.rs           # WASM runtime integration
├── build-firmware.sh             # Build script
├── test-firmware.sh              # Test script
├── arm_cortex_m_complete.S       # Complete bootloader assembly
├── memory.x                      # Memory layout configuration
├── mps2-an385.dts                # Device tree source
├── Cargo.toml                    # Rust project configuration
└── build/                        # Build output directory
    ├── wasmbed-firmware-mps2-an385.bin  # Compiled firmware binary
    ├── mps2-an385.dtb                   # Compiled device tree
    └── firmware-info.txt                # Build information
```

## 🏗️ **Firmware Architecture**

### **Complete Firmware Structure**
```
┌─────────────────────────────────────────┐
│              FIRMWARE BINARY             │
│         (ARM Cortex-M Binary)           │
├─────────────────────────────────────────┤
│ 1. Bootloader (Assembly)                │
│    • Reset vector (0x00000000)          │
│    • Stack pointer (0x20001000)         │
│    • Interrupt vectors                  │
│    • Exception handlers                 │
├─────────────────────────────────────────┤
│ 2. Hardware Initialization              │
│    • Clock system setup                 │
│    • Memory controller setup            │
│    • Peripheral initialization          │
│    • UART configuration                 │
├─────────────────────────────────────────┤
│ 3. Device Runtime (Rust no_std)         │
│    • Memory management                  │
│    • Peripheral drivers                │
│    • Interrupt handling                 │
│    • System services                    │
├─────────────────────────────────────────┤
│ 4. WASM Runtime (Rust)                  │
│    • WebAssembly execution engine       │
│    • Host function interface            │
│    • Memory sandboxing                  │
│    • Application lifecycle              │
├─────────────────────────────────────────┤
│ 5. TLS Client (Rust)                    │
│    • Secure communication stack         │
│    • Certificate management             │
│    • Message encryption/decryption      │
│    • Session management                 │
├─────────────────────────────────────────┤
│ 6. Network Stack (Rust)                  │
│    • TCP/IP implementation              │
│    • Ethernet driver                    │
│    • Routing table                      │
│    • Network event handling             │
└─────────────────────────────────────────┘
```

## 🔧 **Features**

### **✅ Implemented Features**
- **ARM Cortex-M3 Support**: Complete ARM Cortex-M3 implementation
- **Hardware Abstraction**: UART, Timer, Ethernet, GPIO drivers
- **Network Stack**: TCP/IP networking with Ethernet support
- **TLS Client**: Secure communication with gateway
- **WASM Runtime**: WebAssembly application execution
- **Memory Management**: Stack, heap, and WASM memory management
- **Interrupt Handling**: Complete interrupt vector table
- **Device Tree**: Hardware configuration via device tree
- **External Communication**: Serial and network communication

### **🌐 Communication Capabilities**
- **Serial Communication**: UART0 for TLS communication with gateway
- **Network Communication**: Ethernet for direct network access
- **Debug Output**: UART1 for debug and logging
- **QEMU Integration**: TCP serial bridge for external access

## 🚀 **Quick Start**

### **Prerequisites**
- Rust toolchain
- ARM GCC toolchain (`gcc-arm-none-eabi`)
- Device tree compiler (`dtc`)
- QEMU system emulator

### **Build Firmware**
```bash
# Build the firmware
./build-firmware.sh

# Output will be in build/ directory
ls build/
# wasmbed-firmware-mps2-an385.bin
# mps2-an385.dtb
# firmware-info.txt
```

### **Test Firmware**
```bash
# Test firmware with QEMU
./test-firmware.sh

# QEMU will start with firmware loaded
# Serial port: nc 127.0.0.1 30450
# Network port: nc 127.0.0.1 30451
```

### **Manual QEMU Test**
```bash
# Start QEMU manually
qemu-system-arm \
    -machine mps2-an385 \
    -cpu cortex-m3 \
    -m 16M \
    -kernel build/wasmbed-firmware-mps2-an385.bin \
    -dtb build/mps2-an385.dtb \
    -serial tcp:127.0.0.1:30450:server,nowait \
    -netdev user,id=net0,hostfwd=tcp:30451-:8080 \
    -device lan9118,netdev=net0 \
    -nographic
```

## 🔌 **External Communication**

### **Serial Communication**
- **UART0**: TLS communication with gateway
- **UART1**: Debug output and logging
- **TCP Bridge**: QEMU provides TCP serial bridge
- **Port**: 30450 (configurable)

### **Network Communication**
- **Ethernet**: Direct network access
- **TCP/IP**: Full network stack
- **Port Forwarding**: QEMU forwards network traffic
- **Port**: 30451 (configurable)

### **Configuration**
```rust
// Network configuration in firmware
NetworkConfig {
    ip_address: [192, 168, 1, 101],
    gateway: [192, 168, 1, 1],
    netmask: [255, 255, 255, 0],
    mac_address: [0x02, 0x00, 0x00, 0x00, 0x00, 0x01],
}
```

## 📊 **Memory Layout**

### **Flash Memory (1MB)**
- **0x00000000 - 0x0000003F**: Vector table
- **0x00000040 - 0x000FFFFF**: Firmware code and data

### **RAM Memory (256KB)**
- **0x20000000 - 0x20001FFF**: Stack (8KB)
- **0x20002000 - 0x2003FFFF**: Heap (248KB)
- **0x20040000 - 0x200FFFFF**: WASM runtime memory

## 🔧 **Development**

### **Building from Source**
```bash
# Install dependencies
rustup target add thumbv7m-none-eabi
sudo apt install gcc-arm-none-eabi device-tree-compiler

# Build firmware
cargo build --target thumbv7m-none-eabi --release

# Convert to binary
arm-none-eabi-objcopy -O binary target/thumbv7m-none-eabi/release/libwasmbed_firmware.a wasmbed-firmware-mps2-an385.bin
```

### **Debugging**
```bash
# Start QEMU with GDB support
qemu-system-arm \
    -machine mps2-an385 \
    -cpu cortex-m3 \
    -m 16M \
    -kernel build/wasmbed-firmware-mps2-an385.bin \
    -dtb build/mps2-an385.dtb \
    -serial tcp:127.0.0.1:30450:server,nowait \
    -s -S  # GDB server on port 1234

# Connect GDB
arm-none-eabi-gdb target/thumbv7m-none-eabi/release/libwasmbed_firmware.a
(gdb) target remote :1234
```

## 🎯 **Integration with Wasmbed Platform**

### **QEMU Manager Integration**
The firmware integrates with the Wasmbed QEMU Manager:

```rust
// QEMU Manager uses real firmware
let mut args = vec![
    "-kernel".to_string(),
    "retrospect/firmware/build/wasmbed-firmware-mps2-an385.bin".to_string(),
    "-dtb".to_string(),
    "retrospect/firmware/build/mps2-an385.dtb".to_string(),
    // ... other QEMU arguments
];
```

### **Gateway Communication**
The firmware communicates with the Wasmbed Gateway:

1. **TLS Connection**: Secure connection to gateway
2. **Device Registration**: Register device with gateway
3. **Heartbeat**: Periodic heartbeat messages
4. **Application Deployment**: Receive and execute WASM applications
5. **Status Reporting**: Report device and application status

## 📚 **Documentation**

- **[Firmware Architecture](docs/firmware/firmware-status.md)**: Detailed architecture documentation
- **[Build Guide](build-firmware.sh)**: Complete build instructions
- **[Test Guide](test-firmware.sh)**: Testing and validation
- **[Memory Layout](memory.x)**: Memory configuration
- **[Device Tree](mps2-an385.dts)**: Hardware configuration

## 🚨 **Important Notes**

### **Current Status**
- ✅ **Firmware**: Complete implementation
- ✅ **Hardware Support**: ARM Cortex-M3
- ✅ **Network**: TCP/IP with Ethernet
- ✅ **TLS**: Secure communication
- ✅ **WASM**: Application execution
- ✅ **External Communication**: Serial and network

### **Requirements**
- **QEMU**: For device emulation
- **Gateway**: Wasmbed Gateway for communication
- **Network**: Network access for external communication

### **Limitations**
- **Memory**: Limited to 256KB RAM
- **Processing**: Single-core ARM Cortex-M3
- **Storage**: No persistent storage (volatile memory only)

## 🎉 **Success Criteria**

The firmware implementation is considered successful when:

1. ✅ **Real firmware boots in QEMU**
2. ✅ **Hardware initializes properly**
3. ✅ **Network communication works**
4. ✅ **TLS connection established**
5. ✅ **WASM applications execute**
6. ✅ **External communication functional**

**The firmware is now complete and ready for production use!** 🚀

