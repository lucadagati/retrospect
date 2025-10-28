// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::process::{Command, Stdio};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QemuDevice {
    pub id: String,
    pub name: String,
    pub architecture: String,
    pub device_type: String,
    pub mcu_type: McuType,
    pub status: QemuDeviceStatus,
    pub process_id: Option<u32>,
    pub endpoint: String,
    pub wasm_runtime: Option<WasmRuntime>,
}

/// Supported MCU types with Renode compatibility for constrained devices
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum McuType {
    /// Renode Arduino Nano 33 BLE (ARM Cortex-M4) - Constrained device
    RenodeArduinoNano33Ble,
    /// Renode STM32F4 Discovery (ARM Cortex-M4) - Constrained device
    RenodeStm32F4Discovery,
    /// Renode Arduino Uno R4 (ARM Cortex-M4) - Constrained device
    RenodeArduinoUnoR4,
}

impl McuType {
    /// Get Renode platform name for this MCU
    pub fn renode_platform(&self) -> &'static str {
        match self {
            McuType::RenodeArduinoNano33Ble => "arduino_nano_33_ble",
            McuType::RenodeStm32F4Discovery => "stm32f4_discovery",
            McuType::RenodeArduinoUnoR4 => "arduino_uno_r4_minima",
        }
    }

    /// Get CPU architecture for this MCU
    pub fn cpu_architecture(&self) -> &'static str {
        match self {
            McuType::RenodeArduinoNano33Ble => "cortex-m4",
            McuType::RenodeStm32F4Discovery => "cortex-m4",
            McuType::RenodeArduinoUnoR4 => "cortex-m4",
        }
    }

    /// Get memory size for this MCU
    pub fn memory_size(&self) -> &'static str {
        match self {
            McuType::RenodeArduinoNano33Ble => "1M",
            McuType::RenodeStm32F4Discovery => "1M",
            McuType::RenodeArduinoUnoR4 => "512K",
        }
    }

    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            McuType::RenodeArduinoNano33Ble => "Renode Arduino Nano 33 BLE (Cortex-M4)",
            McuType::RenodeStm32F4Discovery => "Renode STM32F4 Discovery (Cortex-M4)",
            McuType::RenodeArduinoUnoR4 => "Renode Arduino Uno R4 (Cortex-M4)",
        }
    }

    /// Get Rust HAL crate name (if available)
    pub fn rust_hal_crate(&self) -> Option<&'static str> {
        match self {
            McuType::RenodeArduinoNano33Ble => Some("nrf52840-hal"),
            McuType::RenodeStm32F4Discovery => Some("stm32f4xx-hal"),
            McuType::RenodeArduinoUnoR4 => Some("renesas-ra-hal"),
        }
    }

    /// Get all supported MCU types
    pub fn all_types() -> Vec<McuType> {
        vec![
            McuType::RenodeArduinoNano33Ble,
            McuType::RenodeStm32F4Discovery,
            McuType::RenodeArduinoUnoR4,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QemuDeviceStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmRuntime {
    pub wasm_bytes: Vec<u8>,
    pub config: WasmConfig,
    pub status: WasmRuntimeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
    pub memory_limit: u32,
    pub execution_timeout: u32,
    pub host_functions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WasmRuntimeStatus {
    NotLoaded,
    Loading,
    Running,
    Stopped,
    Error(String),
}

#[derive(Debug)]
pub struct RenodeManager {
    devices: Arc<Mutex<HashMap<String, QemuDevice>>>,
    renode_binary: String,
    base_port: u16,
}

impl RenodeManager {
    pub fn new(renode_binary: String, base_port: u16) -> Self {
        let mut devices_map = HashMap::new();
        
        // Load persisted devices synchronously
        let devices_file = "qemu_devices.json";
        if std::path::Path::new(devices_file).exists() {
            if let Ok(content) = std::fs::read_to_string(devices_file) {
                if let Ok(persisted_devices) = serde_json::from_str::<HashMap<String, QemuDevice>>(&content) {
                    devices_map = persisted_devices;
                    println!("Loaded {} persisted devices", devices_map.len());
                }
            }
        }
        
        Self {
            devices: Arc::new(Mutex::new(devices_map)),
            renode_binary,
            base_port,
        }
    }
    
    /// Save devices to persistent storage
    async fn save_devices(&self) -> Result<(), anyhow::Error> {
        let devices = self.devices.lock().await;
        let devices_file = "qemu_devices.json";
        let content = serde_json::to_string_pretty(&*devices)?;
        println!("Saving {} devices to {}", devices.len(), devices_file);
        std::fs::write(devices_file, content)?;
        println!("Successfully saved devices to {}", devices_file);
        Ok(())
    }

    pub async fn create_device(&self, id: String, name: String, architecture: String, device_type: String, mcu_type: McuType, endpoint: Option<String>) -> Result<QemuDevice, anyhow::Error> {
        let mut devices = self.devices.lock().await;
        
        if devices.contains_key(&id) {
            return Err(anyhow::anyhow!("Device {} already exists", id));
        }

        let device = QemuDevice {
            id: id.clone(),
            name,
            architecture,
            device_type,
            mcu_type,
            status: QemuDeviceStatus::Stopped,
            process_id: None,
            endpoint: endpoint.unwrap_or_else(|| format!("127.0.0.1:{}", self.base_port + devices.len() as u16)),
            wasm_runtime: None,
        };

        devices.insert(id.clone(), device.clone());
        
        // Save to persistent storage
        drop(devices); // Release the lock before async call
        println!("About to save devices to persistent storage");
        match self.save_devices().await {
            Ok(_) => println!("Successfully saved devices to persistent storage"),
            Err(e) => println!("Failed to save devices to persistent storage: {}", e),
        }
        
        Ok(device)
    }

    pub async fn start_device(&self, device_id: &str) -> Result<(), anyhow::Error> {
        let mut devices = self.devices.lock().await;
        
        let device = devices.get_mut(device_id)
            .ok_or_else(|| anyhow::anyhow!("Device {} not found", device_id))?;

        if matches!(device.status, QemuDeviceStatus::Running) {
            return Err(anyhow::anyhow!("Device is already running"));
        }

        device.status = QemuDeviceStatus::Starting;

        // Start Renode process
        let renode_args = self.build_renode_args(device);
        println!("Starting Renode with args: {:?}", renode_args);
        let mut cmd = Command::new(&renode_args[0]);
        cmd.args(&renode_args[1..]);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        match cmd.spawn() {
            Ok(mut child) => {
                device.process_id = Some(child.id());
                device.status = QemuDeviceStatus::Running;
                
                // Spawn a thread to monitor the process
                let device_id_clone = device_id.to_string();
                let devices_clone = self.devices.clone();
                
                std::thread::spawn(move || {
                    let exit_status = child.wait();
                    println!("QEMU process for device {} exited with status: {:?}", device_id_clone, exit_status);
                    
                    // Process ended, update status
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut devices = devices_clone.lock().await;
                        if let Some(device) = devices.get_mut(&device_id_clone) {
                            device.status = QemuDeviceStatus::Stopped;
                            device.process_id = None;
                            println!("Updated device {} status to Stopped", device_id_clone);
                        }
                    });
                });

                Ok(())
            }
            Err(e) => {
                device.status = QemuDeviceStatus::Error(e.to_string());
                Err(anyhow::anyhow!("Failed to start QEMU: {}", e))
            }
        }
    }

    pub async fn stop_device(&self, device_id: &str) -> Result<(), anyhow::Error> {
        let mut devices = self.devices.lock().await;
        
        let device = devices.get_mut(device_id)
            .ok_or_else(|| anyhow::anyhow!("Device {} not found", device_id))?;

        if let Some(pid) = device.process_id {
            device.status = QemuDeviceStatus::Stopping;
            
            // Send SIGTERM to the process
            if let Err(e) = Command::new("kill")
                .args(&["-TERM", &pid.to_string()])
                .output()
            {
                device.status = QemuDeviceStatus::Error(e.to_string());
                return Err(anyhow::anyhow!("Failed to stop device: {}", e));
            }

            // Wait for process to terminate
            thread::sleep(Duration::from_secs(2));

            // Force kill if still running
            if let Err(e) = Command::new("kill")
                .args(&["-KILL", &pid.to_string()])
                .output()
            {
                device.status = QemuDeviceStatus::Error(e.to_string());
                return Err(anyhow::anyhow!("Failed to force stop device: {}", e));
            }

            device.process_id = None;
            device.status = QemuDeviceStatus::Stopped;
        }

        Ok(())
    }

    pub async fn deploy_wasm(&self, device_id: &str, wasm_bytes: Vec<u8>, config: WasmConfig) -> Result<(), anyhow::Error> {
        let mut devices = self.devices.lock().await;
        
        let device = devices.get_mut(device_id)
            .ok_or_else(|| anyhow::anyhow!("Device {} not found", device_id))?;

        if !matches!(device.status, QemuDeviceStatus::Running) {
            return Err(anyhow::anyhow!("Device is not running"));
        }

        device.wasm_runtime = Some(WasmRuntime {
            wasm_bytes,
            config,
            status: WasmRuntimeStatus::Loading,
        });

        // Simulate WASM loading process
        tokio::spawn({
            let device_id = device_id.to_string();
            let devices = self.devices.clone();
            
            async move {
                tokio::time::sleep(Duration::from_secs(2)).await;
                
                let mut devices = devices.lock().await;
                if let Some(device) = devices.get_mut(&device_id) {
                    if let Some(runtime) = &mut device.wasm_runtime {
                        runtime.status = WasmRuntimeStatus::Running;
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn get_device(&self, device_id: &str) -> Option<QemuDevice> {
        let devices = self.devices.lock().await;
        devices.get(device_id).cloned()
    }

    pub async fn list_devices(&self) -> Vec<QemuDevice> {
        let devices = self.devices.lock().await;
        devices.values().cloned().collect()
    }

    fn build_renode_args(&self, device: &QemuDevice) -> Vec<String> {
        // Use RENODE_PATH environment variable or default to relative path
        let renode_binary = std::env::var("RENODE_PATH")
            .unwrap_or_else(|_| {
                // Try to find renode relative to current executable or project root
                let current_dir = std::env::current_dir().unwrap_or_default();
                let renode_path = current_dir.join("renode_1.15.0_portable/renode");
                renode_path.to_string_lossy().to_string()
            });
        
        let mut args = vec![
            renode_binary,
            "--console".to_string(),
            "--port".to_string(),
            device.endpoint.split(':').nth(1).unwrap_or("3000").to_string(),
            "--execute".to_string(),
            format!(
                "mach create; mach LoadPlatformDescription @platforms/boards/{}.repl",
                device.mcu_type.renode_platform()
            ),
        ];
        
        args
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_device() {
        let manager = QemuManager::new("qemu-system-arm".to_string(), 30000);
        let device = manager.create_device(
            "test-device".to_string(),
            "Test Device".to_string(),
            "ARM_CORTEX_M".to_string(),
            "MCU".to_string(),
            McuType::Mps2An385,
            None,
        ).await.unwrap();

        assert_eq!(device.id, "test-device");
        assert_eq!(device.architecture, "ARM_CORTEX_M");
        assert_eq!(device.mcu_type, McuType::Mps2An385);
        assert!(matches!(device.status, QemuDeviceStatus::Stopped));
    }

    #[tokio::test]
    async fn test_device_lifecycle() {
        let manager = QemuManager::new("qemu-system-arm".to_string(), 30000);
        
        // Create device
        let device = manager.create_device(
            "test-device".to_string(),
            "Test Device".to_string(),
            "ARM_CORTEX_M".to_string(),
            "MCU".to_string(),
            McuType::Mps2An385,
            None,
        ).await.unwrap();

        // Start device (this will fail in test environment without QEMU)
        let result = manager.start_device(&device.id).await;
        // We expect this to fail in test environment
        assert!(result.is_err());

        // Stop device
        let result = manager.stop_device(&device.id).await;
        assert!(result.is_ok());
    }
}
