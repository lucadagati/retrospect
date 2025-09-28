# Wasmbed Configuration Management

## Overview

The Wasmbed platform now supports dynamic configuration management for all components including gateways, devices, and applications. This system allows for runtime configuration changes without requiring system restarts.

## Architecture

### Configuration Structure

The configuration is defined in YAML format and includes:

- **Platform metadata**: Name, version, description
- **Gateway configuration**: Dynamic gateway instances with TLS/HTTP ports, regions, device limits
- **Device configuration**: Device types (ESP32, STM32, RISC-V) with architecture-specific settings
- **Application configuration**: WASM applications with target device specifications
- **Infrastructure configuration**: CA, secret store, monitoring, database settings
- **Security configuration**: TLS, authentication, authorization policies
- **Performance configuration**: Connection pooling, caching, rate limiting
- **Development configuration**: Debug settings, mock data, profiling

### Key Features

1. **Dynamic Configuration**: Add/remove gateways, devices, and applications at runtime
2. **Validation**: Comprehensive validation with integrity checks
3. **CLI Management**: Command-line interface for configuration operations
4. **Real MCU Support**: ESP32 (xtensa), STM32 (arm), RISC-V (riscv64) architectures
5. **Type Safety**: Rust-based configuration with serde serialization

## Configuration File

### Location
- Default: `config/wasmbed-config.yaml`
- Customizable via `--config` parameter

### Structure

```yaml
# Platform metadata
platform:
  name: "Wasmbed"
  version: "1.0.0"
  description: "WebAssembly runtime platform for edge devices"

# Gateway configuration
gateways:
  defaults:
    max_devices: 100
    heartbeat_interval: 30
    connection_timeout: 60
    enrollment_timeout: 300
    tls_port: 30452
    http_port: 30453
    region: "us-west-1"
  
  instances:
    - name: "gateway-1"
      endpoint: "127.0.0.1:30452"
      tls_port: 30452
      http_port: 30453
      max_devices: 50
      region: "us-west-1"
      enabled: true

# Device configuration
devices:
  defaults:
    architecture: "xtensa"
    capabilities:
      - "wasm"
      - "tls"
      - "enrollment"
    heartbeat_interval: 30
    connection_retry_interval: 10
    max_retries: 5
  
  types:
    ESP32:
      architecture: "xtensa"
      memory: "520KB"
      storage: "4MB"
      capabilities:
        - "wasm"
        - "tls"
        - "enrollment"
        - "wifi"
        - "bluetooth"
        - "gpio"
        - "uart"
        - "spi"
        - "i2c"
        - "adc"
        - "dac"
        - "pwm"
    
    STM32:
      architecture: "arm"
      memory: "256KB"
      storage: "1MB"
      capabilities:
        - "wasm"
        - "tls"
        - "enrollment"
        - "gpio"
        - "uart"
        - "spi"
        - "i2c"
        - "adc"
        - "dac"
        - "pwm"
        - "can"
        - "ethernet"
    
    RISC-V:
      architecture: "riscv64"
      memory: "128KB"
      storage: "512KB"
      capabilities:
        - "wasm"
        - "tls"
        - "enrollment"
        - "gpio"
        - "uart"
        - "spi"
        - "i2c"
  
  instances:
    - name: "esp32-board-1"
      device_type: "ESP32"
      architecture: "xtensa"
      gateway: "gateway-1"
      enabled: true

# Application configuration
applications:
  defaults:
    max_size: "1MB"
    timeout: 300
    retries: 3
    validation:
      enabled: true
      strict: true
  
  instances:
    - name: "test-app-1"
      description: "Test Application 1"
      wasm_bytes: "base64-encoded-wasm-1"
      target_devices: ["esp32-board-1", "esp32-board-2"]
      config:
        message: "Test App 1"
        interval: 5000
      enabled: true
```

## CLI Tool

### Installation

```bash
cargo build --package wasmbed-config-manager
```

### Usage

#### Show Configuration
```bash
wasmbed-config show
```

#### Validate Configuration
```bash
wasmbed-config validate
```

#### Gateway Management

**List gateways:**
```bash
wasmbed-config gateway list
```

**Add gateway:**
```bash
wasmbed-config gateway add gateway-4 127.0.0.1:30458 \
  --tls-port 30458 \
  --http-port 30459 \
  --max-devices 25 \
  --region eu-central-1
```

**Update gateway:**
```bash
wasmbed-config gateway update gateway-4 \
  --endpoint 127.0.0.1:30460 \
  --max-devices 30
```

**Remove gateway:**
```bash
wasmbed-config gateway remove gateway-4
```

**Show gateway details:**
```bash
wasmbed-config gateway show gateway-1
```

#### Device Management

**List devices:**
```bash
wasmbed-config device list
```

**Add device:**
```bash
wasmbed-config device add esp32-board-3 ESP32 xtensa gateway-1
```

**Update device:**
```bash
wasmbed-config device update esp32-board-3 \
  --device-type STM32 \
  --architecture arm \
  --gateway gateway-2
```

**Remove device:**
```bash
wasmbed-config device remove esp32-board-3
```

**Show device details:**
```bash
wasmbed-config device show esp32-board-1
```

#### Application Management

**List applications:**
```bash
wasmbed-config application list
```

**Add application:**
```bash
wasmbed-config application add test-app-3 \
  "Test Application 3" \
  "dGVzdC13YXNt" \
  "esp32-board-1,stm32-board-1"
```

**Update application:**
```bash
wasmbed-config application update test-app-3 \
  --description "Updated Test Application 3" \
  --target-devices "esp32-board-1"
```

**Remove application:**
```bash
wasmbed-config application remove test-app-3
```

**Show application details:**
```bash
wasmbed-config application show test-app-1
```

## MCU Architecture Support

### ESP32 (xtensa)
- **Architecture**: Xtensa LX6 dual-core
- **Memory**: 520KB SRAM
- **Storage**: 4MB Flash
- **Capabilities**: WiFi, Bluetooth, GPIO, UART, SPI, I2C, ADC, DAC, PWM
- **Use Cases**: IoT sensors, smart home devices, industrial monitoring

### STM32 (arm)
- **Architecture**: ARM Cortex-M series
- **Memory**: 256KB SRAM
- **Storage**: 1MB Flash
- **Capabilities**: GPIO, UART, SPI, I2C, ADC, DAC, PWM, CAN, Ethernet
- **Use Cases**: Industrial control, automotive, embedded systems

### RISC-V (riscv64)
- **Architecture**: RISC-V 64-bit
- **Memory**: 128KB SRAM
- **Storage**: 512KB Flash
- **Capabilities**: GPIO, UART, SPI, I2C
- **Use Cases**: Research, prototyping, custom embedded systems

## Validation

The configuration system includes comprehensive validation:

1. **Field Validation**: Required fields, data types, value ranges
2. **Reference Validation**: Device-gateway relationships, application target devices
3. **Port Validation**: Unique port assignments, valid port ranges
4. **Architecture Validation**: Supported device types and architectures
5. **WASM Validation**: Base64 encoding, size limits

### Validation Examples

```bash
# Valid configuration
wasmbed-config validate
# ✅ Configuration is valid

# Invalid configuration (missing gateway reference)
wasmbed-config validate
# ❌ Device 'esp32-board-1' references non-existent gateway 'missing-gateway'
```

## Integration

### Rust Integration

```rust
use wasmbed_config::WasmbedConfig;

// Load configuration
let config = WasmbedConfig::load()?;

// Access gateway configurations
for gateway in &config.gateways.instances {
    println!("Gateway: {} at {}", gateway.name, gateway.endpoint);
}

// Access device configurations
for device in &config.devices.instances {
    println!("Device: {} ({})", device.name, device.device_type);
}

// Access application configurations
for app in &config.applications.instances {
    println!("Application: {} targeting {} devices", 
        app.name, app.target_devices.len());
}
```

### Dynamic Updates

The configuration system supports runtime updates:

```rust
// Add new gateway
let new_gateway = GatewayConfig {
    name: "gateway-4".to_string(),
    endpoint: "127.0.0.1:30458".to_string(),
    tls_port: 30458,
    http_port: 30459,
    max_devices: 25,
    region: "eu-central-1".to_string(),
    enabled: true,
};
config.add_gateway(new_gateway);

// Save updated configuration
config.to_file("config/wasmbed-config.yaml")?;
```

## Testing

### Unit Tests

```bash
cargo test --package wasmbed-config
cargo test --package wasmbed-config-manager
```

### Integration Tests

The CLI tool includes comprehensive integration tests:

- Gateway CRUD operations
- Device CRUD operations
- Application CRUD operations
- Configuration validation
- Error handling

### Test Coverage

- ✅ Configuration serialization/deserialization
- ✅ Gateway management operations
- ✅ Device management operations
- ✅ Application management operations
- ✅ Validation logic
- ✅ Error handling
- ✅ CLI argument parsing

## Best Practices

### Configuration Management

1. **Version Control**: Keep configuration files in version control
2. **Validation**: Always validate configuration before deployment
3. **Backup**: Backup configuration before major changes
4. **Testing**: Test configuration changes in development environment
5. **Documentation**: Document custom configurations and changes

### Device Types

1. **Architecture Selection**: Choose appropriate architecture for use case
2. **Capability Mapping**: Map device capabilities to application requirements
3. **Gateway Assignment**: Distribute devices across gateways for load balancing
4. **Naming Conventions**: Use consistent naming for devices and applications

### Security

1. **TLS Configuration**: Use strong TLS settings for production
2. **Authentication**: Enable authentication for production deployments
3. **Authorization**: Implement proper RBAC policies
4. **Certificate Management**: Regular certificate rotation

## Troubleshooting

### Common Issues

1. **Port Conflicts**: Ensure unique port assignments
2. **Gateway References**: Verify device gateway assignments exist
3. **Device References**: Check application target device references
4. **Architecture Mismatches**: Ensure device architecture matches type

### Debug Mode

Enable debug mode for detailed logging:

```yaml
development:
  debug: true
  verbose: true
```

### Validation Errors

Common validation errors and solutions:

- **Missing field**: Add required configuration field
- **Invalid reference**: Fix device/gateway/application references
- **Port conflict**: Change port to available port
- **Architecture mismatch**: Update device type or architecture

## Future Enhancements

1. **Hot Reload**: Configuration changes without restart
2. **Configuration Templates**: Predefined configuration templates
3. **Environment Variables**: Override configuration with environment variables
4. **Configuration Encryption**: Encrypt sensitive configuration data
5. **Multi-Environment**: Support for multiple environment configurations
6. **Configuration Diff**: Show differences between configurations
7. **Rollback Support**: Rollback to previous configuration versions
8. **Configuration Monitoring**: Monitor configuration changes and drift
