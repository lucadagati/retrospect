// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::os::unix::net::UnixStream;
use std::io::{Read, Write};
use std::time::Duration;
use std::thread;

pub struct QemuSerialBridge {
    device_id: String,
    serial_socket_path: String,
    gateway_host: String,
    gateway_port: u16,
    connected: bool,
    enrolled: bool,
    serial_connected: bool,
    heartbeat_interval: Duration,
    last_heartbeat: std::time::Instant,
    applications: std::collections::HashMap<String, ApplicationInfo>,
    microros_active: bool,
    serial_stream: Option<UnixStream>,
    tls_stream: Option<rustls::StreamOwned<rustls::ClientConnection, std::net::TcpStream>>,
}

#[derive(Clone)]
struct ApplicationInfo {
    id: String,
    status: String,
    loaded_at: std::time::Instant,
    platform: String,
}

impl QemuSerialBridge {
    pub fn new(device_id: String, serial_socket_path: String, gateway_host: String, gateway_port: u16) -> Self {
        Self {
            device_id,
            serial_socket_path,
            gateway_host,
            gateway_port,
            connected: false,
            enrolled: false,
            serial_connected: false,
            heartbeat_interval: Duration::from_secs(30),
            last_heartbeat: std::time::Instant::now(),
            applications: std::collections::HashMap::new(),
            microros_active: false,
            serial_stream: None,
            tls_stream: None,
        }
    }
    
    pub fn connect_serial(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let stream = UnixStream::connect(&self.serial_socket_path)?;
        self.serial_stream = Some(stream);
        self.serial_connected = true;
        println!("[{}] Connected to QEMU serial socket: {}", self.device_id, self.serial_socket_path);
        Ok(())
    }
    
    pub fn send_serial_command(&mut self, command: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(ref mut stream) = self.serial_stream {
            let message = format!("{}\n", command);
            stream.write_all(message.as_bytes())?;
            println!("[{}] Sent serial command: {}", self.device_id, command);
        }
        Ok(())
    }
    
    pub fn read_serial_response(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(ref mut stream) = self.serial_stream {
            let mut buffer = [0; 1024];
            let n = stream.read(&mut buffer)?;
            let response = String::from_utf8_lossy(&buffer[..n]).to_string();
            println!("[{}] Received serial response: {}", self.device_id, response.trim());
            Ok(response)
        } else {
            Err("Serial stream not connected".into())
        }
    }
    
    pub fn connect_to_gateway(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.serial_connected {
            return Err("Serial not connected".into());
        }
        
        // Create TLS connection to gateway
        let tcp_stream = std::net::TcpStream::connect(format!("{}:{}", self.gateway_host, self.gateway_port))?;
        
        // Create TLS client configuration
        let mut config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(std::sync::Arc::new(NoVerifier))
            .with_no_client_auth();
        
        let connector = rustls::ClientConnection::new(std::sync::Arc::new(config), self.gateway_host.as_str().try_into()?)?;
        let tls_stream = rustls::StreamOwned::new(connector, tcp_stream);
        
        self.tls_stream = Some(tls_stream);
        self.connected = true;
        println!("[{}] Connected to gateway at {}:{}", self.device_id, self.gateway_host, self.gateway_port);
        Ok(())
    }
    
    pub fn enroll_device(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.connected {
            return Err("Not connected to gateway".into());
        }
        
        let enrollment_msg = serde_json::json!({
            "type": "enrollment",
            "device_id": self.device_id,
            "public_key": format!("qemu_public_key_{}", self.device_id),
            "device_type": "qemu-riscv32",
            "capabilities": ["wasm-execution", "tls-client", "microROS", "serial-communication"],
            "hardware_info": {
                "architecture": "riscv32imac-unknown-none-elf",
                "memory_size": 16,
                "cpu_freq": 100,
                "serial_socket": self.serial_socket_path
            },
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        if let Some(ref mut stream) = self.tls_stream {
            let message = format!("{}\n", enrollment_msg);
            stream.write_all(message.as_bytes())?;
            
            let mut buffer = [0; 1024];
            let n = stream.read(&mut buffer)?;
            let response = String::from_utf8_lossy(&buffer[..n]).to_string();
            let response_data: serde_json::Value = serde_json::from_str(&response)?;
            
            if response_data.get("status").and_then(|s| s.as_str()) == Some("success") {
                self.enrolled = true;
                println!("[{}] Successfully enrolled with gateway", self.device_id);
            } else {
                return Err(format!("Enrollment failed: {}", response_data.get("error").unwrap_or(&serde_json::Value::String("Unknown error".to_string()))).into());
            }
        }
        
        Ok(())
    }
    
    pub fn send_heartbeat(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.connected || !self.enrolled {
            return Err("Not connected or enrolled".into());
        }
        
        let heartbeat_msg = serde_json::json!({
            "type": "heartbeat",
            "device_id": self.device_id,
            "status": "active",
            "uptime": self.last_heartbeat.elapsed().as_secs(),
            "applications": self.applications.keys().collect::<Vec<_>>(),
            "microros_active": self.microros_active,
            "serial_connected": self.serial_connected,
            "architecture": "riscv32imac-unknown-none-elf",
            "memory_size": 16,
            "cpu_freq": 100,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        if let Some(ref mut stream) = self.tls_stream {
            let message = format!("{}\n", heartbeat_msg);
            stream.write_all(message.as_bytes())?;
            self.last_heartbeat = std::time::Instant::now();
            println!("[{}] Heartbeat sent (Architecture: riscv32imac-unknown-none-elf)", self.device_id);
        }
        
        Ok(())
    }
    
    pub fn load_wasm_application(&mut self, app_id: &str, wasm_data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let app_info = ApplicationInfo {
            id: app_id.to_string(),
            status: "loaded".to_string(),
            loaded_at: std::time::Instant::now(),
            platform: "qemu-riscv32".to_string(),
        };
        
        self.applications.insert(app_id.to_string(), app_info);
        println!("[{}] WASM application {} loaded on QEMU", self.device_id, app_id);
        Ok(())
    }
    
    pub fn execute_wasm_function(&mut self, app_id: &str, function_name: &str, args: Option<serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
        if !self.applications.contains_key(app_id) {
            return Err(format!("Application {} not found", app_id).into());
        }
        
        let command = if let Some(args) = args {
            format!("wasm_execute {} {} {}", app_id, function_name, args)
        } else {
            format!("wasm_execute {} {}", app_id, function_name)
        };
        
        self.send_serial_command(&command)?;
        let response = self.read_serial_response()?;
        
        println!("[{}] Executed WASM function {} in {} on QEMU: {}", 
                 self.device_id, function_name, app_id, response.trim());
        Ok(())
    }
    
    pub fn start_microros(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.send_serial_command("microros_start")?;
        let response = self.read_serial_response()?;
        
        if response.to_lowercase().contains("started") {
            self.microros_active = true;
            println!("[{}] microROS started on QEMU", self.device_id);
        }
        
        Ok(())
    }
    
    pub fn publish_microros_topic(&mut self, topic: &str, data: serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
        if !self.microros_active {
            return Err("microROS not active".into());
        }
        
        let command = format!("microros_publish {} {}", topic, data);
        self.send_serial_command(&command)?;
        let response = self.read_serial_response()?;
        
        println!("[{}] QEMU published to topic {}: {}", self.device_id, topic, data);
        Ok(())
    }
    
    pub fn run_device_simulation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("\n=== Starting QEMU Device Simulation: {} ===", self.device_id);
        
        // Connect to QEMU serial
        self.connect_serial()?;
        
        // Connect to gateway
        self.connect_to_gateway()?;
        
        // Enroll device
        self.enroll_device()?;
        
        // Load sample WASM application
        let sample_wasm = b"qemu_wasm_binary_data";
        self.load_wasm_application("microros-px4-bridge", sample_wasm)?;
        
        // Start microROS
        self.start_microros()?;
        
        // Main device loop
        let mut loop_count = 0;
        loop {
            loop_count += 1;
            
            // Send heartbeat every 30 seconds
            if self.last_heartbeat.elapsed() >= self.heartbeat_interval {
                self.send_heartbeat()?;
            }
            
            // Execute WASM functions periodically
            if loop_count % 100 == 0 {
                let args = serde_json::json!({
                    "sensor": "accelerometer",
                    "platform": "qemu"
                });
                self.execute_wasm_function("microros-px4-bridge", "process_sensor_data", Some(args))?;
            }
            
            // Publish microROS data periodically
            if loop_count % 200 == 0 {
                let data = serde_json::json!({
                    "value": 42.5,
                    "unit": "m/s²",
                    "platform": "qemu"
                });
                self.publish_microros_topic("/sensor_data", data)?;
            }
            
            // Show status periodically
            if loop_count % 1000 == 0 {
                println!("[{}] QEMU Status: {} applications, microROS: {}, Serial: {}", 
                         self.device_id, self.applications.len(), self.microros_active, self.serial_connected);
            }
            
            thread::sleep(Duration::from_millis(100)); // 100ms loop
        }
    }
}

// Custom certificate verifier for development
struct NoVerifier;

impl rustls::client::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}
