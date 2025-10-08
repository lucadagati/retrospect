// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use kube::{
    api::{Api, ListParams, Patch, PatchParams, PostParams},
    client::Client,
    runtime::{
        controller::{Action, Controller},
        watcher,
    },
    ResourceExt,
};
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use futures_util::StreamExt;
use std::time::Duration;
use tracing::{error, info, warn};
use wasmbed_k8s_resource::{Device, DeviceStatus, DevicePhase, Gateway, GatewayStatus, GatewayPhase};
use wasmbed_types::GatewayReference;
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
    pods: Api<Pod>,
}

impl DeviceController {
    pub fn new(client: Client) -> Self {
        Self {
            devices: Api::<Device>::namespaced(client.clone(), "wasmbed"),
            pods: Api::<Pod>::namespaced(client.clone(), "wasmbed"),
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
        
        // Check if there are any active gateways
        let gateways_api = Api::<Gateway>::namespaced(self.client.clone(), "wasmbed");
        let gateways = gateways_api.list(&ListParams::default()).await?;
        
        let active_gateways: Vec<_> = gateways
            .items
            .iter()
            .filter(|g| g.status.as_ref()
                .map(|s| matches!(s.phase, GatewayPhase::Running))
                .unwrap_or(false))
            .collect();
        
        if active_gateways.is_empty() {
            info!("No active gateways found for device {}, keeping in enrolling state", device.name_any());
            // Keep device in enrolling state until a gateway becomes available
            return Ok(());
        }
        
        // Select the first available gateway
        let gateway = active_gateways[0];
        let gateway_name = gateway.name_any();
        
        info!("Found active gateway {} for device {}", gateway_name, device.name_any());
        
        // Simulate enrollment process with gateway
        tokio::time::sleep(Duration::from_secs(2)).await;

        let mut status = device.status.clone().unwrap_or_else(|| DeviceStatus {
            phase: DevicePhase::Enrolling,
            gateway: None,
            connected_since: None,
            last_heartbeat: None,
            pairing_mode: false,
        });

        status.phase = DevicePhase::Enrolled;
        status.gateway = Some(GatewayReference::new("wasmbed", &gateway_name));
        status.pairing_mode = true;

        self.update_device_status(device, status).await?;
        info!("Device {} enrolled successfully with gateway {}", device.name_any(), gateway_name);
        Ok(())
    }

    async fn handle_enrolled(&self, device: &Device) -> Result<(), ControllerError> {
        info!("Device {} is enrolled, creating device pod", device.name_any());
        
        // Create Pod for the device
        self.create_device_pod(device).await?;
        
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

    async fn create_device_pod(&self, device: &Device) -> Result<(), ControllerError> {
        let pod_name = format!("{}-pod", device.name_any());
        
        // Check if pod already exists
        match self.pods.get(&pod_name).await {
            Ok(_) => {
                info!("Pod {} already exists", pod_name);
                return Ok(());
            }
            Err(kube::Error::Api(kube::core::ErrorResponse { code: 404, .. })) => {
                // Pod doesn't exist, create it
            }
            Err(e) => return Err(ControllerError::KubeError(e)),
        }

        let pod = Pod {
            metadata: ObjectMeta {
                name: Some(pod_name.clone()),
                namespace: Some("wasmbed".to_string()),
                labels: Some({
                    let mut labels = std::collections::BTreeMap::new();
                    labels.insert("app".to_string(), "wasmbed-device".to_string());
                    labels.insert("device".to_string(), device.name_any());
                    labels
                }),
                ..Default::default()
            },
            spec: Some(k8s_openapi::api::core::v1::PodSpec {
                containers: vec![k8s_openapi::api::core::v1::Container {
                    name: "device-proxy".to_string(),
                    image: Some("nginx:alpine".to_string()), // Using nginx as a proxy for QEMU devices
                    env: Some(vec![
                        k8s_openapi::api::core::v1::EnvVar {
                            name: "DEVICE_NAME".to_string(),
                            value: Some(device.name_any()),
                            ..Default::default()
                        },
                        k8s_openapi::api::core::v1::EnvVar {
                            name: "DEVICE_PUBLIC_KEY".to_string(),
                            value: Some(device.spec.public_key.clone()),
                            ..Default::default()
                        },
                        k8s_openapi::api::core::v1::EnvVar {
                            name: "QEMU_ENDPOINT".to_string(),
                            value: Some(format!("127.0.0.1:{}", 30450 + device.name_any().len() as u16)),
                            ..Default::default()
                        },
                    ]),
                    ports: Some(vec![
                        k8s_openapi::api::core::v1::ContainerPort {
                            container_port: 8080,
                            name: Some("device-api".to_string()),
                            ..Default::default()
                        },
                    ]),
                    ..Default::default()
                }],
                restart_policy: Some("Always".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let params = PostParams::default();
        self.pods.create(&params, &pod).await?;
        info!("Created pod for device: {}", device.name_any());
        Ok(())
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
