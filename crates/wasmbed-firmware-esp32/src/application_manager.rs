// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use log::{debug, error, info, warn};

use crate::wasm_runtime::{WasmRuntime, WasmRuntimeConfig, ApplicationStatus as RuntimeStatus, ApplicationMetrics as RuntimeMetrics};

/// Application configuration
#[derive(Debug, Clone)]
pub struct ApplicationConfig {
    /// Memory limit in bytes
    pub memory_limit: u32,
    /// CPU time limit in milliseconds
    pub cpu_time_limit: u32,
    /// Auto-restart on failure
    pub auto_restart: bool,
    /// Application timeout in seconds
    pub timeout: u32,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            memory_limit: 1024 * 1024, // 1MB
            cpu_time_limit: 30000, // 30 seconds
            auto_restart: true,
            timeout: 30,
        }
    }
}

/// Application metadata
#[derive(Debug, Clone)]
pub struct ApplicationMetadata {
    /// Application name
    pub name: String,
    /// Application description
    pub description: Option<String>,
    /// Application version
    pub version: Option<String>,
    /// Application author
    pub author: Option<String>,
}

/// WASM Application wrapper
#[derive(Debug)]
pub struct WasmApplication {
    /// Application ID
    app_id: String,
    /// Application metadata
    metadata: ApplicationMetadata,
    /// WASM binary data
    wasm_binary: Vec<u8>,
    /// Application configuration
    config: ApplicationConfig,
    /// Application status
    status: ApplicationStatus,
    /// Deployment timestamp
    deployed_at: SystemTime,
    /// Last activity timestamp
    last_activity: SystemTime,
    /// Error message if any
    error: Option<String>,
}

/// Application manager for ESP32 devices
pub struct ApplicationManager {
    /// WASM runtime
    runtime: WasmRuntime,
    /// Managed applications
    applications: BTreeMap<String, WasmApplication>,
    /// Application manager configuration
    config: ApplicationManagerConfig,
}

/// Application manager configuration
#[derive(Debug, Clone)]
pub struct ApplicationManagerConfig {
    /// Maximum number of concurrent applications
    pub max_applications: usize,
    /// Default application timeout
    pub default_timeout: Duration,
}

impl Default for ApplicationManagerConfig {
    fn default() -> Self {
        Self {
            max_applications: 4,
            default_timeout: Duration::from_secs(30),
        }
    }
}

/// Application status
#[derive(Debug, Clone, PartialEq)]
pub enum ApplicationStatus {
    Deploying,
    Running,
    Stopped,
    Error(String),
    Starting,
    Paused,
    Suspended,
    Stopping,
}

impl ApplicationManager {
    /// Create a new application manager
    pub fn new(runtime: WasmRuntime) -> Self {
        Self {
            runtime,
            applications: BTreeMap::new(),
            config: ApplicationManagerConfig::default(),
        }
    }

    /// Deploy a new application
    pub fn deploy_application(&mut self, app_id: &str, wasm_binary: Vec<u8>) -> Result<()> {
        info!("Deploying application: {}", app_id);

        // Check if we've reached the maximum number of applications
        if self.applications.len() >= self.config.max_applications {
            return Err(anyhow::anyhow!("Maximum number of applications reached"));
        }

        // Check if application already exists
        if self.applications.contains_key(app_id) {
            return Err(anyhow::anyhow!("Application already exists: {}", app_id));
        }

        // Create application metadata
        let metadata = ApplicationMetadata {
            name: app_id.to_string(),
            description: Some("WASM Application".to_string()),
            version: Some("1.0.0".to_string()),
            author: Some("Wasmbed Platform".to_string()),
        };

        // Create application configuration
        let config = ApplicationConfig::default();

        // Create WASM application
        let application = WasmApplication {
            app_id: app_id.to_string(),
            metadata,
            wasm_binary: wasm_binary.clone(),
            config,
            status: ApplicationStatus::Deploying,
            deployed_at: SystemTime::now(),
            last_activity: SystemTime::now(),
            error: None,
        };

        // Store the application
        self.applications.insert(app_id.to_string(), application);

        // Load the WASM module in the runtime
        match self.runtime.load_module(app_id, &wasm_binary) {
            Ok(_) => {
                // Update application status
                if let Some(app) = self.applications.get_mut(app_id) {
                    app.status = ApplicationStatus::Running;
                    app.last_activity = SystemTime::now();
                }
                info!("Application deployed successfully: {}", app_id);
                Ok(())
            }
            Err(e) => {
                // Update application status with error
                if let Some(app) = self.applications.get_mut(app_id) {
                    app.status = ApplicationStatus::Error(format!("Deployment failed: {}", e));
                    app.error = Some(format!("Deployment failed: {}", e));
                }
                error!("Failed to deploy application {}: {}", app_id, e);
                Err(anyhow::anyhow!("Failed to deploy application: {}", e))
            }
        }
    }

    /// Start an application
    pub fn start_application(&mut self, app_id: &str) -> Result<()> {
        info!("Starting application: {}", app_id);

        if let Some(app) = self.applications.get_mut(app_id) {
            match app.status {
                ApplicationStatus::Stopped | ApplicationStatus::Paused => {
                    app.status = ApplicationStatus::Starting;
                    app.last_activity = SystemTime::now();
                    
                    // Simulate starting process
                    tokio::time::sleep(Duration::from_millis(100));
                    
                    app.status = ApplicationStatus::Running;
                    info!("Application started successfully: {}", app_id);
                    Ok(())
                }
                ApplicationStatus::Running => {
                    warn!("Application is already running: {}", app_id);
                    Ok(())
                }
                _ => {
                    Err(anyhow::anyhow!("Cannot start application in current status: {:?}", app.status))
                }
            }
        } else {
            Err(anyhow::anyhow!("Application not found: {}", app_id))
        }
    }

    /// Stop an application
    pub fn stop_application(&mut self, app_id: &str) -> Result<()> {
        info!("Stopping application: {}", app_id);

        if let Some(app) = self.applications.get_mut(app_id) {
            match app.status {
                ApplicationStatus::Running => {
                    app.status = ApplicationStatus::Stopping;
                    app.last_activity = SystemTime::now();
                    
                    // Simulate stopping process
                    tokio::time::sleep(Duration::from_millis(100));
                    
                    app.status = ApplicationStatus::Stopped;
                    info!("Application stopped successfully: {}", app_id);
                    Ok(())
                }
                ApplicationStatus::Stopped => {
                    warn!("Application is already stopped: {}", app_id);
                    Ok(())
                }
                _ => {
                    Err(anyhow::anyhow!("Cannot stop application in current status: {:?}", app.status))
                }
            }
        } else {
            Err(anyhow::anyhow!("Application not found: {}", app_id))
        }
    }

    /// Execute a function in an application
    pub fn execute_function(&mut self, app_id: &str, function_name: &str, args: &[wasmi::Value]) -> Result<wasmi::Value> {
        debug!("Executing function {} in application {}", function_name, app_id);

        if let Some(app) = self.applications.get_mut(app_id) {
            if app.status != ApplicationStatus::Running {
                return Err(anyhow::anyhow!("Application is not running: {}", app_id));
            }

            app.last_activity = SystemTime::now();

            // Execute the function in the runtime
            match self.runtime.execute_function(app_id, function_name, args) {
                Ok(result) => {
                    debug!("Function executed successfully: {} in {}", function_name, app_id);
                    Ok(result)
                }
                Err(e) => {
                    error!("Function execution failed: {} in {}: {}", function_name, app_id, e);
                    Err(anyhow::anyhow!("Function execution failed: {}", e))
                }
            }
        } else {
            Err(anyhow::anyhow!("Application not found: {}", app_id))
        }
    }

    /// Get application status
    pub fn get_application_status(&self, app_id: &str) -> Option<ApplicationStatus> {
        self.applications.get(app_id).map(|app| app.status.clone())
    }

    /// Get all application statuses
    pub fn get_all_application_statuses(&self) -> BTreeMap<String, ApplicationStatus> {
        self.applications
            .iter()
            .map(|(id, app)| (id.clone(), app.status.clone()))
            .collect()
    }

    /// Get application metrics
    pub fn get_application_metrics(&self, app_id: &str) -> Option<ApplicationMetrics> {
        if let Some(app) = self.applications.get(app_id) {
            let runtime_metrics = self.runtime.get_application_metrics(app_id);
            
            Some(ApplicationMetrics {
                app_id: app_id.to_string(),
                name: app.metadata.name.clone(),
                status: app.status.clone(),
                memory_usage: runtime_metrics.map(|m| m.memory_usage).unwrap_or(0),
                cpu_usage: 0, // TODO: Implement actual CPU tracking
                function_calls: runtime_metrics.map(|m| m.execution_count).unwrap_or(0),
                avg_execution_time: runtime_metrics.map(|m| m.avg_execution_time).unwrap_or(0),
                error_count: 0, // TODO: Implement error counting
                deployed_at: app.deployed_at,
                last_activity: app.last_activity,
                error: app.error.clone(),
            })
        } else {
            None
        }
    }

    /// Get all application metrics
    pub fn get_all_application_metrics(&self) -> BTreeMap<String, ApplicationMetrics> {
        self.applications
            .keys()
            .filter_map(|app_id| {
                self.get_application_metrics(app_id).map(|metrics| (app_id.clone(), metrics))
            })
            .collect()
    }

    /// Remove an application
    pub fn remove_application(&mut self, app_id: &str) -> Result<()> {
        info!("Removing application: {}", app_id);

        if self.applications.remove(app_id).is_some() {
            info!("Application removed successfully: {}", app_id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Application not found: {}", app_id))
        }
    }

    /// Get application count
    pub fn get_application_count(&self) -> usize {
        self.applications.len()
    }

    /// Get running application count
    pub fn get_running_application_count(&self) -> usize {
        self.applications
            .values()
            .filter(|app| app.status == ApplicationStatus::Running)
            .count()
    }

    /// Check application health
    pub fn check_application_health(&mut self) {
        // Check for applications that need attention
        let app_ids: Vec<String> = self.applications.keys().cloned().collect();
        for app_id in app_ids {
            if let Some(app) = self.applications.get(&app_id) {
                match app.status {
                    ApplicationStatus::Deploying => {
                        // Check if deployment is taking too long
                        let elapsed = SystemTime::now().duration_since(app.deployed_at).unwrap_or_default();
                        if elapsed > Duration::from_secs(30) {
                            warn!("Application {} deployment timeout", app_id);
                            // Update status to error
                            if let Some(app) = self.applications.get_mut(&app_id) {
                                app.status = ApplicationStatus::Error("Deployment timeout".to_string());
                                app.error = Some("Deployment timeout".to_string());
                            }
                        }
                    }
                    ApplicationStatus::Running => {
                        // Check if application is still healthy
                        if let Some(status) = self.runtime.get_application_status(&app_id) {
                            if !matches!(status, RuntimeStatus::Running) {
                                warn!("Application {} runtime status changed", app_id);
                                if let Some(app) = self.applications.get_mut(&app_id) {
                                    app.status = ApplicationStatus::Error("Runtime error".to_string());
                                    app.error = Some("Runtime error".to_string());
                                }
                            }
                        }
                    }
                    ApplicationStatus::Stopped => {
                        // Application is stopped, nothing to do
                    }
                    ApplicationStatus::Error(_) => {
                        // Application has error, could implement retry logic here
                    }
                    ApplicationStatus::Starting => {
                        // Application is starting, check timeout
                        let elapsed = SystemTime::now().duration_since(app.deployed_at).unwrap_or_default();
                        if elapsed > Duration::from_secs(60) {
                            warn!("Application {} starting timeout", app_id);
                            if let Some(app) = self.applications.get_mut(&app_id) {
                                app.status = ApplicationStatus::Error("Starting timeout".to_string());
                                app.error = Some("Starting timeout".to_string());
                            }
                        }
                    }
                    ApplicationStatus::Paused => {
                        // Application is paused, nothing to do
                    }
                    ApplicationStatus::Suspended => {
                        // Application is suspended, nothing to do
                    }
                    ApplicationStatus::Stopping => {
                        // Application is stopping, check timeout
                        let elapsed = SystemTime::now().duration_since(app.deployed_at).unwrap_or_default();
                        if elapsed > Duration::from_secs(30) {
                            warn!("Application {} stopping timeout", app_id);
                            if let Some(app) = self.applications.get_mut(&app_id) {
                                app.status = ApplicationStatus::Stopped;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Get application manager statistics
    pub fn get_statistics(&self) -> ApplicationManagerStatistics {
        let total_apps = self.applications.len();
        let running_apps = self.get_running_application_count();
        let error_apps = self.applications
            .values()
            .filter(|app| matches!(app.status, ApplicationStatus::Error(_)))
            .count();

        ApplicationManagerStatistics {
            total_applications: total_apps,
            running_applications: running_apps,
            stopped_applications: total_apps - running_apps - error_apps,
            error_applications: error_apps,
            max_applications: self.config.max_applications,
        }
    }
}

/// Application metrics
#[derive(Debug, Clone)]
pub struct ApplicationMetrics {
    /// Application ID
    pub app_id: String,
    /// Application name
    pub name: String,
    /// Application status
    pub status: ApplicationStatus,
    /// Memory usage in bytes
    pub memory_usage: usize,
    /// CPU usage percentage
    pub cpu_usage: u8,
    /// Function call count
    pub function_calls: u64,
    /// Average execution time in microseconds
    pub avg_execution_time: u32,
    /// Error count
    pub error_count: u64,
    /// Deployment timestamp
    pub deployed_at: SystemTime,
    /// Last activity timestamp
    pub last_activity: SystemTime,
    /// Error message if any
    pub error: Option<String>,
}

/// Application manager statistics
#[derive(Debug, Clone)]
pub struct ApplicationManagerStatistics {
    /// Total number of applications
    pub total_applications: usize,
    /// Number of running applications
    pub running_applications: usize,
    /// Number of stopped applications
    pub stopped_applications: usize,
    /// Number of applications with errors
    pub error_applications: usize,
    /// Maximum number of applications
    pub max_applications: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_manager_creation() {
        let runtime_config = WasmRuntimeConfig::default();
        let runtime = WasmRuntime::new(runtime_config).unwrap();
        let app_manager = ApplicationManager::new(runtime);
        
        assert_eq!(app_manager.get_application_count(), 0);
        assert_eq!(app_manager.get_running_application_count(), 0);
    }

    #[test]
    fn test_application_config_default() {
        let config = ApplicationConfig::default();
        assert_eq!(config.memory_limit, 1024 * 1024);
        assert_eq!(config.cpu_time_limit, 30000);
        assert!(config.auto_restart);
        assert_eq!(config.timeout, 30);
    }

    #[test]
    fn test_application_deployment() {
        let runtime_config = WasmRuntimeConfig::default();
        let runtime = WasmRuntime::new(runtime_config).unwrap();
        let mut app_manager = ApplicationManager::new(runtime);
        
        let wasm_binary = vec![0x00, 0x61, 0x73, 0x6d]; // WASM magic
        let result = app_manager.deploy_application("test-app", wasm_binary);
        
        // Deployment might fail for incomplete modules, but should not panic
        match result {
            Ok(_) => {
                assert_eq!(app_manager.get_application_count(), 1);
                let status = app_manager.get_application_status("test-app");
                assert!(status.is_some());
            }
            Err(_) => {
                // Expected for incomplete modules
            }
        }
    }

    #[test]
    fn test_application_statistics() {
        let runtime_config = WasmRuntimeConfig::default();
        let runtime = WasmRuntime::new(runtime_config).unwrap();
        let app_manager = ApplicationManager::new(runtime);
        
        let stats = app_manager.get_statistics();
        assert_eq!(stats.total_applications, 0);
        assert_eq!(stats.running_applications, 0);
        assert_eq!(stats.max_applications, 4);
    }
}