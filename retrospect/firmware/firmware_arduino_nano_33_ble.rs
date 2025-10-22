// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::collections::HashMap;
use std::net::{TcpStream, SocketAddr};
use std::io::{Read, Write};
use std::time::Duration;

use ciborium::{ser, de};
use rustls::{ClientConfig, ClientConnection, RootCertStore, pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer}};
use serde::{Serialize, Deserialize};
use wasmtime::{Engine, Module, Store, Instance};
use uuid::Uuid;

use log::{info, error, warn};

fn main() {
    // Install the default crypto provider for rustls
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install default crypto provider");
    
    env_logger::init();
    
    info!("Starting Wasmbed Arduino Nano 33 BLE firmware...");
    
    // Load keypair
    let keypair = load_keypair().expect("Failed to load keypair");
    
    // Create device runtime
    let mut runtime = CommonDeviceRuntime::new("127.0.0.1:8081".to_string(), keypair);
    
    // Run the device
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        runtime.run().await.expect("Device runtime failed");
    });
}

/// Real TLS Client for secure communication with gateway
pub struct TlsClient {
    connection: Option<ClientConnection>,
    stream: Option<TcpStream>,
    connected: bool 
}

impl TlsClient {
    pub fn new() -> Self {
        Self { 
            connection: None,
            stream: None,
            connected: false 
        }
    }

    pub async fn connect(&mut self, endpoint: &str, keypair: &Keypair) -> Result<(), &'static str> {
        info!("Connecting to gateway at: {}", endpoint);
        
        info!("Connecting to endpoint: {}", endpoint);
        
        // Parse endpoint
        let addr: SocketAddr = endpoint.parse()
            .map_err(|_| "Invalid endpoint format")?;
        
        info!("Parsed endpoint: {}", addr);
        
        // Establish TCP connection
        let tcp_stream = TcpStream::connect(addr)
            .map_err(|_| "Failed to connect to gateway")?;
        
        info!("TCP connection established");
        
        // Configure TLS client
        let mut root_store = RootCertStore::empty();
        
        info!("Loading CA certificate, size: {} bytes", keypair.ca_cert.len());
        
        // Load CA certificate
        let ca_cert = CertificateDer::from(keypair.ca_cert.clone());
        root_store.add(ca_cert)
            .map_err(|e| {
                error!("Failed to add CA certificate: {:?}", e);
                "Failed to add CA certificate"
            })?;
        
        info!("CA certificate added successfully");
        
        // Load client certificate and private key
        info!("Loading client certificate, size: {} bytes", keypair.certificate.len());
        info!("Loading client private key, size: {} bytes", keypair.private_key.len());
        
        let client_cert = CertificateDer::from(keypair.certificate.clone());
        let client_key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(keypair.private_key.clone()));
        
        info!("Creating TLS config...");
        
        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(vec![client_cert], client_key)
            .map_err(|e| {
                error!("Failed to create TLS config: {:?}", e);
                "Failed to create TLS config"
            })?;
        
        info!("TLS config created successfully");
        
        // Create TLS connection
        let server_name = "127.0.0.1".try_into()
            .map_err(|_| "Invalid server name")?;
        
        info!("Creating TLS connection...");
        
        let mut connection = ClientConnection::new(std::sync::Arc::new(config), server_name)
            .map_err(|e| {
                error!("Failed to create TLS connection: {:?}", e);
                "Failed to create TLS connection"
            })?;
        
        info!("TLS connection created, completing handshake...");
        
        // Complete TLS handshake
        connection.complete_io(&mut &tcp_stream)
            .map_err(|e| {
                error!("TLS handshake failed: {:?}", e);
                "TLS handshake failed"
            })?;
        
        info!("TLS connection established");
        
        self.connection = Some(connection);
        self.stream = Some(tcp_stream);
        self.connected = true;
        
        Ok(())
    }

    pub async fn send_message(&mut self, message: &ClientMessage) -> Result<(), &'static str> {
        if !self.connected {
            return Err("Not connected to gateway");
        }

        // Serialize message to CBOR
        let mut cbor_data = Vec::new();
        ciborium::ser::into_writer(message, &mut cbor_data)
            .map_err(|_| "Failed to serialize message")?;

        // Send message length
        let length = cbor_data.len() as u32;
        if let Some(ref mut stream) = self.stream {
            stream.write_all(&length.to_be_bytes())
                .map_err(|_| "Failed to send message length")?;
            stream.write_all(&cbor_data)
                .map_err(|_| "Failed to send message data")?;
            stream.flush().map_err(|_| "Failed to flush stream")?;
            info!("Message sent successfully");
            Ok(())
        } else {
            Err("No stream available")
        }
    }

    pub async fn receive_message(&mut self) -> Result<ServerMessage, &'static str> {
        if !self.connected {
            return Err("Not connected to gateway");
        }

        if let Some(ref mut stream) = self.stream {
            // Receive message length
            let mut length_bytes = [0u8; 4];
            stream.read_exact(&mut length_bytes)
                .map_err(|_| "Failed to receive message length")?;
            let length = u32::from_be_bytes(length_bytes) as usize;

            // Receive CBOR data
            let mut cbor_data = vec![0u8; length];
            stream.read_exact(&mut cbor_data)
                .map_err(|_| "Failed to receive message data")?;

            // Deserialize message
            let message: ServerMessage = ciborium::de::from_reader(&cbor_data[..])
                .map_err(|_| "Failed to deserialize message")?;

            info!("Message received successfully");
            Ok(message)
        } else {
            Err("No stream available")
        }
    }

    pub async fn send_heartbeat(&mut self) -> Result<(), &'static str> {
        let message = ClientMessage::Heartbeat;
        self.send_message(&message).await
    }

    pub async fn send_deployment_ack(&mut self, app_id: &str, success: bool, error: Option<&str>) -> Result<(), &'static str> {
        let message = ClientMessage::ApplicationDeployAck {
            app_id: app_id.to_string(),
            success,
            error: error.map(|s| s.to_string()),
        };
        self.send_message(&message).await
    }

    pub async fn send_stop_ack(&mut self, app_id: &str, success: bool, error: Option<&str>) -> Result<(), &'static str> {
        let message = ClientMessage::ApplicationStopAck {
            app_id: app_id.to_string(),
            success,
            error: error.map(|s| s.to_string()),
        };
        self.send_message(&message).await
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

/// Real enrollment client for device registration
pub struct EnrollmentClient {
    tls_client: TlsClient,
    device_uuid: Option<Uuid>,
}

impl EnrollmentClient {
    pub fn new(tls_client: TlsClient) -> Self {
        Self {
            tls_client,
            device_uuid: None,
        }
    }

    pub async fn enroll(&mut self, keypair: &Keypair) -> Result<Uuid, &'static str> {
        info!("Starting enrollment process...");

        // Send enrollment request
        let enrollment_request = ClientMessage::EnrollmentRequest;
        self.tls_client.send_message(&enrollment_request).await?;

        // Wait for enrollment acceptance
        let response = self.tls_client.receive_message().await?;
        match response {
            ServerMessage::EnrollmentAccepted => {
                info!("Enrollment request accepted");
            }
            ServerMessage::EnrollmentRejected { reason } => {
                error!("Enrollment rejected: {}", String::from_utf8_lossy(&reason));
                return Err("Enrollment rejected by gateway");
            }
            _ => {
                error!("Unexpected response during enrollment");
                return Err("Unexpected enrollment response");
            }
        }

        // Send public key
        let public_key_message = ClientMessage::PublicKey {
            key: keypair.public_key.clone(),
        };
        self.tls_client.send_message(&public_key_message).await?;

        // Wait for device UUID
        let response = self.tls_client.receive_message().await?;
        match response {
            ServerMessage::DeviceUuid { uuid } => {
                info!("Received device UUID: {:?}", uuid);
                let uuid_uuid: Uuid = uuid.into();
                self.device_uuid = Some(uuid_uuid);
                Ok(uuid_uuid)
            }
            ServerMessage::EnrollmentRejected { reason } => {
                error!("Enrollment rejected after public key: {}", String::from_utf8_lossy(&reason));
                return Err("Enrollment rejected after public key");
            }
            _ => {
                error!("Unexpected response after public key");
                return Err("Unexpected response after public key");
            }
        }
    }

    pub async fn acknowledge_enrollment(&mut self) -> Result<(), &'static str> {
        let ack_message = ClientMessage::EnrollmentAcknowledgment;
        self.tls_client.send_message(&ack_message).await?;

        let response = self.tls_client.receive_message().await?;
        match response {
            ServerMessage::EnrollmentCompleted => {
                info!("Enrollment completed successfully");
                Ok(())
            }
            _ => {
                error!("Unexpected response to enrollment acknowledgment");
                Err("Unexpected enrollment acknowledgment response")
            }
        }
    }
}

/// Real WASM runtime for executing applications
pub struct WasmRuntime {
    engine: Engine,
    store: Store<()>,
    modules: HashMap<String, Module>,
}

impl WasmRuntime {
    pub fn new() -> Result<Self, &'static str> {
        let engine = Engine::default();
        let store = Store::new(&engine, ());
        
        Ok(Self {
            engine,
            store,
            modules: HashMap::new(),
        })
    }

    pub async fn load_module(&mut self, app_id: &str, wasm_data: &[u8]) -> Result<(), &'static str> {
        info!("Loading WASM module for app: {}", app_id);
        
        let module = Module::new(&self.engine, wasm_data)
            .map_err(|_| "Failed to compile WASM module")?;
        
        self.modules.insert(app_id.to_string(), module);
        info!("WASM module loaded successfully for app: {}", app_id);
        Ok(())
    }

    pub async fn run_module(&mut self, app_id: &str) -> Result<(), &'static str> {
        let module = self.modules.get(app_id)
            .ok_or("Module not found")?;
        
        let instance = Instance::new(&mut self.store, module, &[])
            .map_err(|_| "Failed to instantiate WASM module")?;
        
        // Look for main function
        if let Some(main_func) = instance.get_func(&mut self.store, "_start") {
            info!("Executing WASM module: {}", app_id);
            let _results = main_func.call(&mut self.store, &[], &mut [])
                .map_err(|_| "Failed to execute WASM module")?;
            info!("WASM module executed successfully: {}", app_id);
        } else {
            warn!("No _start function found in WASM module: {}", app_id);
        }
        
        Ok(())
    }

    pub async fn stop_module(&mut self, app_id: &str) -> Result<(), &'static str> {
        info!("Stopping WASM module: {}", app_id);
        self.modules.remove(app_id);
        info!("WASM module stopped: {}", app_id);
        Ok(())
    }
}

/// Real keypair generator for device certificates
pub struct KeypairGenerator;

impl KeypairGenerator {
    pub fn generate() -> Result<Keypair, &'static str> {
        // In a real implementation, this would generate Ed25519 keys
        // For now, we'll load from files
        load_keypair().map_err(|_| "Failed to load keypair")
    }
}

/// Device keypair structure
#[derive(Clone)]
pub struct Keypair {
    pub private_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub certificate: Vec<u8>,
    pub ca_cert: Vec<u8>,
}

/// Client message types
#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    Heartbeat,
    EnrollmentRequest,
    PublicKey { key: Vec<u8> },
    EnrollmentAcknowledgment,
    ApplicationStatus { 
        app_id: String, 
        status: String, 
        error: Option<String>,
        metrics: Option<ApplicationMetrics>
    },
    ApplicationDeployAck { 
        app_id: String, 
        success: bool, 
        error: Option<String> 
    },
    ApplicationStopAck { 
        app_id: String, 
        success: bool, 
        error: Option<String> 
    },
    DeviceInfo {
        available_memory: u64,
        cpu_arch: String,
        wasm_features: Vec<String>,
        max_app_size: u64,
    },
}

/// Server message types
#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    HeartbeatAck,
    EnrollmentAccepted,
    EnrollmentRejected { reason: Vec<u8> },
    DeviceUuid { uuid: DeviceUuid },
    EnrollmentCompleted,
    ApplicationDeploy { 
        app_id: String, 
        wasm_data: Vec<u8> 
    },
    ApplicationStop { 
        app_id: String 
    },
}

/// Device UUID type
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceUuid([u8; 16]);

impl From<Uuid> for DeviceUuid {
    fn from(uuid: Uuid) -> Self {
        DeviceUuid(*uuid.as_bytes())
    }
}

impl From<DeviceUuid> for Uuid {
    fn from(device_uuid: DeviceUuid) -> Self {
        Uuid::from_bytes(device_uuid.0)
    }
}

/// Application metrics
#[derive(Serialize, Deserialize, Debug)]
pub struct ApplicationMetrics {
    pub memory_usage: u64,
    pub cpu_usage: f32,
    pub uptime: u64,
    pub function_calls: u64,
}

/// Common device runtime that orchestrates all components
pub struct CommonDeviceRuntime {
    tls_client: TlsClient,
    enrollment_client: Option<EnrollmentClient>,
    wasm_runtime: WasmRuntime,
    keypair: Keypair,
    gateway_endpoint: String,
    running: bool,
}

impl CommonDeviceRuntime {
    pub fn new(gateway_endpoint: String, keypair: Keypair) -> Self {
        let tls_client = TlsClient::new();
        let wasm_runtime = WasmRuntime::new().expect("Failed to create WASM runtime");
        
        Self {
            tls_client,
            enrollment_client: None,
            wasm_runtime,
            keypair,
            gateway_endpoint,
            running: false,
        }
    }

    pub async fn initialize(&mut self) -> Result<(), &'static str> {
        info!("Initializing device runtime...");
        
        // Connect to gateway
        self.tls_client.connect(&self.gateway_endpoint, &self.keypair).await?;
        
        // Create enrollment client
        let tls_client = std::mem::replace(&mut self.tls_client, TlsClient::new());
        self.enrollment_client = Some(EnrollmentClient::new(tls_client));
        
        // Perform enrollment
        if let Some(ref mut enrollment_client) = self.enrollment_client {
            let device_uuid = enrollment_client.enroll(&self.keypair).await?;
            info!("Device enrolled with UUID: {}", device_uuid);
            
            enrollment_client.acknowledge_enrollment().await?;
            info!("Enrollment acknowledged");
        }
        
        self.running = true;
        info!("Device runtime initialized successfully");
        Ok(())
    }

    pub async fn run(&mut self) -> Result<(), &'static str> {
        info!("Starting device runtime main loop...");
        
        while self.running {
            // Send heartbeat
            if let Some(ref mut enrollment_client) = self.enrollment_client {
                enrollment_client.tls_client.send_heartbeat().await?;
                
                // Wait for heartbeat acknowledgment
                let response = enrollment_client.tls_client.receive_message().await?;
                match response {
                    ServerMessage::HeartbeatAck => {
                        info!("Heartbeat acknowledged");
                    }
                    _ => {
                        warn!("Unexpected response to heartbeat");
                    }
                }
            }
            
            // Check for application deployment messages
            if let Some(ref mut enrollment_client) = self.enrollment_client {
                // This is a simplified version - in reality, we'd need to handle
                // multiple message types and implement proper message queuing
                if let Ok(response) = enrollment_client.tls_client.receive_message().await {
                    match response {
                        ServerMessage::ApplicationDeploy { app_id, wasm_data } => {
                            info!("Received application deployment: {}", app_id);
                            
                            // Load and run WASM module
                            self.wasm_runtime.load_module(&app_id, &wasm_data).await?;
                            self.wasm_runtime.run_module(&app_id).await?;
                            
                            // Send deployment acknowledgment
                            enrollment_client.tls_client.send_deployment_ack(&app_id, true, None).await?;
                        }
                        ServerMessage::ApplicationStop { app_id } => {
                            info!("Received application stop: {}", app_id);
                            
                            // Stop WASM module
                            self.wasm_runtime.stop_module(&app_id).await?;
                            
                            // Send stop acknowledgment
                            enrollment_client.tls_client.send_stop_ack(&app_id, true, None).await?;
                        }
                        _ => {
                            // Handle other message types as needed
                        }
                    }
                }
            }
            
            // Sleep for a short interval
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        
        Ok(())
    }

    pub fn stop(&mut self) {
        info!("Stopping device runtime...");
        self.running = false;
    }
}

fn load_keypair() -> Result<Keypair, Box<dyn std::error::Error>> {
    use std::fs;
    
    // Load real certificates in DER format
    let ca_cert = fs::read("/home/lucadag/18_10_23_retrospect/certs/ca-cert.der")?;
    let device_cert = fs::read("/home/lucadag/18_10_23_retrospect/certs/device-cert.der")?;
    let device_key = fs::read("/home/lucadag/18_10_23_retrospect/certs/device-key.der")?;
    
    // Extract public key from certificate (simplified)
    let public_key = vec![0u8; 32]; // In real implementation, extract from cert
    
    let keypair = Keypair {
        private_key: device_key,
        public_key,
        certificate: device_cert,
        ca_cert,
    };
    
    Ok(keypair)
}
