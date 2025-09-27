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
    pub status: QemuDeviceStatus,
    pub process_id: Option<u32>,
    pub endpoint: String,
    pub wasm_runtime: Option<WasmRuntime>,
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

pub struct QemuManager {
    devices: Arc<Mutex<HashMap<String, QemuDevice>>>,
    qemu_binary: String,
    base_port: u16,
}

impl QemuManager {
    pub fn new(qemu_binary: String, base_port: u16) -> Self {
        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
            qemu_binary,
            base_port,
        }
    }

    pub async fn create_device(&self, id: String, name: String, architecture: String, device_type: String) -> Result<QemuDevice, anyhow::Error> {
        let mut devices = self.devices.lock().await;
        
        if devices.contains_key(&id) {
            return Err(anyhow::anyhow!("Device {} already exists", id));
        }

        let device = QemuDevice {
            id: id.clone(),
            name,
            architecture,
            device_type,
            status: QemuDeviceStatus::Stopped,
            process_id: None,
            endpoint: format!("127.0.0.1:{}", self.base_port + devices.len() as u16),
            wasm_runtime: None,
        };

        devices.insert(id.clone(), device.clone());
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
                    tokio::spawn(async move {
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
        let mut args = vec![
            "-machine".to_string(),
            "virt".to_string(),
            "-cpu".to_string(),
            "cortex-a72".to_string(),
            "-smp".to_string(),
            "1".to_string(),
            "-m".to_string(),
            "128M".to_string(),
            "-nographic".to_string(),
            "-serial".to_string(),
            format!("tcp:{}:server", device.endpoint),
            "-netdev".to_string(),
            "user,id=net0,hostfwd=tcp::8080-:8080".to_string(),
            "-device".to_string(),
            "virtio-net-pci,netdev=net0".to_string(),
        ];

        // Add architecture-specific arguments
        match device.architecture.as_str() {
            "arm64" => {
                args.extend([
                    "-machine".to_string(),
                    "virt".to_string(),
                    "-cpu".to_string(),
                    "cortex-a72".to_string(),
                ]);
            }
            "riscv64" => {
                args.extend([
                    "-machine".to_string(),
                    "virt".to_string(),
                    "-cpu".to_string(),
                    "rv64".to_string(),
                ]);
            }
            "x86_64" => {
                args.extend([
                    "-machine".to_string(),
                    "pc".to_string(),
                    "-cpu".to_string(),
                    "qemu64".to_string(),
                ]);
            }
            _ => {
                // Default to ARM64
                args.extend([
                    "-machine".to_string(),
                    "virt".to_string(),
                    "-cpu".to_string(),
                    "cortex-a72".to_string(),
                ]);
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
        let manager = QemuManager::new("qemu-system-aarch64".to_string(), 30000);
        let device = manager.create_device(
            "test-device".to_string(),
            "Test Device".to_string(),
            "arm64".to_string(),
            "MPU".to_string(),
        ).await.unwrap();

        assert_eq!(device.id, "test-device");
        assert_eq!(device.architecture, "arm64");
        assert!(matches!(device.status, QemuDeviceStatus::Stopped));
    }

    #[tokio::test]
    async fn test_device_lifecycle() {
        let manager = QemuManager::new("qemu-system-aarch64".to_string(), 30000);
        
        // Create device
        let device = manager.create_device(
            "test-device".to_string(),
            "Test Device".to_string(),
            "arm64".to_string(),
            "MPU".to_string(),
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
