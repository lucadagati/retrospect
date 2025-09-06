// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Error, Result};
use clap::Parser;
use tokio::net::TcpStream;
use rustls::{DigitallySignedStruct, RootCertStore};
use rustls::client::{ClientConfig, WebPkiServerVerifier};
use rustls::client::danger::{
    HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier,
};
use rustls_pki_types::{CertificateDer, ServerName, UnixTime};
use tokio_rustls::TlsConnector;
use wasmbed_protocol::{ClientMessage, ServerMessage, DeviceUuid, ClientEnvelope, ServerEnvelope, Version, MessageId};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(long)]
    address: SocketAddr,
    #[arg(long)]
    server_ca: PathBuf,
    #[arg(long)]
    private_key: PathBuf,
    #[arg(long)]
    certificate: PathBuf,
    #[arg(long, default_value = "enrollment")]
    mode: String,
}

#[derive(Debug)]
pub struct NoServerNameVerification {
    inner: Arc<WebPkiServerVerifier>,
}

impl NoServerNameVerification {
    pub fn new(inner: Arc<WebPkiServerVerifier>) -> Self {
        Self { inner }
    }
}

impl ServerCertVerifier for NoServerNameVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        match self.inner.verify_server_cert(
            _end_entity,
            _intermediates,
            _server_name,
            _ocsp,
            _now,
        ) {
            Ok(scv) => Ok(scv),
            Err(rustls::Error::InvalidCertificate(cert_error)) => {
                match cert_error {
                    rustls::CertificateError::NotValidForName
                    | rustls::CertificateError::NotValidForNameContext {
                        ..
                    } => Ok(ServerCertVerified::assertion()),
                    _ => Err(rustls::Error::InvalidCertificate(cert_error)),
                }
            },
            Err(e) => Err(e),
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}

struct ProtocolClient {
    stream: tokio_rustls::client::TlsStream<TcpStream>,
}

impl ProtocolClient {
    fn new(stream: tokio_rustls::client::TlsStream<TcpStream>) -> Self {
        Self { stream }
    }
    
    async fn send(&mut self, message: ClientMessage) -> Result<(), Error> {
        let envelope = ClientEnvelope {
            version: Version::V0,
            message_id: MessageId::default(),
            message,
        };
        
        let mut buffer = Vec::new();
        minicbor::encode(envelope, &mut buffer)?;
        
        let length = (buffer.len() as u32).to_be_bytes();
        self.stream.write_all(&length).await?;
        self.stream.write_all(&buffer).await?;
        self.stream.flush().await?;
        
        Ok(())
    }
    
    async fn recv(&mut self) -> Result<ServerMessage, Error> {
        let mut length_bytes = [0u8; 4];
        self.stream.read_exact(&mut length_bytes).await?;
        let length = u32::from_be_bytes(length_bytes) as usize;
        
        let mut buffer = vec![0u8; length];
        self.stream.read_exact(&mut buffer).await?;
        
        let envelope: ServerEnvelope = minicbor::decode(&buffer)?;
        Ok(envelope.message)
    }
}

async fn test_application_deployment(client: &mut ProtocolClient, wasm_file: &str) -> Result<()> {
    println!("🧪 Testing application deployment workflow...");
    
    // Step 1: Send device info
    println!("📊 Sending device info...");
    let device_info = ClientMessage::DeviceInfo {
        available_memory: 1024 * 1024, // 1MB
        cpu_arch: "riscv32imac".to_string(),
        wasm_features: vec!["i32".to_string(), "i64".to_string()],
        max_app_size: 512 * 1024, // 512KB
    };
    client.send(device_info).await?;
    
    // Step 2: Wait for deployment request
    let response = client.recv().await?;
    match response {
        ServerMessage::DeployApplication { app_id, name, wasm_bytes, config } => {
            println!("📦 Received deployment request:");
            println!("   App ID: {}", app_id);
            println!("   Name: {}", name);
            println!("   WASM size: {} bytes", wasm_bytes.len());
            if let Some(cfg) = config {
                println!("   Memory limit: {} bytes", cfg.memory_limit);
                println!("   CPU time limit: {} ms", cfg.cpu_time_limit);
            }
            
            // Step 3: Simulate deployment process
            println!("🚀 Simulating deployment process...");
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            // Step 4: Send deployment acknowledgment
            println!("✅ Sending deployment acknowledgment...");
            client.send(ClientMessage::ApplicationDeployAck {
                app_id: app_id.clone(),
                success: true,
                error: None,
            }).await?;
            
            // Step 5: Send application status (running)
            println!("📊 Sending application status (running)...");
            let metrics = wasmbed_protocol::ApplicationMetrics {
                memory_usage: 1024, // 1KB
                cpu_usage: 5.0, // 5%
                uptime: 10, // 10 seconds
                function_calls: 42,
            };
            client.send(ClientMessage::ApplicationStatus {
                app_id,
                status: wasmbed_protocol::ApplicationStatus::Running,
                error: None,
                metrics: Some(metrics),
            }).await?;
            
        },
        ServerMessage::RequestDeviceInfo => {
            println!("📊 Received device info request...");
            let device_info = ClientMessage::DeviceInfo {
                available_memory: 1024 * 1024, // 1MB
                cpu_arch: "riscv32imac".to_string(),
                wasm_features: vec!["i32".to_string(), "i64".to_string()],
                max_app_size: 512 * 1024, // 512KB
            };
            client.send(device_info).await?;
        },
        _ => {
            return Err(Error::msg(format!("Unexpected response: {:?}", response)));
        }
    }
    
    println!("🎉 Application deployment test completed successfully!");
    Ok(())
}

async fn test_enrollment(client: &mut ProtocolClient) -> Result<()> {
    println!("🧪 Testing enrollment workflow...");
    
    // Step 1: Send enrollment request
    println!("📝 Sending enrollment request...");
    client.send(ClientMessage::EnrollmentRequest).await?;
    
    // Step 2: Wait for enrollment accepted
    let response = client.recv().await?;
    match response {
        ServerMessage::EnrollmentAccepted => {
            println!("✅ Enrollment request accepted");
        },
        _ => {
            return Err(Error::msg(format!("Unexpected response: {:?}", response)));
        }
    }
    
    // Step 3: Send public key (simulating device public key)
    println!("🔑 Sending public key...");
    let public_key = vec![0x01, 0x02, 0x03, 0x04, 0x05]; // Simulated public key
    client.send(ClientMessage::PublicKey { key: public_key }).await?;
    
    // Step 4: Wait for device UUID
    let response = client.recv().await?;
    match response {
        ServerMessage::DeviceUuid { uuid } => {
            println!("🆔 Received device UUID: {}", uuid.to_string());
        },
        ServerMessage::EnrollmentRejected { reason } => {
            let reason_str = String::from_utf8_lossy(&reason);
            return Err(Error::msg(format!("Enrollment rejected: {}", reason_str)));
        },
        _ => {
            return Err(Error::msg(format!("Unexpected response: {:?}", response)));
        }
    }
    
    // Step 5: Send enrollment acknowledgment
    println!("✅ Sending enrollment acknowledgment...");
    client.send(ClientMessage::EnrollmentAcknowledgment).await?;
    
    // Step 6: Wait for enrollment completed
    let response = client.recv().await?;
    match response {
        ServerMessage::EnrollmentCompleted => {
            println!("🎉 Enrollment completed successfully!");
        },
        _ => {
            return Err(Error::msg(format!("Unexpected response: {:?}", response)));
        }
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let server_ca_bytes =
        std::fs::read(&args.server_ca).with_context(|| {
            format!(
                "Failed to read server CA certificate from {}",
                args.server_ca.display()
            )
        })?;
    let private_key_bytes =
        std::fs::read(&args.private_key).with_context(|| {
            format!(
                "Failed to read private key from {}",
                args.private_key.display()
            )
        })?;
    let certificate_bytes =
        std::fs::read(&args.certificate).with_context(|| {
            format!(
                "Failed to read certificate from {}",
                args.certificate.display()
            )
        })?;

    let mut root_store = RootCertStore::empty();
    root_store
        .add(server_ca_bytes.into())
        .context("failed to add CA certificate to root store")?;

    let mut config = ClientConfig::builder()
        .with_root_certificates(root_store.clone())
        .with_client_auth_cert(
            vec![certificate_bytes.into()],
            private_key_bytes.try_into().map_err(Error::msg)?,
        )
        .context("invalid client cert or key")?;

    config.dangerous().set_certificate_verifier(Arc::new(
        NoServerNameVerification::new(
            WebPkiServerVerifier::builder(Arc::new(root_store.clone()))
                .build()?,
        ),
    ));

    let connector = TlsConnector::from(Arc::new(config));

    let stream = TcpStream::connect(args.address)
        .await
        .context("failed to connect")?;

    let tls_stream = connector
        .connect("example.com".try_into()?, stream)
        .await
        .context("TLS handshake failed")?;

    println!("🔗 Successfully connected and verified TLS");

    // Create protocol client
    let mut client = ProtocolClient::new(tls_stream);
    
    match args.mode.as_str() {
        "enrollment" => {
            test_enrollment(&mut client).await?;
        },
        "deploy" => {
            test_application_deployment(&mut client, "test-app.wasm").await?;
        },
        "heartbeat" => {
            println!("💓 Testing heartbeat...");
            client.send(ClientMessage::Heartbeat).await?;
            let response = client.recv().await?;
            match response {
                ServerMessage::HeartbeatAck => {
                    println!("✅ Heartbeat acknowledged");
                },
                _ => {
                    return Err(Error::msg(format!("Unexpected response: {:?}", response)));
                }
            }
        },
        _ => {
            return Err(Error::msg(format!("Unknown mode: {}", args.mode)));
        }
    }

    println!("🎯 Test completed successfully!");
    Ok(())
}
