// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use kube::{Api, Client, ResourceExt};
use tokio_util::sync::CancellationToken;
use tracing::{Level, error, info, warn, debug};
use tracing_subscriber::FmtSubscriber;

use wasmbed_k8s_resource::{Device, DeviceStatusUpdate, Application};
use wasmbed_protocol::{ClientMessage, ServerMessage, DeviceUuid};
use wasmbed_tls_utils::{TlsUtils, Server, ServerConfig, ServerIdentity, AuthorizationResult, MessageContext};
use wasmbed_types::{GatewayReference, PublicKey};

mod http_api;
use http_api::{HttpApiServer, DeviceCapabilities};

#[derive(Parser)]
#[command(disable_help_subcommand = true)]
struct Args {
    #[arg(long, env = "WASMBED_GATEWAY_BIND_ADDR")]
    bind_addr: SocketAddr,
    #[arg(long, env = "WASMBED_GATEWAY_HTTP_ADDR", default_value = "0.0.0.0:8080")]
    http_addr: SocketAddr,
    #[arg(long, env = "WASMBED_GATEWAY_PRIVATE_KEY")]
    private_key: PathBuf,
    #[arg(long, env = "WASMBED_GATEWAY_CERTIFICATE")]
    certificate: PathBuf,
    #[arg(long, env = "WASMBED_GATEWAY_CLIENT_CA")]
    client_ca: PathBuf,
    #[arg(long, env = "WASMBED_GATEWAY_NAMESPACE")]
    namespace: String,
    #[arg(long, env = "WASMBED_GATEWAY_POD_NAMESPACE")]
    pod_namespace: String,
    #[arg(long, env = "WASMBED_GATEWAY_POD_NAME")]
    pod_name: String,
    #[arg(long, env = "WASMBED_GATEWAY_PAIRING_MODE", default_value = "false")]
    pairing_mode: bool,
    #[arg(long, env = "WASMBED_GATEWAY_PAIRING_TIMEOUT", default_value = "300")]
    pairing_timeout_seconds: u64,
}

struct Callbacks {
    api: Api<Device>,
    gateway_reference: GatewayReference,
    http_server: Arc<HttpApiServer>,
}

impl Callbacks {
    fn on_connect(&self) -> Box<OnClientConnect> {
        let api = self.api.clone();
        let gateway_reference = self.gateway_reference.clone();
        let http_server = self.http_server.clone();
        Box::new(move |public_key: PublicKey<'static>| {
            let api = api.clone();
            let gateway_reference = gateway_reference.clone();
            let http_server = http_server.clone();
            Box::pin(async move {
                // Verify TLS client authentication by checking if the public key
                // from the client certificate matches a registered device
                match Device::find(api.clone(), public_key.clone()).await {
                    Ok(Some(device)) => {
                        // Verify that the public key from the certificate matches the stored device public key
                        if device.spec.public_key == public_key {
                            // Device exists and public key matches, mark as connected
                            info!("TLS client certificate verification successful: public key matches stored device {}", device.name_any());
                            
                            if let Err(e) = DeviceStatusUpdate::default()
                                .mark_connected(gateway_reference.clone())
                                .apply(api.clone(), device.clone())
                                .await
                            {
                                error!("Error updating DeviceStatus: {e}");
                                return AuthorizationResult::Unauthorized;
                            }
                            
                            // Register device in HTTP API
                            let device_id = device.name_any();
                            let public_key_str = public_key.to_base64();
                            let capabilities = DeviceCapabilities {
                                available_memory: 1024 * 1024 * 1024, // 1GB default
                                cpu_arch: "riscv32".to_string(),
                                wasm_features: vec!["core".to_string()],
                                max_app_size: 1024 * 1024, // 1MB default
                            };
                            http_server.register_device(device_id, public_key_str, capabilities).await;
                            
                            info!("TLS client authentication successful for existing device: {}", public_key);
                            AuthorizationResult::Authorized
                        } else {
                            error!("TLS client authentication failed: public key mismatch for device {}", device.name_any());
                            error!("Expected: {}, Got: {}", device.spec.public_key.to_base64(), public_key.to_base64());
                            AuthorizationResult::Unauthorized
                        }
                    },
                    Ok(None) => {
                        // Device doesn't exist, check if pairing mode is enabled for enrollment
                        // TODO: Implement proper pairing mode check from configuration
                        // For now, allow connection for enrollment but log the security consideration
                        warn!("TLS client authentication: unknown device attempting connection for enrollment: {}", public_key);
                        warn!("Consider enabling pairing mode for secure device enrollment");
                        AuthorizationResult::Authorized
                    },
                    Err(e) => {
                        error!("TLS client authentication failed: unable to check Device status: {e}");
                        AuthorizationResult::Unauthorized
                    },
                }
            })
        })
    }

    fn on_disconnect(&self) -> Box<OnClientDisconnect> {
        let api = self.api.clone();
        Box::new(move |public_key: PublicKey<'static>| {
            let api = api.clone();
            Box::pin(async move {
                // Mark device as disconnected when TLS connection is lost
                match Device::find(api.clone(), public_key.clone()).await {
                    Ok(Some(device)) => {
                        if let Err(e) = DeviceStatusUpdate::default()
                            .mark_disconnected()
                            .apply(api.clone(), device.clone())
                            .await
                        {
                            error!("Error updating DeviceStatus on disconnect: {e}");
                        } else {
                            info!("Device marked as disconnected: {}", public_key);
                        }
                    },
                    Ok(None) => {
                        debug!("Unknown device disconnected: {}", public_key);
                    },
                    Err(e) => {
                        error!("Error checking device status on disconnect: {e}");
                    },
                }
            })
        })
    }

    fn on_message(&self) -> Box<OnClientMessage> {
        let api = self.api.clone();
        let gateway_reference = self.gateway_reference.clone();
        let _http_server = self.http_server.clone();
        Box::new(move |ctx: MessageContext| {
            let api = api.clone();
            let gateway_reference = gateway_reference.clone();
            Box::pin(async move {
                match ctx.message() {
                    ClientMessage::Heartbeat => {
                        // Update heartbeat timestamp for the device
                        let public_key = ctx.client_public_key();
                        match Device::find(api.clone(), public_key.clone()).await {
                            Ok(Some(device)) => {
                                if let Err(e) = DeviceStatusUpdate::default()
                                    .update_heartbeat()
                                    .apply(api.clone(), device.clone())
                                    .await
                                {
                                    error!("Error updating heartbeat: {e}");
                                }
                            },
                            Ok(None) => {
                                debug!("Heartbeat from unknown device: {}", public_key);
                            },
                            Err(e) => {
                                error!("Error checking device status for heartbeat: {e}");
                            },
                        }
                        let _ = ctx.reply(ServerMessage::HeartbeatAck);
                    },
                    ClientMessage::EnrollmentRequest => {
                        info!("Received enrollment request from device");
                        
                        // Check if pairing mode is enabled (for now, always accept)
                        // TODO: Implement pairing mode check from configuration
                        let _ = ctx.reply(ServerMessage::EnrollmentAccepted);
                        info!("Enrollment request accepted");
                    },
                    ClientMessage::PublicKey { key } => {
                        info!("Received public key during enrollment: {} bytes", key.len());
                        
                        // Verify that the public key in the message matches the TLS certificate public key
                        let tls_public_key = ctx.client_public_key();
                        let message_public_key = PublicKey::from(key.as_slice());
                        
                        if *tls_public_key != message_public_key {
                            error!("TLS client authentication failed during enrollment: public key mismatch");
                            let _ = ctx.reply(ServerMessage::EnrollmentRejected { 
                                reason: "Public key mismatch with TLS certificate".as_bytes().to_vec() 
                            });
                            return;
                        }
                        
                        info!("TLS client authentication verified during enrollment");
                        
                        // Generate a unique UUID for this device
                        let uuid = uuid::Uuid::new_v4();
                        let device_uuid = DeviceUuid::new(*uuid.as_bytes());
                        
                        // Create a new Device CRD in Kubernetes
                        match create_device_crd(&key, &device_uuid, &api, &gateway_reference).await {
                            Ok(device_name) => {
                                info!("Created Device CRD: {}", device_name);
                                let _ = ctx.reply(ServerMessage::DeviceUuid { uuid: device_uuid });
                            },
                            Err(e) => {
                                error!("Failed to create Device CRD: {}", e);
                                let _ = ctx.reply(ServerMessage::EnrollmentRejected { 
                                    reason: format!("Failed to create device: {}", e).into_bytes() 
                                });
                            }
                        }
                    },
                    ClientMessage::EnrollmentAcknowledgment => {
                        info!("Received enrollment acknowledgment");
                        
                        // Mark enrollment as completed
                        let _ = ctx.reply(ServerMessage::EnrollmentCompleted);
                        info!("Enrollment completed successfully");
                    },
                    ClientMessage::ApplicationStatus { app_id, status, error, metrics } => {
                        info!("Received application status for {}: {:?}", app_id, status);
                        if let Some(err) = error {
                            warn!("Application {} error: {}", app_id, err);
                        }
                        if let Some(m) = metrics {
                            debug!("Application {} metrics: memory={}, cpu={}%, uptime={}s, calls={}", 
                                   app_id, m.memory_usage, m.cpu_usage, m.uptime, m.function_calls);
                        }
                        // TODO: Update Application CRD status
                    },
                    ClientMessage::ApplicationDeployAck { app_id, success, error } => {
                        if success {
                            info!("Application {} deployed successfully", app_id);
                        } else {
                            error!("Application {} deployment failed: {}", app_id, error.as_deref().unwrap_or("Unknown error"));
                        }
                        // TODO: Update Application CRD status
                    },
                    ClientMessage::ApplicationStopAck { app_id, success, error } => {
                        if success {
                            info!("Application {} stopped successfully", app_id);
                        } else {
                            error!("Application {} stop failed: {}", app_id, error.as_deref().unwrap_or("Unknown error"));
                        }
                        // TODO: Update Application CRD status
                    },
                    ClientMessage::DeviceInfo { available_memory, cpu_arch, wasm_features, max_app_size } => {
                        info!("Received device info: arch={}, memory={}MB, max_app_size={}KB, features={:?}", 
                              cpu_arch, available_memory / 1024 / 1024, max_app_size / 1024, wasm_features);
                        
                        // TODO: Update device capabilities in HTTP API when we have device identification
                        // For now, just log the information
                    },
                }
            })
        })
    }
}

/// Create a new Device CRD in Kubernetes during enrollment
async fn create_device_crd(
    public_key: &[u8],
    device_uuid: &DeviceUuid,
    api: &Api<Device>,
    gateway_reference: &GatewayReference,
) -> Result<String, anyhow::Error> {
    // Convert public key to base64 for storage
    let public_key_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, public_key);
    
    // Create device name from UUID
    let device_name = format!("device-{}", device_uuid.to_string().replace("-", ""));
    
    // Create Device spec
    let device_spec = wasmbed_k8s_resource::DeviceSpec {
        public_key: wasmbed_types::PublicKey::from(public_key_b64.into_bytes()),
    };
    
    // Create Device status
    let device_status = wasmbed_k8s_resource::DeviceStatus {
        phase: wasmbed_k8s_resource::DevicePhase::Pending,
        gateway: Some(gateway_reference.clone()),
        connected_since: None,
        last_heartbeat: None,
    };
    
    // Create Device object
    let device = Device {
        metadata: kube::api::ObjectMeta {
            name: Some(device_name.clone()),
            namespace: Some("wasmbed".to_string()),
            ..Default::default()
        },
        spec: device_spec,
        status: Some(device_status),
    };
    
    // Apply to Kubernetes
    api.create(&kube::api::PostParams::default(), &device).await?;
    
    Ok(device_name)
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();

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
    let client_ca_bytes =
        std::fs::read(&args.client_ca).with_context(|| {
            format!(
                "Failed to read client CA certificate from {}",
                args.client_ca.display()
            )
        })?;

    // Parse PEM certificates using our custom TLS utils
    let private_key = TlsUtils::parse_private_key(&private_key_bytes)
        .with_context(|| "Failed to parse private key")?;
    
    let certificate = TlsUtils::parse_certificate(&certificate_bytes)
        .with_context(|| "Failed to parse certificate")?;
    
    let client_ca_certs = TlsUtils::parse_certificates(&client_ca_bytes)
        .with_context(|| "Failed to parse client CA certificates")?;

    let server_key = match private_key {
        rustls_pki_types::PrivateKeyDer::Pkcs8(pkcs8) => pkcs8,
        _ => return Err(anyhow::anyhow!("Only PKCS8 private keys are supported")),
    };
    
    let client_ca = client_ca_certs
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("No CA certificate found in PEM file"))?;

    let identity = ServerIdentity::from_parts(server_key, certificate);

    let gateway_reference =
        GatewayReference::new(&args.pod_namespace, &args.pod_name);

    let shutdown = CancellationToken::new();
    let shutdown_clone = shutdown.clone();

    tokio::spawn(async move {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                info!("Received Ctrl+C, shutting down...");
                shutdown_clone.cancel();
            },
            Err(err) => {
                error!("Unable to listen for shutdown signal: {}", err);
            },
        }
    });

    let client = Client::try_default().await?;
    let api: Api<Device> = Api::namespaced(client.clone(), &args.namespace);
    let application_api: Api<Application> = Api::namespaced(client.clone(), &args.namespace);

    // Create HTTP API server
    let http_server = HttpApiServer::new(api.clone(), application_api)?;
    let http_server = Arc::new(http_server);

    let callbacks = Callbacks {
        api: api.clone(),
        gateway_reference: gateway_reference.clone(),
        http_server: http_server.clone(),
    };

    let config = ServerConfig {
        bind_addr: args.bind_addr,
        identity,
        client_ca,
        on_client_connect: Arc::from(callbacks.on_connect()),
        on_client_disconnect: Arc::from(callbacks.on_disconnect()),
        on_client_message: Arc::from(callbacks.on_message()),
        shutdown: shutdown.clone(),
    };

    let server = Server::new(config);
    
    // Start HTTP API server
    let http_router = http_server.router();
    let http_shutdown = shutdown.clone();
    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(args.http_addr).await.unwrap();
        info!("Starting HTTP API server on {}", args.http_addr);
        
        axum::serve(listener, http_router)
            .with_graceful_shutdown(async move {
                http_shutdown.cancelled().await;
            })
            .await
            .unwrap();
    });

    info!("Starting TLS server on {}", args.bind_addr);
    if let Err(e) = server.run().await {
        error!("Server error: {}", e);
    }

    Ok(())
}
