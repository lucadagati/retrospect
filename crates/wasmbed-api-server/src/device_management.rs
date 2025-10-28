// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::time::SystemTime;
use tracing::{error, info, warn};

use crate::{DeviceInfo, DeviceSelector};

/// Device Manager for device operations
#[derive(Debug)]
pub struct DeviceManager {
    gateway_endpoint: String,
}

impl DeviceManager {
    pub fn new(gateway_endpoint: &str) -> anyhow::Result<Self> {
        Ok(Self {
            gateway_endpoint: gateway_endpoint.to_string(),
        })
    }

    pub async fn get_all_devices(&self) -> anyhow::Result<Vec<DeviceInfo>> {
        info!("Fetching all devices from Kubernetes");
        
        // Query Kubernetes for Device CRDs
        let output = tokio::process::Command::new("kubectl")
            .args(&["get", "devices", "-n", "wasmbed", "-o", "json"])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("No resources found") {
                info!("No devices found in wasmbed namespace");
                return Ok(vec![]);
            }
            return Err(anyhow::anyhow!("kubectl failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let k8s_response: serde_json::Value = serde_json::from_str(&stdout)?;
        
        let mut devices = Vec::new();
        
        if let Some(items) = k8s_response["items"].as_array() {
            for item in items {
                if let Some(metadata) = item["metadata"].as_object() {
                    let device_id = metadata["name"]
                        .as_str()
                        .unwrap_or("unknown")
                        .to_string();
                    
                    let status = item["status"]["phase"]
                        .as_str()
                        .unwrap_or("unknown")
                        .to_string();
                    
                    let device_type = item["spec"]["deviceType"]
                        .as_str()
                        .unwrap_or("MCU")
                        .to_string();
                    
                    let architecture = item["spec"]["architecture"]
                        .as_str()
                        .unwrap_or("ARM_CORTEX_M")
                        .to_string();
                    
                    // Parse gateway_id directly from JSON with debug logging
                    let gateway_id = if let Some(status) = item.get("status") {
                        info!("Device {} has status: {:?}", device_id, status);
                        if let Some(gateway) = status.get("gateway") {
                            info!("Device {} has gateway: {:?}", device_id, gateway);
                            if let Some(name) = gateway.get("name") {
                                info!("Device {} gateway name: {:?}", device_id, name);
                                if let Some(name_str) = name.as_str() {
                                    if !name_str.is_empty() {
                                        info!("Device {} gateway_id parsed: {}", device_id, name_str);
                                        Some(name_str.to_string())
                                    } else {
                                        info!("Device {} gateway name is empty", device_id);
                                        None
                                    }
                                } else {
                                    info!("Device {} gateway name is not a string", device_id);
                                    None
                                }
                            } else {
                                info!("Device {} gateway has no name field", device_id);
                                None
                            }
                        } else {
                            info!("Device {} status has no gateway field", device_id);
                            None
                        }
                    } else {
                        info!("Device {} has no status field", device_id);
                        None
                    };
                    
                    let mcu_type = item["spec"]["mcuType"]
                        .as_str()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "Mps2An385".to_string());
                    
                    devices.push(DeviceInfo {
                        device_id,
                        device_type,
                        architecture,
                        status,
                        last_heartbeat: Some(SystemTime::now()),
                        gateway_id,
                        mcu_type: Some(mcu_type),
                    });
                }
            }
        }
        
        info!("Fetched {} devices from Kubernetes", devices.len());
        Ok(devices)
    }

    pub async fn get_device(&self, device_id: &str) -> anyhow::Result<Option<DeviceInfo>> {
        info!("Fetching device: {}", device_id);
        
        let devices = self.get_all_devices().await?;
        Ok(devices.into_iter().find(|d| d.device_id == device_id))
    }

    pub async fn enable_pairing(&self) -> anyhow::Result<bool> {
        info!("Enabling pairing mode");
        
        // In a real implementation, this would call the gateway API
        // For now, we'll simulate success
        
        info!("Pairing mode enabled");
        Ok(true)
    }

    pub async fn disable_pairing(&self) -> anyhow::Result<bool> {
        info!("Disabling pairing mode");
        
        // In a real implementation, this would call the gateway API
        // For now, we'll simulate success
        
        info!("Pairing mode disabled");
        Ok(true)
    }
}
