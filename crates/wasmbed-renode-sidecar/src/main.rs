// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    routing::{get, post, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StartRenodeRequest {
    device_id: String,
    firmware_path: String,
    script_path: String,
    port: u16,
    gateway_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RenodeStatus {
    device_id: String,
    running: bool,
    container_name: String,
    port: Option<u16>,
}

type DeviceRegistry = Arc<RwLock<HashMap<String, RenodeStatus>>>;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let registry: DeviceRegistry = Arc::new(RwLock::new(HashMap::new()));

    let app = Router::new()
        .route("/health", get(health))
        .route("/devices/:device_id/start", post(start_renode))
        .route("/devices/:device_id/stop", delete(stop_renode))
        .route("/devices/:device_id/status", get(get_status))
        .route("/devices", get(list_devices))
        .with_state(registry);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3002").await.unwrap();
    info!("Renode sidecar listening on 0.0.0.0:3002");
    
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "service": "renode-sidecar"
    })))
}

async fn start_renode(
    axum::extract::State(registry): axum::extract::State<DeviceRegistry>,
    Path(device_id): Path<String>,
    Json(request): Json<StartRenodeRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Starting Renode for device: {}", device_id);

    let container_name = format!("renode-{}", device_id);
    
    // Check if container already exists
    let check_output = Command::new("docker")
        .args(&["ps", "-a", "--filter", &format!("name={}", container_name), "--format", "{{.Names}} {{.Status}}"])
        .output();

    if let Ok(output) = check_output {
        let output_str = String::from_utf8_lossy(&output.stdout);
        if output_str.contains(&container_name) {
            if output_str.contains("Up") || output_str.contains("running") {
                warn!("Container {} already running", container_name);
                // Update registry
                let mut reg = registry.write().await;
                reg.insert(device_id.clone(), RenodeStatus {
                    device_id: device_id.clone(),
                    running: true,
                    container_name: container_name.clone(),
                    port: Some(request.port),
                });
                return Ok(Json(serde_json::json!({
                    "success": true,
                    "message": format!("Renode container {} already running", container_name),
                    "container_name": container_name,
                    "port": request.port
                })));
            } else {
                // Remove stopped container
                info!("Removing stopped container: {}", container_name);
                let _ = Command::new("docker")
                    .args(&["rm", "-f", &container_name])
                    .output();
            }
        }
    }

    // Build docker run command
    let firmware_mount = format!("{}:/firmware:ro", request.firmware_path);
    let script_mount = format!("{}:/script.resc:ro", request.script_path);
    let renode_command = format!(
        "renode --console --plain --port {} /script.resc & RENODE_PID=$!; sleep 2; while kill -0 $RENODE_PID 2>/dev/null; do sleep 1; done; while true; do sleep 3600; done",
        request.port
    );
    
    let docker_args = vec![
        "run",
        "--restart=unless-stopped",
        "--net=host",
        "--name", &container_name,
        "-v", &firmware_mount,
        "-v", &script_mount,
        "antmicro/renode:nightly",
        "sh", "-c",
        &renode_command,
    ];

    info!("Executing: docker {}", docker_args.join(" "));
    
    let output = Command::new("docker")
        .args(&docker_args)
        .output();

    match output {
        Ok(output) if output.status.success() => {
            info!("Renode container {} started successfully", container_name);
            
            // Update registry
            let mut reg = registry.write().await;
            reg.insert(device_id.clone(), RenodeStatus {
                device_id: device_id.clone(),
                running: true,
                container_name: container_name.clone(),
                port: Some(request.port),
            });

            Ok(Json(serde_json::json!({
                "success": true,
                "message": format!("Renode started for device {}", device_id),
                "container_name": container_name,
                "port": request.port
            })))
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to start Renode container: {}", stderr);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(e) => {
            error!("Failed to execute docker command: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn stop_renode(
    axum::extract::State(registry): axum::extract::State<DeviceRegistry>,
    Path(device_id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("Stopping Renode for device: {}", device_id);

    let container_name = format!("renode-{}", device_id);
    
    let output = Command::new("docker")
        .args(&["stop", &container_name])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            info!("Renode container {} stopped", container_name);
            
            // Update registry
            let mut reg = registry.write().await;
            if let Some(status) = reg.get_mut(&device_id) {
                status.running = false;
            }

            Ok(Json(serde_json::json!({
                "success": true,
                "message": format!("Renode stopped for device {}", device_id)
            })))
        }
        Ok(_) => {
            // Container might not exist, which is OK
            warn!("Container {} not found or already stopped", container_name);
            let mut reg = registry.write().await;
            reg.remove(&device_id);
            Ok(Json(serde_json::json!({
                "success": true,
                "message": format!("Container {} not found (may already be stopped)", container_name)
            })))
        }
        Err(e) => {
            error!("Failed to stop Renode container: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_status(
    axum::extract::State(registry): axum::extract::State<DeviceRegistry>,
    Path(device_id): Path<String>,
) -> Result<Json<RenodeStatus>, StatusCode> {
    let reg = registry.read().await;
    if let Some(status) = reg.get(&device_id) {
        Ok(Json(status.clone()))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn list_devices(
    axum::extract::State(registry): axum::extract::State<DeviceRegistry>,
) -> Json<Vec<RenodeStatus>> {
    let reg = registry.read().await;
    Json(reg.values().cloned().collect())
}
