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
use k8s_openapi::api::apps::v1::Deployment;
use k8s_openapi::api::core::v1::Service;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use futures_util::StreamExt;
use std::time::Duration;
use tracing::{error, info, warn};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ControllerError {
    #[error("Kubernetes API error: {0}")]
    KubeError(#[from] kube::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Gateway error: {0}")]
    GatewayError(String),
    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
}

/// Gateway Controller
#[derive(Clone)]
pub struct GatewayController {
    client: Client,
    gateways: Api<wasmbed_k8s_resource::Gateway>,
    devices: Api<wasmbed_k8s_resource::Device>,
    deployments: Api<Deployment>,
    services: Api<Service>,
}

impl GatewayController {
    pub fn new(client: Client) -> Self {
        Self {
            gateways: Api::<wasmbed_k8s_resource::Gateway>::namespaced(client.clone(), "wasmbed"),
            devices: Api::<wasmbed_k8s_resource::Device>::namespaced(client.clone(), "wasmbed"),
            deployments: Api::<Deployment>::namespaced(client.clone(), "wasmbed"),
            services: Api::<Service>::namespaced(client.clone(), "wasmbed"),
            client,
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let gateways_api = Api::<wasmbed_k8s_resource::Gateway>::namespaced(self.client.clone(), "wasmbed");
        let controller = self.clone();
        
        Controller::new(gateways_api, watcher::Config::default())
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

    pub async fn reconcile(&self, gateway: wasmbed_k8s_resource::Gateway) -> Result<Action, ControllerError> {
        let name = gateway.name_any();
        info!("Reconciling gateway: {}", name);

        // Get current status or initialize
        let current_status = self.get_gateway_status(&gateway).await?;
        
        match current_status.phase {
            wasmbed_k8s_resource::GatewayPhase::Pending => {
                self.handle_pending(&gateway).await?;
            }
            wasmbed_k8s_resource::GatewayPhase::Running => {
                self.handle_running(&gateway).await?;
            }
            wasmbed_k8s_resource::GatewayPhase::Failed => {
                self.handle_failed(&gateway).await?;
            }
            wasmbed_k8s_resource::GatewayPhase::Stopped => {
                self.handle_stopped(&gateway).await?;
            }
        }

        Ok(Action::requeue(Duration::from_secs(30)))
    }

    async fn get_gateway_status(&self, gateway: &wasmbed_k8s_resource::Gateway) -> Result<wasmbed_k8s_resource::GatewayStatus, ControllerError> {
        // Get status from gateway or return default
        if let Some(status) = &gateway.status {
            Ok(status.clone())
        } else {
            Ok(wasmbed_k8s_resource::GatewayStatus {
                phase: wasmbed_k8s_resource::GatewayPhase::Pending,
                connected_devices: Some(0),
                enrolled_devices: Some(0),
                last_heartbeat: Some(chrono::Utc::now().to_rfc3339()),
                conditions: None,
            })
        }
    }

    async fn handle_pending(&self, gateway: &wasmbed_k8s_resource::Gateway) -> Result<(), ControllerError> {
        info!("Handling pending gateway: {}", gateway.name_any());
        
        // Create Deployment for the gateway
        self.create_gateway_deployment(gateway).await?;
        
        // Create Service for the gateway
        self.create_gateway_service(gateway).await?;
        
        // Count connected devices
        let devices = self.devices.list(&ListParams::default()).await?;
        let connected_count = devices.items.iter()
            .filter(|device| {
                if let Some(status) = &device.status {
                    matches!(status.phase, wasmbed_k8s_resource::DevicePhase::Connected)
                } else {
                    false
                }
            })
            .count();

        let status = wasmbed_k8s_resource::GatewayStatus {
            phase: wasmbed_k8s_resource::GatewayPhase::Running,
            connected_devices: Some(connected_count as i32),
            enrolled_devices: Some(devices.items.len() as i32),
            last_heartbeat: Some(chrono::Utc::now().to_rfc3339()),
            conditions: Some(vec![wasmbed_k8s_resource::GatewayCondition {
                r#type: "Ready".to_string(),
                status: "True".to_string(),
                reason: Some("GatewayStarted".to_string()),
                message: Some("Gateway is running and accepting connections".to_string()),
                last_transition_time: Some(chrono::Utc::now().to_rfc3339()),
            }]),
        };

        info!("Updating gateway status for {}", gateway.name_any());
        self.update_gateway_status(gateway, status).await?;
        info!("Gateway {} moved to running phase", gateway.name_any());
        Ok(())
    }

    async fn handle_running(&self, gateway: &wasmbed_k8s_resource::Gateway) -> Result<(), ControllerError> {
        info!("Gateway {} is running", gateway.name_any());
        
        // Update device counts
        let devices = self.devices.list(&ListParams::default()).await?;
        let connected_count = devices.items.iter()
            .filter(|device| {
                if let Some(status) = &device.status {
                    matches!(status.phase, wasmbed_k8s_resource::DevicePhase::Connected)
                } else {
                    false
                }
            })
            .count();

        let enrolled_count = devices.items.iter()
            .filter(|device| {
                if let Some(status) = &device.status {
                    matches!(status.phase, wasmbed_k8s_resource::DevicePhase::Enrolled | wasmbed_k8s_resource::DevicePhase::Connected)
                } else {
                    false
                }
            })
            .count();

        let status = wasmbed_k8s_resource::GatewayStatus {
            phase: wasmbed_k8s_resource::GatewayPhase::Running,
            connected_devices: Some(connected_count as i32),
            enrolled_devices: Some(enrolled_count as i32),
            last_heartbeat: Some(chrono::Utc::now().to_rfc3339()),
            conditions: Some(vec![wasmbed_k8s_resource::GatewayCondition {
                r#type: "Ready".to_string(),
                status: "True".to_string(),
                reason: Some("GatewayRunning".to_string()),
                message: Some(format!("Gateway is running with {} connected devices", connected_count)),
                last_transition_time: Some(chrono::Utc::now().to_rfc3339()),
            }]),
        };
        
        self.update_gateway_status(gateway, status).await?;
        Ok(())
    }

    async fn handle_failed(&self, gateway: &wasmbed_k8s_resource::Gateway) -> Result<(), ControllerError> {
        warn!("Gateway {} is in failed state", gateway.name_any());
        Ok(())
    }

    async fn handle_stopped(&self, gateway: &wasmbed_k8s_resource::Gateway) -> Result<(), ControllerError> {
        info!("Gateway {} is stopped", gateway.name_any());
        Ok(())
    }

    async fn update_gateway_status(&self, gateway: &wasmbed_k8s_resource::Gateway, status: wasmbed_k8s_resource::GatewayStatus) -> Result<(), ControllerError> {
        info!("Attempting to update status for gateway {}", gateway.name_any());
        
        let patch = serde_json::json!({
            "status": status
        });

        let params = PatchParams::apply("wasmbed-gateway-controller");
        let patch = Patch::Merge(patch);
        
        // Try patch_status first, fallback to regular patch
        match self.gateways
            .patch_status(&gateway.name_any(), &params, &patch)
            .await {
            Ok(_) => {
                info!("Successfully updated gateway status using patch_status");
                Ok(())
            },
            Err(kube::Error::Api(kube::core::ErrorResponse { code: 404, .. })) => {
                info!("Status subresource not available, trying regular patch");
                // Status subresource not available, use regular patch
                self.gateways
                    .patch(&gateway.name_any(), &params, &patch)
                    .await?;
                info!("Successfully updated gateway status using regular patch");
                Ok(())
            }
            Err(e) => {
                error!("Failed to update gateway status: {}", e);
                Err(ControllerError::KubeError(e))
            },
        }
    }

    async fn create_gateway_deployment(&self, gateway: &wasmbed_k8s_resource::Gateway) -> Result<(), ControllerError> {
        let deployment_name = format!("{}-deployment", gateway.name_any());
        
        // Check if deployment already exists
        match self.deployments.get(&deployment_name).await {
            Ok(_) => {
                info!("Deployment {} already exists", deployment_name);
                return Ok(());
            }
            Err(kube::Error::Api(kube::core::ErrorResponse { code: 404, .. })) => {
                // Deployment doesn't exist, create it
            }
            Err(e) => return Err(ControllerError::KubeError(e)),
        }

        let deployment = Deployment {
            metadata: ObjectMeta {
                name: Some(deployment_name.clone()),
                namespace: Some("wasmbed".to_string()),
                labels: Some({
                    let mut labels = std::collections::BTreeMap::new();
                    labels.insert("app".to_string(), "wasmbed-gateway".to_string());
                    labels.insert("gateway".to_string(), gateway.name_any());
                    labels
                }),
                ..Default::default()
            },
            spec: Some(k8s_openapi::api::apps::v1::DeploymentSpec {
                replicas: Some(1),
                selector: k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector {
                    match_labels: Some({
                        let mut labels = std::collections::BTreeMap::new();
                        labels.insert("app".to_string(), "wasmbed-gateway".to_string());
                        labels.insert("gateway".to_string(), gateway.name_any());
                        labels
                    }),
                    ..Default::default()
                },
                template: k8s_openapi::api::core::v1::PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some({
                            let mut labels = std::collections::BTreeMap::new();
                            labels.insert("app".to_string(), "wasmbed-gateway".to_string());
                            labels.insert("gateway".to_string(), gateway.name_any());
                            labels
                        }),
                        ..Default::default()
                    }),
                    spec: Some(k8s_openapi::api::core::v1::PodSpec {
                        containers: vec![k8s_openapi::api::core::v1::Container {
                            name: "gateway".to_string(),
                            image: Some("nginx:alpine".to_string()), // Using nginx for testing
                            ports: Some(vec![
                                k8s_openapi::api::core::v1::ContainerPort {
                                    container_port: 8080,
                                    name: Some("http".to_string()),
                                    ..Default::default()
                                },
                                k8s_openapi::api::core::v1::ContainerPort {
                                    container_port: 8443,
                                    name: Some("https".to_string()),
                                    ..Default::default()
                                },
                            ]),
                            env: Some(vec![
                                k8s_openapi::api::core::v1::EnvVar {
                                    name: "GATEWAY_NAME".to_string(),
                                    value: Some(gateway.name_any()),
                                    ..Default::default()
                                },
                                k8s_openapi::api::core::v1::EnvVar {
                                    name: "GATEWAY_ENDPOINT".to_string(),
                                    value: Some(gateway.spec.endpoint.clone()),
                                    ..Default::default()
                                },
                            ]),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                },
                ..Default::default()
            }),
            ..Default::default()
        };

        let params = PostParams::default();
        self.deployments.create(&params, &deployment).await?;
        info!("Created deployment for gateway: {}", gateway.name_any());
        Ok(())
    }

    async fn create_gateway_service(&self, gateway: &wasmbed_k8s_resource::Gateway) -> Result<(), ControllerError> {
        let service_name = format!("{}-service", gateway.name_any());
        
        // Check if service already exists
        match self.services.get(&service_name).await {
            Ok(_) => {
                info!("Service {} already exists", service_name);
                return Ok(());
            }
            Err(kube::Error::Api(kube::core::ErrorResponse { code: 404, .. })) => {
                // Service doesn't exist, create it
            }
            Err(e) => return Err(ControllerError::KubeError(e)),
        }

        let service = Service {
            metadata: ObjectMeta {
                name: Some(service_name.clone()),
                namespace: Some("wasmbed".to_string()),
                labels: Some({
                    let mut labels = std::collections::BTreeMap::new();
                    labels.insert("app".to_string(), "wasmbed-gateway".to_string());
                    labels.insert("gateway".to_string(), gateway.name_any());
                    labels
                }),
                ..Default::default()
            },
            spec: Some(k8s_openapi::api::core::v1::ServiceSpec {
                selector: Some({
                    let mut labels = std::collections::BTreeMap::new();
                    labels.insert("app".to_string(), "wasmbed-gateway".to_string());
                    labels.insert("gateway".to_string(), gateway.name_any());
                    labels
                }),
                ports: Some(vec![
                    k8s_openapi::api::core::v1::ServicePort {
                        name: Some("http".to_string()),
                        port: 8080,
                        target_port: Some(k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(8080)),
                        ..Default::default()
                    },
                    k8s_openapi::api::core::v1::ServicePort {
                        name: Some("https".to_string()),
                        port: 8443,
                        target_port: Some(k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(8443)),
                        ..Default::default()
                    },
                ]),
                type_: Some("ClusterIP".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let params = PostParams::default();
        self.services.create(&params, &service).await?;
        info!("Created service for gateway: {}", gateway.name_any());
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let client = Client::try_default().await?;
    let controller = GatewayController::new(client);
    
    info!("Starting Gateway Controller...");
    controller.run().await?;
    
    Ok(())
}