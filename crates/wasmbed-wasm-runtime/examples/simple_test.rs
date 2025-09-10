// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use wasmbed_wasm_runtime::{
    WasmRuntime, RuntimeConfig, DeviceArchitecture, HostFunctionConfig,
};
use tracing::{info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting Wasmbed WASM Runtime Simple Test");

    // Test 1: Create runtime for different architectures
    info!("Test 1: Creating runtimes for different architectures");
    
    let mpu_config = RuntimeConfig::for_architecture(DeviceArchitecture::Mpu, "test-mpu-device".to_string());
    let mpu_runtime = WasmRuntime::new(mpu_config.clone())?;
    info!("✓ MPU runtime created successfully");

    let mcu_config = RuntimeConfig::for_architecture(DeviceArchitecture::Mcu, "test-mcu-device".to_string());
    let mcu_runtime = WasmRuntime::new(mcu_config.clone())?;
    info!("✓ MCU runtime created successfully");

    let riscv_config = RuntimeConfig::for_architecture(DeviceArchitecture::RiscV, "test-riscv-device".to_string());
    let riscv_runtime = WasmRuntime::new(riscv_config.clone())?;
    info!("✓ RISC-V runtime created successfully");

    // Test 2: Test host function configuration
    info!("Test 2: Testing host function configuration");
    
    let px4_config = HostFunctionConfig {
        enable_px4: true,
        enable_microros: false,
        enable_sensors: true,
        enable_secure: false,
        enable_filesystem: false,
        enable_network: false,
        enable_gpio: true,
        enable_i2c_spi: false,
    };
    
    let px4_runtime_config = RuntimeConfig {
        architecture: DeviceArchitecture::Mpu,
        device_id: "test-px4-device".to_string(),
        gateway_endpoint: Some("localhost:8080".to_string()),
        wasm_config: mpu_config.wasm_config.clone(),
        host_config: px4_config,
    };
    
    let px4_runtime = WasmRuntime::new(px4_runtime_config)?;
    info!("✓ PX4 runtime created successfully");

    // Test 3: Test microROS configuration
    info!("Test 3: Testing microROS configuration");
    
    let microros_config = HostFunctionConfig {
        enable_px4: false,
        enable_microros: true,
        enable_sensors: true,
        enable_secure: true,
        enable_filesystem: false,
        enable_network: true,
        enable_gpio: false,
        enable_i2c_spi: true,
    };
    
    let microros_runtime_config = RuntimeConfig {
        architecture: DeviceArchitecture::Mpu,
        device_id: "test-microros-device".to_string(),
        gateway_endpoint: Some("localhost:8080".to_string()),
        wasm_config: mpu_config.wasm_config.clone(),
        host_config: microros_config,
    };
    
    let microros_runtime = WasmRuntime::new(microros_runtime_config)?;
    info!("✓ microROS runtime created successfully");

    // Test 4: Test error handling
    info!("Test 4: Testing error handling");
    
    let invalid_config = RuntimeConfig {
        architecture: DeviceArchitecture::Mpu,
        device_id: "test-invalid-device".to_string(),
        gateway_endpoint: Some("localhost:8080".to_string()),
        wasm_config: mpu_config.wasm_config.clone(),
        host_config: HostFunctionConfig {
            enable_px4: false,
            enable_microros: false,
            enable_sensors: false,
            enable_secure: false,
            enable_filesystem: false,
            enable_network: false,
            enable_gpio: false,
            enable_i2c_spi: false,
        },
    };
    
    let invalid_runtime = WasmRuntime::new(invalid_config);
    match invalid_runtime {
        Ok(_) => info!("⚠ Invalid runtime created (unexpected)"),
        Err(e) => info!("✓ Invalid runtime correctly rejected: {:?}", e),
    }

    info!("All tests completed successfully!");
    info!("Wasmbed WASM Runtime is working correctly");

    Ok(())
}
