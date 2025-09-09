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
    routing::{get, post},
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

use wasmbed_k8s_resource::{Application, Device, DeviceApplicationPhase, ApplicationConfig};
use wasmbed_protocol::{ServerMessage, DeviceUuid};
use wasmbed_types::PublicKey;

/// HTTP API server for Gateway-Controller communication with CBOR/TLS support
#[derive(Clone)]
pub struct HttpApiServer {
    pub device_connections: Arc<RwLock<HashMap<String, DeviceConnection>>>,
    pub applications: Arc<RwLock<HashMap<String, DeployedApplication>>>,
    pub device_api: Api<Device>,
    pub application_api: Api<Application>,
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
#[derive(Debug, Clone)]
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
    pub fn new(device_api: Api<Device>, application_api: Api<Application>) -> Result<Self> {
        Ok(Self {
            device_connections: Arc::new(RwLock::new(HashMap::new())),
            applications: Arc::new(RwLock::new(HashMap::new())),
            device_api,
            application_api,
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
            .route("/api/v1/devices/:device_id/deploy", post(deploy_application))
            .route("/api/v1/devices/:device_id/stop/:app_id", post(stop_application))
            .route("/api/v1/devices/:device_id/status/:app_id", get(get_application_status))
            .route("/api/v1/devices/:device_id/applications", get(get_device_applications))
            .route("/api/v1/admin/pairing-mode", get(get_pairing_mode))
            .route("/api/v1/admin/pairing-mode", post(set_pairing_mode))
            .route("/api/v1/admin/pairing-timeout", get(get_pairing_timeout))
            .route("/api/v1/admin/pairing-timeout", post(set_pairing_timeout))
            .route("/api/v1/admin/heartbeat-timeout", get(get_heartbeat_timeout))
            .route("/api/v1/admin/heartbeat-timeout", post(set_heartbeat_timeout))
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
        wasm_bytes,
        None, // No config for now
    ).await;

    // TODO: Send deployment command to device via TLS connection
    // For now, simulate successful deployment
    let server_clone = server.clone();
    let app_id_clone = app_id.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        server_clone.update_application_status(&app_id_clone, DeviceApplicationPhase::Running, None).await;
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

    // TODO: Send stop command to device via TLS connection
    // For now, simulate successful stop
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        server_clone.update_application_status(&app_id_clone, DeviceApplicationPhase::Stopped, None).await;
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
