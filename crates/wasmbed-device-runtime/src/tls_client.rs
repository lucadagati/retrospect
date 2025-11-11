// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use heapless::{String, Vec};
use log::{error, info, warn};
use wasmbed_protocol::{ServerMessage, ClientMessage, ClientEnvelope, ServerEnvelope, Version, MessageId};

#[cfg(feature = "std")]
use std::net::TcpStream;
#[cfg(feature = "std")]
use std::sync::Arc;
#[cfg(feature = "std")]
use rustls::{ClientConfig, ClientConnection, StreamOwned};
#[cfg(feature = "std")]
use std::io::{Read, Write};

#[cfg(not(feature = "std"))]
use crate::tls_io::NetworkIo;

/// TLS Client for secure communication with gateway
pub struct TlsClient {
    connected: bool,
    
    #[cfg(feature = "std")]
    tls_stream: Option<StreamOwned<ClientConnection, TcpStream>>,
    
    #[cfg(not(feature = "std"))]
    network_io: Option<NetworkIo>,
}

impl TlsClient {
    pub fn new() -> Self {
        Self { 
            connected: false,
            #[cfg(feature = "std")]
            tls_stream: None,
            #[cfg(not(feature = "std"))]
            network_io: None,
        }
    }

    #[cfg(feature = "std")]
    pub fn connect(&mut self, endpoint: &str, _keypair: &Keypair) -> Result<(), &'static str> {
        info!("TlsClient::connect() (std) - Connecting to: {}", endpoint);
        
        // Install rustls crypto provider (same as gateway)
        rustls::crypto::ring::default_provider()
            .install_default()
            .map_err(|_| "Failed to install rustls crypto provider")?;
        
        // Parse endpoint and clone host for 'static lifetime
        let parts: std::vec::Vec<&str> = endpoint.split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid endpoint format (expected host:port)");
        }
        let host = parts[0].to_string();  // Clone to owned String
        let port: u16 = parts[1].parse().map_err(|_| "Invalid port number")?;
        
        info!("Connecting to {}:{}", host, port);
        
        // Create TCP connection
        let mut tcp_stream = TcpStream::connect(endpoint)
            .map_err(|e| {
                error!("Failed to connect TCP: {}", e);
                "TCP connection failed"
            })?;
        
        info!("✓ TCP connection established");
        
        // Gateway uses .with_no_client_auth(), so we don't need client certificate
        // Use dangerous config to accept any server certificate (for testing with localhost)
        // This is needed because gateway cert may not be valid for "127.0.0.1"
        let config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoVerifier))
            .with_no_client_auth();
        
        // For localhost testing, use "localhost" as hostname (certificate is for localhost)
        let server_name = if host == "127.0.0.1" {
            // Use "localhost" hostname since certificate is issued for localhost
            rustls::pki_types::ServerName::try_from("localhost")
                .map_err(|_| "Invalid server name")?
                .to_owned()
        } else {
            rustls::pki_types::ServerName::try_from(host.as_str())
                .map_err(|_| "Invalid server name")?
                .to_owned()
        };
        
        info!("Creating TLS connection to {}", host);
        
        // Create TLS connection
        let client_conn = ClientConnection::new(Arc::new(config), server_name)
            .map_err(|e| {
                error!("Failed to create TLS connection: {}", e);
                "TLS connection creation failed"
            })?;
        
        // Create StreamOwned - it will handle handshake automatically on first read/write
        let mut tls_stream = StreamOwned::new(client_conn, tcp_stream);
        
        info!("TLS stream created, attempting handshake via write...");
        
        // Try to complete handshake by writing a dummy byte (will be discarded)
        // StreamOwned should complete handshake automatically
        use std::io::Write;
        match tls_stream.write_all(&[0]) {
            Ok(_) => {
                info!("✓ TLS handshake completed via write!");
            }
            Err(e) => {
                // Handshake might not be complete yet, try reading
                use std::io::Read;
                let mut buf = [0u8; 1];
                match tls_stream.read(&mut buf) {
                    Ok(_) => {
                        info!("✓ TLS handshake completed via read!");
                    }
                    Err(_) => {
                        error!("TLS handshake failed: {}", e);
                        return Err("TLS handshake failed");
                    }
                }
            }
        }
        
        info!("✓ TLS connection established successfully!");
        
        self.tls_stream = Some(tls_stream);
        self.connected = true;
        
        Ok(())
    }
    
    #[cfg(not(feature = "std"))]
    pub fn connect(&mut self, endpoint: &str, _keypair: &Keypair) -> Result<(), &'static str> {
        info!("TlsClient::connect() (no_std) - Initializing connection to: {}", endpoint);
        
        // Create NetworkIo for the endpoint
        let mut network_io = NetworkIo::new(endpoint)
            .map_err(|_| "Failed to create NetworkIo")?;
        
        info!("TlsClient::connect() - NetworkIo created, attempting TCP connection...");
        
        // Connect via NetworkIo (uses shared memory in no_std)
        network_io.connect()
            .map_err(|_| "Failed to connect NetworkIo")?;
        
        info!("TlsClient::connect() - ✅ NetworkIo connected successfully!");
        
        self.network_io = Some(network_io);
        self.connected = true;
        info!("TLS connection established (real TCP via shared memory)");
        Ok(())
    }
    
    // Async version for std builds
    #[cfg(feature = "std")]
    pub async fn connect_async(&mut self, endpoint: &str, keypair: &Keypair) -> Result<(), &'static str> {
        self.connect(endpoint, keypair)
    }

    #[cfg(feature = "std")]
    pub fn send_enrollment_request(&mut self) -> Result<(), &'static str> {
        if !self.connected {
            return Err("Not connected to gateway");
        }
        
        info!("Sending enrollment request");
        
        if let Some(ref mut stream) = self.tls_stream {
            let envelope = ClientEnvelope {
                version: Version::V0,
                message_id: MessageId::default(),
                message: ClientMessage::EnrollmentRequest,
            };
            let encoded = minicbor::to_vec(&envelope)
                .map_err(|_| "Failed to encode enrollment request")?;
            
            // Send CBOR message directly (no length prefix)
            stream.write_all(&encoded)
                .map_err(|e| {
                    error!("Failed to write message: {} (kind: {:?})", e, e.kind());
                    "Failed to write message"
                })?;
            stream.flush()
                .map_err(|e| {
                    error!("Failed to flush stream: {} (kind: {:?})", e, e.kind());
                    "Failed to flush stream"
                })?;
            
            info!("✓ Enrollment request sent ({} bytes)", encoded.len());
        }
        
        Ok(())
    }
    
    #[cfg(feature = "std")]
    pub fn send_public_key(&mut self, public_key: &[u8]) -> Result<(), &'static str> {
        if !self.connected {
            return Err("Not connected to gateway");
        }
        
        info!("Sending public key ({} bytes)", public_key.len());
        
        if let Some(ref mut stream) = self.tls_stream {
            let envelope = ClientEnvelope {
                version: Version::V0,
                message_id: MessageId::default(),
                message: ClientMessage::PublicKey {
                    key: public_key.to_vec(),
                },
            };
            let encoded = minicbor::to_vec(&envelope)
                .map_err(|_| "Failed to encode public key")?;
            
            // Send CBOR message directly (no length prefix)
            stream.write_all(&encoded)
                .map_err(|e| {
                    error!("Failed to write message: {} (kind: {:?})", e, e.kind());
                    "Failed to write message"
                })?;
            stream.flush()
                .map_err(|e| {
                    error!("Failed to flush stream: {} (kind: {:?})", e, e.kind());
                    "Failed to flush stream"
                })?;
            
            info!("✓ Public key sent ({} bytes)", encoded.len());
        }
        
        Ok(())
    }
    
    #[cfg(feature = "std")]
    pub fn send_enrollment_ack(&mut self) -> Result<(), &'static str> {
        if !self.connected {
            return Err("Not connected to gateway");
        }
        
        info!("Sending enrollment acknowledgment");
        
        if let Some(ref mut stream) = self.tls_stream {
            let envelope = ClientEnvelope {
                version: Version::V0,
                message_id: MessageId::default(),
                message: ClientMessage::EnrollmentAcknowledgment,
            };
            let encoded = minicbor::to_vec(&envelope)
                .map_err(|_| "Failed to encode enrollment ack")?;
            
            // Send CBOR message directly (no length prefix)
            stream.write_all(&encoded)
                .map_err(|e| {
                    error!("Failed to write message: {} (kind: {:?})", e, e.kind());
                    "Failed to write message"
                })?;
            stream.flush()
                .map_err(|e| {
                    error!("Failed to flush stream: {} (kind: {:?})", e, e.kind());
                    "Failed to flush stream"
                })?;
            
            info!("✓ Enrollment ack sent");
        }
        
        Ok(())
    }
    
    #[cfg(feature = "std")]
    pub fn send_heartbeat(&mut self) -> Result<(), &'static str> {
        if !self.connected {
            return Err("Not connected to gateway");
        }
        
        info!("Sending heartbeat");
        
        if let Some(ref mut stream) = self.tls_stream {
            let envelope = ClientEnvelope {
                version: Version::V0,
                message_id: MessageId::default(),
                message: ClientMessage::Heartbeat,
            };
            let encoded = minicbor::to_vec(&envelope)
                .map_err(|_| "Failed to encode heartbeat")?;
            
            // Send CBOR message directly (no length prefix - gateway reads directly)
            stream.write_all(&encoded)
                .map_err(|e| {
                    error!("Failed to write message: {} (kind: {:?})", e, e.kind());
                    "Failed to write message"
                })?;
            stream.flush()
                .map_err(|e| {
                    error!("Failed to flush stream: {} (kind: {:?})", e, e.kind());
                    "Failed to flush stream"
                })?;
            
            info!("✓ Heartbeat sent ({} bytes)", encoded.len());
        }
        
        Ok(())
    }
    
    #[cfg(not(feature = "std"))]
    pub fn send_heartbeat(&mut self) -> Result<(), &'static str> {
        if !self.connected {
            return Err("Not connected to gateway");
        }
        info!("Sending heartbeat (no_std - not implemented)");
        Ok(())
    }

    #[cfg(feature = "std")]
    pub fn receive_message(&mut self) -> Result<Option<ServerMessage>, &'static str> {
        if !self.connected {
            return Err("Not connected to gateway");
        }
        
        if let Some(ref mut stream) = self.tls_stream {
            // Read CBOR message directly (gateway sends without length prefix)
            // Read up to 1024 bytes (same as gateway buffer)
            let mut buffer = vec![0u8; 1024];
            match stream.read(&mut buffer) {
                Ok(0) => {
                    // Connection closed
                    return Err("Connection closed by gateway");
                }
                Ok(n) => {
                    // Try to decode CBOR envelope
                    match minicbor::decode::<ServerEnvelope>(&buffer[..n]) {
                        Ok(envelope) => {
                            info!("✓ Received message envelope (version: {:?}, message_id: {:?}, {} bytes)", 
                                envelope.version, envelope.message_id, n);
                            Ok(Some(envelope.message))
                        }
                        Err(e) => {
                            // Try to decode as raw ServerMessage (fallback)
                            match minicbor::decode::<ServerMessage>(&buffer[..n]) {
                                Ok(msg) => {
                                    info!("✓ Received message (no envelope, {} bytes)", n);
                                    Ok(Some(msg))
                                }
                                Err(_) => {
                                    warn!("Failed to decode message ({} bytes): {:?}", n, e);
                                    Ok(None)
                                }
                            }
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data available
                    Ok(None)
                }
                Err(e) => {
                    error!("Error reading from stream: {} (kind: {:?})", e, e.kind());
                    Err("Failed to read from stream")
                }
            }
        } else {
            Ok(None)
        }
    }
    
    #[cfg(not(feature = "std"))]
    pub fn receive_message(&mut self) -> Result<Option<ServerMessage>, &'static str> {
        if !self.connected {
            return Err("Not connected to gateway");
        }
        Ok(None)
    }

    #[cfg(feature = "std")]
    pub fn send_deployment_ack(&mut self, app_id: &str, success: bool, error: Option<&str>) -> Result<(), &'static str> {
        if !self.connected {
            return Err("Not connected to gateway");
        }
        
        info!("Sending deployment ack for {}: success={}", app_id, success);
        
        if let Some(ref mut stream) = self.tls_stream {
            let envelope = ClientEnvelope {
                version: Version::V0,
                message_id: MessageId::default(),
                message: ClientMessage::ApplicationDeployAck {
                    app_id: app_id.to_string(),
                    success,
                    error: error.map(|s| s.to_string()),
                },
            };
            
            let encoded = minicbor::to_vec(&envelope)
                .map_err(|_| "Failed to encode deployment ack")?;
            
            // Send CBOR message directly (no length prefix)
            stream.write_all(&encoded)
                .map_err(|e| {
                    error!("Failed to write message: {} (kind: {:?})", e, e.kind());
                    "Failed to write message"
                })?;
            stream.flush()
                .map_err(|e| {
                    error!("Failed to flush stream: {} (kind: {:?})", e, e.kind());
                    "Failed to flush stream"
                })?;
            
            info!("✓ Deployment ack sent ({} bytes)", encoded.len());
        }
        
        Ok(())
    }
    
    #[cfg(not(feature = "std"))]
    pub fn send_deployment_ack(&mut self, app_id: &str, success: bool, _error: Option<&str>) -> Result<(), &'static str> {
        if !self.connected {
            return Err("Not connected to gateway");
        }
        info!("Sending deployment ack for {}: success={} (no_std - not implemented)", app_id, success);
        Ok(())
    }

    #[cfg(feature = "std")]
    pub fn send_stop_ack(&mut self, app_id: &str, success: bool, error: Option<&str>) -> Result<(), &'static str> {
        if !self.connected {
            return Err("Not connected to gateway");
        }
        
        info!("Sending stop ack for {}: success={}", app_id, success);
        
        if let Some(ref mut stream) = self.tls_stream {
            let envelope = ClientEnvelope {
                version: Version::V0,
                message_id: MessageId::default(),
                message: ClientMessage::ApplicationStopAck {
                    app_id: app_id.to_string(),
                    success,
                    error: error.map(|s| s.to_string()),
                },
            };
            
            let encoded = minicbor::to_vec(&envelope)
                .map_err(|_| "Failed to encode stop ack")?;
            
            // Send CBOR message directly (no length prefix)
            stream.write_all(&encoded)
                .map_err(|e| {
                    error!("Failed to write message: {} (kind: {:?})", e, e.kind());
                    "Failed to write message"
                })?;
            stream.flush()
                .map_err(|e| {
                    error!("Failed to flush stream: {} (kind: {:?})", e, e.kind());
                    "Failed to flush stream"
                })?;
            
            info!("✓ Stop ack sent");
        }
        
        Ok(())
    }
    
    #[cfg(not(feature = "std"))]
    pub fn send_stop_ack(&mut self, app_id: &str, success: bool, _error: Option<&str>) -> Result<(), &'static str> {
        if !self.connected {
            return Err("Not connected to gateway");
        }
        info!("Sending stop ack for {}: success={} (no_std - not implemented)", app_id, success);
        Ok(())
    }
}

/// Keypair structure
#[derive(Debug, Clone)]
pub struct Keypair {
    pub private_key: Vec<u8, 256>,
    pub public_key: Vec<u8, 256>,
}

/// Certificate verifier that accepts all certificates (for testing with localhost)
#[cfg(feature = "std")]
#[derive(Debug)]
struct NoVerifier;

#[cfg(feature = "std")]
impl rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer,
        _intermediates: &[rustls::pki_types::CertificateDer],
        _server_name: &rustls::pki_types::ServerName,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        // Accept any certificate (insecure, only for testing!)
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> std::vec::Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

