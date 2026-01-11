# Current Status - Wasmbed Platform

**Last Updated**: 2026-01-11

## Summary

The Wasmbed platform is operational with the following status:

### Working Components

1. **Kubernetes Infrastructure**
   - k3d cluster running
   - All services deployed (Gateway, API Server, Dashboard, Infrastructure)
   - CRDs (Device, Application, Gateway) registered
   - RBAC configured correctly

2. **Gateway Service**
   - HTTP API on port 8080
   - TLS server on port 8081
   - Device enrollment endpoint working
   - Certificate management functional

3. **API Server**
   - REST API on port 3001
   - Device management endpoints working
   - Renode container orchestration functional
   - Gateway endpoint resolution working (pod IP, not 127.0.0.1)

4. **Dashboard**
   - Web UI accessible on port 3000
   - Network topology visualization working
   - Infrastructure status correctly displayed
   - System health monitoring functional

5. **MCU Type Mapping**
   - `Nrf52840DK` correctly mapped to `nrf52840dk_nrf52840.repl`
   - `Stm32F4Disco` supported
   - Legacy types (`RenodeArduinoNano33Ble`, `Mps2An385`) still supported
   - Serialization/deserialization from CRD to in-memory working

6. **Renode Integration**
   - Renode containers start correctly
   - Firmware volumes mounted
   - Gateway endpoint written to memory (0x20001000)
   - Correct platform files loaded (e.g., `nrf52840dk_nrf52840.repl`)

7. **TLS Connection Logic**
   - Gateway endpoint resolution from pod IP working
   - No more `127.0.0.1` fallback
   - Endpoint correctly written to memory for Zephyr to read

### Known Issues

1. **Zephyr Firmware Execution in Renode**
   - **Status**: Zephyr boots but terminates after initialization
   - **Symptoms**: 
     - Renode process exits with status 256
     - No UART output visible
     - Machine pauses after ~5 seconds
   - **Root Cause**: 
     - nRF52840DK has no Ethernet in Renode
     - Zephyr network stack may crash without network interface
     - UART logging may be failing (writes to invalid addresses)
   - **Impact**: Cannot test TLS connection end-to-end

2. **Network Support in Renode**
   - **Issue**: Official Zephyr boards (nRF52840DK, STM32F4 Discovery) don't have Ethernet in Renode
   - **Workaround Needed**: 
     - Use boards with Ethernet support (e.g., STM32F7 Discovery, i.MX RT1064)
     - OR configure BLE/802.15.4 network in Renode (complex)
     - OR use QEMU instead of Renode for network testing

3. **UART Logging**
   - **Issue**: Zephyr UART writes cause UsageFault in Renode
   - **Symptoms**: `WriteByte to non existing peripheral at 0x8F691C9`
   - **Impact**: No console output from Zephyr
   - **Workaround**: Disable UART logging in Zephyr (`CONFIG_LOG=n`)

### In Progress

1. **Zephyr Firmware Compilation**
   - Need to compile Zephyr for a board with Ethernet support in Renode
   - Options:
     - `stm32f746g_disco` (STM32F7 Discovery with Ethernet)
     - `mimxrt1064_evk` (i.MX RT1064 EVK with Ethernet)
     - `frdm_k64f` (FRDM-K64F with Ethernet)

2. **Network Configuration**
   - Need to configure Ethernet in Renode for the chosen board
   - Update Renode script to enable network interface
   - Configure DHCP or static IP for Zephyr

### Next Steps

1. **Immediate (High Priority)**
   - [ ] Compile Zephyr firmware for a board with Ethernet support
   - [ ] Update `McuType` enum to include Ethernet-capable boards
   - [ ] Test Renode with Ethernet-enabled platform
   - [ ] Verify TLS connection from Zephyr to Gateway

2. **Short Term**
   - [ ] Fix UART logging in Zephyr/Renode
   - [ ] Add network configuration to Renode scripts
   - [ ] Test complete workflow: device enrollment → app deployment → execution

3. **Medium Term**
   - [ ] Support multiple network interfaces (Ethernet, BLE, 802.15.4)
   - [ ] Add real hardware support (not just emulation)
   - [ ] Implement device persistence across restarts

4. **Long Term**
   - [ ] Certificate rotation and proper CA management
   - [ ] Multi-gateway load balancing
   - [ ] Production-ready TLS configuration

## Architecture Decisions

### Why Official Zephyr Boards?

- **Reason**: Leverage official Zephyr firmware with full network stack, TLS, and WAMR
- **Benefit**: No need to write custom firmware or network drivers
- **Challenge**: Need to find boards with Ethernet support in Renode

### Why Direct TLS Connection?

- **Reason**: Eliminate TCP bridge complexity and make connections persistent
- **Benefit**: Simpler architecture, better security, production-ready
- **Implementation**: Gateway pod IP resolved dynamically, written to memory for Zephyr

### Why Renode?

- **Reason**: Best emulation platform for ARM Cortex-M devices
- **Benefit**: Accurate hardware emulation, UART analyzer, debugging support
- **Limitation**: Limited network support for some boards

## Testing Status

### Tested and Working

- [x] Kubernetes deployment
- [x] Gateway HTTP API
- [x] Gateway TLS server (port 8081)
- [x] API Server endpoints
- [x] Dashboard UI
- [x] Device CRD creation
- [x] Renode container startup
- [x] Firmware volume mounting
- [x] Gateway endpoint resolution
- [x] MCU type mapping (CRD → in-memory)

### Not Yet Tested

- [ ] Zephyr TLS connection to Gateway
- [ ] Device enrollment via TLS
- [ ] WebAssembly module deployment
- [ ] WAMR execution on device
- [ ] End-to-end workflow

## Documentation Status

### Updated Documentation

- [x] `ARCHITECTURE.md` - Complete system architecture
- [x] `TLS_CONNECTION.md` - TLS connection flow and implementation
- [x] `CURRENT_STATUS.md` - This file

### Needs Update

- [ ] `DEPLOYMENT.md` - Add troubleshooting for Zephyr/Renode
- [ ] `FIRMWARE.md` - Add instructions for Ethernet-capable boards
- [ ] `README.md` - Update with current status

## Known Workarounds

### 1. Firmware Not Copied to Volume

**Issue**: API Server pod doesn't have Docker socket access

**Workaround**:
```bash
# Manually copy firmware to volume
docker run --rm -v firmware-DEVICE_ID:/firmware \
    -v /path/to/zephyr.elf:/tmp/zephyr.elf:ro \
    alpine cp /tmp/zephyr.elf /firmware/
```

### 2. Renode Exits Immediately

**Issue**: Zephyr terminates after initialization

**Workaround**: Use a board with Ethernet support (see "Next Steps")

### 3. No UART Output

**Issue**: Zephyr UART writes cause faults

**Workaround**: Disable UART logging in `prj.conf`:
```
CONFIG_LOG=n
CONFIG_SERIAL=n
```

## Performance Metrics

- **Device Creation**: ~2-5 seconds
- **Renode Startup**: ~3-5 seconds
- **Zephyr Boot**: ~1-2 seconds (before termination)
- **Gateway Endpoint Resolution**: ~100-200ms

## Resource Usage

- **Kubernetes Cluster**: ~500MB RAM
- **Gateway Pod**: ~50MB RAM
- **API Server Pod**: ~100MB RAM
- **Dashboard Pod**: ~50MB RAM
- **Renode Container**: ~100-200MB RAM per device

## Contact & Support

For issues or questions:
- Check logs: `kubectl logs -n wasmbed POD_NAME`
- Check device status: `kubectl get devices -n wasmbed`
- Check Renode logs: `docker logs renode-DEVICE_ID`
