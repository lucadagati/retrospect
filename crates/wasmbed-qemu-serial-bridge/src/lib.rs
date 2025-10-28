// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error, debug};

/// Serial bridge message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerialMessage {
    /// Data from external client to QEMU
    DataToQemu { data: Vec<u8> },
    /// Data from QEMU to external client
    DataFromQemu { data: Vec<u8> },
    /// Heartbeat message
    Heartbeat,
    /// Device status update
    StatusUpdate { status: String },
    /// Error message
    Error { message: String },
}

/// Connection state
#[derive(Debug, Clone)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error(String),
}

/// Device connection information
#[derive(Debug, Clone)]
pub struct DeviceConnection {
    pub device_id: String,
    pub state: ConnectionState,
    pub external_addr: Option<SocketAddr>,
    pub qemu_addr: Option<SocketAddr>,
    pub last_heartbeat: Option<std::time::Instant>,
}

/// TCP Serial Bridge for QEMU ARM Cortex-M communication
pub struct QemuSerialBridge {
    devices: Arc<RwLock<HashMap<String, DeviceConnection>>>,
    external_port: u16,
    qemu_base_port: u16,
}

impl QemuSerialBridge {
    /// Create a new serial bridge
    pub fn new(external_port: u16, qemu_base_port: u16) -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            external_port,
            qemu_base_port,
        }
    }

    /// Start the serial bridge server
    pub async fn start(&self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.external_port)).await?;
        info!("Serial bridge listening on port {}", self.external_port);

        loop {
            let (stream, addr) = listener.accept().await?;
            info!("New external connection from {}", addr);
            
            let bridge = self.clone();
            tokio::spawn(async move {
                if let Err(e) = bridge.handle_external_connection(stream, addr).await {
                    error!("Error handling external connection: {}", e);
                }
            });
        }
    }

    /// Handle external client connection
    async fn handle_external_connection(&self, mut external_stream: TcpStream, addr: SocketAddr) -> anyhow::Result<()> {
        // Read device ID from the first message
        let mut buffer = [0u8; 1024];
        let n = external_stream.read(&mut buffer).await?;
        
        if n == 0 {
            return Ok(());
        }

        // Parse device ID from the first message
        let device_id = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
        info!("External client {} connecting for device {}", addr, device_id);

        // Update device connection state
        {
            let mut devices = self.devices.write().await;
            devices.insert(device_id.clone(), DeviceConnection {
                device_id: device_id.clone(),
                state: ConnectionState::Connecting,
                external_addr: Some(addr),
                qemu_addr: None,
                last_heartbeat: Some(std::time::Instant::now()),
            });
        }

        // Connect to QEMU serial port
        let qemu_addr = format!("127.0.0.1:{}", self.qemu_base_port + self.get_device_index(&device_id));
        let qemu_stream = match TcpStream::connect(&qemu_addr).await {
            Ok(stream) => {
                info!("Connected to QEMU at {}", qemu_addr);
                stream
            }
            Err(e) => {
                error!("Failed to connect to QEMU at {}: {}", qemu_addr, e);
                return Err(anyhow::anyhow!("QEMU connection failed: {}", e));
            }
        };

        // Update device connection state
        {
            let mut devices = self.devices.write().await;
            if let Some(device) = devices.get_mut(&device_id) {
                device.state = ConnectionState::Connected;
                device.qemu_addr = Some(qemu_addr.parse()?);
            }
        }

        // Create bidirectional communication
        let (mut qemu_read, mut qemu_write) = qemu_stream.into_split();
        let (mut external_read, mut external_write) = external_stream.into_split();

        // Channel for coordination
        let (tx1, mut rx1) = mpsc::channel::<()>(1);
        let (tx2, mut rx2) = mpsc::channel::<()>(1);

        // Spawn task to forward data from external client to QEMU
        let device_id_clone = device_id.clone();
        let devices_clone = self.devices.clone();
        tokio::spawn(async move {
            let mut buffer = [0u8; 4096];
            loop {
                tokio::select! {
                    result = external_read.read(&mut buffer) => {
                        match result {
                            Ok(0) => {
                                info!("External client disconnected for device {}", device_id_clone);
                                break;
                            }
                            Ok(n) => {
                                debug!("Forwarding {} bytes from external to QEMU for device {}", n, device_id_clone);
                                if let Err(e) = qemu_write.write_all(&buffer[..n]).await {
                                    error!("Failed to write to QEMU for device {}: {}", device_id_clone, e);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Error reading from external client for device {}: {}", device_id_clone, e);
                                break;
                            }
                        }
                    }
                    _ = rx1.recv() => {
                        info!("Shutting down external->QEMU forwarder for device {}", device_id_clone);
                        break;
                    }
                }
            }
            
            // Update device state
            let mut devices = devices_clone.write().await;
            if let Some(device) = devices.get_mut(&device_id_clone) {
                device.state = ConnectionState::Disconnected;
            }
        });

        // Spawn task to forward data from QEMU to external client
        let device_id_clone = device_id.clone();
        let devices_clone = self.devices.clone();
        tokio::spawn(async move {
            let mut buffer = [0u8; 4096];
            loop {
                tokio::select! {
                    result = qemu_read.read(&mut buffer) => {
                        match result {
                            Ok(0) => {
                                info!("QEMU disconnected for device {}", device_id_clone);
                                break;
                            }
                            Ok(n) => {
                                debug!("Forwarding {} bytes from QEMU to external for device {}", n, device_id_clone);
                                if let Err(e) = external_write.write_all(&buffer[..n]).await {
                                    error!("Failed to write to external client for device {}: {}", device_id_clone, e);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Error reading from QEMU for device {}: {}", device_id_clone, e);
                                break;
                            }
                        }
                    }
                    _ = rx2.recv() => {
                        info!("Shutting down QEMU->external forwarder for device {}", device_id_clone);
                        break;
                    }
                }
            }
            
            // Update device state
            let mut devices = devices_clone.write().await;
            if let Some(device) = devices.get_mut(&device_id_clone) {
                device.state = ConnectionState::Disconnected;
            }
        });

        // Keep the connection alive
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        // Send shutdown signal
        let _ = tx1.send(()).await;
        let _ = tx2.send(()).await;

        Ok(())
    }

    /// Get device connection status
    pub async fn get_device_status(&self, device_id: &str) -> Option<DeviceConnection> {
        let devices = self.devices.read().await;
        devices.get(device_id).cloned()
    }

    /// List all device connections
    pub async fn list_devices(&self) -> Vec<DeviceConnection> {
        let devices = self.devices.read().await;
        devices.values().cloned().collect()
    }

    /// Send data to a specific device
    pub async fn send_to_device(&self, device_id: &str, data: &[u8]) -> anyhow::Result<()> {
        let devices = self.devices.read().await;
        if let Some(device) = devices.get(device_id) {
            if let Some(qemu_addr) = device.qemu_addr {
                let mut stream = TcpStream::connect(qemu_addr).await?;
                stream.write_all(data).await?;
                info!("Sent {} bytes to device {}", data.len(), device_id);
                Ok(())
            } else {
                Err(anyhow::anyhow!("Device {} not connected to QEMU", device_id))
            }
        } else {
            Err(anyhow::anyhow!("Device {} not found", device_id))
        }
    }

    /// Get device index for port calculation
    fn get_device_index(&self, device_id: &str) -> u16 {
        // Simple hash-based device index
        device_id.chars().map(|c| c as u16).sum::<u16>() % 1000
    }
}

impl Clone for QemuSerialBridge {
    fn clone(&self) -> Self {
        Self {
            devices: self.devices.clone(),
            external_port: self.external_port,
            qemu_base_port: self.qemu_base_port,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_serial_bridge_creation() {
        let bridge = QemuSerialBridge::new(30001, 30000);
        assert_eq!(bridge.external_port, 30001);
        assert_eq!(bridge.qemu_base_port, 30000);
    }

    #[tokio::test]
    async fn test_device_index_calculation() {
        let bridge = QemuSerialBridge::new(30001, 30000);
        let index1 = bridge.get_device_index("device-1");
        let index2 = bridge.get_device_index("device-2");
        assert_ne!(index1, index2);
    }
}
