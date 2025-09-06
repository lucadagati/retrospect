// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::time::Duration;

use anyhow::Result;
use log::{error, info};
use wasmbed_protocol::{ClientMessage, ServerMessage, DeviceUuid};

use crate::wasm_runtime::{WasmRuntime, WasmRuntimeConfig};
use crate::application_manager::ApplicationManager;
use crate::wifi_manager::{WifiManager, WifiConfig};
use crate::monitoring::MonitoringSystem;
use crate::wasmbed_client::WasmbedClient;
use crate::security::SecurityManager;

mod wasm_runtime;
mod wasmbed_client;
mod security;
mod wifi_manager;
mod application_manager;
mod monitoring;

/// Main firmware for ESP32 devices
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    info!(" Starting Wasmbed ESP32 Firmware");
    
    // Create device UUID
    let device_uuid = DeviceUuid::new();
    info!("Device UUID: {}", device_uuid);
    
    // Initialize components
    let runtime_config = WasmRuntimeConfig {
        max_memory_per_app: 1024 * 1024, // 1MB
        max_concurrent_apps: 4,
        default_timeout: Duration::from_secs(30),
        max_stack_size: 64 * 1024, // 64KB
    };
    
    let runtime = WasmRuntime::new(runtime_config)?;
    let mut app_manager = ApplicationManager::new(runtime);
    let mut monitoring_system = MonitoringSystem::new();
    
    // Initialize WiFi
    let wifi_config = WifiConfig::default();
    let mut wifi_manager = WifiManager::new(wifi_config);
    
    // Connect to WiFi
    info!("Connecting to WiFi...");
    wifi_manager.connect().await?;
    
    if !wifi_manager.is_connected() {
        error!("Failed to connect to WiFi");
        return Err(anyhow::anyhow!("WiFi connection failed"));
    }
    
    info!("WiFi connected successfully");
    
    // Initialize Wasmbed client
    let mut client = WasmbedClient::new(
        device_uuid,
        "localhost".to_string(), // TODO: Get from configuration
        8443,
        wifi_config,
    )?;
    
    // Connect to gateway
    info!("Connecting to gateway...");
    client.connect().await?;
    
    if !client.is_connected() {
        error!("Failed to connect to gateway");
        return Err(anyhow::anyhow!("Gateway connection failed"));
    }
    
    info!("Gateway connected successfully");
    
    // Register device with gateway
    info!("Registering device with gateway...");
    client.register_device().await?;
    
    info!("Device registered successfully");
    
    // Main application loop
    info!("Starting main application loop");
    run_main_loop(&mut app_manager, &mut monitoring_system, &mut client).await?;
    
    Ok(())
}

/// Main application loop
async fn run_main_loop(
    app_manager: &mut ApplicationManager,
    monitoring_system: &mut MonitoringSystem,
    client: &mut WasmbedClient,
) -> Result<()> {
    let mut loop_count = 0;
    
    loop {
        loop_count += 1;
        
        // Collect metrics
        monitoring_system.collect_metrics(app_manager);
        
        // Send heartbeat
        if let Err(e) = client.send_heartbeat().await {
            error!("Failed to send heartbeat: {}", e);
        }
        
        // Check application health
        app_manager.check_application_health();
        
        // Send application status updates
        let app_statuses = app_manager.get_all_application_statuses();
        for (app_id, status) in app_statuses {
            if let Err(e) = client.send_application_status(&app_id, status).await {
                error!("Failed to send application status for {}: {}", app_id, e);
            }
        }
        
        // Log status every 10 iterations
        if loop_count % 10 == 0 {
            let summary = monitoring_system.get_summary();
            info!("Status: {} total apps, {} running, health score: {}", 
                  summary.total_applications, 
                  summary.running_applications, 
                  summary.system_health_score);
        }
        
        // Sleep for 5 seconds
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

/// Test function for development
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_uuid_creation() {
        let device_uuid = DeviceUuid::new();
        assert!(!device_uuid.to_string().is_empty());
    }

    #[test]
    fn test_runtime_creation() {
        let config = WasmRuntimeConfig {
            max_memory_per_app: 1024 * 1024,
            max_concurrent_apps: 4,
            default_timeout: Duration::from_secs(30),
            max_stack_size: 64 * 1024,
        };
        
        let runtime = WasmRuntime::new(config);
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_wifi_config_default() {
        let config = WifiConfig::default();
        assert_eq!(config.ssid, "WasmbedNetwork");
        assert_eq!(config.password, "wasmbed123");
    }

    #[tokio::test]
    async fn test_wifi_connection() {
        let config = WifiConfig::default();
        let mut wifi_manager = WifiManager::new(config);
        
        let result = wifi_manager.connect().await;
        assert!(result.is_ok());
        assert!(wifi_manager.is_connected());
    }
}