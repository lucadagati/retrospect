// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use wasmbed_qemu_serial_bridge::{QemuDeviceManager, QemuDeviceConfig, QemuDeviceType, FirmwareManager};
use std::path::PathBuf;
use std::collections::HashMap;
use tracing::{info, error, warn};
use tokio::time::{sleep, Duration};

/// Complete QEMU integration example demonstrating the full system
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting Wasmbed QEMU Integration Example");
    
    // Create firmware manager
    let storage_dir = PathBuf::from("/tmp/wasmbed-firmware");
    let mut firmware_manager = FirmwareManager::new(storage_dir)?;
    
    // Create QEMU device manager
    let qemu_manager = QemuDeviceManager::new();
    
    // Add sample firmware images
    info!("Adding sample firmware images...");
    
    // RISC-V firmware
    let riscv_firmware_data = b"riscv_firmware_binary_data";
    let mut riscv_metadata = HashMap::new();
    riscv_metadata.insert("description".to_string(), "RISC-V SiFive HiFive1 firmware".to_string());
    riscv_metadata.insert("author".to_string(), "Wasmbed Team".to_string());
    
    firmware_manager.add_firmware_image(
        "riscv-hifive1-v1.0".to_string(),
        "RISC-V HiFive1 Firmware".to_string(),
        "riscv".to_string(),
        "riscv32imac".to_string(),
        "1.0.0".to_string(),
        riscv_firmware_data,
        riscv_metadata,
    ).await?;
    
    // ARM Cortex-M firmware
    let arm_firmware_data = b"arm_cortex_m_firmware_binary_data";
    let mut arm_metadata = HashMap::new();
    arm_metadata.insert("description".to_string(), "ARM Cortex-M STM32 firmware".to_string());
    arm_metadata.insert("author".to_string(), "Wasmbed Team".to_string());
    
    firmware_manager.add_firmware_image(
        "arm-stm32-v1.0".to_string(),
        "ARM STM32 Firmware".to_string(),
        "arm".to_string(),
        "armv7m".to_string(),
        "1.0.0".to_string(),
        arm_firmware_data,
        arm_metadata,
    ).await?;
    
    // ESP32 firmware
    let esp32_firmware_data = b"esp32_firmware_binary_data";
    let mut esp32_metadata = HashMap::new();
    esp32_metadata.insert("description".to_string(), "ESP32 Xtensa firmware".to_string());
    esp32_metadata.insert("author".to_string(), "Wasmbed Team".to_string());
    
    firmware_manager.add_firmware_image(
        "esp32-v1.0".to_string(),
        "ESP32 Firmware".to_string(),
        "esp32".to_string(),
        "xtensa".to_string(),
        "1.0.0".to_string(),
        esp32_firmware_data,
        esp32_metadata,
    ).await?;
    
    info!("Added {} firmware images", firmware_manager.list_firmware_images().len());
    
    // Add QEMU devices
    info!("Adding QEMU devices...");
    
    // RISC-V device
    let riscv_config = QemuDeviceConfig::riscv_default(
        "riscv-device-1".to_string(),
        "/tmp/riscv-firmware.bin".to_string(),
    );
    qemu_manager.add_device(riscv_config).await?;
    
    // ARM Cortex-M device
    let arm_config = QemuDeviceConfig::arm_cortex_m_default(
        "arm-device-1".to_string(),
        "/tmp/arm-firmware.bin".to_string(),
    );
    qemu_manager.add_device(arm_config).await?;
    
    // ESP32 device
    let esp32_config = QemuDeviceConfig::esp32_default(
        "esp32-device-1".to_string(),
        "/tmp/esp32-firmware.bin".to_string(),
    );
    qemu_manager.add_device(esp32_config).await?;
    
    info!("Added {} QEMU devices", qemu_manager.device_count());
    
    // Deploy firmware to devices
    info!("Deploying firmware to devices...");
    
    // Deploy RISC-V firmware
    let riscv_deployment_id = firmware_manager.deploy_firmware(
        "riscv-device-1".to_string(),
        "riscv-hifive1-v1.0".to_string(),
    ).await?;
    
    // Deploy ARM firmware
    let arm_deployment_id = firmware_manager.deploy_firmware(
        "arm-device-1".to_string(),
        "arm-stm32-v1.0".to_string(),
    ).await?;
    
    // Deploy ESP32 firmware
    let esp32_deployment_id = firmware_manager.deploy_firmware(
        "esp32-device-1".to_string(),
        "esp32-v1.0".to_string(),
    ).await?;
    
    info!("Started {} firmware deployments", firmware_manager.list_deployments().len());
    
    // Monitor deployments
    info!("Monitoring firmware deployments...");
    for _ in 0..10 {
        // Check deployment status
        if let Some(deployment) = firmware_manager.get_deployment_status(&riscv_deployment_id) {
            info!("RISC-V deployment status: {:?} ({}%)", deployment.status, deployment.progress);
        }
        
        if let Some(deployment) = firmware_manager.get_deployment_status(&arm_deployment_id) {
            info!("ARM deployment status: {:?} ({}%)", deployment.status, deployment.progress);
        }
        
        if let Some(deployment) = firmware_manager.get_deployment_status(&esp32_deployment_id) {
            info!("ESP32 deployment status: {:?} ({}%)", deployment.status, deployment.progress);
        }
        
        sleep(Duration::from_millis(500)).await;
    }
    
    // Update device heartbeats
    info!("Updating device heartbeats...");
    for device_id in qemu_manager.list_devices() {
        qemu_manager.update_heartbeat(&device_id)?;
        qemu_manager.set_serial_connected(&device_id, true)?;
        info!("Updated heartbeat for device: {}", device_id);
    }
    
    // Show device statuses
    info!("Device statuses:");
    for (device_id, status) in qemu_manager.get_all_device_statuses() {
        info!("Device {}: {:?}", device_id, status);
    }
    
    // Show firmware images
    info!("Available firmware images:");
    for image in firmware_manager.list_firmware_images() {
        info!("Firmware: {} ({}) - {} - {} bytes", 
              image.name, image.id, image.architecture, image.file_size);
    }
    
    // Show deployments
    info!("Firmware deployments:");
    for deployment in firmware_manager.list_deployments() {
        info!("Deployment {}: {} -> {} ({:?})", 
              deployment.id, deployment.firmware_id, deployment.device_id, deployment.status);
    }
    
    // Verify firmware integrity
    info!("Verifying firmware integrity...");
    for image in firmware_manager.list_firmware_images() {
        let is_valid = firmware_manager.verify_firmware(&image.id).await?;
        info!("Firmware {} integrity: {}", image.id, if is_valid { "VALID" } else { "INVALID" });
    }
    
    // Simulate device operation
    info!("Simulating device operation...");
    for i in 1..=5 {
        info!("Simulation cycle {}", i);
        
        // Update heartbeats
        for device_id in qemu_manager.list_devices() {
            qemu_manager.update_heartbeat(&device_id)?;
        }
        
        // Check device statuses
        for (device_id, status) in qemu_manager.get_all_device_statuses() {
            info!("Device {} status: {:?}", device_id, status);
        }
        
        sleep(Duration::from_secs(2)).await;
    }
    
    // Cleanup
    info!("Cleaning up...");
    
    // Stop all devices
    for device_id in qemu_manager.list_devices() {
        qemu_manager.stop_device(&device_id).await?;
        info!("Stopped device: {}", device_id);
    }
    
    // Remove devices
    for device_id in qemu_manager.list_devices() {
        qemu_manager.remove_device(&device_id).await?;
        info!("Removed device: {}", device_id);
    }
    
    info!("QEMU integration example completed successfully");
    Ok(())
}
