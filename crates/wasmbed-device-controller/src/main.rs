// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use kube::{
    api::{Api, Patch, PatchParams},
    client::Client,
    runtime::{
        controller::{Action, Controller},
        watcher,
    },
    ResourceExt,
};
use futures_util::StreamExt;
use std::time::Duration;
use tracing::{error, info, warn};
use wasmbed_k8s_resource::{Device, DeviceStatus, DevicePhase};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] kube::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Device error: {0}")]
    DeviceError(String),
}

/// Device Controller
#[derive(Clone)]
pub struct DeviceController {
    client: Client,
    devices: Api<Device>,
}

impl DeviceController {
    pub fn new(client: Client) -> Self {
        Self {
            devices: Api::<Device>::namespaced(client.clone(), "wasmbed"),
            client,
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let devices_api = Api::<Device>::namespaced(self.client.clone(), "wasmbed");
        let controller = self.clone();
        
        Controller::new(devices_api, watcher::Config::default())
            .shutdown_on_signal()
            .run(
                move |obj, _ctx| {
                    let controller = controller.clone();
                    async move {
                        controller.reconcile((*obj).clone()).await
                    }
                },
                move |_obj, _err, _ctx| {
                    Action::requeue(Duration::from_secs(30))
                },
                std::sync::Arc::new(()),
            )
            .for_each(|res| async move {
                match res {
                    Ok(o) => info!("reconciled {:?}", o),
                    Err(e) => error!("reconcile failed: {}", e),
                }
            })
            .await;

        Ok(())
    }
}

impl DeviceController {
    pub async fn reconcile(&self, device: Device) -> Result<Action, ControllerError> {
        let name = device.name_any();
        info!("Reconciling device: {}", name);

        match device.status.as_ref().map(|s| &s.phase) {
            Some(DevicePhase::Pending) => {
                self.handle_pending(&device).await?;
            }
            Some(DevicePhase::Enrolling) => {
                self.handle_enrollment(&device).await?;
            }
            Some(DevicePhase::Enrolled) => {
                self.handle_enrolled(&device).await?;
            }
            Some(DevicePhase::Connected) => {
                self.handle_connected(&device).await?;
            }
            Some(DevicePhase::Disconnected) => {
                self.handle_disconnected(&device).await?;
            }
            Some(DevicePhase::Unreachable) => {
                self.handle_unreachable(&device).await?;
            }
            None => {
                // Initialize device status
                self.initialize_device(&device).await?;
            }
        }

        Ok(Action::requeue(Duration::from_secs(30)))
    }
}

impl DeviceController {
    async fn handle_pending(&self, device: &Device) -> Result<(), ControllerError> {
        info!("Handling pending device: {}", device.name_any());
        
        // Move to enrolling phase
        let mut status = device.status.clone().unwrap_or_else(|| DeviceStatus {
            phase: DevicePhase::Pending,
            gateway: None,
            connected_since: None,
            last_heartbeat: None,
            pairing_mode: false,
        });
        
        status.phase = DevicePhase::Enrolling;
        self.update_device_status(device, status).await?;
        
        Ok(())
    }

    async fn initialize_device(&self, device: &Device) -> Result<(), ControllerError> {
        let status = DeviceStatus {
            phase: DevicePhase::Pending,
            gateway: None,
            connected_since: None,
            last_heartbeat: None,
            pairing_mode: false,
        };

        self.update_device_status(device, status).await?;
        info!("Initialized device: {}", device.name_any());
        Ok(())
    }

    async fn handle_enrollment(&self, device: &Device) -> Result<(), ControllerError> {
        info!("Handling enrollment for device: {}", device.name_any());
        
        // In a real implementation, this would communicate with the gateway
        // For now, we'll simulate the enrollment process
        tokio::time::sleep(Duration::from_secs(2)).await;

        let mut status = device.status.clone().unwrap_or_else(|| DeviceStatus {
            phase: DevicePhase::Enrolling,
            gateway: None,
            connected_since: None,
            last_heartbeat: None,
            pairing_mode: false,
        });

        status.phase = DevicePhase::Enrolled;
        status.pairing_mode = true;

        self.update_device_status(device, status).await?;
        info!("Device {} enrolled successfully", device.name_any());
        Ok(())
    }

    async fn handle_enrolled(&self, device: &Device) -> Result<(), ControllerError> {
        info!("Device {} is enrolled, waiting for connection", device.name_any());
        Ok(())
    }

    async fn handle_connected(&self, device: &Device) -> Result<(), ControllerError> {
        info!("Device {} is connected", device.name_any());
        
        // Update heartbeat
        let mut status = device.status.clone().unwrap();
        status.last_heartbeat = Some(chrono::Utc::now());
        
        self.update_device_status(device, status).await?;
        Ok(())
    }

    async fn handle_disconnected(&self, device: &Device) -> Result<(), ControllerError> {
        info!("Device {} disconnected", device.name_any());
        Ok(())
    }

    async fn handle_unreachable(&self, device: &Device) -> Result<(), ControllerError> {
        warn!("Device {} is unreachable", device.name_any());
        Ok(())
    }

    async fn update_device_status(&self, device: &Device, status: DeviceStatus) -> Result<(), ControllerError> {
        let patch = serde_json::json!({
            "status": status
        });

        let params = PatchParams::apply("wasmbed-device-controller");
        let patch = Patch::Merge(patch);
        
        // Try patch_status first, fallback to patch if status doesn't exist
        match self.devices.patch_status(&device.name_any(), &params, &patch).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // If patch_status fails, try regular patch
                warn!("patch_status failed, trying regular patch: {}", e);
                self.devices
                    .patch(&device.name_any(), &params, &patch)
                    .await?;
                Ok(())
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let client = Client::try_default().await?;
    let controller = DeviceController::new(client);
    
    info!("Starting Device Controller...");
    controller.run().await?;
    
    Ok(())
}
