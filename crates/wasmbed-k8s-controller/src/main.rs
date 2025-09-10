// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::sync::Arc;
use std::time::Duration;
use std::collections::BTreeMap;

use anyhow::{Context, Result};
use kube::{
    api::{Api, ListParams, Patch, PatchParams},
    client::Client,
    ResourceExt,
    runtime::{
        controller::{Action, Controller},
        events::Recorder,
        watcher,
    },
};
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use axum::{
    routing::get,
    Router,
    http::StatusCode,
    response::Json,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use futures_util::StreamExt;
use k8s_openapi;

use wasmbed_k8s_resource::{
    Application, ApplicationPhase, ApplicationSpec, Device, DevicePhase,
    ApplicationStatus, DeviceApplicationStatus, DeviceApplicationPhase,
};
use wasmbed_protocol::ApplicationConfig;

/// Custom error type for the controller
#[derive(Debug, thiserror::Error)]
pub enum ControllerError {
    #[error("Application error: {0}")]
    Application(#[from] anyhow::Error),
    #[error("Kubernetes error: {0}")]
    Kubernetes(#[from] kube::Error),
}

/// Complete Application Controller for Wasmbed with Kubernetes Integration
pub struct ApplicationController {
    client: Client,
    gateway_client: Arc<GatewayClient>,
    app_status_cache: Arc<tokio::sync::RwLock<BTreeMap<String, ApplicationStatus>>>,
    retry_config: RetryConfig,
}

/// Configuration for retry logic
#[derive(Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Real Gateway Client with proper error handling
pub struct GatewayClient {
    http_client: reqwest::Client,
    gateway_url: String,
    timeout: Duration,
}

impl GatewayClient {
    pub fn new(gateway_url: String) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            gateway_url,
            timeout: Duration::from_secs(30),
        }
    }

    /// Deploy an application to a device with retry logic
    pub async fn deploy_application(
        &self,
        device_id: &str,
        app_id: &str,
        app_name: &str,
        wasm_bytes: &[u8],
        _config: Option<ApplicationConfig>,
    ) -> Result<()> {
        let url = format!("{}/api/v1/devices/{}/deploy", self.gateway_url, device_id);
        
        // Create payload without config for now (config needs Serialize)
        let payload = serde_json::json!({
            "app_id": app_id,
            "name": app_name,
            "wasm_bytes": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, wasm_bytes),
        });

        let response = self.http_client
            .post(&url)
            .json(&payload)
            .send()
            .await
            .context("Failed to send deployment request to gateway")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Gateway deployment failed: {}", error_text));
        }

        info!("Successfully deployed application {} to device {}", app_id, device_id);
        Ok(())
    }

    /// Stop an application on a device
    pub async fn stop_application(&self, device_id: &str, app_id: &str) -> Result<()> {
        let url = format!("{}/api/v1/devices/{}/stop/{}", self.gateway_url, device_id, app_id);
        
        let response = self.http_client
            .post(&url)
            .send()
            .await
            .context("Failed to send stop request to gateway")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Gateway stop failed: {}", error_text));
        }

        info!("Successfully stopped application {} on device {}", app_id, device_id);
        Ok(())
    }

    /// Get application status from device
    pub async fn get_application_status(&self, device_id: &str, app_id: &str) -> Result<DeviceApplicationStatus> {
        let url = format!("{}/api/v1/devices/{}/status/{}", self.gateway_url, device_id, app_id);
        
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .context("Failed to get application status from gateway")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Gateway status request failed: {}", error_text));
        }

        let status: DeviceApplicationStatus = response.json().await
            .context("Failed to parse application status")?;

        Ok(status)
    }
}

impl ApplicationController {
    pub fn new(client: Client, gateway_url: String) -> Self {
        let gateway_client = Arc::new(GatewayClient::new(gateway_url));
        let app_status_cache = Arc::new(tokio::sync::RwLock::new(BTreeMap::new()));
        
        Self {
            client,
            gateway_client,
            app_status_cache,
            retry_config: RetryConfig::default(),
        }
    }

    /// Reconcile a single application with continuous monitoring
    pub async fn reconcile(&self, app: Arc<Application>) -> Result<Action, ControllerError> {
        let app_name = app.name_any();
        let app_namespace = app.namespace().unwrap_or_default();
        
        info!("Reconciling Application {} in namespace {}", app_name, app_namespace);

        // Get current application status from Kubernetes
        let current_status = app.status()
            .map(|s| s.phase)
            .unwrap_or(ApplicationPhase::Creating);

        // Handle different phases
        match current_status {
            ApplicationPhase::Creating => {
                self.handle_creating_phase(&app).await?;
                Ok(Action::requeue(Duration::from_secs(5)))
            },
            ApplicationPhase::Deploying => {
                self.handle_deploying_phase(&app).await?;
                Ok(Action::requeue(Duration::from_secs(10)))
            },
            ApplicationPhase::Running | ApplicationPhase::PartiallyRunning => {
                self.handle_running_phase(&app).await?;
                Ok(Action::requeue(Duration::from_secs(30)))
            },
            ApplicationPhase::Stopping => {
                self.handle_stopping_phase(&app).await?;
                Ok(Action::requeue(Duration::from_secs(10)))
            },
            ApplicationPhase::Stopped => {
                self.handle_stopped_phase(&app).await?;
                Ok(Action::requeue(Duration::from_secs(60)))
            },
            ApplicationPhase::Failed => {
                self.handle_failed_phase(&app).await?;
                Ok(Action::requeue(Duration::from_secs(120)))
            },
            ApplicationPhase::Deleting => {
                self.handle_deleting_phase(&app).await?;
                Ok(Action::requeue(Duration::from_secs(5)))
            },
        }
    }

    /// Enable pairing mode for device enrollment
    pub async fn enable_pairing_mode(&self) -> Result<()> {
        info!("Enabling pairing mode for device enrollment");
        
        // Create a ConfigMap to store pairing mode state
        let config_map = k8s_openapi::api::core::v1::ConfigMap {
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                name: Some("wasmbed-pairing-mode".to_string()),
                namespace: Some("wasmbed".to_string()),
                ..Default::default()
            },
            data: Some(std::collections::BTreeMap::from([
                ("enabled".to_string(), "true".to_string()),
                ("enabled_at".to_string(), chrono::Utc::now().to_rfc3339()),
            ])),
            ..Default::default()
        };

        let config_api: Api<k8s_openapi::api::core::v1::ConfigMap> = Api::namespaced(self.client.clone(), "wasmbed");
        
        match config_api.get("wasmbed-pairing-mode").await {
            Ok(_) => {
                // Update existing ConfigMap
                config_api.replace("wasmbed-pairing-mode", &Default::default(), &config_map).await?;
            },
            Err(_) => {
                // Create new ConfigMap
                config_api.create(&Default::default(), &config_map).await?;
            }
        }

        info!("Pairing mode enabled successfully");
        Ok(())
    }

    /// Disable pairing mode
    pub async fn disable_pairing_mode(&self) -> Result<()> {
        info!("Disabling pairing mode");
        
        let config_api: Api<k8s_openapi::api::core::v1::ConfigMap> = Api::namespaced(self.client.clone(), "wasmbed");
        config_api.delete("wasmbed-pairing-mode", &Default::default()).await?;
        
        info!("Pairing mode disabled successfully");
        Ok(())
    }

    /// Check if pairing mode is enabled
    pub async fn is_pairing_mode_enabled(&self) -> Result<bool> {
        let config_api: Api<k8s_openapi::api::core::v1::ConfigMap> = Api::namespaced(self.client.clone(), "wasmbed");
        
        match config_api.get("wasmbed-pairing-mode").await {
            Ok(config_map) => {
                if let Some(data) = config_map.data {
                    Ok(data.get("enabled").map(|v| v == "true").unwrap_or(false))
                } else {
                    Ok(false)
                }
            },
            Err(_) => Ok(false)
        }
    }

    /// Handle Creating phase
    async fn handle_creating_phase(&self, app: &Application) -> Result<()> {
        let app_name = app.name_any();
        info!("Handling Creating phase for Application {}", app_name);

        // Validate application specification
        self.validate_application_spec(&app.spec)?;

        // Find target devices from Kubernetes
        let target_devices = self.find_target_devices(&app.spec).await?;
        
        if target_devices.is_empty() {
            self.update_application_status(app, ApplicationPhase::Failed, 
                "No target devices found").await?;
            return Ok(());
        }

        // Update status to Deploying in Kubernetes
        self.update_application_status(app, ApplicationPhase::Deploying, 
            "Starting deployment").await?;
        
        Ok(())
    }

    /// Handle Deploying phase
    async fn handle_deploying_phase(&self, app: &Application) -> Result<()> {
        let app_name = app.name_any();
        info!("Handling Deploying phase for Application {}", app_name);

        // Find target devices from Kubernetes
        let target_devices = self.find_target_devices(&app.spec).await?;
        
        // Decode WASM bytes
        let wasm_bytes = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &app.spec.wasm_bytes)
            .context("Failed to decode WASM bytes")?;

        // Deploy to each target device with retry logic
        let mut deployed_count = 0;
        let mut failed_count = 0;
        let mut device_statuses = BTreeMap::new();

        for device in target_devices {
            match self.deploy_to_device_with_retry(app, &device, &wasm_bytes).await {
                Ok(_) => {
                    deployed_count += 1;
                    // Mark as deploying initially - MCU feedback will update to Running/Failed
                    device_statuses.insert(device.name_any(), 
                        DeviceApplicationStatus {
                            status: DeviceApplicationPhase::Deploying,
                            last_heartbeat: Some(chrono::Utc::now().to_rfc3339()),
                            metrics: None,
                            error: None,
                            restart_count: 0,
                        });
                    info!("Deployment request sent for {} to device {}", app_name, device.name_any());
                },
                Err(e) => {
                    failed_count += 1;
                    device_statuses.insert(device.name_any(), 
                        DeviceApplicationStatus {
                            status: DeviceApplicationPhase::Failed,
                            last_heartbeat: Some(chrono::Utc::now().to_rfc3339()),
                            metrics: None,
                            error: Some(e.to_string()),
                            restart_count: 0,
                        });
                    error!("Failed to deploy {} to device {}: {}", app_name, device.name_any(), e);
                }
            }
        }

        // Update status in Kubernetes based on deployment results
        if failed_count == 0 {
            // Check if all deployments are still in progress
            let all_deploying = device_statuses.values()
                .all(|s| matches!(s.status, DeviceApplicationPhase::Deploying));
            
            if all_deploying {
                // All deployments are still in progress, keep in Deploying phase
                self.update_application_status_with_devices(app, ApplicationPhase::Deploying, 
                    "Deployment requests sent, waiting for MCU feedback", device_statuses).await?;
            } else {
                // Some deployments have completed, check if all are running
                let running_count = device_statuses.values()
                    .filter(|s| matches!(s.status, DeviceApplicationPhase::Running))
                    .count();
                
                if running_count == deployed_count {
                    self.update_application_status_with_devices(app, ApplicationPhase::Running, 
                        "All devices deployed successfully", device_statuses).await?;
                } else {
                    self.update_application_status_with_devices(app, ApplicationPhase::PartiallyRunning, 
                        &format!("Deployed to {} devices, {} still deploying", running_count, deployed_count - running_count), 
                        device_statuses).await?;
                }
            }
        } else if deployed_count > 0 {
            self.update_application_status_with_devices(app, ApplicationPhase::PartiallyRunning, 
                &format!("Deployment requests sent to {} devices, {} failed", deployed_count, failed_count), 
                device_statuses).await?;
        } else {
            self.update_application_status_with_devices(app, ApplicationPhase::Failed, 
                &format!("Failed to send deployment requests to any devices ({} failed)", failed_count), 
                device_statuses).await?;
        }

        Ok(())
    }

    /// Handle Running phase with monitoring
    async fn handle_running_phase(&self, app: &Application) -> Result<()> {
        let app_name = app.name_any();
        debug!("Handling Running phase for Application {}", app_name);

        // Monitor application status on all devices
        let target_devices = self.find_target_devices(&app.spec).await?;
        let mut all_healthy = true;
        let mut device_statuses = BTreeMap::new();

        for device in target_devices {
            let device_id = device.spec.public_key.to_string();
            let app_id = self.get_app_id_from_status(app).await?;
            
            match self.gateway_client.get_application_status(&device_id, &app_id).await {
                Ok(status) => {
                    if matches!(status.status, DeviceApplicationPhase::Failed) {
                        all_healthy = false;
                    }
                    device_statuses.insert(device.name_any(), status);
                },
                Err(e) => {
                    warn!("Failed to get status for device {}: {}", device_id, e);
                    all_healthy = false;
                }
            }
        }

        // Update status in Kubernetes if needed
        if !all_healthy {
            self.update_application_status_with_devices(app, ApplicationPhase::PartiallyRunning, 
                "Some devices are unhealthy", device_statuses).await?;
        }

        Ok(())
    }

    /// Handle Stopping phase
    async fn handle_stopping_phase(&self, app: &Application) -> Result<()> {
        let app_name = app.name_any();
        info!("Handling Stopping phase for Application {}", app_name);

        // Stop application on all devices
        let target_devices = self.find_target_devices(&app.spec).await?;
        let mut stopped_count = 0;
        let mut failed_count = 0;

        for device in target_devices {
            let device_id = device.spec.public_key.to_string();
            let app_id = self.get_app_id_from_status(app).await?;
            
            match self.gateway_client.stop_application(&device_id, &app_id).await {
                Ok(_) => {
                    stopped_count += 1;
                    info!("Successfully stopped {} on device {}", app_name, device_id);
                },
                Err(e) => {
                    failed_count += 1;
                    error!("Failed to stop {} on device {}: {}", app_name, device_id, e);
                }
            }
        }

        // Update status to Stopped in Kubernetes
        self.update_application_status(app, ApplicationPhase::Stopped, 
            &format!("Stopped on {} devices, {} failed", stopped_count, failed_count)).await?;

        Ok(())
    }

    /// Handle other phases
    async fn handle_stopped_phase(&self, app: &Application) -> Result<()> {
        debug!("Application {} is stopped", app.name_any());
        Ok(())
    }

    async fn handle_failed_phase(&self, app: &Application) -> Result<()> {
        debug!("Application {} is in failed state", app.name_any());
        Ok(())
    }

    async fn handle_deleting_phase(&self, app: &Application) -> Result<()> {
        let app_name = app.name_any();
        info!("Handling Deleting phase for Application {}", app_name);

        // Stop application on all devices before deletion
        self.handle_stopping_phase(app).await?;

        Ok(())
    }

    /// Validate application specification
    fn validate_application_spec(&self, spec: &ApplicationSpec) -> Result<()> {
        if spec.name.is_empty() {
            return Err(anyhow::anyhow!("Application name cannot be empty"));
        }

        if spec.wasm_bytes.is_empty() {
            return Err(anyhow::anyhow!("WASM bytes cannot be empty"));
        }

        // Validate base64 encoding
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &spec.wasm_bytes)
            .context("WASM bytes must be valid base64")?;

        // Validate configuration
        if let Some(config) = &spec.config {
            if config.memory_limit == 0 {
                return Err(anyhow::anyhow!("Memory limit must be greater than 0"));
            }
            if config.cpu_time_limit == 0 {
                return Err(anyhow::anyhow!("CPU time limit must be greater than 0"));
            }
        }

        Ok(())
    }

    /// Find target devices from Kubernetes
    async fn find_target_devices(&self, spec: &ApplicationSpec) -> Result<Vec<Device>> {
        let devices_api: Api<Device> = Api::all(self.client.clone());
        let mut target_devices = Vec::new();

        // Handle device_names selector
        if let Some(device_names) = &spec.target_devices.device_names {
            for device_name in device_names {
                if let Ok(device) = devices_api.get(device_name).await {
                    // Only include connected devices
                    if device.status.as_ref()
                        .and_then(|s| Some(matches!(s.phase, DevicePhase::Connected)))
                        .unwrap_or(false) {
                        target_devices.push(device);
                    }
                }
            }
        }

        // Handle all_devices
        if spec.target_devices.all_devices.unwrap_or(false) {
            let devices = devices_api.list(&ListParams::default()).await?;
            
            for device in devices {
                // Only include connected devices
                if device.status.as_ref()
                    .and_then(|s| Some(matches!(s.phase, DevicePhase::Connected)))
                    .unwrap_or(false) {
                    target_devices.push(device);
                }
            }
        }

        info!("Found {} target devices", target_devices.len());
        Ok(target_devices)
    }

    /// Deploy application to device with retry logic
    async fn deploy_to_device_with_retry(&self, app: &Application, device: &Device, wasm_bytes: &[u8]) -> Result<()> {
        let mut delay = self.retry_config.initial_delay;
        
        for attempt in 0..=self.retry_config.max_retries {
            match self.deploy_to_device(app, device, wasm_bytes).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    if attempt == self.retry_config.max_retries {
                        return Err(e);
                    }
                    
                    warn!("Deployment attempt {} failed, retrying in {:?}: {}", 
                        attempt + 1, delay, e);
                    
                    sleep(delay).await;
                    delay = std::cmp::min(
                        Duration::from_secs_f32(delay.as_secs_f32() * self.retry_config.backoff_multiplier),
                        self.retry_config.max_delay
                    );
                }
            }
        }
        
        unreachable!()
    }

    /// Deploy application to a specific device
    async fn deploy_to_device(&self, app: &Application, device: &Device, wasm_bytes: &[u8]) -> Result<()> {
        let app_id = Uuid::new_v4().to_string();
        let device_id = device.spec.public_key.to_string();

        // Deploy via gateway
        self.gateway_client.deploy_application(
            &device_id,
            &app_id,
            &app.spec.name,
            wasm_bytes,
            None, // config not serializable yet
        ).await?;

        Ok(())
    }

    /// Update application status in Kubernetes
    async fn update_application_status(&self, app: &Application, phase: ApplicationPhase, message: &str) -> Result<()> {
        let apps_api: Api<Application> = Api::all(self.client.clone());
        
        // Validate state transition
        let current_phase = app.status().as_ref().map(|s| s.phase).unwrap_or(ApplicationPhase::Creating);
        if !ApplicationPhase::validate_transition(current_phase, phase) {
            warn!("Invalid state transition from {:?} to {:?} for application {}", current_phase, phase, app.name_any());
            // Still proceed with the update but log the invalid transition
        }
        
        let status = ApplicationStatus {
            phase: phase.clone(),
            device_statuses: Some(BTreeMap::new()),
            statistics: Some(wasmbed_k8s_resource::ApplicationStatistics {
                total_devices: 0,
                deployed_devices: 0,
                running_devices: 0,
                failed_devices: 0,
                stopped_devices: 0,
            }),
            last_updated: Some(chrono::Utc::now().to_rfc3339()),
            error: if matches!(phase, ApplicationPhase::Failed) {
                Some(message.to_string())
            } else {
                None
            },
        };

        let patch = serde_json::json!({
            "status": status
        });

        let pp = PatchParams::default();
        apps_api.patch(&app.name_any(), &pp, &Patch::Merge(patch)).await?;

        info!("Updated Application {} status to {:?}: {}", app.name_any(), phase, message);
        Ok(())
    }

    /// Update application status with device statuses in Kubernetes
    async fn update_application_status_with_devices(&self, app: &Application, phase: ApplicationPhase, 
        message: &str, device_statuses: BTreeMap<String, DeviceApplicationStatus>) -> Result<()> {
        let apps_api: Api<Application> = Api::all(self.client.clone());
        
        // Validate state transition
        let current_phase = app.status().as_ref().map(|s| s.phase).unwrap_or(ApplicationPhase::Creating);
        if !ApplicationPhase::validate_transition(current_phase, phase) {
            warn!("Invalid state transition from {:?} to {:?} for application {}", current_phase, phase, app.name_any());
            // Still proceed with the update but log the invalid transition
        }
        
        // Calculate statistics
        let total_devices = device_statuses.len() as u32;
        let running_devices = device_statuses.values()
            .filter(|s| matches!(s.status, DeviceApplicationPhase::Running))
            .count() as u32;
        let failed_devices = device_statuses.values()
            .filter(|s| matches!(s.status, DeviceApplicationPhase::Failed))
            .count() as u32;
        
        let status = ApplicationStatus {
            phase: phase.clone(),
            device_statuses: Some(device_statuses),
            statistics: Some(wasmbed_k8s_resource::ApplicationStatistics {
                total_devices,
                deployed_devices: total_devices,
                running_devices,
                failed_devices,
                stopped_devices: total_devices - running_devices - failed_devices,
            }),
            last_updated: Some(chrono::Utc::now().to_rfc3339()),
            error: if matches!(phase, ApplicationPhase::Failed) {
                Some(message.to_string())
            } else {
                None
            },
        };

        let patch = serde_json::json!({
            "status": status
        });

        let pp = PatchParams::default();
        apps_api.patch(&app.name_any(), &pp, &Patch::Merge(patch)).await?;

        info!("Updated Application {} status to {:?}: {}", app.name_any(), phase, message);
        Ok(())
    }

    /// Get app ID from status (in real implementation, this would be stored)
    async fn get_app_id_from_status(&self, app: &Application) -> Result<String> {
        // In a real implementation, this would be stored in the application status
        // For now, generate a consistent ID based on the app name
        Ok(format!("app-{}", app.spec.name))
    }
}

/// Health check endpoint
async fn health_check() -> StatusCode {
    StatusCode::OK
}

/// Readiness check endpoint
async fn readiness_check() -> StatusCode {
    StatusCode::OK
}

/// Metrics endpoint
async fn metrics() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "controller": "wasmbed-k8s-controller"
    }))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting Wasmbed Application Controller with Continuous Reconciliation...");

    // Create Kubernetes client
    let client = Client::try_default().await?;

    // Gateway URL (in production, this would come from config)
    let gateway_url = std::env::var("WASMBED_GATEWAY_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Create controller
    let controller = ApplicationController::new(client.clone(), gateway_url);
    let controller = Arc::new(controller);

    // Create API for Applications
    let apps_api: Api<Application> = Api::all(client.clone());

    // Create recorder for events
    let recorder = Recorder::new(client.clone(), "wasmbed-controller".to_string().into());

    // Set up HTTP server for health checks
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        .route("/metrics", get(metrics));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Starting HTTP server on {}", addr);

    // Start HTTP server in background
    let server_handle = tokio::spawn(async move {
        let listener = TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    // Create the controller with continuous reconciliation
    let controller_handle = Controller::new(apps_api, watcher::Config::default())
        .shutdown_on_signal()
        .run(
            move |obj, _ctx| {
                let controller = controller.clone();
                async move {
                    controller.reconcile(obj).await
                }
            },
            move |_obj, _err, _ctx| {
                Action::requeue(Duration::from_secs(30))
            },
            Arc::new(recorder),
        )
        .for_each(|res| async move {
            match res {
                Ok(o) => info!("Reconciled {:?}", o),
                Err(e) => warn!("Reconciliation error: {}", e),
            }
        });

    info!("Starting continuous reconciliation...");
    
    // Run both the HTTP server and the controller
    tokio::select! {
        _ = server_handle => {
            info!("HTTP server stopped");
        }
        _ = controller_handle => {
            info!("Controller stopped");
        }
    }

    info!("Application Controller finished");
    Ok(())
}
