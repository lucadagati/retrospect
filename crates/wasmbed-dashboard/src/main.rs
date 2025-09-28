// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use clap::Parser;
use axum::{
    extract::{State, WebSocketUpgrade, Path},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post, delete, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
    process::Stdio,
};
use tokio::sync::RwLock;
use tokio::net::TcpListener;
use tracing::{error, info, warn};

mod device_management;
mod application_management;
mod gateway_management;
mod monitoring;
mod templates;

use device_management::DeviceManager;
use application_management::ApplicationManager;
use gateway_management::GatewayManager;
use monitoring::MonitoringDashboard;
use templates::DashboardTemplates;

#[derive(Parser)]
#[command(name = "wasmbed-dashboard")]
#[command(about = "Wasmbed Dashboard - Web UI for managing edge devices")]
struct Args {
    #[arg(long, env = "WASMBED_DASHBOARD_PORT", default_value = "3000")]
    port: u16,
    #[arg(long, env = "WASMBED_DASHBOARD_GATEWAY_ENDPOINT", default_value = "http://localhost:30431")]
    gateway_endpoint: String,
    #[arg(long, env = "WASMBED_DASHBOARD_INFRASTRUCTURE_ENDPOINT", default_value = "http://localhost:30432")]
    infrastructure_endpoint: String,
}

/// Dashboard configuration
#[derive(Debug, Clone)]
pub struct DashboardConfig {
    pub port: u16,
    pub gateway_endpoint: String,
    pub infrastructure_endpoint: String,
    pub refresh_interval: Duration,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            gateway_endpoint: "http://localhost:8080".to_string(),
            infrastructure_endpoint: "http://localhost:8080".to_string(),
            refresh_interval: Duration::from_secs(5),
        }
    }
}

/// Dashboard state
#[derive(Debug, Clone)]
pub struct DashboardState {
    pub config: DashboardConfig,
    pub device_manager: Arc<DeviceManager>,
    pub application_manager: Arc<ApplicationManager>,
    pub gateway_manager: Arc<GatewayManager>,
    pub monitoring: Arc<MonitoringDashboard>,
    pub templates: Arc<DashboardTemplates>,
    pub system_status: Arc<RwLock<SystemStatus>>,
}

/// System status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub devices: DeviceStats,
    pub applications: ApplicationStats,
    pub gateways: GatewayStats,
    pub infrastructure: InfrastructureStats,
    pub uptime: u64,
    pub last_update: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStats {
    pub total: u32,
    pub connected: u32,
    pub enrolled: u32,
    pub unreachable: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationStats {
    pub total: u32,
    pub running: u32,
    pub pending: u32,
    pub failed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayStats {
    pub total: u32,
    pub active: u32,
    pub inactive: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureStats {
    pub ca_status: String,
    pub secret_store_status: String,
    pub monitoring_status: String,
    pub logging_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TerminalExecuteRequest {
    pub command: String,
    pub command_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TerminalExecuteResponse {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PodInfo {
    pub name: String,
    pub ready: String,
    pub status: String,
    pub restarts: u32,
    pub age: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceInfo {
    pub name: String,
    pub r#type: String,
    pub cluster_ip: String,
    pub external_ip: String,
    pub ports: String,
    pub age: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PodMetric {
    pub name: String,
    pub cpu: String,
    pub memory: String,
}

/// Dashboard API handlers
pub struct DashboardApi;

impl DashboardApi {
    /// Get dashboard home page
    pub async fn home(State(state): State<Arc<DashboardState>>) -> Result<Html<String>, StatusCode> {
        let system_status = state.system_status.read().await;
        let html = state.templates.render_dashboard(&system_status).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Html(html))
    }

    /// Get devices page
    pub async fn devices(State(state): State<Arc<DashboardState>>) -> Result<Html<String>, StatusCode> {
        let devices = state.device_manager.get_all_devices().await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let html = state.templates.render_devices(&devices).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Html(html))
    }

    /// Get applications page
    pub async fn applications(State(state): State<Arc<DashboardState>>) -> Result<Html<String>, StatusCode> {
        let applications = state.application_manager.get_all_applications().await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let html = state.templates.render_applications(&applications).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Html(html))
    }

    /// Get gateways page
    pub async fn gateways(State(state): State<Arc<DashboardState>>) -> Result<Html<String>, StatusCode> {
        let gateways = state.gateway_manager.get_all_gateways().await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let html = state.templates.render_gateways(&gateways).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Html(html))
    }

    /// Get monitoring page
    pub async fn monitoring(State(state): State<Arc<DashboardState>>) -> Result<Html<String>, StatusCode> {
        let metrics = state.monitoring.get_metrics().await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let html = state.templates.render_monitoring(&metrics).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Html(html))
    }

    /// Health check endpoint
    pub async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
        Ok(Json(serde_json::json!({
            "status": "healthy",
            "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        })))
    }

    /// Get system status API
    pub async fn api_status(State(state): State<Arc<DashboardState>>) -> Result<Json<SystemStatus>, StatusCode> {
        let system_status = state.system_status.read().await;
        Ok(Json(system_status.clone()))
    }

    /// Get devices API
    pub async fn api_devices(State(state): State<Arc<DashboardState>>) -> Result<Json<serde_json::Value>, StatusCode> {
        match tokio::time::timeout(Duration::from_secs(5), state.device_manager.get_all_devices()).await {
            Ok(Ok(devices)) => Ok(Json(serde_json::json!({
                "devices": devices
            }))),
            Ok(Err(_)) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            Err(_) => Err(StatusCode::REQUEST_TIMEOUT),
        }
    }

    /// Get applications API
    pub async fn api_applications(State(state): State<Arc<DashboardState>>) -> Result<Json<serde_json::Value>, StatusCode> {
        match tokio::time::timeout(Duration::from_secs(5), state.application_manager.get_all_applications()).await {
            Ok(Ok(applications)) => Ok(Json(serde_json::json!({
                "applications": applications
            }))),
            Ok(Err(_)) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            Err(_) => Err(StatusCode::REQUEST_TIMEOUT),
        }
    }

    /// Get gateways API
    pub async fn api_gateways(State(state): State<Arc<DashboardState>>) -> Result<Json<serde_json::Value>, StatusCode> {
        match tokio::time::timeout(Duration::from_secs(5), state.gateway_manager.get_all_gateways()).await {
            Ok(Ok(gateways)) => Ok(Json(serde_json::json!({
                "gateways": gateways
            }))),
            Ok(Err(_)) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            Err(_) => Err(StatusCode::REQUEST_TIMEOUT),
        }
    }

    /// Delete gateway
    pub async fn delete_gateway(
        State(_state): State<Arc<DashboardState>>,
        Path(id): Path<String>,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Deleting gateway: {}", id);

        let output = tokio::process::Command::new("kubectl")
            .args(&["delete", "gateway", &id, "-n", "wasmbed"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await;

        match output {
            Ok(output) => {
                if output.status.success() {
                    Ok(Json(serde_json::json!({
                        "success": true,
                        "message": format!("Gateway {} deleted successfully", id)
                    })))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("Failed to delete gateway {}: {}", id, stderr);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            Err(e) => {
                error!("Failed to execute kubectl delete for gateway {}: {}", id, e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Delete device
    pub async fn delete_device(
        State(_state): State<Arc<DashboardState>>,
        Path(id): Path<String>,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Deleting device: {}", id);

        let output = tokio::process::Command::new("kubectl")
            .args(&["delete", "device", &id, "-n", "wasmbed"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await;

        match output {
            Ok(output) => {
                if output.status.success() {
                    Ok(Json(serde_json::json!({
                        "success": true,
                        "message": format!("Device {} deleted successfully", id)
                    })))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("Failed to delete device {}: {}", id, stderr);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            Err(e) => {
                error!("Failed to execute kubectl delete for device {}: {}", id, e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Delete application
    pub async fn delete_application(
        State(_state): State<Arc<DashboardState>>,
        Path(id): Path<String>,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Deleting application: {}", id);

        let output = tokio::process::Command::new("kubectl")
            .args(&["delete", "application", &id, "-n", "wasmbed"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await;

        match output {
            Ok(output) => {
                if output.status.success() {
                    Ok(Json(serde_json::json!({
                        "success": true,
                        "message": format!("Application {} deleted successfully", id)
                    })))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("Failed to delete application {}: {}", id, stderr);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            Err(e) => {
                error!("Failed to execute kubectl delete for application {}: {}", id, e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Toggle gateway status
    pub async fn toggle_gateway(
        State(_state): State<Arc<DashboardState>>,
        Path(id): Path<String>,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Toggling gateway: {}", id);

        // Get current gateway status
        let output = tokio::process::Command::new("kubectl")
            .args(&["get", "gateway", &id, "-n", "wasmbed", "-o", "json"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await;

        match output {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if let Ok(gateway_json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                        let current_status = gateway_json["status"]["phase"]
                            .as_str()
                            .unwrap_or("Pending");

                        let new_status = if current_status == "Running" || current_status == "Active" {
                            "Stopped"
                        } else {
                            "Running"
                        };

                        // Update gateway status using kubectl patch
                        let patch_output = tokio::process::Command::new("kubectl")
                            .args(&[
                                "patch", "gateway", &id, "-n", "wasmbed",
                                "--type", "merge",
                                "--patch", &format!("{{\"status\":{{\"phase\":\"{}\"}}}}", new_status)
                            ])
                            .stdout(std::process::Stdio::piped())
                            .stderr(std::process::Stdio::piped())
                            .output()
                            .await;

                        match patch_output {
                            Ok(patch_output) => {
                                if patch_output.status.success() {
                                    Ok(Json(serde_json::json!({
                                        "success": true,
                                        "message": format!("Gateway {} status changed to {}", id, new_status),
                                        "status": new_status
                                    })))
                                } else {
                                    let stderr = String::from_utf8_lossy(&patch_output.stderr);
                                    error!("Failed to patch gateway {}: {}", id, stderr);
                                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                                }
                            }
                            Err(e) => {
                                error!("Failed to execute kubectl patch for gateway {}: {}", id, e);
                                Err(StatusCode::INTERNAL_SERVER_ERROR)
                            }
                        }
                    } else {
                        error!("Failed to parse gateway JSON for {}", id);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("Failed to get gateway {}: {}", id, stderr);
                    Err(StatusCode::NOT_FOUND)
                }
            }
            Err(e) => {
                error!("Failed to execute kubectl get for gateway {}: {}", id, e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Update gateway configuration
    pub async fn update_gateway(
        State(_state): State<Arc<DashboardState>>,
        Path(id): Path<String>,
        Json(request): Json<serde_json::Value>,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Updating gateway {}: {:?}", id, request);
        
        let endpoint = request.get("endpoint")
            .and_then(|v| v.as_str())
            .unwrap_or("127.0.0.1:30452");
        
        let capabilities = request.get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_else(|| vec!["tls", "enrollment", "deployment"]);
        
        // Update gateway configuration using kubectl patch
        let patch = serde_json::json!({
            "spec": {
                "endpoint": endpoint,
                "capabilities": capabilities
            }
        });
        
        let patch_str = serde_json::to_string(&patch).unwrap_or_else(|_| "{}".to_string());
        
        let output = tokio::process::Command::new("kubectl")
            .args(&["patch", "gateway", &id, "-n", "wasmbed", "--type", "merge", "--patch", &patch_str])
            .output()
            .await;
            
        match output {
            Ok(output) => {
                if output.status.success() {
                    info!("Gateway {} configuration updated successfully", id);
                    Ok(Json(serde_json::json!({
                        "success": true,
                        "message": format!("Gateway {} configuration updated successfully", id)
                    })))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("Failed to update gateway {}: {}", id, stderr);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            Err(e) => {
                error!("Failed to execute kubectl for gateway update {}: {}", id, e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Create gateways
    pub async fn create_gateway(
        State(state): State<Arc<DashboardState>>,
        Json(request): Json<serde_json::Value>,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Creating gateways: {:?}", request);
        
        let count = request.get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;
        
        let base_endpoint = request.get("endpoint")
            .and_then(|v| v.as_str())
            .unwrap_or("127.0.0.1");
        
        let base_port = request.get("basePort")
            .and_then(|v| v.as_u64())
            .unwrap_or(30452) as u16;
        
        let mut created_gateways = Vec::new();
        let mut errors = Vec::new();
        
        for i in 1..=count {
            let name = format!("gateway-{}", i);
            let port = base_port + (i as u16 - 1) * 2; // Increment by 2 for each gateway
            let endpoint = format!("{}:{}", base_endpoint, port);
            
            // Create Gateway CRD in Kubernetes
            let gateway_yaml = format!(
                r#"apiVersion: wasmbed.io/v1
kind: Gateway
metadata:
  name: {}
  namespace: wasmbed
spec:
  endpoint: {}
  capabilities:
    - tls
    - enrollment
    - deployment
  config:
    connectionTimeout: "10m"
    enrollmentTimeout: "5m"
    heartbeatInterval: "30s"
status:
  phase: Pending
  connectedDevices: 0
  enrolledDevices: 0"#,
                name, endpoint
            );
            
            // Apply the Gateway CRD using kubectl
            let output = tokio::process::Command::new("kubectl")
                .args(&["apply", "-f", "-"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();
                
            match output {
                Ok(mut child) => {
                    if let Some(mut stdin) = child.stdin.take() {
                        use tokio::io::AsyncWriteExt;
                        if let Err(e) = stdin.write_all(gateway_yaml.as_bytes()).await {
                            error!("Failed to write to kubectl stdin for gateway {}: {}", name, e);
                            errors.push(format!("Failed to create gateway {}: {}", name, e));
                            continue;
                        }
                    }
                    
                    match child.wait_with_output().await {
                        Ok(output) => {
                            if output.status.success() {
                                info!("Gateway {} created successfully", name);
                                created_gateways.push(serde_json::json!({
                                    "id": name,
                                    "name": name,
                                    "status": "Pending",
                                    "endpoint": endpoint
                                }));
                            } else {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                error!("Failed to create gateway {}: {}", name, stderr);
                                errors.push(format!("Failed to create gateway {}: {}", name, stderr));
                            }
                        }
                        Err(e) => {
                            error!("Failed to execute kubectl for gateway {}: {}", name, e);
                            errors.push(format!("Failed to create gateway {}: {}", name, e));
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to spawn kubectl for gateway {}: {}", name, e);
                    errors.push(format!("Failed to create gateway {}: {}", name, e));
                }
            }
        }
        
        if created_gateways.is_empty() {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        
        let message = if errors.is_empty() {
            format!("Successfully created {} gateways", created_gateways.len())
        } else {
            format!("Created {} gateways, {} errors: {}", 
                   created_gateways.len(), 
                   errors.len(), 
                   errors.join("; "))
        };
        
        Ok(Json(serde_json::json!({
            "success": true,
            "message": message,
            "gateways": created_gateways,
            "errors": errors
        })))
    }

    /// Create devices
    pub async fn create_device(
        State(state): State<Arc<DashboardState>>,
        Json(request): Json<serde_json::Value>,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Creating devices: {:?}", request);
        
        let count = request.get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;
        
        let device_type = request.get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("RISC-V MCU");
        
        let gateway_id = request.get("gatewayId")
            .and_then(|v| v.as_str())
            .unwrap_or("gateway-1");
        
        let mut created_devices = Vec::new();
        let mut errors = Vec::new();
        
        for i in 1..=count {
            let name = format!("device-{}-{}", gateway_id, i);
            
            // Create Device CRD in Kubernetes
            let device_yaml = format!(
                r#"apiVersion: wasmbed.github.io/v0
kind: Device
metadata:
  name: {}
  namespace: wasmbed
spec:
  publicKey: "-----BEGIN PUBLIC KEY-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA...\n-----END PUBLIC KEY-----"
status:
  phase: Pending
  pairingMode: false"#,
                name
            );
            
            // Apply the Device CRD using kubectl
            let output = tokio::process::Command::new("kubectl")
                .args(&["apply", "-f", "-"])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();
                
            match output {
                Ok(mut child) => {
                    if let Some(mut stdin) = child.stdin.take() {
                        use tokio::io::AsyncWriteExt;
                        if let Err(e) = stdin.write_all(device_yaml.as_bytes()).await {
                            error!("Failed to write to kubectl stdin for device {}: {}", name, e);
                            errors.push(format!("Failed to create device {}: {}", name, e));
                            continue;
                        }
                    }
                    
                    match child.wait_with_output().await {
                        Ok(output) => {
                            if output.status.success() {
                                info!("Device {} created successfully", name);
                                created_devices.push(serde_json::json!({
                                    "id": name,
                                    "name": name,
                                    "type": device_type,
                                    "status": "Pending"
                                }));
                            } else {
                                let stderr = String::from_utf8_lossy(&output.stderr);
                                error!("Failed to create device {}: {}", name, stderr);
                                errors.push(format!("Failed to create device {}: {}", name, stderr));
                            }
                        }
                        Err(e) => {
                            error!("Failed to execute kubectl for device {}: {}", name, e);
                            errors.push(format!("Failed to create device {}: {}", name, e));
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to spawn kubectl for device {}: {}", name, e);
                    errors.push(format!("Failed to create device {}: {}", name, e));
                }
            }
        }
        
        if created_devices.is_empty() {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        
        let message = if errors.is_empty() {
            format!("Successfully created {} devices", created_devices.len())
        } else {
            format!("Created {} devices, {} errors: {}", 
                   created_devices.len(), 
                   errors.len(), 
                   errors.join("; "))
        };
        
        Ok(Json(serde_json::json!({
            "success": true,
            "message": message,
            "devices": created_devices,
            "errors": errors
        })))
    }

    /// Enroll device to gateway
    pub async fn enroll_device(
        State(state): State<Arc<DashboardState>>,
        Path(device_id): Path<String>,
        Json(request): Json<serde_json::Value>,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Enrolling device {}: {:?}", device_id, request);
        
        let gateway_id = request.get("gatewayId")
            .and_then(|v| v.as_str())
            .unwrap_or("gateway-1");
        
        // Update device status to "enrolled" using kubectl patch
        let patch = serde_json::json!({
            "status": {
                "phase": "Enrolled",
                "gatewayId": gateway_id
            }
        });
        
        let patch_str = serde_json::to_string(&patch).unwrap_or_else(|_| "{}".to_string());
        
        let output = tokio::process::Command::new("kubectl")
            .args(&["patch", "device", &device_id, "-n", "wasmbed", "--type", "merge", "--patch", &patch_str])
            .output()
            .await;
            
        match output {
            Ok(output) => {
                if output.status.success() {
                    info!("Device {} enrolled to gateway {} successfully", device_id, gateway_id);
                    Ok(Json(serde_json::json!({
                        "success": true,
                        "message": format!("Device {} enrolled to gateway {} successfully", device_id, gateway_id)
                    })))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("Failed to enroll device {}: {}", device_id, stderr);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            Err(e) => {
                error!("Failed to execute kubectl for device enrollment {}: {}", device_id, e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Connect device
    pub async fn connect_device(
        State(state): State<Arc<DashboardState>>,
        Path(device_id): Path<String>,
        Json(request): Json<serde_json::Value>,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Connecting device {}: {:?}", device_id, request);
        
        // Update device status to "connected" using kubectl patch
        let patch = serde_json::json!({
            "status": {
                "phase": "Connected"
            }
        });
        
        let patch_str = serde_json::to_string(&patch).unwrap_or_else(|_| "{}".to_string());
        
        let output = tokio::process::Command::new("kubectl")
            .args(&["patch", "device", &device_id, "-n", "wasmbed", "--type", "merge", "--patch", &patch_str])
            .output()
            .await;
            
        match output {
            Ok(output) => {
                if output.status.success() {
                    info!("Device {} connected successfully", device_id);
                    Ok(Json(serde_json::json!({
                        "success": true,
                        "message": format!("Device {} connected successfully", device_id)
                    })))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("Failed to connect device {}: {}", device_id, stderr);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            Err(e) => {
                error!("Failed to execute kubectl for device connection {}: {}", device_id, e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Create applications
    pub async fn create_application(
        State(state): State<Arc<DashboardState>>,
        Json(request): Json<serde_json::Value>,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Creating application: {:?}", request);
        
        let name = request.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("application-1");
        
        let wasm_bytes = request.get("wasmBytes")
            .and_then(|v| v.as_str())
            .unwrap_or("dGVzdA==");
        
        let target_devices = request.get("targetDevices")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_else(|| vec!["device-gateway-1-1"]);
        
        // Create Application CRD in Kubernetes
        let application_yaml = format!(
            r#"apiVersion: wasmbed.github.io/v1alpha1
kind: Application
metadata:
  name: {}
  namespace: wasmbed
spec:
  name: {}
  wasmBytes: {}
  targetDevices:
    deviceNames: [{}]
status:
  phase: Pending"#,
            name,
            name,
            wasm_bytes,
            target_devices.iter().map(|d| format!("\"{}\"", d)).collect::<Vec<_>>().join(", ")
        );
        
        // Apply the Application CRD using kubectl
        let output = tokio::process::Command::new("kubectl")
            .args(&["apply", "-f", "-"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();
            
        match output {
            Ok(mut child) => {
                if let Some(mut stdin) = child.stdin.take() {
                    use tokio::io::AsyncWriteExt;
                    if let Err(e) = stdin.write_all(application_yaml.as_bytes()).await {
                        error!("Failed to write to kubectl stdin for application {}: {}", name, e);
                        return Err(StatusCode::INTERNAL_SERVER_ERROR);
                    }
                }
                
                match child.wait_with_output().await {
                    Ok(output) => {
                        if output.status.success() {
                            info!("Application {} created successfully", name);
                            Ok(Json(serde_json::json!({
                                "success": true,
                                "message": format!("Successfully created application {}", name),
                                "application": {
                                    "id": name,
                                    "name": name,
                                    "status": "Pending",
                                    "targetDevices": target_devices
                                }
                            })))
                        } else {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            error!("Failed to create application {}: {}", name, stderr);
                            Err(StatusCode::INTERNAL_SERVER_ERROR)
                        }
                    }
                    Err(e) => {
                        error!("Failed to execute kubectl for application {}: {}", name, e);
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }
            }
            Err(e) => {
                error!("Failed to spawn kubectl for application {}: {}", name, e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Deploy application by ID
    pub async fn deploy_application_by_id(
        State(state): State<Arc<DashboardState>>,
        Path(app_id): Path<String>,
        Json(request): Json<serde_json::Value>,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Deploying application {}: {:?}", app_id, request);
        
        // Update application status to "Running" using kubectl patch
        let patch = serde_json::json!({
            "status": {
                "phase": "Running"
            }
        });
        
        let patch_str = serde_json::to_string(&patch).unwrap_or_else(|_| "{}".to_string());
        
        let output = tokio::process::Command::new("kubectl")
            .args(&["patch", "application", &app_id, "-n", "wasmbed", "--type", "merge", "--patch", &patch_str])
            .output()
            .await;
            
        match output {
            Ok(output) => {
                if output.status.success() {
                    info!("Application {} deployed successfully", app_id);
                    Ok(Json(serde_json::json!({
                        "success": true,
                        "message": format!("Application {} deployed successfully", app_id)
                    })))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("Failed to deploy application {}: {}", app_id, stderr);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            Err(e) => {
                error!("Failed to execute kubectl for application deployment {}: {}", app_id, e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Stop application by ID
    pub async fn stop_application_by_id(
        State(state): State<Arc<DashboardState>>,
        Path(app_id): Path<String>,
        Json(request): Json<serde_json::Value>,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Stopping application {}: {:?}", app_id, request);
        
        // Update application status to "Stopped" using kubectl patch
        let patch = serde_json::json!({
            "status": {
                "phase": "Stopped"
            }
        });
        
        let patch_str = serde_json::to_string(&patch).unwrap_or_else(|_| "{}".to_string());
        
        let output = tokio::process::Command::new("kubectl")
            .args(&["patch", "application", &app_id, "-n", "wasmbed", "--type", "merge", "--patch", &patch_str])
            .output()
            .await;
            
        match output {
            Ok(output) => {
                if output.status.success() {
                    info!("Application {} stopped successfully", app_id);
                    Ok(Json(serde_json::json!({
                        "success": true,
                        "message": format!("Application {} stopped successfully", app_id)
                    })))
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    error!("Failed to stop application {}: {}", app_id, stderr);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            Err(e) => {
                error!("Failed to execute kubectl for application stop {}: {}", app_id, e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Deploy application
    pub async fn deploy_application(
        State(state): State<Arc<DashboardState>>,
        Json(request): Json<DeployApplicationRequest>,
    ) -> Result<Json<DeployApplicationResponse>, StatusCode> {
        info!("Deploying application: {}", request.name);

        let result = state.application_manager.deploy_application(&request).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(DeployApplicationResponse {
            success: result.success,
            message: result.message,
            application_id: result.application_id,
        }))
    }

    /// Enable pairing mode
    pub async fn enable_pairing(State(state): State<Arc<DashboardState>>) -> Result<Json<PairingResponse>, StatusCode> {
        info!("Enabling pairing mode");

        let result = state.device_manager.enable_pairing().await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(PairingResponse {
            success: result,
            message: "Pairing mode enabled".to_string(),
        }))
    }

    /// Disable pairing mode
    pub async fn disable_pairing(State(state): State<Arc<DashboardState>>) -> Result<Json<PairingResponse>, StatusCode> {
        info!("Disabling pairing mode");

        let result = state.device_manager.disable_pairing().await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(Json(PairingResponse {
            success: result,
            message: "Pairing mode disabled".to_string(),
        }))
    }

    /// WebSocket handler for real-time updates
    pub async fn websocket_handler(
        ws: WebSocketUpgrade,
        State(_state): State<Arc<DashboardState>>,
    ) -> axum::response::Response {
        ws.on_upgrade(|_socket| async move {
            // Handle WebSocket connection for real-time updates
            info!("WebSocket connection established");
            
            // In a real implementation, this would:
            // 1. Send periodic status updates
            // 2. Handle client commands
            // 3. Manage connection lifecycle
        })
    }

    /// Execute terminal command
    pub async fn execute_terminal_command(
        State(_state): State<Arc<DashboardState>>,
        Json(request): Json<TerminalExecuteRequest>,
    ) -> Result<Json<TerminalExecuteResponse>, StatusCode> {
        info!("Executing terminal command: {}", request.command);

        // Allowed commands for security
        let allowed_commands = vec![
            "kubectl get pods -n wasmbed",
            "kubectl get devices -n wasmbed",
            "kubectl get applications -n wasmbed",
            "kubectl get gateways -n wasmbed",
            "kubectl get svc -n wasmbed",
            "kubectl top pods -n wasmbed",
            "kubectl logs -n wasmbed --tail=50",
            "kubectl get nodes",
            "kubectl get namespaces",
            "kubectl get crd",
            "kubectl get events -n wasmbed",
            "kubectl describe pods -n wasmbed",
            "kubectl get configmaps -n wasmbed",
            "kubectl get secrets -n wasmbed",
            "kubectl get pv",
            "kubectl get pvc -n wasmbed",
            "kubectl get ingress -n wasmbed",
            "kubectl get networkpolicies -n wasmbed",
            "kubectl get serviceaccounts -n wasmbed",
            "kubectl get roles -n wasmbed",
            "kubectl get devices -n wasmbed -o wide",
            "kubectl get applications -n wasmbed -o wide",
            "kubectl get gateways -n wasmbed -o wide",
            "kubectl get events -n wasmbed --sort-by=.metadata.creationTimestamp",
            "kubectl get certificates -n wasmbed",
            "kubectl get all -n wasmbed",
            "kubectl get pods -n wasmbed -l app=wasmbed-wasm-runtime",
            "kubectl logs -n wasmbed -l app=wasmbed-application-controller",
            "kubectl logs -n wasmbed -l app=wasmbed-gateway",
            "kubectl describe device -n wasmbed",
            "curl -s http://localhost:30461/health",
            "curl -s http://localhost:30461/logs",
            "curl -s http://localhost:30453/api/v1/status",
            "cd /home/lucadag/27_9_25_retrospect/retrospect && ./target/release/wasmbed-gateway-controller --kubeconfig ~/.kube/config &",
            "cd /home/lucadag/27_9_25_retrospect/retrospect && ./target/release/wasmbed-device-controller --kubeconfig ~/.kube/config &",
            "cd /home/lucadag/27_9_25_retrospect/retrospect && ./target/release/wasmbed-application-controller --kubeconfig ~/.kube/config &",
        ];

        if !allowed_commands.contains(&request.command.as_str()) {
            return Ok(Json(TerminalExecuteResponse {
                success: false,
                output: String::new(),
                error: Some(format!("Command '{}' is not allowed", request.command)),
            }));
        }

        // Execute the command
        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&request.command)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                let stderr = String::from_utf8_lossy(&result.stderr);
                
                if result.status.success() {
                    // Combine stdout and stderr for successful commands
                    // kubectl often sends informational messages to stderr even on success
                    let combined_output = if stdout.trim().is_empty() && !stderr.trim().is_empty() {
                        stderr.to_string()
                    } else if !stdout.trim().is_empty() && !stderr.trim().is_empty() {
                        format!("{}{}", stdout, stderr)
                    } else {
                        stdout.to_string()
                    };
                    
                    Ok(Json(TerminalExecuteResponse {
                        success: true,
                        output: combined_output,
                        error: None,
                    }))
                } else {
                    Ok(Json(TerminalExecuteResponse {
                        success: false,
                        output: stdout.to_string(),
                        error: Some(stderr.to_string()),
                    }))
                }
            }
            Err(e) => {
                error!("Failed to execute command: {}", e);
                Ok(Json(TerminalExecuteResponse {
                    success: false,
                    output: String::new(),
                    error: Some(format!("Failed to execute command: {}", e)),
                }))
            }
        }
    }

    /// Get pods information
    pub async fn get_pods(State(_state): State<Arc<DashboardState>>) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Getting pods information");

        let output = tokio::process::Command::new("kubectl")
            .args(&["get", "pods", "-n", "wasmbed", "-o", "json"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match output {
            Ok(result) => {
                if result.status.success() {
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    match serde_json::from_str::<serde_json::Value>(&stdout) {
                        Ok(json) => Ok(Json(json)),
                        Err(e) => {
                            error!("Failed to parse kubectl output: {}", e);
                            Err(StatusCode::INTERNAL_SERVER_ERROR)
                        }
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    error!("kubectl get pods failed: {}", stderr);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            Err(e) => {
                error!("Failed to execute kubectl: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Get services information
    pub async fn get_services(State(_state): State<Arc<DashboardState>>) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Getting services information");

        let output = tokio::process::Command::new("kubectl")
            .args(&["get", "svc", "-n", "wasmbed", "-o", "json"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match output {
            Ok(result) => {
                if result.status.success() {
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    match serde_json::from_str::<serde_json::Value>(&stdout) {
                        Ok(json) => Ok(Json(json)),
                        Err(e) => {
                            error!("Failed to parse kubectl output: {}", e);
                            Err(StatusCode::INTERNAL_SERVER_ERROR)
                        }
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    error!("kubectl get svc failed: {}", stderr);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            Err(e) => {
                error!("Failed to execute kubectl: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Get pod metrics
    pub async fn get_pod_metrics(State(_state): State<Arc<DashboardState>>) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Getting pod metrics");

        let output = tokio::process::Command::new("kubectl")
            .args(&["top", "pods", "-n", "wasmbed"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match output {
            Ok(result) => {
                if result.status.success() {
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    // Parse the output and convert to JSON
                    let lines: Vec<&str> = stdout.lines().collect();
                    let mut metrics = Vec::new();
                    
                    for line in lines.iter().skip(1) { // Skip header
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 3 {
                            metrics.push(serde_json::json!({
                                "name": parts[0],
                                "cpu": parts[1],
                                "memory": parts[2]
                            }));
                        }
                    }
                    
                    Ok(Json(serde_json::json!({
                        "metrics": metrics
                    })))
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    error!("kubectl top pods failed: {}", stderr);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            Err(e) => {
                error!("Failed to execute kubectl: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }

    /// Get logs
    pub async fn get_logs(State(_state): State<Arc<DashboardState>>) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Getting logs");

        let output = tokio::process::Command::new("kubectl")
            .args(&["logs", "-n", "wasmbed", "--tail=50", "--all-containers=true"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await;

        match output {
            Ok(result) => {
                if result.status.success() {
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    let logs: Vec<&str> = stdout.lines().collect();
                    
                    Ok(Json(serde_json::json!({
                        "logs": logs
                    })))
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    error!("kubectl logs failed: {}", stderr);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            Err(e) => {
                error!("Failed to execute kubectl: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_type: String,
    pub architecture: String,
    pub status: String,
    pub last_heartbeat: Option<SystemTime>,
    pub gateway_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationInfo {
    pub app_id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub deployed_devices: Vec<String>,
    pub created_at: SystemTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayInfo {
    pub gateway_id: String,
    pub endpoint: String,
    pub status: String,
    pub connected_devices: u32,
    pub enrolled_devices: u32,
}

#[derive(Debug, Deserialize)]
pub struct DeployApplicationRequest {
    pub name: String,
    pub image: String,
    pub device_selector: DeviceSelector,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceSelector {
    pub device_type: Option<String>,
    pub architecture: Option<String>,
    pub capabilities: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct DeployApplicationResponse {
    pub success: bool,
    pub message: String,
    pub application_id: String,
}

#[derive(Debug, Serialize)]
pub struct PairingResponse {
    pub success: bool,
    pub message: String,
}

/// Dashboard implementation
#[derive(Clone)]
pub struct Dashboard {
    config: DashboardConfig,
    state: Arc<DashboardState>,
}

impl Dashboard {
    pub fn new(config: DashboardConfig) -> anyhow::Result<Self> {
        let device_manager = Arc::new(DeviceManager::new(&config.gateway_endpoint)?);
        let application_manager = Arc::new(ApplicationManager::new(&config.gateway_endpoint)?);
        let gateway_manager = Arc::new(GatewayManager::new(&config.gateway_endpoint)?);
        let monitoring = Arc::new(MonitoringDashboard::new(&config.infrastructure_endpoint)?);
        let templates = Arc::new(DashboardTemplates::new());

        let system_status = Arc::new(RwLock::new(SystemStatus {
            devices: DeviceStats {
                total: 0,
                connected: 0,
                enrolled: 0,
                unreachable: 0,
            },
            applications: ApplicationStats {
                total: 0,
                running: 0,
                pending: 0,
                failed: 0,
            },
            gateways: GatewayStats {
                total: 0,
                active: 0,
                inactive: 0,
            },
            infrastructure: InfrastructureStats {
                ca_status: "unknown".to_string(),
                secret_store_status: "unknown".to_string(),
                monitoring_status: "unknown".to_string(),
                logging_status: "unknown".to_string(),
            },
            uptime: 0,
            last_update: SystemTime::now(),
        }));

        let state = Arc::new(DashboardState {
            config: config.clone(),
            device_manager,
            application_manager,
            gateway_manager,
            monitoring,
            templates,
            system_status,
        });

        Ok(Self { config, state })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        info!("Starting Wasmbed Dashboard...");

        let dashboard1 = self.clone();
        let dashboard2 = self.clone();
        
        // Start status update task
        let status_update_handle = tokio::spawn(async move {
            dashboard1.update_system_status().await;
        });

        // Start REST API server
        let api_handle = tokio::spawn(async move {
            if let Err(e) = dashboard2.run_web_server().await {
                error!("Web server error: {}", e);
            }
        });

        info!("Dashboard started successfully");
        info!("Web UI: http://localhost:{}", self.config.port);

        // Wait for all tasks
        tokio::try_join!(status_update_handle, api_handle)?;

        Ok(())
    }

    async fn update_system_status(&self) {
        loop {
            // Update system status with timeout protection
            let mut status = self.state.system_status.write().await;
            
            // Update device stats with timeout
            match tokio::time::timeout(Duration::from_secs(5), self.state.device_manager.get_all_devices()).await {
                Ok(Ok(devices)) => {
                    status.devices.total = devices.len() as u32;
                    status.devices.connected = devices.iter()
                        .filter(|d| d.status == "connected")
                        .count() as u32;
                    status.devices.enrolled = devices.iter()
                        .filter(|d| d.status == "enrolled")
                        .count() as u32;
                    status.devices.unreachable = devices.iter()
                        .filter(|d| d.status == "unreachable")
                        .count() as u32;
                },
                Ok(Err(e)) => {
                    warn!("Failed to fetch devices: {}", e);
                },
                Err(_) => {
                    warn!("Timeout fetching devices");
                }
            }

            // Update application stats with timeout
            match tokio::time::timeout(Duration::from_secs(5), self.state.application_manager.get_all_applications()).await {
                Ok(Ok(applications)) => {
                    status.applications.total = applications.len() as u32;
                    status.applications.running = applications.iter()
                        .filter(|a| a.status == "running")
                        .count() as u32;
                    status.applications.pending = applications.iter()
                        .filter(|a| a.status == "pending")
                        .count() as u32;
                    status.applications.failed = applications.iter()
                        .filter(|a| a.status == "failed")
                        .count() as u32;
                },
                Ok(Err(e)) => {
                    warn!("Failed to fetch applications: {}", e);
                },
                Err(_) => {
                    warn!("Timeout fetching applications");
                }
            }

            // Update gateway stats with timeout
            match tokio::time::timeout(Duration::from_secs(5), self.state.gateway_manager.get_all_gateways()).await {
                Ok(Ok(gateways)) => {
                    status.gateways.total = gateways.len() as u32;
                    status.gateways.active = gateways.iter()
                        .filter(|g| g.status == "active")
                        .count() as u32;
                    status.gateways.inactive = gateways.iter()
                        .filter(|g| g.status == "inactive")
                        .count() as u32;
                },
                Ok(Err(e)) => {
                    warn!("Failed to fetch gateways: {}", e);
                },
                Err(_) => {
                    warn!("Timeout fetching gateways");
                }
            }

            // Update infrastructure status
            self.update_infrastructure_status(&mut status).await;

            status.last_update = SystemTime::now();
            status.uptime = status.last_update
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            drop(status);

            // Wait before next update
            tokio::time::sleep(self.config.refresh_interval).await;
        }
    }

    async fn update_infrastructure_status(&self, status: &mut SystemStatus) {
        // Check Certificate Authority status
        match self.check_ca_status().await {
            Ok(ca_status) => status.infrastructure.ca_status = ca_status,
            Err(_) => status.infrastructure.ca_status = "error".to_string(),
        }

        // Check Secret Store status
        match self.check_secret_store_status().await {
            Ok(secret_status) => status.infrastructure.secret_store_status = secret_status,
            Err(_) => status.infrastructure.secret_store_status = "error".to_string(),
        }

        // Check Monitoring status
        match self.check_monitoring_status().await {
            Ok(monitoring_status) => status.infrastructure.monitoring_status = monitoring_status,
            Err(_) => status.infrastructure.monitoring_status = "error".to_string(),
        }

        // Check Logging status
        match self.check_logging_status().await {
            Ok(logging_status) => status.infrastructure.logging_status = logging_status,
            Err(_) => status.infrastructure.logging_status = "error".to_string(),
        }
    }

    async fn check_ca_status(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Check if CA certificates exist in Kubernetes
        let output = tokio::process::Command::new("kubectl")
            .args(&["get", "secrets", "-n", "wasmbed", "--field-selector", "type=kubernetes.io/tls"])
            .output()
            .await?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("ca-") || output_str.contains("tls-") {
                Ok("healthy".to_string())
            } else {
                Ok("not_configured".to_string())
            }
        } else {
            Ok("not_available".to_string())
        }
    }

    async fn check_secret_store_status(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Check if secret store is available
        let output = tokio::process::Command::new("kubectl")
            .args(&["get", "secrets", "-n", "wasmbed"])
            .output()
            .await?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.lines().count() > 1 { // More than just header
                Ok("healthy".to_string())
            } else {
                Ok("empty".to_string())
            }
        } else {
            Ok("not_available".to_string())
        }
    }

    async fn check_monitoring_status(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Check if monitoring service is running
        let output = tokio::process::Command::new("kubectl")
            .args(&["get", "pods", "-n", "wasmbed", "-l", "app=wasmbed-infrastructure"])
            .output()
            .await?;

        if output.status.success() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            if output_str.contains("Running") {
                Ok("healthy".to_string())
            } else if output_str.contains("Pending") {
                Ok("starting".to_string())
            } else {
                Ok("not_running".to_string())
            }
        } else {
            Ok("not_available".to_string())
        }
    }

    async fn check_logging_status(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Check if logging is working by testing the infrastructure service
        match reqwest::get("http://localhost:30461/logs").await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok("healthy".to_string())
                } else {
                    Ok("error".to_string())
                }
            },
            Err(_) => Ok("not_available".to_string()),
        }
    }

    async fn run_web_server(self) -> anyhow::Result<()> {
        let app = Router::new()
            .route("/", get(DashboardApi::home))
            .route("/devices", get(DashboardApi::devices))
            .route("/applications", get(DashboardApi::applications))
            .route("/gateways", get(DashboardApi::gateways))
            .route("/monitoring", get(DashboardApi::monitoring))
            .route("/health", get(DashboardApi::health_check))
            .route("/logs", get(DashboardApi::get_logs))
            .route("/api/status", get(DashboardApi::api_status))
            .route("/api/devices", get(DashboardApi::api_devices))
            .route("/api/applications", get(DashboardApi::api_applications))
            .route("/api/gateways", get(DashboardApi::api_gateways))
            .route("/api/v1/status", get(DashboardApi::api_status))
            .route("/api/v1/devices", get(DashboardApi::api_devices))
            .route("/api/v1/applications", get(DashboardApi::api_applications))
            .route("/api/v1/applications", post(DashboardApi::create_application))
            .route("/api/v1/gateways", get(DashboardApi::api_gateways))
            .route("/api/v1/gateways", post(DashboardApi::create_gateway))
            .route("/api/v1/gateways/:id/toggle", post(DashboardApi::toggle_gateway))
            .route("/api/v1/gateways/:id", delete(DashboardApi::delete_gateway))
            .route("/api/v1/gateways/:id", put(DashboardApi::update_gateway))
            .route("/api/v1/devices", post(DashboardApi::create_device))
            .route("/api/v1/devices/:id", delete(DashboardApi::delete_device))
            .route("/api/v1/devices/:id/enroll", post(DashboardApi::enroll_device))
            .route("/api/v1/devices/:id/connect", post(DashboardApi::connect_device))
            .route("/api/v1/applications/:id", delete(DashboardApi::delete_application))
            .route("/api/v1/applications/:id/deploy", post(DashboardApi::deploy_application_by_id))
            .route("/api/v1/applications/:id/stop", post(DashboardApi::stop_application_by_id))
            .route("/api/deploy", post(DashboardApi::deploy_application))
            .route("/api/pairing/enable", post(DashboardApi::enable_pairing))
            .route("/api/pairing/disable", post(DashboardApi::disable_pairing))
            .route("/api/v1/terminal/execute", post(DashboardApi::execute_terminal_command))
            .route("/api/v1/pods", get(DashboardApi::get_pods))
            .route("/api/v1/services", get(DashboardApi::get_services))
            .route("/api/v1/metrics", get(DashboardApi::get_pod_metrics))
            .route("/api/v1/logs", get(DashboardApi::get_logs))
            .route("/ws", get(DashboardApi::websocket_handler))
            .layer(
                tower_http::cors::CorsLayer::new()
                    .allow_origin(tower_http::cors::Any)
                    .allow_methods(tower_http::cors::Any)
                    .allow_headers(tower_http::cors::Any)
            )
            .with_state(self.state);

        let addr = SocketAddr::from(([0, 0, 0, 0], self.config.port));
        
        info!("Starting web server on {}", addr);
        
        let listener = TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    
    let config = DashboardConfig {
        port: args.port,
        gateway_endpoint: args.gateway_endpoint,
        infrastructure_endpoint: args.infrastructure_endpoint,
        refresh_interval: Duration::from_secs(5),
    };
    
    let dashboard = Dashboard::new(config)?;
    
    dashboard.run().await?;
    
    Ok(())
}
