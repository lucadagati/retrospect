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

/// Supported MCU types with QEMU and Rust compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum McuType {
    /// ARM MPS2-AN385 (Cortex-M3) - Default, most compatible
    Mps2An385,
    /// ARM MPS2-AN386 (Cortex-M4) - Enhanced with FPU
    Mps2An386,
    /// ARM MPS2-AN500 (Cortex-M7) - High performance
    Mps2An500,
    /// ARM MPS2-AN505 (Cortex-M33) - TrustZone support
    Mps2An505,
    /// STM32VLDISCOVERY (Cortex-M3) - STMicroelectronics
    Stm32Vldiscovery,
    /// Olimex STM32-H405 (Cortex-M4) - Olimex board
    OlimexStm32H405,
}

impl McuType {
    /// Get QEMU machine name for this MCU
    pub fn qemu_machine(&self) -> &'static str {
        match self {
            McuType::Mps2An385 => "mps2-an385",
            McuType::Mps2An386 => "mps2-an386", 
            McuType::Mps2An500 => "mps2-an500",
            McuType::Mps2An505 => "mps2-an505",
            McuType::Stm32Vldiscovery => "stm32vldiscovery",
            McuType::OlimexStm32H405 => "olimex-stm32-h405",
        }
    }

    /// Get QEMU CPU name for this MCU
    pub fn qemu_cpu(&self) -> &'static str {
        match self {
            McuType::Mps2An385 => "cortex-m3",
            McuType::Mps2An386 => "cortex-m4",
            McuType::Mps2An500 => "cortex-m7", 
            McuType::Mps2An505 => "cortex-m33",
            McuType::Stm32Vldiscovery => "cortex-m3",
            McuType::OlimexStm32H405 => "cortex-m4",
        }
    }

    /// Get memory size for this MCU
    pub fn memory_size(&self) -> &'static str {
        match self {
            McuType::Mps2An385 => "16M",
            McuType::Mps2An386 => "16M",
            McuType::Mps2An500 => "16M",
            McuType::Mps2An505 => "16M", 
            McuType::Stm32Vldiscovery => "8M",
            McuType::OlimexStm32H405 => "16M",
        }
    }

    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            McuType::Mps2An385 => "ARM MPS2-AN385 (Cortex-M3)",
            McuType::Mps2An386 => "ARM MPS2-AN386 (Cortex-M4)",
            McuType::Mps2An500 => "ARM MPS2-AN500 (Cortex-M7)",
            McuType::Mps2An505 => "ARM MPS2-AN505 (Cortex-M33)",
            McuType::Stm32Vldiscovery => "STM32VLDISCOVERY (Cortex-M3)",
            McuType::OlimexStm32H405 => "Olimex STM32-H405 (Cortex-M4)",
        }
    }

    /// Get Rust HAL crate name (if available)
    pub fn rust_hal_crate(&self) -> Option<&'static str> {
        match self {
            McuType::Mps2An385 => Some("cortex-m"),
            McuType::Mps2An386 => Some("cortex-m"),
            McuType::Mps2An500 => Some("cortex-m"),
            McuType::Mps2An505 => Some("cortex-m"),
            McuType::Stm32Vldiscovery => Some("stm32f1xx-hal"),
            McuType::OlimexStm32H405 => Some("stm32f4xx-hal"),
        }
    }

    /// Get all supported MCU types
    pub fn all_types() -> Vec<McuType> {
        vec![
            McuType::Mps2An385,
            McuType::Mps2An386,
            McuType::Mps2An500,
            McuType::Mps2An505,
            McuType::Stm32Vldiscovery,
            McuType::OlimexStm32H405,
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
pub struct QemuManager {
    devices: Arc<Mutex<HashMap<String, QemuDevice>>>,
    qemu_binary: String,
    base_port: u16,
}

impl QemuManager {
    pub fn new(qemu_binary: String, base_port: u16) -> Self {
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
            qemu_binary,
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

        // Start QEMU process
        let qemu_args = self.build_qemu_args(device);
        let mut cmd = Command::new(&self.qemu_binary);
        cmd.args(&qemu_args);
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
                    let _ = child.wait();
                    // Process ended, update status
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let mut devices = devices_clone.lock().await;
                        if let Some(device) = devices.get_mut(&device_id_clone) {
                            device.status = QemuDeviceStatus::Stopped;
                            device.process_id = None;
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

    fn build_qemu_args(&self, device: &QemuDevice) -> Vec<String> {
        // MCU-specific configuration for optimal Rust support
        let mut args = vec![
            "-machine".to_string(),
            device.mcu_type.qemu_machine().to_string(),
            "-cpu".to_string(),
            device.mcu_type.qemu_cpu().to_string(),
            "-m".to_string(),
            device.mcu_type.memory_size().to_string(),
            "-nographic".to_string(),
            "-serial".to_string(),
            format!("tcp:{}:server,nowait", device.endpoint), // TCP serial bridge
            "-monitor".to_string(),
            format!("tcp:{}:server,nowait", device.endpoint.replace(":", ":1")), // QEMU monitor
            "-kernel".to_string(),
            "/dev/zero".to_string(), // Use /dev/zero as kernel (minimal approach)
            "-dtb".to_string(),
            "/dev/null".to_string(), // Device tree (optional)
        ];

        // Add networking for supported MCU types
        match device.mcu_type {
            McuType::Mps2An385 | McuType::Mps2An386 | McuType::Mps2An500 | McuType::Mps2An505 => {
                // MPS2 boards support networking
                args.push("-netdev".to_string());
                args.push(format!("user,id=net0,hostfwd=tcp::{}-:8080", device.endpoint.split(':').nth(1).unwrap_or("8080")));
                args.push("-device".to_string());
                args.push("lan9118,netdev=net0".to_string()); // Ethernet controller
            }
            McuType::Stm32Vldiscovery | McuType::OlimexStm32H405 => {
                // STM32 boards may not have networking in QEMU
                // Skip networking configuration
            }
        }

        // Add firmware and device tree for supported MCU types
        match device.mcu_type {
            McuType::Mps2An385 => {
                args.push("-kernel".to_string());
                args.push("wasmbed-firmware-mps2-an385.bin".to_string());
                args.push("-dtb".to_string());
                args.push("mps2-an385.dtb".to_string());
            }
            McuType::Mps2An386 => {
                args.push("-kernel".to_string());
                args.push("wasmbed-firmware-mps2-an386.bin".to_string());
                args.push("-dtb".to_string());
                args.push("mps2-an386.dtb".to_string());
            }
            McuType::Mps2An500 => {
                args.push("-kernel".to_string());
                args.push("wasmbed-firmware-mps2-an500.bin".to_string());
                args.push("-dtb".to_string());
                args.push("mps2-an500.dtb".to_string());
            }
            McuType::Mps2An505 => {
                args.push("-kernel".to_string());
                args.push("wasmbed-firmware-mps2-an505.bin".to_string());
                args.push("-dtb".to_string());
                args.push("mps2-an505.dtb".to_string());
            }
            McuType::Stm32Vldiscovery => {
                args.push("-kernel".to_string());
                args.push("wasmbed-firmware-stm32vldiscovery.bin".to_string());
            }
            McuType::OlimexStm32H405 => {
                args.push("-kernel".to_string());
                args.push("wasmbed-firmware-olimex-stm32-h405.bin".to_string());
            }
        }

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
