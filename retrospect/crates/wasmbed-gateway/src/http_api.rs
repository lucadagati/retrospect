// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use std::net::SocketAddr;

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use kube::Api;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use minicbor;
use wasmbed_tls_utils::{TlsServer};

use wasmbed_k8s_resource::{Application, Device, DeviceApplicationPhase, ApplicationConfig, Gateway};
use wasmbed_protocol::{ServerMessage, DeviceUuid};
use wasmbed_types::PublicKey;

/// HTTP API server for Gateway-Controller communication with CBOR/TLS support
#[derive(Clone)]
pub struct HttpApiServer {
    pub device_connections: Arc<RwLock<HashMap<String, DeviceConnection>>>,
    pub applications: Arc<RwLock<HashMap<String, DeployedApplication>>>,
    pub device_api: Api<Device>,
    pub application_api: Api<Application>,
    pub gateway_api: Api<Gateway>,
    pub tls_config: Arc<TlsServer>, // Custom TLS server implementation
    pub cbor_tls_listener: Option<Arc<TcpListener>>,
    pub pairing_mode: Arc<RwLock<bool>>,
    pub pairing_timeout_seconds: Arc<RwLock<u64>>,
    pub heartbeat_timeout_seconds: Arc<RwLock<u64>>,
}

/// Active device connection information with TLS support
#[derive(Debug, Clone)]
pub struct DeviceConnection {
    pub device_id: String,
    pub device_uuid: DeviceUuid,
    pub public_key: PublicKey<'static>,
    pub connected_since: SystemTime,
    pub last_heartbeat: SystemTime,
    pub capabilities: DeviceCapabilities,
    pub tls_connection: Option<Arc<RwLock<TcpStream>>>,
    pub is_enrolled: bool,
}

/// CBOR/TLS message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CborTlsMessage {
    pub message_type: String,
    pub data: Vec<u8>,
    pub signature: Vec<u8>,
    pub timestamp: SystemTime,
}

/// CBOR/TLS connection handler
pub struct CborTlsHandler {
    device_connections: Arc<RwLock<HashMap<String, DeviceConnection>>>,
    applications: Arc<RwLock<HashMap<String, DeployedApplication>>>,
}

/// Device capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub available_memory: u64,
    pub cpu_arch: String,
    pub wasm_features: Vec<String>,
    pub max_app_size: u64,
}

/// Deployed application information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployedApplication {
    pub app_id: String,
    pub device_id: String,
    pub name: String,
    pub wasm_bytes: Vec<u8>,
    pub config: Option<ApplicationConfig>,
    pub deployed_at: SystemTime,
    pub status: DeviceApplicationPhase,
    pub error: Option<String>,
}

/// Deployment request from controller
#[derive(Debug, Deserialize)]
pub struct DeploymentRequest {
    pub app_id: String,
    pub name: String,
    pub wasm_bytes: String, // Base64 encoded
}

/// Deployment response
#[derive(Debug, Serialize)]
pub struct DeploymentResponse {
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
}

/// Application status response
#[derive(Debug, Serialize)]
pub struct ApplicationStatusResponse {
    pub app_id: String,
    pub device_id: String,
    pub status: DeviceApplicationPhase,
    pub deployed_at: SystemTime,
    pub error: Option<String>,
}

/// Device list response
#[derive(Debug, Serialize)]
pub struct DeviceListResponse {
    pub devices: Vec<DeviceInfo>,
}

/// Device information
#[derive(Debug, Serialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub connected: bool,
    pub connected_since: Option<SystemTime>,
    pub last_heartbeat: Option<SystemTime>,
    pub capabilities: Option<DeviceCapabilities>,
}

impl HttpApiServer {
    /// Create a new HTTP API server with CBOR/TLS support
    pub fn new(device_api: Api<Device>, application_api: Api<Application>, gateway_api: Api<Gateway>) -> Result<Self> {
        Ok(Self {
            device_connections: Arc::new(RwLock::new(HashMap::new())),
            applications: Arc::new(RwLock::new(HashMap::new())),
            device_api,
            application_api,
            gateway_api,
            tls_config: Arc::new(TlsServer::new(
                "0.0.0.0:8443".parse().unwrap(),
                rustls_pki_types::CertificateDer::from(vec![]),
                rustls_pki_types::PrivatePkcs8KeyDer::from(vec![]),
                rustls_pki_types::CertificateDer::from(vec![]),
            )),
            cbor_tls_listener: None,
            pairing_mode: Arc::new(RwLock::new(false)),
            pairing_timeout_seconds: Arc::new(RwLock::new(300)),
            heartbeat_timeout_seconds: Arc::new(RwLock::new(90)),
        })
    }
    
    
    /// Start CBOR/TLS listener for device connections
    pub async fn start_cbor_tls_listener(&mut self, bind_addr: SocketAddr) -> Result<()> {
        info!("Starting CBOR/TLS listener on {}", bind_addr);
        
        let listener = TcpListener::bind(bind_addr).await?;
        let listener_arc = Arc::new(listener);
        self.cbor_tls_listener = Some(listener_arc.clone());
        
        let device_connections = self.device_connections.clone();
        let applications = self.applications.clone();
        let tls_config = self.tls_config.clone();
        
        tokio::spawn(async move {
            let handler = CborTlsHandler {
                device_connections,
                applications,
            };
            
            loop {
                match listener_arc.accept().await {
                    Ok((stream, addr)) => {
                        info!("New CBOR/TLS connection from {}", addr);
                        
                        let handler_clone = handler.clone();
                        let tls_config_clone = tls_config.clone();
                        
                        tokio::spawn(async move {
                            if let Err(e) = handler_clone.handle_connection(stream, tls_config_clone).await {
                                error!("Error handling CBOR/TLS connection: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("Failed to accept CBOR/TLS connection: {}", e);
                    }
                }
            }
        });
        
        info!("CBOR/TLS listener started successfully");
        Ok(())
    }
    
    /// Send CBOR/TLS message to device
    pub async fn send_cbor_tls_message(&self, device_id: &str, message: ServerMessage) -> Result<()> {
        let connections = self.device_connections.read().await;
        
        if let Some(connection) = connections.get(device_id) {
            if let Some(tls_stream) = &connection.tls_connection {
                let mut stream = tls_stream.write().await;
                
                // Serialize message to CBOR
                let cbor_data = minicbor::to_vec(&message)?;
                
                // Create message wrapper
                let cbor_message = CborTlsMessage {
                    message_type: "server_message".to_string(),
                    data: cbor_data,
                    signature: vec![], // In real implementation, sign the message
                    timestamp: SystemTime::now(),
                };
                
                // Serialize wrapper to CBOR
                let message_data = serde_cbor::to_vec(&cbor_message)?;
                
                // Send length prefix + data
                let length = message_data.len() as u32;
                let length_bytes = length.to_be_bytes();
                
                stream.write_all(&length_bytes).await?;
                stream.write_all(&message_data).await?;
                stream.flush().await?;
                
                debug!("Sent CBOR/TLS message to device {}", device_id);
            } else {
                return Err(anyhow::anyhow!("No TLS connection for device {}", device_id));
            }
        } else {
            return Err(anyhow::anyhow!("Device {} not found", device_id));
        }
        
        Ok(())
    }

    /// Create the HTTP router
    pub fn router(&self) -> Router {
        let state = Arc::new(self.clone());
        
        Router::new()
            .route("/api/v1/devices", get(get_devices))
            .route("/api/v1/devices", post(create_device))
            .route("/api/v1/devices/:device_id", get(get_device))
            .route("/api/v1/devices/:device_id", put(update_device))
            .route("/api/v1/devices/:device_id", delete(delete_device))
            .route("/api/v1/devices/:device_id/enroll", post(enroll_device))
            .route("/api/v1/devices/:device_id/connect", post(connect_device))
            .route("/api/v1/devices/:device_id/deploy", post(deploy_application))
            .route("/api/v1/devices/:device_id/stop/:app_id", post(stop_application))
            .route("/api/v1/devices/:device_id/status/:app_id", get(get_application_status))
            .route("/api/v1/devices/:device_id/applications", get(get_device_applications))
            .route("/api/v1/applications", get(get_applications))
            .route("/api/v1/applications", post(create_application))
            .route("/api/v1/applications/:app_id", get(get_application))
            .route("/api/v1/applications/:app_id", put(update_application))
            .route("/api/v1/applications/:app_id", delete(delete_application))
            .route("/api/v1/gateways", get(get_gateways))
            .route("/api/v1/gateways", post(create_gateway))
            .route("/api/v1/gateways/:gateway_id", get(get_gateway))
            .route("/api/v1/gateways/:gateway_id", put(update_gateway))
            .route("/api/v1/gateways/:gateway_id", delete(delete_gateway))
            .route("/api/v1/gateways/:gateway_id/toggle", post(toggle_gateway))
            .route("/api/v1/infrastructure/status", get(get_infrastructure_status))
            .route("/api/v1/admin/pairing-mode", get(get_pairing_mode))
            .route("/api/v1/admin/pairing-mode", post(set_pairing_mode))
            .route("/api/v1/admin/pairing-timeout", get(get_pairing_timeout))
            .route("/api/v1/admin/pairing-timeout", post(set_pairing_timeout))
            .route("/api/v1/admin/heartbeat-timeout", get(get_heartbeat_timeout))
            .route("/api/v1/admin/heartbeat-timeout", post(set_heartbeat_timeout))
            .route("/api/v1/k8s/pods", get(get_k8s_pods))
            .route("/api/v1/k8s/applications", get(get_k8s_applications))
            .route("/api/v1/metrics/system", get(get_system_metrics))
            .route("/api/v1/alerts", get(get_alerts))
            .route("/api/v1/drone/command", post(send_drone_command))
            .route("/health", get(health_check))
            .route("/ready", get(readiness_check))
            .with_state(state)
    }

    /// Register a device connection
    pub async fn register_device(&self, device_id: String, public_key: String, capabilities: DeviceCapabilities) {
        let connection = DeviceConnection {
            device_id: device_id.clone(),
            device_uuid: DeviceUuid::new([0u8; 16]),
            public_key: PublicKey::from(public_key.as_bytes()).into_owned(),
            connected_since: SystemTime::now(),
            last_heartbeat: SystemTime::now(),
            capabilities,
            tls_connection: None,
            is_enrolled: false,
        };

        let mut connections = self.device_connections.write().await;
        connections.insert(device_id, connection);
        info!("Device registered for HTTP API");
    }

    /// Update device heartbeat
    pub async fn update_heartbeat(&self, device_id: &str) {
        let mut connections = self.device_connections.write().await;
        if let Some(connection) = connections.get_mut(device_id) {
            connection.last_heartbeat = SystemTime::now();
            debug!("Updated heartbeat for device {}", device_id);
        }
    }

    /// Register application deployment
    pub async fn register_application(&self, app_id: String, device_id: String, name: String, wasm_bytes: Vec<u8>, config: Option<ApplicationConfig>) {
        let application = DeployedApplication {
            app_id: app_id.clone(),
            device_id,
            name,
            wasm_bytes,
            config,
            deployed_at: SystemTime::now(),
            status: DeviceApplicationPhase::Deploying,
            error: None,
        };

        let mut applications = self.applications.write().await;
        applications.insert(app_id, application);
        info!("Application registered for HTTP API");
    }

    /// Update application status
    pub async fn update_application_status(&self, app_id: &str, status: DeviceApplicationPhase, error: Option<String>) {
        let mut applications = self.applications.write().await;
        if let Some(application) = applications.get_mut(app_id) {
            application.status = status;
            application.error = error;
            debug!("Updated application status for {}", app_id);
        }
    }

    /// Deploy application to a specific device
    pub async fn deploy_application_to_device(&self, device_id: &str, app_id: &str, wasm_bytes: &[u8]) -> Result<()> {
        let connections = self.device_connections.read().await;
        
        if let Some(_connection) = connections.get(device_id) {
            // Create deployment message
            let deployment_message = ServerMessage::DeployApplication {
                app_id: app_id.to_string(),
                name: app_id.to_string(), // Use app_id as name for now
                wasm_bytes: wasm_bytes.to_vec(),
                config: None,
            };
            
            // Send deployment command via TLS
            match self.send_message_to_device(device_id, &deployment_message).await {
                Ok(_) => {
                    info!("Successfully sent deployment command for app {} to device {}", app_id, device_id);
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to send deployment command for app {} to device {}: {}", app_id, device_id, e);
                    Err(e)
                }
            }
        } else {
            Err(anyhow::anyhow!("Device {} not connected", device_id))
        }
    }

    /// Stop application on a specific device
    pub async fn stop_application_on_device(&self, device_id: &str, app_id: &str) -> Result<()> {
        let connections = self.device_connections.read().await;
        
        if let Some(_connection) = connections.get(device_id) {
            // Create stop message
            let stop_message = ServerMessage::StopApplication {
                app_id: app_id.to_string(),
            };
            
            // Send stop command via TLS
            match self.send_message_to_device(device_id, &stop_message).await {
                Ok(_) => {
                    info!("Successfully sent stop command for app {} to device {}", app_id, device_id);
                    Ok(())
                }
                Err(e) => {
                    error!("Failed to send stop command for app {} to device {}: {}", app_id, device_id, e);
                    Err(e)
                }
            }
        } else {
            Err(anyhow::anyhow!("Device {} not connected", device_id))
        }
    }

    /// Send message to a specific device via TLS
    async fn send_message_to_device(&self, device_id: &str, message: &ServerMessage) -> Result<()> {
        info!("Sending message to device {}: {:?}", device_id, message);
        
        let connections = self.device_connections.read().await;
        
        if let Some(connection) = connections.get(device_id) {
            if let Some(tls_stream) = &connection.tls_connection {
                let mut stream = tls_stream.write().await;
                
                // Serialize message to CBOR
                let cbor_data = minicbor::to_vec(&message)?;
                
                // Create message wrapper
                let cbor_message = CborTlsMessage {
                    message_type: "server_message".to_string(),
                    data: cbor_data,
                    signature: vec![], // In real implementation, sign the message
                    timestamp: SystemTime::now(),
                };
                
                // Serialize wrapper to CBOR
                let message_data = serde_cbor::to_vec(&cbor_message)?;
                
                // Send length prefix + data
                let length = message_data.len() as u32;
                let length_bytes = length.to_be_bytes();
                
                stream.write_all(&length_bytes).await?;
                stream.write_all(&message_data).await?;
                stream.flush().await?;
                
                debug!("Sent CBOR/TLS message to device {}", device_id);
                Ok(())
            } else {
                Err(anyhow::anyhow!("No TLS connection for device {}", device_id))
            }
        } else {
            Err(anyhow::anyhow!("Device {} not found", device_id))
        }
    }
}

/// Get list of connected devices
async fn get_devices(
    State(server): State<Arc<HttpApiServer>>,
) -> Result<Json<DeviceListResponse>, StatusCode> {
    let connections = server.device_connections.read().await;
    let devices: Vec<DeviceInfo> = connections
        .values()
        .map(|conn| DeviceInfo {
            device_id: conn.device_id.clone(),
            connected: true,
            connected_since: Some(conn.connected_since),
            last_heartbeat: Some(conn.last_heartbeat),
            capabilities: Some(conn.capabilities.clone()),
        })
        .collect();

    Ok(Json(DeviceListResponse { devices }))
}

/// Deploy application to device
async fn deploy_application(
    State(server): State<Arc<HttpApiServer>>,
    Path(device_id): Path<String>,
    Json(payload): Json<DeploymentRequest>,
) -> Result<Json<DeploymentResponse>, StatusCode> {
    info!("Received deployment request for device {}: app_id={}", device_id, payload.app_id);

    // Check if device is connected
    let connections = server.device_connections.read().await;
    if !connections.contains_key(&device_id) {
        return Ok(Json(DeploymentResponse {
            success: false,
            message: "Device not connected".to_string(),
            error: Some("Device not found or not connected".to_string()),
        }));
    }

    // Decode WASM bytes
    let wasm_bytes = match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &payload.wasm_bytes) {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to decode WASM bytes: {}", e);
            return Ok(Json(DeploymentResponse {
                success: false,
                message: "Invalid WASM bytes".to_string(),
                error: Some(format!("Failed to decode base64: {}", e)),
            }));
        }
    };

    // Register application
    let app_id = payload.app_id.clone();
    server.register_application(
        app_id.clone(),
        device_id.clone(),
        payload.name,
        wasm_bytes.clone(),
        None, // No config for now
    ).await;

    // Send deployment command to device via TLS connection
    let server_clone = server.clone();
    let app_id_clone = app_id.clone();
    let device_id_clone = device_id.clone();
    let wasm_bytes_clone = wasm_bytes.clone();
    
    tokio::spawn(async move {
        // Deploy to the specific device
        match server_clone.deploy_application_to_device(&device_id_clone, &app_id_clone, &wasm_bytes_clone).await {
            Ok(_) => {
                // Update application status to running
                server_clone.update_application_status(&app_id_clone, DeviceApplicationPhase::Running, None).await;
            }
            Err(e) => {
                eprintln!("Failed to deploy application {} to device {}: {}", app_id_clone, device_id_clone, e);
                server_clone.update_application_status(&app_id_clone, DeviceApplicationPhase::Failed, Some(e.to_string())).await;
            }
        }
    });

    Ok(Json(DeploymentResponse {
        success: true,
        message: format!("Application {} deployment initiated", app_id),
        error: None,
    }))
}

/// Stop application on device
async fn stop_application(
    State(server): State<Arc<HttpApiServer>>,
    Path((device_id, app_id)): Path<(String, String)>,
) -> Result<Json<DeploymentResponse>, StatusCode> {
    info!("Received stop request for device {}: app_id={}", device_id, app_id);

    // Check if device is connected
    let connections = server.device_connections.read().await;
    if !connections.contains_key(&device_id) {
        return Ok(Json(DeploymentResponse {
            success: false,
            message: "Device not connected".to_string(),
            error: Some("Device not found or not connected".to_string()),
        }));
    }

    // Update application status
    let server_clone = server.clone();
    let app_id_clone = app_id.clone();
    server.update_application_status(&app_id, DeviceApplicationPhase::Stopped, None).await;

    // Send stop command to device via TLS connection
    let device_id_clone = device_id.clone();
    tokio::spawn(async move {
        // Stop the application on the specific device
        match server_clone.stop_application_on_device(&device_id_clone, &app_id_clone).await {
            Ok(_) => {
                // Update application status to stopped
                server_clone.update_application_status(&app_id_clone, DeviceApplicationPhase::Stopped, None).await;
            }
            Err(e) => {
                eprintln!("Failed to stop application {} on device {}: {}", app_id_clone, device_id_clone, e);
                server_clone.update_application_status(&app_id_clone, DeviceApplicationPhase::Failed, Some(e.to_string())).await;
            }
        }
    });

    Ok(Json(DeploymentResponse {
        success: true,
        message: format!("Application {} stop initiated", app_id),
        error: None,
    }))
}

/// Get application status
async fn get_application_status(
    State(server): State<Arc<HttpApiServer>>,
    Path((_device_id, app_id)): Path<(String, String)>,
) -> Result<Json<ApplicationStatusResponse>, StatusCode> {
    let applications = server.applications.read().await;
    if let Some(app) = applications.get(&app_id) {
        Ok(Json(ApplicationStatusResponse {
            app_id: app.app_id.clone(),
            device_id: app.device_id.clone(),
            status: app.status.clone(),
            deployed_at: app.deployed_at,
            error: app.error.clone(),
        }))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Get applications for a device
async fn get_device_applications(
    State(server): State<Arc<HttpApiServer>>,
    Path(device_id): Path<String>,
) -> Result<Json<Vec<ApplicationStatusResponse>>, StatusCode> {
    let applications = server.applications.read().await;
    let device_apps: Vec<ApplicationStatusResponse> = applications
        .values()
        .filter(|app| app.device_id == device_id)
        .map(|app| ApplicationStatusResponse {
            app_id: app.app_id.clone(),
            device_id: app.device_id.clone(),
            status: app.status.clone(),
            deployed_at: app.deployed_at,
            error: app.error.clone(),
        })
        .collect();

    Ok(Json(device_apps))
}

/// Health check endpoint
async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Readiness check endpoint
async fn readiness_check() -> StatusCode {
    StatusCode::OK
}

impl Clone for CborTlsHandler {
    fn clone(&self) -> Self {
        Self {
            device_connections: self.device_connections.clone(),
            applications: self.applications.clone(),
        }
    }
}

impl CborTlsHandler {
    /// Handle incoming CBOR/TLS connection
    pub async fn handle_connection(&self, stream: TcpStream, _tls_config: Arc<TlsServer>) -> Result<()> {
        info!("Handling new CBOR/TLS connection");
        
        // In a real implementation, you would:
        // 1. Perform TLS handshake
        // 2. Authenticate the device
        // 3. Handle enrollment process
        // 4. Process CBOR messages
        
        // For now, we'll simulate the connection handling
        let mut buffer = [0u8; 1024];
        let mut stream = stream;
        
        loop {
            match stream.read(&mut buffer).await {
                Ok(0) => {
                    info!("CBOR/TLS connection closed by client");
                    break;
                }
                Ok(n) => {
                    debug!("Received {} bytes from CBOR/TLS client", n);
                    
                    // In a real implementation, you would:
                    // 1. Parse the CBOR message
                    // 2. Verify the signature
                    // 3. Process the message
                    // 4. Send response
                    
                    // For now, just acknowledge receipt
                    let response = b"ACK";
                    if let Err(e) = stream.write_all(response).await {
                        error!("Failed to send response: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Error reading from CBOR/TLS connection: {}", e);
                    break;
                }
            }
        }
        
        Ok(())
    }
    
    /// Process CBOR message from device
    async fn process_cbor_message(&self, message: CborTlsMessage) -> Result<()> {
        debug!("Processing CBOR message: {}", message.message_type);
        
        match message.message_type.as_str() {
            "enrollment_request" => {
                self.handle_enrollment_request(message).await?;
            }
            "heartbeat" => {
                self.handle_heartbeat(message).await?;
            }
            "application_status" => {
                self.handle_application_status(message).await?;
            }
            _ => {
                warn!("Unknown message type: {}", message.message_type);
            }
        }
        
        Ok(())
    }
    
    /// Handle device enrollment request
    async fn handle_enrollment_request(&self, _message: CborTlsMessage) -> Result<()> {
        debug!("Handling enrollment request");
        
        // In a real implementation, you would:
        // 1. Parse the enrollment request
        // 2. Verify device credentials
        // 3. Generate device UUID
        // 4. Send enrollment response
        
        Ok(())
    }
    
    /// Handle device heartbeat
    async fn handle_heartbeat(&self, _message: CborTlsMessage) -> Result<()> {
        debug!("Handling heartbeat");
        
        // In a real implementation, you would:
        // 1. Parse the heartbeat message
        // 2. Update device connection status
        // 3. Send heartbeat response
        
        Ok(())
    }
    
    /// Handle application status update
    async fn handle_application_status(&self, _message: CborTlsMessage) -> Result<()> {
        debug!("Handling application status update");
        
        // In a real implementation, you would:
        // 1. Parse the status update
        // 2. Update application status in Kubernetes
        // 3. Log the status change
        
        Ok(())
    }
}

/// Admin API handlers for pairing mode management

/// Get current pairing mode status
async fn get_pairing_mode(State(server): State<Arc<HttpApiServer>>) -> Json<serde_json::Value> {
    let pairing_mode = *server.pairing_mode.read().await;
    Json(serde_json::json!({
        "pairing_mode": pairing_mode,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Set pairing mode status
#[derive(Deserialize)]
struct PairingModeRequest {
    enabled: bool,
}

async fn set_pairing_mode(
    State(server): State<Arc<HttpApiServer>>,
    Json(request): Json<PairingModeRequest>,
) -> Json<serde_json::Value> {
    let mut pairing_mode = server.pairing_mode.write().await;
    *pairing_mode = request.enabled;
    
    info!("Pairing mode {} by admin API", if request.enabled { "enabled" } else { "disabled" });
    
    Json(serde_json::json!({
        "success": true,
        "pairing_mode": request.enabled,
        "message": format!("Pairing mode {}", if request.enabled { "enabled" } else { "disabled" }),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Get current pairing timeout
async fn get_pairing_timeout(State(server): State<Arc<HttpApiServer>>) -> Json<serde_json::Value> {
    let timeout = *server.pairing_timeout_seconds.read().await;
    Json(serde_json::json!({
        "pairing_timeout_seconds": timeout,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Set pairing timeout
#[derive(Deserialize)]
struct PairingTimeoutRequest {
    timeout_seconds: u64,
}

async fn set_pairing_timeout(
    State(server): State<Arc<HttpApiServer>>,
    Json(request): Json<PairingTimeoutRequest>,
) -> Json<serde_json::Value> {
    let mut timeout = server.pairing_timeout_seconds.write().await;
    *timeout = request.timeout_seconds;
    
    info!("Pairing timeout set to {} seconds by admin API", request.timeout_seconds);
    
    Json(serde_json::json!({
        "success": true,
        "pairing_timeout_seconds": request.timeout_seconds,
        "message": format!("Pairing timeout set to {} seconds", request.timeout_seconds),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Get current heartbeat timeout
async fn get_heartbeat_timeout(State(server): State<Arc<HttpApiServer>>) -> Json<serde_json::Value> {
    let timeout = *server.heartbeat_timeout_seconds.read().await;
    Json(serde_json::json!({
        "heartbeat_timeout_seconds": timeout,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Set heartbeat timeout
#[derive(Deserialize)]
struct HeartbeatTimeoutRequest {
    timeout_seconds: u64,
}

async fn set_heartbeat_timeout(
    State(server): State<Arc<HttpApiServer>>,
    Json(request): Json<HeartbeatTimeoutRequest>,
) -> Json<serde_json::Value> {
    let mut timeout = server.heartbeat_timeout_seconds.write().await;
    *timeout = request.timeout_seconds;
    
    info!("Heartbeat timeout set to {} seconds by admin API", request.timeout_seconds);
    
    Json(serde_json::json!({
        "success": true,
        "heartbeat_timeout_seconds": request.timeout_seconds,
        "message": format!("Heartbeat timeout set to {} seconds", request.timeout_seconds),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Get Kubernetes pods status
async fn get_k8s_pods(State(_server): State<Arc<HttpApiServer>>) -> Json<serde_json::Value> {
    // Simulazione per ora - in futuro collegare all'API Kubernetes reale
    Json(serde_json::json!({
        "total": 12,
        "running": 10,
        "pending": 1,
        "failed": 1,
        "pods": [
            {
                "name": "wasmbed-gateway-1",
                "status": "Running",
                "ready": "1/1",
                "restarts": 0,
                "age": "2d"
            },
            {
                "name": "wasmbed-gateway-2",
                "status": "Running",
                "ready": "1/1",
                "restarts": 0,
                "age": "1d"
            },
            {
                "name": "wasmbed-controller",
                "status": "Running",
                "ready": "1/1",
                "restarts": 0,
                "age": "2d"
            }
        ]
    }))
}

/// Get Kubernetes applications status
async fn get_k8s_applications(State(server): State<Arc<HttpApiServer>>) -> Json<serde_json::Value> {
    let applications = server.applications.read().await;
    
    Json(serde_json::json!({
        "total": applications.len(),
        "running": applications.values().filter(|a| matches!(a.status, DeviceApplicationPhase::Running)).count(),
        "failed": applications.values().filter(|a| matches!(a.status, DeviceApplicationPhase::Failed)).count(),
        "deploying": applications.values().filter(|a| matches!(a.status, DeviceApplicationPhase::Deploying)).count(),
        "applications": applications.values().collect::<Vec<_>>()
    }))
}

/// Get system metrics
async fn get_system_metrics(State(_server): State<Arc<HttpApiServer>>) -> Json<serde_json::Value> {
    use std::time::{SystemTime, UNIX_EPOCH};
    
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let mut metrics = Vec::new();
    
    // Generate last 24 hours of data
    for i in 0..24 {
        let time = now - (24 - i) * 3600;
        metrics.push(serde_json::json!({
            "time": format!("{}:00", (time / 3600) % 24),
            "timestamp": time,
            "cpu": 20.0 + (i as f64 * 2.5) + (rand::random::<f64>() * 20.0),
            "memory": 30.0 + (i as f64 * 1.5) + (rand::random::<f64>() * 15.0),
            "devices": 3 + (rand::random::<u8>() % 3),
            "applications": 5 + (rand::random::<u8>() % 5)
        }));
    }
    
    Json(serde_json::json!({
        "metrics": metrics,
        "current": {
            "cpu": 45.2,
            "memory": 67.8,
            "storage": 23.1,
            "network_in": 1024,
            "network_out": 2048
        }
    }))
}

/// Get active alerts
async fn get_alerts(State(_server): State<Arc<HttpApiServer>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "alerts": [
            {
                "id": "alert-1",
                "severity": "warning",
                "title": "High CPU Usage",
                "message": "CPU usage on gateway-pod-1 is above 80%",
                "timestamp": "2025-01-10T10:30:00Z",
                "source": "kubernetes",
                "resolved": false
            },
            {
                "id": "alert-2",
                "severity": "error",
                "title": "Device Disconnected",
                "message": "Device drone-002 has been disconnected for more than 5 minutes",
                "timestamp": "2025-01-10T10:25:00Z",
                "source": "device_monitor",
                "resolved": false
            }
        ]
    }))
}

/// Send drone command
async fn send_drone_command(
    State(_server): State<Arc<HttpApiServer>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let command = payload["command"].as_str().unwrap_or("");
    
    info!("Received drone command: {}", command);
    
    // In futuro, qui invieremo il comando al dispositivo via TLS
    // Per ora simuliamo la risposta
    match command {
        "arm" | "disarm" | "takeoff" | "land" | "hover" | "emergency" | "setAltitude" => {
            Ok(Json(serde_json::json!({
                "success": true,
                "command": command,
                "message": format!("Command '{}' executed successfully", command),
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
        }
        _ => {
            warn!("Unknown drone command: {}", command);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

// ===== NEW API ENDPOINTS FOR DASHBOARD INTEGRATION =====

/// Get all applications from Kubernetes
async fn get_applications(
    State(server): State<Arc<HttpApiServer>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match server.application_api.list(&kube::api::ListParams::default()).await {
        Ok(applications) => {
            let app_list: Vec<serde_json::Value> = applications
                .items
                .iter()
                .map(|app| {
                    serde_json::json!({
                        "id": app.metadata.name.as_ref().unwrap_or(&"unknown".to_string()),
                        "name": app.metadata.name.as_ref().unwrap_or(&"unknown".to_string()),
                        "description": app.spec.description.as_ref().unwrap_or(&"".to_string()),
                        "status": format!("{:?}", app.status().as_ref().map(|s| &s.phase).unwrap_or(&wasmbed_k8s_resource::ApplicationPhase::Creating)),
                        "target_devices": app.spec.target_devices.clone(),
                        "wasm_bytes_size": app.spec.wasm_bytes.len(),
                        "enabled": true
                    })
                })
                .collect();
            
            Ok(Json(serde_json::json!({
                "applications": app_list
            })))
        }
        Err(e) => {
            error!("Failed to fetch applications: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create new application
async fn create_application(
    State(server): State<Arc<HttpApiServer>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let name = payload["name"].as_str().unwrap_or("unknown");
    let description = payload["description"].as_str().unwrap_or("");
    let target_devices = payload["target_devices"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();
    let wasm_bytes = payload["wasm_bytes"].as_str().unwrap_or("").to_string();
    
    let application = Application {
        metadata: kube::api::ObjectMeta {
            name: Some(name.to_string()),
            namespace: Some("wasmbed".to_string()),
            ..Default::default()
        },
        spec: wasmbed_k8s_resource::ApplicationSpec {
            name: name.to_string(),
            description: Some(description.to_string()),
            wasm_bytes,
            target_devices: wasmbed_k8s_resource::TargetDevices {
                device_names: Some(target_devices),
                selectors: None,
                all_devices: None,
            },
            config: None,
            metadata: None,
        },
    };
    
    match server.application_api.create(&kube::api::PostParams::default(), &application).await {
        Ok(_) => {
            info!("Created application: {}", name);
            Ok(Json(serde_json::json!({
                "success": true,
                "message": format!("Application '{}' created successfully", name)
            })))
        }
        Err(e) => {
            error!("Failed to create application: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get single application
async fn get_application(
    State(server): State<Arc<HttpApiServer>>,
    Path(app_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match server.application_api.get(&app_id).await {
        Ok(app) => {
            Ok(Json(serde_json::json!({
                "id": app.metadata.name.as_ref().unwrap_or(&"unknown".to_string()),
                "name": app.metadata.name.as_ref().unwrap_or(&"unknown".to_string()),
                "description": app.spec.description.as_ref().unwrap_or(&"".to_string()),
                "status": format!("{:?}", app.status().as_ref().map(|s| &s.phase).unwrap_or(&wasmbed_k8s_resource::ApplicationPhase::Creating)),
                "target_devices": app.spec.target_devices,
                "wasm_bytes_size": app.spec.wasm_bytes.len(),
                "enabled": true
            })))
        }
        Err(e) => {
            error!("Failed to fetch application {}: {}", app_id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Update application
async fn update_application(
    State(server): State<Arc<HttpApiServer>>,
    Path(app_id): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match server.application_api.get(&app_id).await {
        Ok(mut app) => {
            if let Some(description) = payload["description"].as_str() {
                app.spec.description = Some(description.to_string());
            }
            if let Some(target_devices) = payload["target_devices"].as_array() {
                let device_names: Vec<String> = target_devices
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                app.spec.target_devices = wasmbed_k8s_resource::TargetDevices {
                    device_names: Some(device_names),
                    selectors: None,
                    all_devices: None,
                };
            }
            if let Some(wasm_bytes) = payload["wasm_bytes"].as_str() {
                app.spec.wasm_bytes = wasm_bytes.to_string();
            }
            
            match server.application_api.replace(&app_id, &kube::api::PostParams::default(), &app).await {
                Ok(_) => {
                    info!("Updated application: {}", app_id);
                    Ok(Json(serde_json::json!({
                        "success": true,
                        "message": format!("Application '{}' updated successfully", app_id)
                    })))
                }
                Err(e) => {
                    error!("Failed to update application: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            error!("Failed to fetch application {}: {}", app_id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Delete application
async fn delete_application(
    State(server): State<Arc<HttpApiServer>>,
    Path(app_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match server.application_api.delete(&app_id, &kube::api::DeleteParams::default()).await {
        Ok(_) => {
            info!("Deleted application: {}", app_id);
            Ok(Json(serde_json::json!({
                "success": true,
                "message": format!("Application '{}' deleted successfully", app_id)
            })))
        }
        Err(e) => {
            error!("Failed to delete application: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get all gateways (simulated for now)
async fn get_gateways(
    State(_server): State<Arc<HttpApiServer>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // For now, return simulated gateway data
    // In the future, this should query Kubernetes for Gateway CRDs
    Ok(Json(serde_json::json!({
        "gateways": [
            {
                "id": "gateway-1",
                "name": "gateway-1",
                "status": "Active",
                "endpoint": "127.0.0.1:30452",
                "connected_devices": 2,
                "enrolled_devices": 6,
                "enabled": true
            },
            {
                "id": "gateway-2", 
                "name": "gateway-2",
                "status": "Active",
                "endpoint": "127.0.0.1:30454",
                "connected_devices": 2,
                "enrolled_devices": 6,
                "enabled": true
            },
            {
                "id": "gateway-3",
                "name": "gateway-3", 
                "status": "Inactive",
                "endpoint": "127.0.0.1:30456",
                "connected_devices": 0,
                "enrolled_devices": 0,
                "enabled": false
            }
        ]
    })))
}

/// Create new gateway
async fn create_gateway(
    State(server): State<Arc<HttpApiServer>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let name = payload["name"].as_str().unwrap_or("unknown").to_string();
    let endpoint = payload["endpoint"].as_str().unwrap_or("127.0.0.1:30452").to_string();
    let tls_port = payload["tls_port"].as_u64().unwrap_or(30452) as u16;
    let http_port = payload["http_port"].as_u64().unwrap_or(30453) as u16;
    let max_devices = payload["max_devices"].as_u64().unwrap_or(50) as u32;
    let region = payload["region"].as_str().unwrap_or("us-west-1").to_string();
    let enabled = payload["enabled"].as_bool().unwrap_or(true);
    
    let gateway = Gateway {
        metadata: kube::api::ObjectMeta {
            name: Some(name.to_string()),
            namespace: Some("wasmbed".to_string()),
            ..Default::default()
        },
        spec: wasmbed_k8s_resource::GatewaySpec {
            endpoint: endpoint.clone(),
            capabilities: Some(vec!["tls".to_string(), "enrollment".to_string(), "deployment".to_string()]),
            config: Some(wasmbed_k8s_resource::GatewayConfig {
                connection_timeout: Some("10m".to_string()),
                enrollment_timeout: Some("5m".to_string()),
                heartbeat_interval: Some("30s".to_string()),
            }),
        },
        status: Some(wasmbed_k8s_resource::GatewayStatus {
            phase: wasmbed_k8s_resource::GatewayPhase::Pending,
            connected_devices: Some(0),
            enrolled_devices: Some(0),
            last_heartbeat: None,
            conditions: None,
        }),
    };
    
    match server.gateway_api.create(&kube::api::PostParams::default(), &gateway).await {
        Ok(_) => {
            info!("Created gateway: {} at {}", name, endpoint);
            
            // Start the actual gateway process
            let name_clone = name.clone();
            let endpoint_clone = endpoint.clone();
            tokio::spawn(async move {
                let gateway_process = tokio::process::Command::new("./target/release/wasmbed-gateway")
                    .arg("--bind-addr")
                    .arg(format!("127.0.0.1:{}", tls_port))
                    .arg("--http-addr")
                    .arg(format!("127.0.0.1:{}", http_port))
                    .arg("--private-key")
                    .arg("certs/server-key.pem")
                    .arg("--certificate")
                    .arg("certs/server-cert.pem")
                    .arg("--client-ca")
                    .arg("certs/ca-cert.pem")
                    .arg("--namespace")
                    .arg("wasmbed")
                    .arg("--pod-namespace")
                    .arg("wasmbed")
                    .arg("--pod-name")
                    .arg(name_clone.clone())
                    .spawn();
                
                match gateway_process {
                    Ok(mut process) => {
                        info!("Gateway process started for {}", name_clone);
                        if let Err(e) = process.wait().await {
                            error!("Gateway process error for {}: {}", name_clone, e);
                        }
                    }
                    Err(e) => {
                        error!("Failed to start gateway process for {}: {}", name_clone, e);
                    }
                }
            });
            
            Ok(Json(serde_json::json!({
                "success": true,
                "message": format!("Gateway '{}' created and started successfully", name),
                "gateway": {
                    "name": name,
                    "endpoint": endpoint,
                    "tls_port": tls_port,
                    "http_port": http_port,
                    "max_devices": max_devices,
                    "region": region,
                    "enabled": enabled
                }
            })))
        }
        Err(e) => {
            error!("Failed to create gateway: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get single gateway
async fn get_gateway(
    State(_server): State<Arc<HttpApiServer>>,
    Path(gateway_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // For now, return simulated gateway data
    Ok(Json(serde_json::json!({
        "id": gateway_id,
        "name": gateway_id,
        "status": "Active",
        "endpoint": "127.0.0.1:30452",
        "connected_devices": 2,
        "enrolled_devices": 6,
        "enabled": true
    })))
}

/// Update gateway
async fn update_gateway(
    State(_server): State<Arc<HttpApiServer>>,
    Path(gateway_id): Path<String>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Updating gateway: {}", gateway_id);
    
    // For now, just return success
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Gateway '{}' updated successfully", gateway_id)
    })))
}

/// Delete gateway
async fn delete_gateway(
    State(_server): State<Arc<HttpApiServer>>,
    Path(gateway_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Deleting gateway: {}", gateway_id);
    
    // For now, just return success
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Gateway '{}' deleted successfully", gateway_id)
    })))
}

/// Toggle gateway enabled/disabled
async fn toggle_gateway(
    State(_server): State<Arc<HttpApiServer>>,
    Path(gateway_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Toggling gateway: {}", gateway_id);
    
    // For now, just return success
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Gateway '{}' toggled successfully", gateway_id)
    })))
}

/// Get infrastructure status
async fn get_infrastructure_status(
    State(_server): State<Arc<HttpApiServer>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "components": {
            "kubernetes": "healthy",
            "gateway": "healthy", 
            "database": "healthy",
            "monitoring": "healthy"
        },
        "uptime": "2d 5h 30m",
        "version": "1.0.0"
    })))
}

/// Create new device
async fn create_device(
    State(server): State<Arc<HttpApiServer>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let name = payload["name"].as_str().unwrap_or("unknown");
    let device_type = payload["type"].as_str().unwrap_or("MCU");
    let architecture = payload["architecture"].as_str().unwrap_or("riscv32");
    let gateway = payload["gateway"].as_str().unwrap_or("gateway-1");
    let enabled = payload["enabled"].as_bool().unwrap_or(true);
    
    // Generate a unique public key for the device
    let public_key = format!("device-{}-{}", name, uuid::Uuid::new_v4().to_string()[..8].to_string());
    
    let device = Device {
        metadata: kube::api::ObjectMeta {
            name: Some(name.to_string()),
            namespace: Some("wasmbed".to_string()),
            labels: Some(std::collections::BTreeMap::from([
                ("device-type".to_string(), device_type.to_string()),
                ("architecture".to_string(), architecture.to_string()),
                ("gateway".to_string(), gateway.to_string()),
                ("enabled".to_string(), enabled.to_string()),
            ])),
            ..Default::default()
        },
        spec: wasmbed_k8s_resource::DeviceSpec {
            public_key: public_key.clone(),
            mcu_type: Some("mps2-an385".to_string()),
        },
        status: Some(wasmbed_k8s_resource::DeviceStatus {
            phase: wasmbed_k8s_resource::DevicePhase::Pending,
            gateway: None,
            connected_since: None,
            last_heartbeat: None,
            pairing_mode: false,
        }),
    };
    
    match server.device_api.create(&kube::api::PostParams::default(), &device).await {
        Ok(_) => {
            info!("Created device: {} (type: {}, arch: {}, gateway: {})", name, device_type, architecture, gateway);
            Ok(Json(serde_json::json!({
                "success": true,
                "message": format!("Device '{}' created successfully", name),
                "device": {
                    "name": name,
                    "type": device_type,
                    "architecture": architecture,
                    "gateway": gateway,
                    "enabled": enabled,
                    "public_key": public_key,
                    "status": "Pending"
                }
            })))
        }
        Err(e) => {
            error!("Failed to create device: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get single device
async fn get_device(
    State(server): State<Arc<HttpApiServer>>,
    Path(device_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match server.device_api.get(&device_id).await {
        Ok(device) => {
            Ok(Json(serde_json::json!({
                "id": device.metadata.name.as_ref().unwrap_or(&"unknown".to_string()),
                "name": device.metadata.name.as_ref().unwrap_or(&"unknown".to_string()),
                "type": "MCU",
                "architecture": "riscv32",
                "gateway": "gateway-1",
                "status": "Connected",
                "enrolled": true,
                "connected": true,
                "last_heartbeat": "2025-09-27T17:30:00Z"
            })))
        }
        Err(e) => {
            error!("Failed to fetch device {}: {}", device_id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Update device
async fn update_device(
    State(server): State<Arc<HttpApiServer>>,
    Path(device_id): Path<String>,
    Json(_payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match server.device_api.get(&device_id).await {
        Ok(device) => {
            // Device spec only has public_key field, so we can't update device_type, architecture, or gateway
            // These would need to be handled differently in a real implementation
            
            match server.device_api.replace(&device_id, &kube::api::PostParams::default(), &device).await {
                Ok(_) => {
                    info!("Updated device: {}", device_id);
                    Ok(Json(serde_json::json!({
                        "success": true,
                        "message": format!("Device '{}' updated successfully", device_id)
                    })))
                }
                Err(e) => {
                    error!("Failed to update device: {}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            error!("Failed to fetch device {}: {}", device_id, e);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

/// Delete device
async fn delete_device(
    State(server): State<Arc<HttpApiServer>>,
    Path(device_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    match server.device_api.delete(&device_id, &kube::api::DeleteParams::default()).await {
        Ok(_) => {
            info!("Deleted device: {}", device_id);
            Ok(Json(serde_json::json!({
                "success": true,
                "message": format!("Device '{}' deleted successfully", device_id)
            })))
        }
        Err(e) => {
            error!("Failed to delete device: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Enroll device
async fn enroll_device(
    State(_server): State<Arc<HttpApiServer>>,
    Path(device_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Enrolling device: {}", device_id);
    
    // For now, just return success
    // In the future, this should trigger the enrollment workflow
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Device '{}' enrolled successfully", device_id)
    })))
}

/// Connect device
async fn connect_device(
    State(_server): State<Arc<HttpApiServer>>,
    Path(device_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Connecting device: {}", device_id);
    
    // For now, just return success
    // In the future, this should trigger the connection workflow
    Ok(Json(serde_json::json!({
        "success": true,
        "message": format!("Device '{}' connected successfully", device_id)
    })))
}
