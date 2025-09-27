// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use clap::Parser;
use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
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
        let devices = state.device_manager.get_all_devices().await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(devices))
    }

    /// Get applications API
    pub async fn api_applications(State(state): State<Arc<DashboardState>>) -> Result<Json<Vec<ApplicationInfo>>, StatusCode> {
        let applications = state.application_manager.get_all_applications().await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(applications))
    }

    /// Get gateways API
    pub async fn api_gateways(State(state): State<Arc<DashboardState>>) -> Result<Json<Vec<GatewayInfo>>, StatusCode> {
        let gateways = state.gateway_manager.get_all_gateways().await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(gateways))
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
        State(state): State<Arc<DashboardState>>,
    ) -> axum::response::Response {
        ws.on_upgrade(|socket| async move {
            // Handle WebSocket connection for real-time updates
            info!("WebSocket connection established");
            
            // In a real implementation, this would:
            // 1. Send periodic status updates
            // 2. Handle client commands
            // 3. Manage connection lifecycle
        })
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
            // Update system status
            let mut status = self.state.system_status.write().await;
            
            // Update device stats
            if let Ok(devices) = self.state.device_manager.get_all_devices().await {
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
            }

            // Update application stats
            if let Ok(applications) = self.state.application_manager.get_all_applications().await {
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
            }

            // Update gateway stats
            if let Ok(gateways) = self.state.gateway_manager.get_all_gateways().await {
                status.gateways.total = gateways.len() as u32;
                status.gateways.active = gateways.iter()
                    .filter(|g| g.status == "active")
                    .count() as u32;
                status.gateways.inactive = gateways.iter()
                    .filter(|g| g.status == "inactive")
                    .count() as u32;
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
            .route("/api/deploy", post(DashboardApi::deploy_application))
            .route("/api/pairing/enable", post(DashboardApi::enable_pairing))
            .route("/api/pairing/disable", post(DashboardApi::disable_pairing))
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
