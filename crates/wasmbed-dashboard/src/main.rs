// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use clap::Parser;
use axum::{
    extract::{State, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
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

    /// Get system status API
    pub async fn api_status(State(state): State<Arc<DashboardState>>) -> Result<Json<SystemStatus>, StatusCode> {
        let system_status = state.system_status.read().await;
        Ok(Json(system_status.clone()))
    }

    /// Get devices API
    pub async fn api_devices(State(state): State<Arc<DashboardState>>) -> Result<Json<Vec<DeviceInfo>>, StatusCode> {
        match tokio::time::timeout(Duration::from_secs(5), state.device_manager.get_all_devices()).await {
            Ok(Ok(devices)) => Ok(Json(devices)),
            Ok(Err(_)) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            Err(_) => Err(StatusCode::REQUEST_TIMEOUT),
        }
    }

    /// Get applications API
    pub async fn api_applications(State(state): State<Arc<DashboardState>>) -> Result<Json<Vec<ApplicationInfo>>, StatusCode> {
        match tokio::time::timeout(Duration::from_secs(5), state.application_manager.get_all_applications()).await {
            Ok(Ok(applications)) => Ok(Json(applications)),
            Ok(Err(_)) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            Err(_) => Err(StatusCode::REQUEST_TIMEOUT),
        }
    }

    /// Get gateways API
    pub async fn api_gateways(State(state): State<Arc<DashboardState>>) -> Result<Json<Vec<GatewayInfo>>, StatusCode> {
        match tokio::time::timeout(Duration::from_secs(5), state.gateway_manager.get_all_gateways()).await {
            Ok(Ok(gateways)) => Ok(Json(gateways)),
            Ok(Err(_)) => Err(StatusCode::INTERNAL_SERVER_ERROR),
            Err(_) => Err(StatusCode::REQUEST_TIMEOUT),
        }
    }

    /// Create gateway
    pub async fn create_gateway(
        State(state): State<Arc<DashboardState>>,
        Json(request): Json<serde_json::Value>,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Creating gateway: {:?}", request);
        
        // For now, return a mock response
        Ok(Json(serde_json::json!({
            "success": true,
            "message": "Gateway created successfully",
            "gateway": {
                "id": "gateway-1",
                "name": request.get("name").unwrap_or(&serde_json::Value::String("gateway-1".to_string())),
                "status": "Active",
                "endpoint": "127.0.0.1:30452"
            }
        })))
    }

    /// Create device
    pub async fn create_device(
        State(state): State<Arc<DashboardState>>,
        Json(request): Json<serde_json::Value>,
    ) -> Result<Json<serde_json::Value>, StatusCode> {
        info!("Creating device: {:?}", request);
        
        // For now, return a mock response
        Ok(Json(serde_json::json!({
            "success": true,
            "message": "Device created successfully",
            "device": {
                "id": "device-1",
                "name": request.get("name").unwrap_or(&serde_json::Value::String("device-1".to_string())),
                "type": request.get("type").unwrap_or(&serde_json::Value::String("RISC-V MCU".to_string())),
                "status": "Pending"
            }
        })))
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

    async fn run_web_server(self) -> anyhow::Result<()> {
        let app = Router::new()
            .route("/", get(DashboardApi::home))
            .route("/devices", get(DashboardApi::devices))
            .route("/applications", get(DashboardApi::applications))
            .route("/gateways", get(DashboardApi::gateways))
            .route("/monitoring", get(DashboardApi::monitoring))
            .route("/api/status", get(DashboardApi::api_status))
            .route("/api/devices", get(DashboardApi::api_devices))
            .route("/api/applications", get(DashboardApi::api_applications))
            .route("/api/gateways", get(DashboardApi::api_gateways))
            .route("/api/v1/status", get(DashboardApi::api_status))
            .route("/api/v1/devices", get(DashboardApi::api_devices))
            .route("/api/v1/applications", get(DashboardApi::api_applications))
            .route("/api/v1/gateways", get(DashboardApi::api_gateways))
            .route("/api/v1/gateways", post(DashboardApi::create_gateway))
            .route("/api/v1/devices", post(DashboardApi::create_device))
            .route("/api/deploy", post(DashboardApi::deploy_application))
            .route("/api/pairing/enable", post(DashboardApi::enable_pairing))
            .route("/api/pairing/disable", post(DashboardApi::disable_pairing))
            .route("/api/v1/terminal/execute", post(DashboardApi::execute_terminal_command))
            .route("/api/v1/pods", get(DashboardApi::get_pods))
            .route("/api/v1/services", get(DashboardApi::get_services))
            .route("/api/v1/metrics", get(DashboardApi::get_pod_metrics))
            .route("/api/v1/logs", get(DashboardApi::get_logs))
            .route("/ws", get(DashboardApi::websocket_handler))
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
