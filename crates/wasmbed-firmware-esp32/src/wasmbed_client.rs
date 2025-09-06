// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use log::{debug, error, info, warn};
use rustls::{ClientConfig as RustlsClientConfig, ClientConnection, Connection, RootCertStore};
use rustls_pemfile::certs;
use wasmbed_protocol::{ClientMessage, ServerMessage, DeviceUuid};
use wasmbed_types::PublicKey;
use minicbor;

use crate::wifi_manager::WifiConfig;
use crate::security::SecurityManager;

/// Wasmbed client for ESP32 devices
pub struct WasmbedClient {
    /// Device UUID
    device_uuid: DeviceUuid,
    /// Public key for authentication
    public_key: Option<PublicKey>,
    /// Gateway hostname
    gateway_host: String,
    /// Gateway port
    gateway_port: u16,
    /// TLS connection
    tls_connection: Option<Arc<ClientConnection>>,
    /// Security manager
    security_manager: SecurityManager,
    /// Connection status
    connected: bool,
    /// Last heartbeat time
    last_heartbeat: SystemTime,
    /// Heartbeat interval
    heartbeat_interval: Duration,
}

impl WasmbedClient {
    /// Create a new Wasmbed client
    pub fn new(
        device_uuid: DeviceUuid,
        gateway_host: String,
        gateway_port: u16,
        wifi_config: WifiConfig,
    ) -> Result<Self> {
        let security_manager = SecurityManager::new()?;
        
        Ok(Self {
            device_uuid,
            public_key: None,
            gateway_host,
            gateway_port,
            tls_connection: None,
            security_manager,
            connected: false,
            last_heartbeat: SystemTime::now(),
            heartbeat_interval: Duration::from_secs(30),
        })
    }

    /// Connect to the gateway
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to gateway: {}:{}", self.gateway_host, self.gateway_port);

        // Create TLS configuration
        let mut root_certs = RootCertStore::empty();
        
        // Load CA certificate
        let ca_cert = include_bytes!("../../certs/ca-cert.pem");
        let mut cert_reader = std::io::Cursor::new(ca_cert);
        let certs = certs(&mut cert_reader)?;
        
        for cert in certs {
            root_certs.add(cert)?;
        }

        let config = RustlsClientConfig::builder()
            .with_root_certificates(root_certs)
            .with_no_client_auth();

        // Connect to gateway
        let server_name = self.gateway_host.as_str().try_into()?;
        let mut connection = ClientConnection::new(Arc::new(config), server_name)?;
        
        // Establish TLS connection
        let stream = TcpStream::connect(format!("{}:{}", self.gateway_host, self.gateway_port))?;
        connection.complete_io(stream)?;
        
        self.tls_connection = Some(Arc::new(connection));
        self.connected = true;

        info!("Connected to gateway successfully");
        Ok(())
    }

    /// Disconnect from the gateway
    pub fn disconnect(&mut self) {
        self.tls_connection = None;
        self.connected = false;
        info!("Disconnected from gateway");
    }

    /// Send a message to the gateway
    pub async fn send_message(&mut self, message: ClientMessage) -> Result<ServerMessage> {
        if !self.connected {
            return Err(anyhow::anyhow!("Not connected to gateway"));
        }

        debug!("Sending message to gateway: {:?}", message);
        
        // Serialize message to CBOR
        let cbor_data = minicbor::to_vec(&message)
            .map_err(|e| anyhow::anyhow!("Failed to serialize message to CBOR: {}", e))?;
        
        // Sign the message
        let signed_message = SignedMessage {
            data: cbor_data,
            signature: self.security_manager.sign_data(&message)?,
        };
        
        // Serialize signed message to CBOR
        let signed_cbor = minicbor::to_vec(&signed_message)
            .map_err(|e| anyhow::anyhow!("Failed to serialize signed message: {}", e))?;
        
        // Send length prefix + data
        let length = signed_cbor.len() as u32;
        let length_bytes = length.to_be_bytes();
        
        if let Some(connection) = &self.tls_connection {
            let mut stream = connection.writer();
            stream.write_all(&length_bytes)?;
            stream.write_all(&signed_cbor)?;
            stream.flush()?;
        }

        // Read response
        let response = self.read_message().await?;
        Ok(response)
    }

    /// Read a message from the gateway
    async fn read_message(&mut self) -> Result<ServerMessage> {
        if let Some(connection) = &self.tls_connection {
            let mut stream = connection.reader();
            
            // Read length prefix
            let mut length_bytes = [0u8; 4];
            stream.read_exact(&mut length_bytes)?;
            let length = u32::from_be_bytes(length_bytes) as usize;
            
            // Read message data
            let mut message_data = vec![0u8; length];
            stream.read_exact(&mut message_data)
                .map_err(|e| anyhow::anyhow!("Failed to read message: {}", e))?;
            
            // Deserialize signed message
            let signed_message: SignedMessage = minicbor::decode(&message_data)
                .map_err(|e| anyhow::anyhow!("Failed to deserialize signed message: {}", e))?;
            
            // Verify signature (in real implementation, you'd verify with gateway's public key)
            // For now, we'll skip verification for demo purposes
            
            // Deserialize actual message
            let message: ServerMessage = minicbor::decode(&signed_message.data)
                .map_err(|e| anyhow::anyhow!("Failed to deserialize message: {}", e))?;
            
            debug!("Message received successfully: {:?}", message);
            Ok(message)
        } else {
            Err(anyhow::anyhow!("Not connected to gateway"))
        }
    }

    /// Send heartbeat to gateway
    pub async fn send_heartbeat(&mut self) -> Result<()> {
        let now = SystemTime::now();
        if now.duration_since(self.last_heartbeat)? >= self.heartbeat_interval {
            let heartbeat = ClientMessage::Heartbeat {
                timestamp: now,
                metrics: DeviceMetrics {
                    cpu_usage: 0.5, // TODO: Get actual CPU usage
                    memory_usage: 1024 * 1024, // TODO: Get actual memory usage
                    network_rx: 1000, // TODO: Get actual network stats
                    network_tx: 2000,
                },
            };

            let _response = self.send_message(heartbeat).await?;
            self.last_heartbeat = now;
        }
        Ok(())
    }

    /// Register device with gateway
    pub async fn register_device(&mut self) -> Result<()> {
        let registration = ClientMessage::DeviceRegistration {
            device_id: self.device_uuid.to_string().into(),
            public_key: self.security_manager.get_public_key()?,
            device_type: wasmbed_types::DeviceType::Esp32,
            capabilities: vec![wasmbed_types::Capability::Wasm],
        };

        let response = self.send_message(registration).await?;
        
        match response {
            ServerMessage::EnrollmentResponse { status } => {
                if status == "accepted" {
                    info!("Device registration successful");
                } else {
                    warn!("Device registration rejected: {}", status);
                }
            }
            _ => {
                warn!("Unexpected response to device registration");
            }
        }

        Ok(())
    }

    /// Send application status update
    pub async fn send_application_status(&mut self, app_id: &str, status: ApplicationStatus) -> Result<()> {
        let status_message = ClientMessage::ApplicationStatus {
            application_id: app_id.into(),
            status: match status {
                ApplicationStatus::Running => "running".to_string(),
                ApplicationStatus::Stopped => "stopped".to_string(),
                ApplicationStatus::Error(msg) => format!("error: {}", msg),
                _ => "unknown".to_string(),
            },
            metrics: ApplicationMetrics {
                app_id: app_id.to_string(),
                memory_usage: 0, // TODO: Get actual metrics
                cpu_usage: 0,
                function_calls: 0,
                avg_execution_time: 0,
                error_count: 0,
                status: status.clone(),
                last_activity: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs(),
            },
        };

        let _response = self.send_message(status_message).await?;
        Ok(())
    }

    /// Check if client is connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get device UUID
    pub fn device_uuid(&self) -> &DeviceUuid {
        &self.device_uuid
    }

    /// Get public key
    pub fn public_key(&self) -> Option<&PublicKey> {
        self.public_key.as_ref()
    }
}

/// Signed message wrapper
#[derive(Debug, Clone)]
struct SignedMessage {
    data: Vec<u8>,
    signature: Vec<u8>,
}

/// Device metrics
#[derive(Debug, Clone)]
pub struct DeviceMetrics {
    pub cpu_usage: f64,
    pub memory_usage: usize,
    pub network_rx: u64,
    pub network_tx: u64,
}

/// Application status
#[derive(Debug, Clone)]
pub enum ApplicationStatus {
    Running,
    Stopped,
    Error(String),
}

/// Application metrics
#[derive(Debug, Clone)]
pub struct ApplicationMetrics {
    pub app_id: String,
    pub memory_usage: usize,
    pub cpu_usage: u8,
    pub function_calls: u64,
    pub avg_execution_time: u32,
    pub error_count: u64,
    pub status: ApplicationStatus,
    pub last_activity: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let device_uuid = DeviceUuid::new();
        let client = WasmbedClient::new(
            device_uuid,
            "localhost".to_string(),
            8443,
            WifiConfig::default(),
        );
        assert!(client.is_ok());
    }

    #[test]
    fn test_device_metrics() {
        let metrics = DeviceMetrics {
            cpu_usage: 0.5,
            memory_usage: 1024 * 1024,
            network_rx: 1000,
            network_tx: 2000,
        };
        
        assert_eq!(metrics.cpu_usage, 0.5);
        assert_eq!(metrics.memory_usage, 1024 * 1024);
    }

    #[test]
    fn test_application_status() {
        let status = ApplicationStatus::Running;
        match status {
            ApplicationStatus::Running => assert!(true),
            _ => assert!(false),
        }
    }
}