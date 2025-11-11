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
        let mut devices = self.devices.lock().await;
        
        let device = devices.get_mut(device_id)
            .ok_or_else(|| anyhow::anyhow!("Device {} not found", device_id))?;

        if matches!(device.status, QemuDeviceStatus::Running) {
            return Err(anyhow::anyhow!("Device is already running"));
        }

        // Update gateway endpoint if provided
        if let Some(gateway_ep) = gateway_endpoint {
            device.gateway_endpoint = Some(gateway_ep.clone());
            
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

        device.status = QemuDeviceStatus::Starting;

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

    fn build_renode_args(&self, device: &QemuDevice, device_id: &str) -> Result<Vec<String>, std::io::Error> {
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
            // Use RENODE_PATH environment variable or use latest version from /tmp/renode_latest
            std::env::var("RENODE_PATH")
                .unwrap_or_else(|_| {
                    // Use latest Renode from /tmp/renode_latest
                    let latest_path = std::path::Path::new("/tmp/renode_latest/renode");
                    if latest_path.exists() {
                        latest_path.to_string_lossy().to_string()
                    } else {
                        panic!("Renode not found at /tmp/renode_latest/renode. Please download it from https://builds.renode.io/renode-latest.linux-portable.tar.gz");
                    }
                })
        };
        
        // Find firmware path (ARM build) - use absolute path
        let current_dir = std::env::current_dir().unwrap_or_default();
        
        // For Renode, prefer firmware compiled with std feature (for real TCP connections)
        // Fallback to no_std firmware if std version doesn't exist
        let std_firmware = current_dir.join("target/release/wasmbed-device-runtime");
        let no_std_firmware_nrf52840 = current_dir.join("target/thumbv7em-none-eabihf/release/wasmbed-device-runtime-nrf52840");
        let no_std_firmware_stm32f4 = current_dir.join("target/thumbv7em-none-eabihf/release/wasmbed-device-runtime-stm32f4");
        let no_std_firmware_default = current_dir.join("target/thumbv7em-none-eabihf/release/wasmbed-device-runtime");
        
        // Use firmware compiled for each specific MCU (compiled with correct linker script)
        // Prefer std version for Renode (real TCP connections), fallback to no_std
        let (firmware_path, post_load_commands) = match device.mcu_type {
            McuType::RenodeArduinoNano33Ble => {
                // nRF52840: prefer std version, fallback to no_std
                if std_firmware.exists() {
                    (std_firmware, String::new())
                } else if no_std_firmware_nrf52840.exists() {
                    (no_std_firmware_nrf52840, String::new())
                } else {
                    (no_std_firmware_default, String::new())
                }
            },
            McuType::RenodeStm32F4Discovery => {
                // STM32F4: prefer std version, fallback to no_std
                if std_firmware.exists() {
                    (std_firmware, String::new())
                } else if no_std_firmware_stm32f4.exists() {
                    (no_std_firmware_stm32f4, String::new())
                } else {
                    (no_std_firmware_default, String::new())
                }
            },
            McuType::RenodeArduinoUnoR4 => {
                // Arduino Uno R4: prefer std version, fallback to no_std
                if std_firmware.exists() {
                    (std_firmware, String::new())
                } else if no_std_firmware_stm32f4.exists() {
                    (no_std_firmware_stm32f4, String::new())
                } else {
                    (no_std_firmware_default, String::new())
                }
            },
        };
        let firmware_path_str = firmware_path.to_string_lossy().to_string();
        
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
            McuType::RenodeArduinoUnoR4 => "uart0",
        };
        
        // For Docker, use /firmware_dir/firmware_filename path (mounted volume), otherwise use absolute path
        // Note: Renode uses @ prefix for relative paths, but we need absolute paths in container
        let firmware_path_in_container = if use_docker {
            let firmware_filename = std::path::Path::new(&firmware_path_str)
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("wasmbed-device-runtime"))
                .to_string_lossy()
                .to_string();
            // Use absolute path in container (no @ prefix)
            format!("/firmware_dir/{}", firmware_filename)
        } else {
            // For non-Docker, use @ prefix for Renode-relative paths
            format!("@{}", firmware_path_str)
        };
        
        // Renode command format: mach add "name" (not mach create "name")
        // For nRF52840, we need to set PC and SP after loading firmware
        let renode_commands = if !post_load_commands.is_empty() {
            format!(
                "mach add \"{}\"; include @platforms/boards/{}.repl; showAnalyzer sysbus.{}; sysbus LoadELF \"{}\"; mach set \"{}\"; {}start",
                device.id,
                device.mcu_type.renode_platform(),
                uart_name,
                firmware_path_in_container,
                device.id,
                post_load_commands
            )
        } else {
            // For nRF52840, add PC and SP setup after loading firmware
            let pc_sp_commands = if device.mcu_type == McuType::RenodeArduinoNano33Ble {
                // nRF52840: firmware entry point is typically at 0x866b (from previous logs)
                // SP should be set to end of RAM (0x20020000 for nRF52840)
                "; sysbus.cpu PC 0x866b; sysbus.cpu SP 0x20020000"
            } else {
                ""
            };
            format!(
                "mach add \"{}\"; include @platforms/boards/{}.repl; showAnalyzer sysbus.{}; sysbus LoadELF \"{}\"; mach set \"{}\"{}{} start",
                device.id,
                device.mcu_type.renode_platform(),
                uart_name,
                firmware_path_in_container,
                device.id,
                pc_sp_commands,
                if pc_sp_commands.is_empty() { "" } else { ";" }
            )
        };
        
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
        let script_path = std::env::temp_dir().join(format!("renode_{}.resc", device.id));
        // Add a command to keep Renode running after start
        // Renode will stay active as long as the machine is running
        // The 'start' command begins execution and Renode stays in monitor mode
        let script_content = format!("{}\n# Machine will continue running after start command\n", renode_commands_with_endpoint);
        std::fs::write(&script_path, script_content)?;
        
        // Start Renode with script file - it will execute script and stay running
        let args = if use_docker {
            // Docker command format:
            // docker run --rm --net=host -v firmware:/firmware:ro -v script:/script:ro antmicro/renode:nightly --console --plain --port PORT /script/file.resc
            // Get parent directory of firmware for mounting (so /firmware points to the firmware file)
            let firmware_dir = std::path::Path::new(&firmware_path_str)
                .parent()
                .unwrap_or_else(|| std::path::Path::new("/"))
                .to_string_lossy()
                .to_string();
            let firmware_filename = std::path::Path::new(&firmware_path_str)
                .file_name()
                .unwrap_or_else(|| std::ffi::OsStr::new("wasmbed-device-runtime"))
                .to_string_lossy()
                .to_string();
            
            // Use sh -c to execute Renode and keep container running
            vec![
                renode_binary, // "docker"
                "run".to_string(),
                "--rm".to_string(),
                "--net=host".to_string(),
                "--name".to_string(),
                format!("renode-{}", device.id), // Container name for easy cleanup
                "-v".to_string(),
                format!("{}:/firmware_dir:ro", firmware_dir),
                "-v".to_string(),
                format!("{}:/script.resc:ro", script_path.to_string_lossy()),
                "antmicro/renode:nightly".to_string(),
                "sh".to_string(),
                "-c".to_string(),
                format!("renode --console --plain --port {} /script.resc; while true; do sleep 3600; done", port.clone()), // Keep container running
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
