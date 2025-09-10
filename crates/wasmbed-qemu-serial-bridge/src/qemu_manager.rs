// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tracing::{info, error, warn, debug};

/// QEMU device types supported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QemuDeviceType {
    /// RISC-V SiFive HiFive1
    RiscV,
    /// ARM Cortex-M STM32
    ArmCortexM,
    /// ESP32 Xtensa
    Esp32,
}

/// QEMU device configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QemuDeviceConfig {
    /// Device type
    pub device_type: QemuDeviceType,
    /// Device ID
    pub device_id: String,
    /// Firmware image path
    pub firmware_path: String,
    /// Serial port for communication
    pub serial_port: u16,
    /// Monitor port for QEMU monitor
    pub monitor_port: u16,
    /// Memory size in MB
    pub memory_mb: u32,
    /// CPU cores
    pub cpu_cores: u32,
    /// Network configuration
    pub network_config: Option<NetworkConfig>,
    /// Additional QEMU arguments
    pub extra_args: Vec<String>,
}

/// Network configuration for QEMU devices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Host forward port for SSH
    pub ssh_port: u16,
    /// Host forward port for HTTP
    pub http_port: u16,
    /// Network device type
    pub device_type: String,
}

/// QEMU device status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceStatus {
    /// Device is starting up
    Starting,
    /// Device is running
    Running,
    /// Device is stopping
    Stopping,
    /// Device has stopped
    Stopped,
    /// Device encountered an error
    Error(String),
}

/// QEMU device information
#[derive(Debug)]
pub struct QemuDevice {
    /// Device configuration
    pub config: QemuDeviceConfig,
    /// QEMU process handle
    pub process: Option<Child>,
    /// Device status
    pub status: DeviceStatus,
    /// Start time
    pub start_time: Option<Instant>,
    /// Last heartbeat
    pub last_heartbeat: Option<Instant>,
    /// Serial bridge connection
    pub serial_connected: bool,
}

/// QEMU Device Manager for managing multiple QEMU instances
pub struct QemuDeviceManager {
    /// Active devices
    devices: Arc<Mutex<HashMap<String, QemuDevice>>>,
    /// Heartbeat interval
    heartbeat_interval: Duration,
    /// Device timeout
    device_timeout: Duration,
}

impl QemuDeviceManager {
    /// Create a new QEMU device manager
    pub fn new() -> Self {
        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
            heartbeat_interval: Duration::from_secs(30),
            device_timeout: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Add a new QEMU device
    pub async fn add_device(&self, config: QemuDeviceConfig) -> Result<(), Box<dyn std::error::Error>> {
        let device_id = config.device_id.clone();
        
        info!("Adding QEMU device: {} ({:?})", device_id, config.device_type);
        
        let device = QemuDevice {
            config,
            process: None,
            status: DeviceStatus::Starting,
            start_time: None,
            last_heartbeat: None,
            serial_connected: false,
        };

        {
            let mut devices = self.devices.lock().unwrap();
            devices.insert(device_id.clone(), device);
        }

        // Start the device
        self.start_device(&device_id).await?;
        
        Ok(())
    }

    /// Start a QEMU device
    pub async fn start_device(&self, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config = {
            let devices = self.devices.lock().unwrap();
            let device = devices.get(device_id)
                .ok_or_else(|| format!("Device {} not found", device_id))?;
            device.config.clone()
        };

        info!("Starting QEMU device: {}", device_id);

        // Build QEMU command based on device type
        let mut cmd = self.build_qemu_command(&config)?;
        
        // Start the process
        let process = cmd.spawn()
            .map_err(|e| format!("Failed to start QEMU process: {}", e))?;

        // Update device status
        {
            let mut devices = self.devices.lock().unwrap();
            let device = devices.get_mut(device_id).unwrap();
            device.process = Some(process);
            device.status = DeviceStatus::Running;
            device.start_time = Some(Instant::now());
            device.last_heartbeat = Some(Instant::now());
        }

        info!("QEMU device {} started successfully", device_id);
        Ok(())
    }

    /// Stop a QEMU device
    pub async fn stop_device(&self, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let process = {
            let mut devices = self.devices.lock().unwrap();
            let device = devices.get_mut(device_id)
                .ok_or_else(|| format!("Device {} not found", device_id))?;
            device.status = DeviceStatus::Stopping;
            device.process.take()
        };

        info!("Stopping QEMU device: {}", device_id);

        if let Some(mut process) = process {
            // Try graceful shutdown first
            if let Err(e) = process.kill() {
                warn!("Failed to kill QEMU process for {}: {}", device_id, e);
            }
            
            // Wait for process to exit
            let _ = process.wait();
        }

        // Update device status
        {
            let mut devices = self.devices.lock().unwrap();
            let device = devices.get_mut(device_id).unwrap();
            device.status = DeviceStatus::Stopped;
            device.serial_connected = false;
        }

        info!("QEMU device {} stopped", device_id);
        Ok(())
    }

    /// Remove a QEMU device
    pub async fn remove_device(&self, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Stop the device first
        self.stop_device(device_id).await?;
        
        // Remove from devices map
        {
            let mut devices = self.devices.lock().unwrap();
            devices.remove(device_id);
        }

        info!("QEMU device {} removed", device_id);
        Ok(())
    }

    /// Get device status
    pub fn get_device_status(&self, device_id: &str) -> Option<DeviceStatus> {
        let devices = self.devices.lock().unwrap();
        devices.get(device_id).map(|d| d.status.clone())
    }

    /// Get all device statuses
    pub fn get_all_device_statuses(&self) -> HashMap<String, DeviceStatus> {
        let devices = self.devices.lock().unwrap();
        devices.iter().map(|(id, device)| (id.clone(), device.status.clone())).collect()
    }

    /// Update device heartbeat
    pub fn update_heartbeat(&self, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut devices = self.devices.lock().unwrap();
        let device = devices.get_mut(device_id)
            .ok_or_else(|| format!("Device {} not found", device_id))?;

        device.last_heartbeat = Some(Instant::now());
        Ok(())
    }

    /// Set serial connection status
    pub fn set_serial_connected(&self, device_id: &str, connected: bool) -> Result<(), Box<dyn std::error::Error>> {
        let mut devices = self.devices.lock().unwrap();
        let device = devices.get_mut(device_id)
            .ok_or_else(|| format!("Device {} not found", device_id))?;

        device.serial_connected = connected;
        debug!("Device {} serial connection: {}", device_id, connected);
        Ok(())
    }

    /// Build QEMU command for a device configuration
    fn build_qemu_command(&self, config: &QemuDeviceConfig) -> Result<Command, Box<dyn std::error::Error>> {
        let mut cmd = Command::new("qemu-system-riscv32"); // Default, will be changed based on device type
        
        match config.device_type {
            QemuDeviceType::RiscV => {
                cmd = Command::new("qemu-system-riscv32");
                cmd.arg("-machine").arg("sifive_u");
                cmd.arg("-cpu").arg("rv32");
            },
            QemuDeviceType::ArmCortexM => {
                cmd = Command::new("qemu-system-arm");
                cmd.arg("-machine").arg("stm32-p103");
                cmd.arg("-cpu").arg("cortex-m3");
            },
            QemuDeviceType::Esp32 => {
                cmd = Command::new("qemu-system-xtensa");
                cmd.arg("-machine").arg("esp32");
                cmd.arg("-cpu").arg("esp32");
            },
        }

        // Common arguments
        cmd.arg("-smp").arg(config.cpu_cores.to_string());
        cmd.arg("-m").arg(format!("{}M", config.memory_mb));
        cmd.arg("-nographic");
        
        // Monitor configuration
        cmd.arg("-monitor")
            .arg(format!("tcp:localhost:{},server,nowait", config.monitor_port));
        
        // Serial configuration
        cmd.arg("-serial")
            .arg(format!("tcp:localhost:{},server,nowait", config.serial_port));
        
        // Network configuration
        if let Some(ref net_config) = config.network_config {
            cmd.arg("-netdev")
                .arg(format!("user,id=net0,hostfwd=tcp::{}:22,hostfwd=tcp::{}:8080", 
                    net_config.ssh_port, net_config.http_port));
            cmd.arg("-device")
                .arg(format!("{},netdev=net0", net_config.device_type));
        }
        
        // Firmware
        cmd.arg("-kernel").arg(&config.firmware_path);
        
        // Extra arguments
        for arg in &config.extra_args {
            cmd.arg(arg);
        }
        
        // Set up stdio
        cmd.stdin(Stdio::null());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        Ok(cmd)
    }

    /// Monitor devices and handle timeouts
    pub async fn monitor_devices(&self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let now = Instant::now();
            let mut devices_to_restart = Vec::new();
            
            {
                let mut devices = self.devices.lock().unwrap();
                for (device_id, device) in devices.iter_mut() {
                    // Check for heartbeat timeout
                    if let Some(last_heartbeat) = device.last_heartbeat {
                        if now.duration_since(last_heartbeat) > self.device_timeout {
                            warn!("Device {} heartbeat timeout, restarting", device_id);
                            devices_to_restart.push(device_id.clone());
                        }
                    }
                    
                    // Check if process is still running
                    if let Some(ref mut process) = device.process {
                        if let Ok(Some(_)) = process.try_wait() {
                            warn!("Device {} process exited unexpectedly", device_id);
                            device.status = DeviceStatus::Error("Process exited".to_string());
                            devices_to_restart.push(device_id.clone());
                        }
                    }
                }
            }
            
            // Restart devices that need it
            for device_id in devices_to_restart {
                if let Err(e) = self.restart_device(&device_id).await {
                    error!("Failed to restart device {}: {}", device_id, e);
                }
            }
            
            sleep(self.heartbeat_interval).await;
        }
    }

    /// Restart a device
    async fn restart_device(&self, device_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Restarting QEMU device: {}", device_id);
        
        // Stop the device
        self.stop_device(device_id).await?;
        
        // Wait a bit
        sleep(Duration::from_secs(2)).await;
        
        // Start the device again
        self.start_device(device_id).await?;
        
        info!("QEMU device {} restarted successfully", device_id);
        Ok(())
    }

    /// Get device configuration
    pub fn get_device_config(&self, device_id: &str) -> Option<QemuDeviceConfig> {
        let devices = self.devices.lock().unwrap();
        devices.get(device_id).map(|d| d.config.clone())
    }

    /// List all device IDs
    pub fn list_devices(&self) -> Vec<String> {
        let devices = self.devices.lock().unwrap();
        devices.keys().cloned().collect()
    }

    /// Get device count
    pub fn device_count(&self) -> usize {
        let devices = self.devices.lock().unwrap();
        devices.len()
    }

    /// Check if device exists
    pub fn device_exists(&self, device_id: &str) -> bool {
        let devices = self.devices.lock().unwrap();
        devices.contains_key(device_id)
    }
}

impl Default for QemuDeviceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Default configurations for different device types
impl QemuDeviceConfig {
    /// Create default RISC-V configuration
    pub fn riscv_default(device_id: String, firmware_path: String) -> Self {
        Self {
            device_type: QemuDeviceType::RiscV,
            device_id,
            firmware_path,
            serial_port: 4444,
            monitor_port: 4445,
            memory_mb: 128,
            cpu_cores: 2,
            network_config: Some(NetworkConfig {
                ssh_port: 2222,
                http_port: 8080,
                device_type: "virtio-net-device".to_string(),
            }),
            extra_args: vec![],
        }
    }

    /// Create default ARM Cortex-M configuration
    pub fn arm_cortex_m_default(device_id: String, firmware_path: String) -> Self {
        Self {
            device_type: QemuDeviceType::ArmCortexM,
            device_id,
            firmware_path,
            serial_port: 4447,
            monitor_port: 4446,
            memory_mb: 128,
            cpu_cores: 1,
            network_config: Some(NetworkConfig {
                ssh_port: 2223,
                http_port: 8081,
                device_type: "virtio-net-device".to_string(),
            }),
            extra_args: vec![],
        }
    }

    /// Create default ESP32 configuration
    pub fn esp32_default(device_id: String, firmware_path: String) -> Self {
        Self {
            device_type: QemuDeviceType::Esp32,
            device_id,
            firmware_path,
            serial_port: 4449,
            monitor_port: 4448,
            memory_mb: 4,
            cpu_cores: 1,
            network_config: Some(NetworkConfig {
                ssh_port: 2224,
                http_port: 8082,
                device_type: "virtio-net-device".to_string(),
            }),
            extra_args: vec![],
        }
    }
}
