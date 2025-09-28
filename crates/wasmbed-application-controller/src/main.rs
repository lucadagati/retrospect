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
    #[error("Application error: {0}")]
    ApplicationError(String),
    #[error("Anyhow error: {0}")]
    AnyhowError(#[from] anyhow::Error),
}

/// Application Controller
#[derive(Clone)]
pub struct ApplicationController {
    client: Client,
    applications: Api<wasmbed_k8s_resource::Application>,
    devices: Api<wasmbed_k8s_resource::Device>,
    gateways: Api<wasmbed_k8s_resource::Gateway>,
    deployments: Api<Deployment>,
}

impl ApplicationController {
    pub fn new(client: Client) -> Self {
        Self {
            applications: Api::<wasmbed_k8s_resource::Application>::namespaced(client.clone(), "wasmbed"),
            devices: Api::<wasmbed_k8s_resource::Device>::namespaced(client.clone(), "wasmbed"),
            gateways: Api::<wasmbed_k8s_resource::Gateway>::namespaced(client.clone(), "wasmbed"),
            deployments: Api::<Deployment>::namespaced(client.clone(), "wasmbed"),
            client,
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let applications_api = Api::<wasmbed_k8s_resource::Application>::namespaced(self.client.clone(), "wasmbed");
        let controller = self.clone();
        
        Controller::new(applications_api, watcher::Config::default())
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

    pub async fn reconcile(&self, application: wasmbed_k8s_resource::Application) -> Result<Action, ControllerError> {
        let name = application.name_any();
        info!("Reconciling application: {}", name);

        // Get current status or initialize
        let current_status = self.get_application_status(&application).await?;
        
        match current_status.phase {
            wasmbed_k8s_resource::ApplicationPhase::Creating => {
                self.handle_creating(&application).await?;
            }
            wasmbed_k8s_resource::ApplicationPhase::Deploying => {
                self.handle_deploying(&application).await?;
            }
            wasmbed_k8s_resource::ApplicationPhase::Running => {
                self.handle_running(&application).await?;
            }
            wasmbed_k8s_resource::ApplicationPhase::PartiallyRunning => {
                self.handle_partially_running(&application).await?;
            }
            wasmbed_k8s_resource::ApplicationPhase::Failed => {
                self.handle_failed(&application).await?;
            }
            wasmbed_k8s_resource::ApplicationPhase::Stopping => {
                self.handle_stopping(&application).await?;
            }
            wasmbed_k8s_resource::ApplicationPhase::Stopped => {
                self.handle_stopped(&application).await?;
            }
            wasmbed_k8s_resource::ApplicationPhase::Deleting => {
                self.handle_deleting(&application).await?;
            }
        }

        Ok(Action::requeue(Duration::from_secs(30)))
    }

    async fn get_application_status(&self, application: &wasmbed_k8s_resource::Application) -> Result<wasmbed_k8s_resource::ApplicationStatus, ControllerError> {
        // For now, return a default status since we don't have status field access
        Ok(wasmbed_k8s_resource::ApplicationStatus {
            phase: wasmbed_k8s_resource::ApplicationPhase::Creating,
            device_statuses: None,
            statistics: None,
            last_updated: Some(chrono::Utc::now().to_rfc3339()),
            error: None,
        })
    }

    async fn handle_creating(&self, application: &wasmbed_k8s_resource::Application) -> Result<(), ControllerError> {
        info!("Handling creating application: {}", application.name_any());
        
        // Find target devices
        let target_devices = self.find_target_devices(&application.spec.target_devices).await?;
        
        if target_devices.is_empty() {
            warn!("No target devices found for application: {}", application.name_any());
            return Ok(());
        }

        // Update status to deploying
        let status = wasmbed_k8s_resource::ApplicationStatus {
            phase: wasmbed_k8s_resource::ApplicationPhase::Deploying,
            device_statuses: None,
            statistics: Some(wasmbed_k8s_resource::ApplicationStatistics {
                total_devices: target_devices.len() as u32,
                deployed_devices: 0,
                running_devices: 0,
                failed_devices: 0,
                stopped_devices: 0,
            }),
            last_updated: Some(chrono::Utc::now().to_rfc3339()),
            error: None,
        };

        self.update_application_status(application, status).await?;
        info!("Application {} moved to deploying phase", application.name_any());
        Ok(())
    }

    async fn handle_deploying(&self, application: &wasmbed_k8s_resource::Application) -> Result<(), ControllerError> {
        info!("Handling deploying application: {}", application.name_any());
        
        // Create Deployment for the application
        self.create_application_deployment(application).await?;
        
        // Find target devices again
        let target_devices = self.find_target_devices(&application.spec.target_devices).await?;
        
        let mut device_statuses = std::collections::BTreeMap::new();
        let mut running_count = 0;
        
        for device in &target_devices {
            let device_status = wasmbed_k8s_resource::DeviceApplicationStatus {
                status: wasmbed_k8s_resource::DeviceApplicationPhase::Running,
                last_heartbeat: Some(chrono::Utc::now().to_rfc3339()),
                metrics: None,
                error: None,
                restart_count: 0,
            };
            device_statuses.insert(device.name_any(), device_status);
            running_count += 1;
        }

        let status = wasmbed_k8s_resource::ApplicationStatus {
            phase: wasmbed_k8s_resource::ApplicationPhase::Running,
            device_statuses: Some(device_statuses),
            statistics: Some(wasmbed_k8s_resource::ApplicationStatistics {
                total_devices: target_devices.len() as u32,
                deployed_devices: target_devices.len() as u32,
                running_devices: running_count,
                failed_devices: 0,
                stopped_devices: 0,
            }),
            last_updated: Some(chrono::Utc::now().to_rfc3339()),
            error: None,
        };

        self.update_application_status(application, status).await?;
        info!("Application {} deployed successfully", application.name_any());
        Ok(())
    }

    async fn handle_running(&self, application: &wasmbed_k8s_resource::Application) -> Result<(), ControllerError> {
        info!("Application {} is running", application.name_any());
        
        // Update heartbeat timestamps
        let mut device_statuses = std::collections::BTreeMap::new();
        let target_devices = self.find_target_devices(&application.spec.target_devices).await?;
        
        for device in &target_devices {
            let device_status = wasmbed_k8s_resource::DeviceApplicationStatus {
                status: wasmbed_k8s_resource::DeviceApplicationPhase::Running,
                last_heartbeat: Some(chrono::Utc::now().to_rfc3339()),
                metrics: None,
                error: None,
                restart_count: 0,
            };
            device_statuses.insert(device.name_any(), device_status);
        }
        
        let status = wasmbed_k8s_resource::ApplicationStatus {
            phase: wasmbed_k8s_resource::ApplicationPhase::Running,
            device_statuses: Some(device_statuses),
            statistics: Some(wasmbed_k8s_resource::ApplicationStatistics {
                total_devices: target_devices.len() as u32,
                deployed_devices: target_devices.len() as u32,
                running_devices: target_devices.len() as u32,
                failed_devices: 0,
                stopped_devices: 0,
            }),
            last_updated: Some(chrono::Utc::now().to_rfc3339()),
            error: None,
        };
        
        self.update_application_status(application, status).await?;
        Ok(())
    }

    async fn handle_partially_running(&self, application: &wasmbed_k8s_resource::Application) -> Result<(), ControllerError> {
        info!("Application {} is partially running", application.name_any());
        Ok(())
    }

    async fn handle_failed(&self, application: &wasmbed_k8s_resource::Application) -> Result<(), ControllerError> {
        warn!("Application {} is in failed state", application.name_any());
        Ok(())
    }

    async fn handle_stopping(&self, application: &wasmbed_k8s_resource::Application) -> Result<(), ControllerError> {
        info!("Application {} is stopping", application.name_any());
        
        // Update status to stopped
        let status = wasmbed_k8s_resource::ApplicationStatus {
            phase: wasmbed_k8s_resource::ApplicationPhase::Stopped,
            device_statuses: None,
            statistics: None,
            last_updated: Some(chrono::Utc::now().to_rfc3339()),
            error: None,
        };
        
        self.update_application_status(application, status).await?;
        Ok(())
    }

    async fn handle_stopped(&self, application: &wasmbed_k8s_resource::Application) -> Result<(), ControllerError> {
        info!("Application {} is stopped", application.name_any());
        Ok(())
    }

    async fn handle_deleting(&self, application: &wasmbed_k8s_resource::Application) -> Result<(), ControllerError> {
        info!("Application {} is being deleted", application.name_any());
        Ok(())
    }

    async fn find_target_devices(&self, target_devices: &wasmbed_k8s_resource::TargetDevices) -> Result<Vec<wasmbed_k8s_resource::Device>, ControllerError> {
        let all_devices = self.devices.list(&ListParams::default()).await?;
        let mut matching_devices = vec![];

        // If all_devices is true, return all devices
        if target_devices.all_devices.unwrap_or(false) {
            return Ok(all_devices.items);
        }

        // If device_names is specified, match by name
        if let Some(device_names) = &target_devices.device_names {
            for device in all_devices.items {
                if device_names.contains(&device.name_any()) {
                    matching_devices.push(device);
                }
            }
        }

        Ok(matching_devices)
    }

    async fn update_application_status(&self, application: &wasmbed_k8s_resource::Application, status: wasmbed_k8s_resource::ApplicationStatus) -> Result<(), ControllerError> {
        let patch = serde_json::json!({
            "status": status
        });

        let params = PatchParams::apply("wasmbed-application-controller");
        let patch = Patch::Merge(patch);
        
        // Try patch_status first, fallback to patch if status doesn't exist
        match self.applications.patch_status(&application.name_any(), &params, &patch).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // If patch_status fails, try regular patch
                warn!("patch_status failed, trying regular patch: {}", e);
                self.applications
                    .patch(&application.name_any(), &params, &patch)
                    .await?;
                Ok(())
            }
        }
    }

    async fn create_application_deployment(&self, application: &wasmbed_k8s_resource::Application) -> Result<(), ControllerError> {
        let deployment_name = format!("{}-deployment", application.name_any());
        
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
                    labels.insert("app".to_string(), "wasmbed-application".to_string());
                    labels.insert("application".to_string(), application.name_any());
                    labels
                }),
                ..Default::default()
            },
            spec: Some(k8s_openapi::api::apps::v1::DeploymentSpec {
                replicas: Some(1),
                selector: k8s_openapi::apimachinery::pkg::apis::meta::v1::LabelSelector {
                    match_labels: Some({
                        let mut labels = std::collections::BTreeMap::new();
                        labels.insert("app".to_string(), "wasmbed-application".to_string());
                        labels.insert("application".to_string(), application.name_any());
                        labels
                    }),
                    ..Default::default()
                },
                template: k8s_openapi::api::core::v1::PodTemplateSpec {
                    metadata: Some(ObjectMeta {
                        labels: Some({
                            let mut labels = std::collections::BTreeMap::new();
                            labels.insert("app".to_string(), "wasmbed-application".to_string());
                            labels.insert("application".to_string(), application.name_any());
                            labels
                        }),
                        ..Default::default()
                    }),
                    spec: Some(k8s_openapi::api::core::v1::PodSpec {
                        containers: vec![k8s_openapi::api::core::v1::Container {
                            name: "application".to_string(),
                            image: Some("wasmbed/application:latest".to_string()),
                            env: Some(vec![
                                k8s_openapi::api::core::v1::EnvVar {
                                    name: "APPLICATION_NAME".to_string(),
                                    value: Some(application.name_any()),
                                    ..Default::default()
                                },
                                k8s_openapi::api::core::v1::EnvVar {
                                    name: "WASM_BYTES".to_string(),
                                    value: Some(application.spec.wasm_bytes.clone()),
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
        info!("Created deployment for application: {}", application.name_any());
        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    let client = Client::try_default().await?;
    let controller = ApplicationController::new(client);
    
    info!("Starting Application Controller...");
    controller.run().await?;
    
    Ok(())
}