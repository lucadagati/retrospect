// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use kube::{
    api::{Api, ListParams, Patch, PatchParams},
    client::Client,
    ResourceExt,
};
use wasmbed_k8s_resource::{Application, Device, ApplicationStatus, ApplicationPhase, DeviceApplicationPhase};
use wasmbed_qemu_manager::{QemuManager, WasmConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{info, error, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentRequest {
    pub application_id: String,
    pub device_id: String,
    pub wasm_bytes: Vec<u8>,
    pub config: WasmConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStatus {
    pub application_id: String,
    pub device_id: String,
    pub status: DeploymentPhase,
    pub error: Option<String>,
    pub deployed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentPhase {
    Pending,
    Deploying,
    Deployed,
    Failed,
    Stopped,
}

pub struct QemuDeploymentService {
    client: Client,
    applications: Api<Application>,
    devices: Api<Device>,
    qemu_manager: QemuManager,
    deployments: Mutex<HashMap<String, DeploymentStatus>>,
}

impl QemuDeploymentService {
    pub fn new(client: Client, qemu_manager: QemuManager) -> Self {
        Self {
            applications: Api::<Application>::all(client.clone()),
            devices: Api::<Device>::all(client.clone()),
            client,
            qemu_manager,
            deployments: Mutex::new(HashMap::new()),
        }
    }

    pub async fn deploy_application(&self, application: &Application) -> Result<(), String> {
        let app_id = application.name_any();
        info!("Deploying application: {}", app_id);

        // Find target devices
        let target_devices = self.find_target_devices(&application.spec.target_devices).await?;
        
        if target_devices.is_empty() {
            return Err("No target devices found".to_string());
        }

        // Deploy to each target device
        for device in target_devices {
            let device_id = device.name_any();
            let deployment_key = format!("{}:{}", app_id, device_id);

            // Check if device is running in QEMU
            if let Some(qemu_device) = self.qemu_manager.get_device(&device_id).await {
                if !matches!(qemu_device.status, wasmbed_qemu_manager::QemuDeviceStatus::Running) {
                    warn!("Device {} is not running in QEMU, starting it...", device_id);
                    if let Err(e) = self.qemu_manager.start_device(&device_id).await {
                        error!("Failed to start device {}: {}", device_id, e);
                        continue;
                    }
                }

                // Deploy WASM to the device
                let config = WasmConfig {
                    memory_limit: 128 * 1024 * 1024, // 128MB
                    execution_timeout: 30, // 30 seconds
                    host_functions: vec![
                        "print_message".to_string(),
                        "get_timestamp".to_string(),
                    ],
                };

                if let Err(e) = self.qemu_manager.deploy_wasm(&device_id, application.spec.wasm_bytes.as_bytes().to_vec(), config).await {
                    error!("Failed to deploy WASM to device {}: {}", device_id, e);
                    
                    // Update deployment status
                    let mut deployments = self.deployments.lock().await;
                    deployments.insert(deployment_key.clone(), DeploymentStatus {
                        application_id: app_id.clone(),
                        device_id: device_id.clone(),
                        status: DeploymentPhase::Failed,
                        error: Some(e.to_string()),
                        deployed_at: None,
                    });
                } else {
                    info!("Successfully deployed WASM to device: {}", device_id);
                    
                    // Update deployment status
                    let mut deployments = self.deployments.lock().await;
                    deployments.insert(deployment_key.clone(), DeploymentStatus {
                        application_id: app_id.clone(),
                        device_id: device_id.clone(),
                        status: DeploymentPhase::Deployed,
                        error: None,
                        deployed_at: Some(chrono::Utc::now()),
                    });
                }
            } else {
                warn!("Device {} not found in QEMU manager", device_id);
            }
        }

        // Update application status in Kubernetes
        self.update_application_status(application).await?;

        Ok(())
    }

    pub async fn stop_application(&self, application: &Application) -> Result<(), String> {
        let app_id = application.name_any();
        info!("Stopping application: {}", app_id);

        // Find target devices
        let target_devices = self.find_target_devices(&application.spec.target_devices).await?;

        // Stop deployment on each device
        for device in target_devices {
            let device_id = device.name_any();
            let deployment_key = format!("{}:{}", app_id, device_id);

            // Update deployment status
            let mut deployments = self.deployments.lock().await;
            if let Some(deployment) = deployments.get_mut(&deployment_key) {
                deployment.status = DeploymentPhase::Stopped;
                deployment.deployed_at = None;
            }
        }

        // Update application status in Kubernetes
        let mut status = application.status().cloned().unwrap_or_else(|| ApplicationStatus {
            phase: ApplicationPhase::Creating,
            device_statuses: None,
            statistics: None,
            last_updated: None,
            error: None,
        });
        status.phase = ApplicationPhase::Stopped;
        status.last_updated = Some(chrono::Utc::now().to_rfc3339());
        
        self.update_application_status_with_status(application, status).await?;

        Ok(())
    }

    pub async fn get_deployment_status(&self, application_id: &str, device_id: &str) -> Option<DeploymentStatus> {
        let deployments = self.deployments.lock().await;
        let key = format!("{}:{}", application_id, device_id);
        deployments.get(&key).cloned()
    }

    pub async fn list_deployments(&self) -> Vec<DeploymentStatus> {
        let deployments = self.deployments.lock().await;
        deployments.values().cloned().collect()
    }

    async fn find_target_devices(&self, target_devices_selector: &wasmbed_k8s_resource::TargetDevices) -> Result<Vec<Device>, String> {
        let all_devices = self.devices.list(&ListParams::default()).await
            .map_err(|e| format!("Failed to list devices: {}", e))?;

        let mut matching_devices = vec![];

        // If all_devices is true, return all devices
        if target_devices_selector.all_devices.unwrap_or(false) {
            return Ok(all_devices.items);
        }

        // If device_names is specified, match by name
        if let Some(device_names) = &target_devices_selector.device_names {
            for device in all_devices.items {
                if device_names.contains(&device.name_any()) {
                    matching_devices.push(device);
                }
            }
        }

        Ok(matching_devices)
    }

    async fn update_application_status(&self, application: &Application) -> Result<(), String> {
        let mut status = application.status().cloned().unwrap_or_else(|| ApplicationStatus {
            phase: ApplicationPhase::Creating,
            device_statuses: None,
            statistics: None,
            last_updated: None,
            error: None,
        });
        
        // Count successful deployments
        let deployments = self.deployments.lock().await;
        let app_deployments: Vec<_> = deployments.values()
            .filter(|d| d.application_id == application.name_any())
            .collect();

        let deployed_count = app_deployments.iter()
            .filter(|d| matches!(d.status, DeploymentPhase::Deployed))
            .count();

        let failed_count = app_deployments.iter()
            .filter(|d| matches!(d.status, DeploymentPhase::Failed))
            .count();

        let total_count = app_deployments.len();

        // Update phase based on deployment results
        if deployed_count == total_count && total_count > 0 {
            status.phase = ApplicationPhase::Running;
        } else if deployed_count > 0 {
            status.phase = ApplicationPhase::PartiallyRunning;
        } else if failed_count > 0 {
            status.phase = ApplicationPhase::Failed;
        } else {
            status.phase = ApplicationPhase::Deploying;
        }

        // Update statistics
        if let Some(stats) = &mut status.statistics {
            stats.deployed_devices = deployed_count as u32;
            stats.failed_devices = failed_count as u32;
            stats.running_devices = deployed_count as u32;
        }

        status.last_updated = Some(chrono::Utc::now().to_rfc3339());

        self.update_application_status_with_status(application, status).await
    }

    async fn update_application_status_with_status(&self, application: &Application, status: ApplicationStatus) -> Result<(), String> {
        let patch = serde_json::json!({
            "status": status
        });

        let params = PatchParams::apply("wasmbed-qemu-deployment-service");
        let patch = Patch::Apply(patch);

        self.applications
            .patch_status(&application.name_any(), &params, &patch)
            .await
            .map_err(|e| format!("Failed to update application status: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmbed_k8s_resource::{ApplicationSpec, TargetDevices};

    #[tokio::test]
    async fn test_deployment_service() {
        // This test uses a real Kubernetes client
        // For now, just test the basic structure
        let qemu_manager = QemuManager::new("qemu-system-aarch64".to_string(), 30000);
        // Note: We can't create a real QemuDeploymentService without a Kubernetes client
        // This would need to be tested in integration tests
    }
}
