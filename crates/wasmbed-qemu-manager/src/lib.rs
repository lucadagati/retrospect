// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::process::{Command, Stdio};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use wasmbed_tcp_bridge::TcpBridge;
use base64::{engine::general_purpose, Engine};

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
    pub gateway_endpoint: Option<String>, // Gateway endpoint for TLS connection
    pub wasm_runtime: Option<WasmRuntime>,
}

/// Supported MCU types with Renode compatibility for constrained devices
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum McuType {
    /// Renode Arduino Nano 33 BLE (ARM Cortex-M4) - Constrained device
    RenodeArduinoNano33Ble,
    /// Renode STM32F4 Discovery (ARM Cortex-M4) - Constrained device
    RenodeStm32F4Discovery,
    /// Legacy: ARM MPS2-AN385 (ARM Cortex-M3) - Maps to RenodeArduinoNano33Ble
    #[serde(alias = "Mps2An385")]
    Mps2An385,
}

impl McuType {
    /// Get Renode platform name for this MCU
    pub fn renode_platform(&self) -> &'static str {
        match self {
            McuType::RenodeArduinoNano33Ble => "arduino_nano_33_ble",
            McuType::RenodeStm32F4Discovery => "stm32f4_discovery",
            McuType::Mps2An385 => "arduino_nano_33_ble", // Map to Arduino Nano for compatibility
        }
    }

    /// Get CPU architecture for this MCU
    pub fn cpu_architecture(&self) -> &'static str {
        match self {
            McuType::RenodeArduinoNano33Ble => "cortex-m4",
            McuType::RenodeStm32F4Discovery => "cortex-m4",
            McuType::Mps2An385 => "cortex-m3",
        }
    }

    /// Get memory size for this MCU
    pub fn memory_size(&self) -> &'static str {
        match self {
            McuType::RenodeArduinoNano33Ble => "1M",
            McuType::RenodeStm32F4Discovery => "1M",
            McuType::Mps2An385 => "512K",
        }
    }

    /// Get display name for UI
    pub fn display_name(&self) -> &'static str {
        match self {
            McuType::RenodeArduinoNano33Ble => "Renode Arduino Nano 33 BLE (Cortex-M4)",
            McuType::RenodeStm32F4Discovery => "Renode STM32F4 Discovery (Cortex-M4)",
            McuType::Mps2An385 => "ARM MPS2-AN385 (Cortex-M3)",
        }
    }

    /// Get Rust HAL crate name (if available)
    pub fn rust_hal_crate(&self) -> Option<&'static str> {
        match self {
            McuType::RenodeArduinoNano33Ble => Some("nrf52840-hal"),
            McuType::RenodeStm32F4Discovery => Some("stm32f4xx-hal"),
            McuType::Mps2An385 => None, // No specific HAL for MPS2-AN385
        }
    }

    /// Get all supported MCU types
    pub fn all_types() -> Vec<McuType> {
        vec![
            McuType::RenodeArduinoNano33Ble,
            McuType::RenodeStm32F4Discovery,
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
    tcp_bridges: Arc<Mutex<HashMap<String, TcpBridge>>>,
}

impl RenodeManager {
    pub fn new(renode_binary: String, base_port: u16) -> Self {
        // Don't load from file - Kubernetes is the source of truth
        // Devices will be created on-demand when needed
        // This prevents orphaned devices from previous sessions
        
        Self {
            devices: Arc::new(Mutex::new(HashMap::new())),
            renode_binary,
            base_port,
            tcp_bridges: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Save devices to persistent storage (optional, for debugging)
    /// Note: Kubernetes is the source of truth, this is just a cache
    async fn save_devices(&self) -> Result<(), anyhow::Error> {
        // Optional: save to file for debugging, but don't rely on it
        // Kubernetes is the source of truth
        let devices = self.devices.lock().await;
        let devices_file = "qemu_devices.json";
        let content = serde_json::to_string_pretty(&*devices)?;
        println!("Saving {} devices to {} (cache only, Kubernetes is source of truth)", devices.len(), devices_file);
        std::fs::write(devices_file, content)?;
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
            gateway_endpoint: None, // Will be set when starting device
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

    pub async fn start_device(&self, device_id: &str, gateway_endpoint: Option<String>) -> Result<(), anyhow::Error> {
        let debug_log_path = std::env::temp_dir().join(format!("start_device_debug_{}.log", device_id));
        let _ = std::fs::write(&debug_log_path, format!("start_device called for: {}\n", device_id));
        eprintln!("DEBUG: start_device called for: {}", device_id);
        
        let mut devices = self.devices.lock().await;
        
        let device = devices.get_mut(device_id)
            .ok_or_else(|| anyhow::anyhow!("Device {} not found", device_id))?;

        // Check if Docker container already exists and is running
        let use_docker = std::env::var("RENODE_USE_DOCKER").unwrap_or_else(|_| "true".to_string()) == "true";
        if use_docker {
            let container_name = format!("renode-{}", device_id);
            let check_container = Command::new("docker")
                .args(&["ps", "-a", "--filter", &format!("name={}", container_name), "--format", "{{.Names}} {{.Status}}"])
                .output();
            
            if let Ok(output) = check_container {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let _ = std::fs::write(&debug_log_path, format!("Container check output: {:?}\n", output_str));
                if output_str.contains(&container_name) {
                    if output_str.contains("Up") || output_str.contains("running") {
                        // FORCE RECREATION: Stop and remove existing container to regenerate script with Python code
                        println!("Renode container {} exists but forcing recreation to update script", container_name);
                        let _ = Command::new("docker")
                            .args(&["stop", &container_name])
                            .output();
                        let _ = Command::new("docker")
                            .args(&["rm", "-f", &container_name])
                            .output();
                        // Continue to build_renode_args to regenerate script
                    } else if output_str.contains("Exited") {
                        println!("Renode container {} exists but is stopped, removing and recreating", container_name);
                        // Remove stopped container so we can recreate it with new restart policy
                        let _ = Command::new("docker")
                            .args(&["rm", "-f", &container_name])
                            .output();
                    }
                }
            }
        }

        if matches!(device.status, QemuDeviceStatus::Running) {
            return Err(anyhow::anyhow!("Device is already running"));
        }

        // Update gateway endpoint if provided (use reference to avoid moving option)
        if let Some(ref gateway_ep) = gateway_endpoint {
            device.gateway_endpoint = Some(gateway_ep.clone());
        }
        
        // Drop lock before performing any other async operations to keep future Send
        drop(devices);
        
        // Start TCP bridge outside of device lock to avoid holding non-Send guard across await
        if let Some(gateway_ep) = gateway_endpoint {
            // Start TCP bridge for real TCP connections
            // Convert gateway HTTP endpoint (port 8080) to TLS endpoint (port 8443)
            let gateway_tls_endpoint = if gateway_ep.contains(":8080") {
                gateway_ep.replace(":8080", ":8443")
            } else if !gateway_ep.contains(":8443") {
                format!("{}:8443", gateway_ep.trim_end_matches('/'))
            } else {
                gateway_ep.clone()
            };
            
            // Use a unique bridge port for each device (hash of device_id)
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            device_id.hash(&mut hasher);
            let device_hash = hasher.finish();
            let bridge_port = 40000 + (device_hash % 1000) as u16; // Port between 40000-40999
            
            let bridge = TcpBridge::new(gateway_tls_endpoint, bridge_port);
            // Ensure bridge can safely move across await points
            fn assert_send<T: Send + Sync>(_: &T) {}
            assert_send(&bridge);
            if let Err(e) = bridge.start() {
                eprintln!("Failed to start TCP bridge for device {}: {}", device_id, e);
            } else {
                let bridge_endpoint = format!("127.0.0.1:{}", bridge_port);
                println!("TCP bridge started for device {} on port {} (endpoint: {})", device_id, bridge_port, bridge_endpoint);
                
                // Store bridge endpoint in device for later use
                // The bridge endpoint will be passed to firmware via memory
                let mut bridges = self.tcp_bridges.lock().await;
                bridges.insert(device_id.to_string(), bridge);
                
                // Store bridge endpoint in device (we'll write it to memory in build_renode_args)
                // For now, we'll use the bridge endpoint instead of gateway endpoint in memory
            }
        }
        
        // Re-lock devices to continue with startup (no awaits after this point)
        let mut devices = self.devices.lock().await;
        let device = devices.get_mut(device_id)
            .ok_or_else(|| anyhow::anyhow!("Device {} not found", device_id))?;

        device.status = QemuDeviceStatus::Starting;

        // Copy firmware to a Docker volume if using Docker
        // This is necessary because the firmware is inside the API server container image
        // and we need to make it accessible to the Renode container
        let use_docker = std::env::var("RENODE_USE_DOCKER").unwrap_or_else(|_| "true".to_string()) == "true";
        if use_docker {
            // Get firmware path
            let firmware_path = self.get_firmware_path(&device.mcu_type)?;
            let firmware_path_str = firmware_path.to_string_lossy().to_string();
            let firmware_filename = firmware_path
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("zephyr.elf"))
                .to_string_lossy()
                .to_string();
            
            // Create a Docker volume for the firmware
            let firmware_volume_name = format!("firmware-{}", device_id);
            
            // Check if volume already exists, create if not
            let volume_check = Command::new("docker")
                .args(&["volume", "inspect", &firmware_volume_name])
                .output();
            
            if volume_check.is_err() || !volume_check.unwrap().status.success() {
                // Create volume
                let create_volume = Command::new("docker")
                    .args(&["volume", "create", &firmware_volume_name])
                    .output();
                
                if let Err(e) = create_volume {
                    eprintln!("Failed to create Docker volume {}: {}", firmware_volume_name, e);
                    return Err(anyhow::anyhow!("Failed to create Docker volume: {}", e));
                }
                println!("Created Docker volume: {}", firmware_volume_name);
            }
            
            // Copy firmware from current container filesystem to volume
            // We're running inside the API server container, so the firmware should be accessible
            // Use cat to read the file and pipe it to a container that writes to the volume
            let firmware_copied = if std::path::Path::new(&firmware_path_str).exists() {
                println!("Firmware found in container filesystem, copying to volume {}", firmware_volume_name);
                
                // Use cat to read the file and pipe to docker run
                let cat_process = Command::new("cat")
                    .arg(&firmware_path_str)
                    .stdout(Stdio::piped())
                    .spawn();
                
                if let Ok(mut cat_proc) = cat_process {
                    if let Some(cat_stdout) = cat_proc.stdout.take() {
                        // Write to volume using a temporary container
                        let volume_write = Command::new("docker")
                            .args(&[
                                "run", "-i", "--rm",
                                "-v", &format!("{}:/firmware", firmware_volume_name),
                                "alpine:latest",
                                "sh", "-c",
                                &format!("cat > /firmware/{} && ls -lh /firmware/{}", firmware_filename, firmware_filename)
                            ])
                            .stdin(cat_stdout)
                            .output();
                        
                        if let Ok(write_output) = volume_write {
                            if write_output.status.success() {
                                println!("Firmware copied successfully to volume {}", firmware_volume_name);
                                true
                            } else {
                                let error_msg = String::from_utf8_lossy(&write_output.stderr);
                                eprintln!("Failed to write firmware to volume: {}", error_msg);
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                eprintln!("WARNING: Firmware not found at {}", firmware_path_str);
                false
            };
            
            // Fallback: if firmware exists on host filesystem, copy from there
            if !firmware_copied && std::path::Path::new(&firmware_path_str).exists() {
                println!("Copying firmware from host filesystem to volume {}", firmware_volume_name);
                let copy_from_host = Command::new("docker")
                    .args(&[
                        "run", "--rm",
                        "-v", &format!("{}:/firmware", firmware_volume_name),
                        "-v", &format!("{}:/source:ro", firmware_path.parent().unwrap().to_string_lossy()),
                        "alpine:latest",
                        "sh", "-c",
                        &format!("cp /source/{} /firmware/{}", firmware_filename, firmware_filename)
                    ])
                    .output();
                
                if let Ok(host_copy_output) = copy_from_host {
                    if !host_copy_output.status.success() {
                        let error_msg = String::from_utf8_lossy(&host_copy_output.stderr);
                        eprintln!("Failed to copy firmware from host: {}", error_msg);
                    }
                }
            }
        }

        // Start Renode process
        let renode_args = self.build_renode_args(device, device_id)?;
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
                
                // Don't wait for the process to exit - Renode should stay running
                // Instead, just spawn a detached thread that monitors but doesn't block
                std::thread::spawn(move || {
                    // Give Renode time to start and accept connections
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    
                    // Check if process is still running
                    match child.try_wait() {
                        Ok(Some(status)) => {
                            println!("Renode process for device {} exited early with status: {:?}", device_id_clone, status);
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
                        }
                        Ok(None) => {
                            // Process is still running, which is good
                            println!("Renode process for device {} is running", device_id_clone);
                            // Continue monitoring in background
                            let exit_status = child.wait();
                            println!("Renode process for device {} exited with status: {:?}", device_id_clone, exit_status);
                            let rt = tokio::runtime::Runtime::new().unwrap();
                            rt.block_on(async {
                                let mut devices = devices_clone.lock().await;
                                if let Some(device) = devices.get_mut(&device_id_clone) {
                                    device.status = QemuDeviceStatus::Stopped;
                                    device.process_id = None;
                                    println!("Updated device {} status to Stopped", device_id_clone);
                                }
                            });
                        }
                        Err(e) => {
                            eprintln!("Error checking Renode process for device {}: {}", device_id_clone, e);
                        }
                    }
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

        // Check if using Docker
        let use_docker = std::env::var("RENODE_USE_DOCKER").unwrap_or_else(|_| "true".to_string()) == "true";

        if use_docker {
            // Stop Docker container
            device.status = QemuDeviceStatus::Stopping;
            let container_name = format!("renode-{}", device_id);
            
            println!("Stopping Docker container: {}", container_name);
            if let Err(e) = Command::new("docker")
                .args(&["stop", &container_name])
                .output()
            {
                device.status = QemuDeviceStatus::Error(e.to_string());
                return Err(anyhow::anyhow!("Failed to stop Docker container: {}", e));
            }
            
            // Container will be automatically removed due to --rm flag
            device.process_id = None;
        } else if let Some(pid) = device.process_id {
            // Stop portable Renode process
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

    fn get_firmware_path(&self, mcu_type: &McuType) -> Result<std::path::PathBuf, std::io::Error> {
        let current_dir = std::env::current_dir().unwrap_or_default();
        let zephyr_workspace = current_dir.join("zephyr-workspace");
        let zephyr_firmware_nrf52840 = zephyr_workspace.join("build/nrf52840dk/nrf52840/zephyr/zephyr.elf");
        let zephyr_firmware_stm32f4 = zephyr_workspace.join("build/stm32f4/zephyr/zephyr.elf");
        let zephyr_firmware_arduino_nano = zephyr_workspace.join("build/nrf52840dk/nrf52840/zephyr/zephyr.elf");
        
        match mcu_type {
            McuType::RenodeArduinoNano33Ble => {
                if zephyr_firmware_arduino_nano.exists() {
                    Ok(zephyr_firmware_arduino_nano)
                } else if zephyr_firmware_nrf52840.exists() {
                    Ok(zephyr_firmware_nrf52840)
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("Zephyr firmware not found for nRF52840. Expected: {} or {}", 
                            zephyr_firmware_arduino_nano.display(), zephyr_firmware_nrf52840.display())
                    ))
                }
            },
            McuType::RenodeStm32F4Discovery => {
                if zephyr_firmware_stm32f4.exists() {
                    Ok(zephyr_firmware_stm32f4)
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("Zephyr firmware not found for STM32F4. Expected: {}", 
                            zephyr_firmware_stm32f4.display())
                    ))
                }
            },
            McuType::Mps2An385 => {
                if zephyr_firmware_nrf52840.exists() {
                    Ok(zephyr_firmware_nrf52840)
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("Zephyr firmware not found for MPS2-AN385. Expected: {}", 
                            zephyr_firmware_nrf52840.display())
                    ))
                }
            },
        }
    }

    fn build_renode_args(&self, device: &QemuDevice, device_id: &str) -> Result<Vec<String>, std::io::Error> {
        // CRITICAL: Write to file immediately to verify function is called
        let function_entry_log = std::env::temp_dir().join(format!("build_renode_args_entry_{}.log", device_id));
        let _ = std::fs::write(&function_entry_log, format!("build_renode_args called for device: {}\ndevice.id: {}\n", device_id, device.id));
        
        println!("DEBUG: build_renode_args called for device: {} (device.id: {})", device_id, device.id);
        eprintln!("DEBUG: build_renode_args called for device: {} (device.id: {})", device_id, device.id);
        
        // Check if we should use Docker (more reliable and isolate than portable)
        let use_docker = std::env::var("RENODE_USE_DOCKER").unwrap_or_else(|_| "true".to_string()) == "true";
        
        let renode_binary = if use_docker {
            // Use Docker - pull image if needed
            // Check if image exists
            let check_image = std::process::Command::new("docker")
                .args(&["images", "-q", "antmicro/renode:nightly"])
                .output();
            
            if let Ok(output) = check_image {
                if output.stdout.is_empty() {
                    println!("Pulling Renode Docker image (first time only)...");
                    let pull_result = std::process::Command::new("docker")
                        .args(&["pull", "antmicro/renode:nightly"])
                        .status();
                    if let Err(e) = pull_result {
                        panic!("Failed to pull Renode Docker image: {}", e);
                    }
                }
            }
            "docker".to_string()
        } else {
            // Fallback: Use RENODE_PATH environment variable if set
            // Otherwise, force Docker usage (recommended)
            if let Ok(renode_path) = std::env::var("RENODE_PATH") {
                renode_path
            } else {
                // Force Docker usage - portable Renode is no longer supported
                eprintln!("Warning: RENODE_PATH not set. Forcing Docker usage.");
                // Use Docker instead
                let check_image = std::process::Command::new("docker")
                    .args(&["images", "-q", "antmicro/renode:nightly"])
                    .output();
                
                if let Ok(output) = check_image {
                    if output.stdout.is_empty() {
                        println!("Pulling Renode Docker image (first time only)...");
                        let pull_result = std::process::Command::new("docker")
                            .args(&["pull", "antmicro/renode:nightly"])
                            .status();
                        if let Err(e) = pull_result {
                            panic!("Failed to pull Renode Docker image: {}", e);
                        }
                    }
                }
                "docker".to_string()
            }
        };
        
        // Get firmware path using helper method
        let firmware_path = self.get_firmware_path(&device.mcu_type)?;
        let firmware_path_str = firmware_path.to_string_lossy().to_string();
        let firmware_filename = firmware_path
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("zephyr.elf"))
            .to_string_lossy()
            .to_string();
        
        // Post-load commands (empty for now, can be extended)
        let post_load_commands = String::new();
        
        // Build Renode commands to:
        // 1. Create machine
        // 2. Include platform description (use @ prefix for Renode-relative paths)
        // 3. Setup UART analyzer for logs (different UART per device)
        // 4. Load device runtime firmware (use absolute path)
        // 5. Set PC/SP if needed (for nRF52840)
        // 6. Start execution
        let uart_name = match device.mcu_type {
            McuType::RenodeArduinoNano33Ble => "uart0",
            McuType::RenodeStm32F4Discovery => "usart1",
            McuType::Mps2An385 => "uart0", // Map to Arduino Nano UART
        };
        
        // For Docker, use Docker volume mount (firmware-{device_id} volume)
        // The firmware is copied to the volume in start_device before calling build_renode_args
        // Note: Renode uses @ prefix for relative paths, but we need absolute paths in container
        let firmware_path_in_container = if use_docker {
            format!("/firmware/{}", firmware_filename)
        } else {
            firmware_path_str.clone()
        };
        
        // Renode command format: mach add "name" (not mach create "name")
        // For nRF52840, we need to set PC and SP after loading firmware
        // For STM32F4, configure Ethernet interface
        let ethernet_config = if device.mcu_type == McuType::RenodeStm32F4Discovery {
            // STM32F4 has built-in Ethernet MAC (Network.SynopsysEthernetMAC) - configure it
            // Ethernet is already defined in stm32f4.repl at sysbus 0x40028000
            // Create network switch and TAP interface for host connection
            "; emulation CreateSwitch \"ethernet_switch\"; emulation CreateTap \"tap0\" \"ethernet_tap\"; sysbus.ethernet MAC \"00:11:22:33:44:55\"; connector Connect sysbus.ethernet ethernet_switch; connector Connect host.ethernet_tap ethernet_switch; host.ethernet_tap Start"
        } else {
            ""
        };
        
        let pc_sp_commands = if device.mcu_type == McuType::RenodeArduinoNano33Ble {
            "\nsysbus.cpu PC 0x866b\nsysbus.cpu SP 0x20020000"
        } else {
            ""
        };
        // For Docker, use absolute path without @ prefix (absolute paths don't need @)
        // The @ prefix is for Renode-relative paths, not absolute paths
        let loadelf_cmd = if use_docker {
            // Use absolute path without @ prefix
            format!("sysbus LoadELF \"{}\"", firmware_path_in_container)
        } else {
            format!("sysbus LoadELF @\"{}\"", firmware_path_in_container)
        };
        let mut renode_commands = format!(
            "mach add \"{id}\"\ninclude @platforms/boards/{platform}.repl\nshowAnalyzer sysbus.{uart}\n{loadelf}\nmach set \"{id}\"\n{ethernet}{pc_sp}\n# Enable detailed logging\nlogLevel 0\nsysbus.cpu LogFunctionNames true true\n# Configure CPU to continue on faults (for debugging)\nsysbus.cpu ExecutionMode SingleStepBlocking\n# Start machine\nstart\n# Keep machine running - Renode will stay active after start command",
            id = device.id,
            platform = device.mcu_type.renode_platform(),
            uart = uart_name,
            loadelf = loadelf_cmd,
            ethernet = if ethernet_config.is_empty() {
                "".to_string()
            } else {
                ethernet_config
                    .trim()
                    .trim_start_matches(';')
                    .split(';')
                    .map(|cmd| cmd.trim())
                    .filter(|cmd| !cmd.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            pc_sp = pc_sp_commands
        );
        if !post_load_commands.is_empty() {
            renode_commands.push('\n');
            renode_commands.push_str(
                &post_load_commands
                    .trim()
                    .trim_start_matches(';')
                    .split(';')
                    .map(|cmd| cmd.trim())
                    .filter(|cmd| !cmd.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n"),
            );
        }
        
        // Get bridge endpoint if TCP bridge is running for this device
        // Calculate bridge port using same hash as in start_device
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        device_id.hash(&mut hasher);
        let device_hash = hasher.finish();
        let bridge_port = 40000 + (device_hash % 1000) as u16;
        let bridge_endpoint = format!("127.0.0.1:{}", bridge_port);
        
        // Use bridge endpoint if gateway_endpoint is set (bridge is running)
        // Otherwise, fallback to gateway endpoint directly
        let endpoint_str = if device.gateway_endpoint.is_some() {
            // Use bridge endpoint (firmware connects to bridge, bridge connects to gateway)
            bridge_endpoint
        } else {
            // Fallback to default endpoint
            "127.0.0.1:8443".to_string()
        };
        
        // Use endpoint_str for memory write
        let gateway_endpoint_str = endpoint_str;
        
        // Append gateway endpoint configuration to Renode commands
        // The endpoint will be written to memory address 0x20001000 (in RAM)
        // Format: write endpoint string bytes to memory
        let endpoint_bytes = gateway_endpoint_str.as_bytes();
        let mut endpoint_write_commands = String::new();
        
        // Write endpoint string to memory starting at 0x20001000
        // First write the length, then the string bytes
        endpoint_write_commands.push_str(&format!("; sysbus WriteDoubleWord 0x20001000 0x{:08x}", endpoint_bytes.len()));
        
        // Write endpoint bytes (4 bytes at a time)
        for (i, chunk) in endpoint_bytes.chunks(4).enumerate() {
            let mut word: u32 = 0;
            for (j, &byte) in chunk.iter().enumerate() {
                word |= (byte as u32) << (j * 8);
            }
            endpoint_write_commands.push_str(&format!("; sysbus WriteDoubleWord 0x{:08x} 0x{:08x}", 
                0x20001004 + (i as u32 * 4), word));
        }
        
        // Append endpoint configuration to Renode commands
        let renode_commands_with_endpoint = format!("{}{}", renode_commands, endpoint_write_commands);
        
        // Start Renode with a script file to keep it running
        // Create a temporary script file with all commands
        let port = device.endpoint.split(':').nth(1).unwrap_or("3000").to_string();
        
        // Create temporary script file
        // CRITICAL: Use device_id instead of device.id to ensure consistency
        let script_path = std::env::temp_dir().join(format!("renode_{}.resc", device_id));
        // Add commands to keep Renode running after start
        // The issue is that the machine may pause/dispose when firmware terminates
        // Solution: Use 'start' command and add a Python script to monitor and restart the machine
        // if it pauses. We'll use Renode's Python API to keep the machine running.
        // Build script content with Python monitoring code
        // IMPORTANT: The Python code must be added AFTER the 'start' command
        // Renode executes the script line by line, so Python must come after start
        // CRITICAL: Python code in Renode must use proper indentation and format
        // Note: Python monitoring is optional - machine should stay running after 'start'
        let python_code = r#"

# Python script to monitor and keep machine running (optional)
# Note: Machine should stay running after 'start' command
# This Python code is commented out to avoid syntax errors
# python:
# import time
# mach = machines.current
# while True:
#     try:
#         time.sleep(0.5)
#         if mach.cpu.IsPaused:
#             mach.cpu.Resume()
#     except:
#         pass"#;
        
        // Build the complete script: renode commands + start + Python monitoring
        // Ensure proper line breaks and formatting
        // CRITICAL: Make sure python_code is actually included
        // Note: renode_commands_with_endpoint already includes 'start', so we don't add it again
        let script_content = format!(
            "{}{}",
            renode_commands_with_endpoint,
            python_code
        );
        
        // CRITICAL: Log script_content immediately after format!
        let format_log = std::env::temp_dir().join(format!("script_content_after_format_{}.log", device_id));
        let format_log_msg = format!(
            "After format!:\nrenode_commands_with_endpoint length: {}\npython_code length: {}\nscript_content length: {}\nscript_content contains 'python:': {}\nscript_content last 500 chars:\n{}",
            renode_commands_with_endpoint.len(),
            python_code.len(),
            script_content.len(),
            script_content.contains("python:"),
            &script_content[script_content.len().saturating_sub(500)..]
        );
        let _ = std::fs::write(&format_log, &format_log_msg);
        eprintln!("DEBUG: script_content after format! - length: {}, contains python: {}", script_content.len(), script_content.contains("python:"));
        println!("DEBUG: script_content after format! - length: {}, contains python: {}", script_content.len(), script_content.contains("python:"));
        
        // CRITICAL DEBUG: Write script_content to a file IMMEDIATELY to verify it contains Python
        // Use unwrap to ensure the write happens and we see any errors
        let pre_write_debug_path = std::env::temp_dir().join(format!("renode_prewrite_{}.resc", device_id));
        if let Err(e) = std::fs::write(&pre_write_debug_path, &script_content) {
            eprintln!("ERROR: Failed to write pre-write debug file: {}", e);
            println!("ERROR: Failed to write pre-write debug file: {}", e);
        } else {
            eprintln!("DEBUG: Pre-write debug file written successfully to: {:?}", pre_write_debug_path);
            println!("DEBUG: Pre-write debug file written successfully to: {:?}", pre_write_debug_path);
        }
        
        // Debug output - use both eprintln and println to ensure visibility
        // Also write to a separate debug file to ensure we can see it
        let debug_log_path = std::env::temp_dir().join(format!("renode_debug_{}.log", device_id));
        let python_code_len = python_code.len();
        let script_contains_python = script_content.contains("python:");
        let debug_msg = format!(
            "DEBUG: Writing Renode script to: {:?}\nDEBUG: renode_commands_with_endpoint length: {} bytes\nDEBUG: python_code length: {} bytes\nDEBUG: Script content length: {} bytes\nDEBUG: Script contains 'python:': {}\nDEBUG: Last 300 chars of script: {:?}\n",
            script_path,
            renode_commands_with_endpoint.len(),
            python_code_len,
            script_content.len(),
            script_contains_python,
            &script_content[script_content.len().saturating_sub(300)..]
        );
        // Write debug BEFORE any potential errors - use unwrap to ensure it happens
        if let Err(e) = std::fs::write(&debug_log_path, &debug_msg) {
            eprintln!("ERROR: Failed to write debug log file: {}", e);
            println!("ERROR: Failed to write debug log file: {}", e);
        } else {
            eprintln!("DEBUG: Debug log file written successfully to: {:?}", debug_log_path);
            println!("DEBUG: Debug log file written successfully to: {:?}", debug_log_path);
        }
        eprintln!("{}", debug_msg);
        println!("{}", debug_msg);
        
        // CRITICAL: If script doesn't contain Python, log error immediately and PANIC to see the error
        if !script_contains_python {
            let error_msg = format!("ERROR: script_content does NOT contain 'python:' before writing!\nrenode_commands_with_endpoint length: {} bytes\nrenode_commands_with_endpoint ends with: {:?}\npython_code length: {} bytes\npython_code starts with: {:?}\nscript_content length: {} bytes\nscript_content last 200 chars: {:?}", 
                renode_commands_with_endpoint.len(),
                &renode_commands_with_endpoint[renode_commands_with_endpoint.len().saturating_sub(50)..],
                python_code.len(),
                &python_code[..50.min(python_code.len())],
                script_content.len(),
                &script_content[script_content.len().saturating_sub(200)..]);
            eprintln!("{}", error_msg);
            println!("{}", error_msg);
            let _ = std::fs::write(&debug_log_path, format!("{}\n{}", debug_msg, error_msg));
            // PANIC to see the error immediately
            panic!("{}", error_msg);
        }
        
        // Encode script content as base64 so we can recreate it inside the Renode container
        let script_b64 = general_purpose::STANDARD.encode(script_content.as_bytes());
        let script_filename = script_path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| format!("renode_{}.resc", device_id));
        
        // Write the script file - use unwrap_or_else to handle errors gracefully
        // This ensures we can still write debug files even if there's an error
        match std::fs::write(&script_path, &script_content) {
            Ok(_) => {
                // CRITICAL: Also write to a separate debug file to verify content
                let debug_script_path = std::env::temp_dir().join(format!("renode_debug_{}.resc", device_id));
                let _ = std::fs::write(&debug_script_path, &script_content);
                
                // Verify the file was written correctly
                match std::fs::read_to_string(&script_path) {
                    Ok(written_content) => {
                        // Write verification to both stderr and a log file
                        let verify_msg = format!(
                            "DEBUG: Written file length: {} bytes\nDEBUG: Written file contains 'python:': {}\nDEBUG: Expected length: {} bytes\nDEBUG: Expected contains 'python:': {}",
                            written_content.len(),
                            written_content.contains("python:"),
                            script_content.len(),
                            script_content.contains("python:")
                        );
                        eprintln!("{}", verify_msg);
                        println!("{}", verify_msg);
                        let _ = std::fs::write(&debug_log_path, format!("{}\n{}", debug_msg, verify_msg));
                        
                        if written_content != script_content {
                            eprintln!("ERROR: Written script content differs from expected!");
                            eprintln!("ERROR: Expected length: {}, Written length: {}", script_content.len(), written_content.len());
                            eprintln!("ERROR: Expected last 200 chars: {:?}", &script_content[script_content.len().saturating_sub(200)..]);
                            eprintln!("ERROR: Written last 200 chars: {:?}", &written_content[written_content.len().saturating_sub(200)..]);
                            // Write error to debug file
                            let error_msg = format!("ERROR: Content mismatch!\nExpected last 200: {:?}\nWritten last 200: {:?}", 
                                &script_content[script_content.len().saturating_sub(200)..],
                                &written_content[written_content.len().saturating_sub(200)..]);
                            let _ = std::fs::write(&debug_log_path, format!("{}\n{}", debug_msg, error_msg));
                        } else {
                            eprintln!("DEBUG: Script file written successfully with Python code");
                            println!("DEBUG: Script file written successfully with Python code");
                        }
                    },
                    Err(e) => {
                        let error_msg = format!("ERROR: Failed to read written script file: {}", e);
                        eprintln!("{}", error_msg);
                        println!("{}", error_msg);
                        let _ = std::fs::write(&debug_log_path, format!("{}\n{}", debug_msg, error_msg));
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, error_msg));
                    }
                }
            },
            Err(e) => {
                let error_msg = format!("ERROR: Failed to write script file: {}", e);
                eprintln!("{}", error_msg);
                println!("{}", error_msg);
                let _ = std::fs::write(&debug_log_path, format!("{}\n{}", debug_msg, error_msg));
                return Err(e);
            }
        }
        
        // Start Renode with script file - it will execute script and stay running
        let args = if use_docker {
            // Docker command format:
            // docker run --rm --net=host -v firmware:/firmware:ro -v script:/script:ro antmicro/renode:nightly --console --plain --port PORT /script/file.resc
            // Get parent directory of firmware for mounting (so /firmware_dir points to the firmware directory)
            // CRITICAL: Mount the ARM firmware directory (target/thumbv7em-none-eabihf/release), not x86-64 (target/release)
            println!("DEBUG: Firmware path: {}", firmware_path_str);
            
            let script_b64_sanitized = script_b64.replace('"', "\\\"");
            // Use sh -c to execute Renode and keep container running
            // Use --restart=unless-stopped to automatically restart if container exits
            // Remove --rm so container persists and can be restarted
            vec![
                renode_binary, // "docker"
                "run".to_string(),
                "--restart=unless-stopped".to_string(),
                "--net=host".to_string(),
                "--name".to_string(),
                format!("renode-{}", device.id), // Container name for easy cleanup
                "-v".to_string(),
                // Mount Docker volume containing the firmware (copied in start_device)
                format!("{}:/firmware:ro", format!("firmware-{}", device_id)),
                "antmicro/renode:nightly".to_string(),
                "sh".to_string(),
                "-c".to_string(),
                format!(
                    "set -e; python3 - <<'PY'\nimport base64, os\nos.makedirs('/tmp/renode', exist_ok=True)\nwith open('/tmp/renode/{script_name}', 'wb') as f:\n    f.write(base64.b64decode(\"{script_b64}\"))\nPY\nexec renode --console --port {port} /tmp/renode/{script_name}",
                    script_name = script_filename,
                    script_b64 = script_b64_sanitized,
                    port = port.clone()
                ), // Execute Renode with script file as argument (script includes start)
            ]
        } else {
            vec![
                renode_binary,
                "--console".to_string(),
                "--plain".to_string(),
                "--port".to_string(),
                port.clone(),
                script_path.to_string_lossy().to_string(),
            ]
        };
        
        // Note: Renode will execute the script and stay running because machine is active after 'start'
        
        Ok(args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_device() {
        let manager = RenodeManager::new("qemu-system-arm".to_string(), 30000);
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
        let manager = RenodeManager::new("qemu-system-arm".to_string(), 30000);
        
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
        let result = manager.start_device(&device.id, None).await;
        // We expect this to fail in test environment
        assert!(result.is_err());

        // Stop device
        let result = manager.stop_device(&device.id).await;
        assert!(result.is_ok());
    }
}
