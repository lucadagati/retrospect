// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error, warn};

/// MCU Device Client for testing
pub struct McuDeviceClient {
    device_id: String,
    gateway_host: String,
    gateway_port: u16,
    connected: bool,
}

impl McuDeviceClient {
    pub fn new(device_id: String, gateway_host: String, gateway_port: u16) -> Self {
        Self {
            device_id,
            gateway_host,
            gateway_port,
            connected: false,
        }
    }

    /// Simulate device enrollment
    pub async fn enroll(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Device {} starting enrollment process", self.device_id);
        
        // Simulate enrollment steps
        sleep(Duration::from_millis(500)).await;
        info!("Device {} generating keypair", self.device_id);
        
        sleep(Duration::from_millis(300)).await;
        info!("Device {} sending enrollment request", self.device_id);
        
        sleep(Duration::from_millis(200)).await;
        info!("Device {} sending public key", self.device_id);
        
        sleep(Duration::from_millis(100)).await;
        info!("Device {} received device UUID", self.device_id);
        
        sleep(Duration::from_millis(100)).await;
        info!("Device {} enrollment completed", self.device_id);
        
        Ok(())
    }

    /// Simulate device connection
    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Device {} connecting to gateway {}:{}", 
              self.device_id, self.gateway_host, self.gateway_port);
        
        // Simulate TLS connection
        sleep(Duration::from_millis(200)).await;
        info!("Device {} TLS handshake completed", self.device_id);
        
        sleep(Duration::from_millis(100)).await;
        info!("Device {} authentication successful", self.device_id);
        
        self.connected = true;
        info!("Device {} connected successfully", self.device_id);
        
        Ok(())
    }

    /// Simulate heartbeat
    pub async fn send_heartbeat(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.connected {
            return Err("Device not connected".into());
        }
        
        info!("Device {} sending heartbeat", self.device_id);
        sleep(Duration::from_millis(50)).await;
        info!("Device {} heartbeat acknowledged", self.device_id);
        
        Ok(())
    }

    /// Simulate WASM application execution
    pub async fn execute_wasm_app(&self, app_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        if !self.connected {
            return Err("Device not connected".into());
        }
        
        info!("Device {} executing WASM application: {}", self.device_id, app_name);
        
        // Simulate WASM execution
        sleep(Duration::from_millis(100)).await;
        info!("Device {} WASM application {} loaded", self.device_id, app_name);
        
        sleep(Duration::from_millis(200)).await;
        info!("Device {} WASM application {} running", self.device_id, app_name);
        
        Ok(())
    }

    /// Simulate microROS communication
    pub async fn microros_communication(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.connected {
            return Err("Device not connected".into());
        }
        
        info!("Device {} starting microROS communication", self.device_id);
        
        // Simulate microROS topics
        let topics = vec![
            "/fmu/out/vehicle_status",
            "/fmu/out/battery_status", 
            "/fmu/out/vehicle_local_position",
            "/fmu/in/vehicle_command",
            "/fmu/in/position_setpoint"
        ];
        
        for topic in topics {
            sleep(Duration::from_millis(100)).await;
            info!("Device {} subscribed to topic: {}", self.device_id, topic);
        }
        
        // Simulate data exchange
        sleep(Duration::from_millis(500)).await;
        info!("Device {} microROS communication active", self.device_id);
        
        Ok(())
    }

    /// Run complete device simulation
    pub async fn run_simulation(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting device simulation for {}", self.device_id);
        
        // Enrollment
        self.enroll().await?;
        
        // Connection
        self.connect().await?;
        
        // Execute microROS application
        self.execute_wasm_app("microros-px4-bridge").await?;
        
        // Start microROS communication
        self.microros_communication().await?;
        
        // Send heartbeats
        for i in 1..=5 {
            self.send_heartbeat().await?;
            sleep(Duration::from_secs(2)).await;
        }
        
        info!("Device {} simulation completed", self.device_id);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting Wasmbed MCU Device Simulation");
    
    // Create multiple device clients
    let devices = vec![
        "mcu-device-1",
        "mcu-device-2", 
        "mcu-device-3",
        "mcu-device-4"
    ];
    
    let mut clients = Vec::new();
    
    for device_id in devices {
        let mut client = McuDeviceClient::new(
            device_id.to_string(),
            "172.19.0.2".to_string(),
            30423
        );
        
        // Run simulation for each device
        if let Err(e) = client.run_simulation().await {
            error!("Device {} simulation failed: {}", device_id, e);
        } else {
            info!("Device {} simulation completed successfully", device_id);
        }
        
        clients.push(client);
    }
    
    info!("All device simulations completed");
    Ok(())
}

