// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::time::Duration;

use crate::wasm_runtime::{WasmRuntime, WasmRuntimeError, ApplicationStatus};

/// Application configuration
pub struct ApplicationConfig {
    /// Memory limit in bytes
    pub memory_limit: u32,
    /// CPU time limit in milliseconds
    pub cpu_time_limit: u32,
    /// Auto-restart on failure
    pub auto_restart: bool,
    /// Maximum restart attempts
    pub max_restarts: u8,
    /// Application timeout
    pub timeout: u32,
}

impl Default for ApplicationConfig {
    fn default() -> Self {
        Self {
            memory_limit: 1024 * 1024, // 1MB
            cpu_time_limit: 1000,      // 1 second
            auto_restart: true,
            max_restarts: 3,
            timeout: 30,               // 30 seconds
        }
    }
}

/// Application metadata
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

/// Application metrics
#[derive(Debug, Clone)]
pub struct ApplicationMetrics {
    /// Memory usage in bytes
    pub memory_usage: u32,
    /// CPU usage percentage
    pub cpu_usage: u8,
    /// Uptime in seconds
    pub uptime: u64,
    /// Number of function calls
    pub function_calls: u32,
    /// Number of errors
    pub error_count: u32,
}

/// WASM Application
pub struct WasmApplication {
    /// Application ID
    app_id: String,
    /// Application metadata
    metadata: ApplicationMetadata,
    /// Application configuration
    config: ApplicationConfig,
    /// Application status
    status: ApplicationStatus,
    /// Application metrics
    metrics: ApplicationMetrics,
    /// Restart count
    restart_count: u8,
    /// Start time
    start_time: u64,
}

/// Application Manager
pub struct ApplicationManager {
    /// WASM runtime
    runtime: WasmRuntime,
    /// Active applications
    applications: BTreeMap<String, WasmApplication>,
    /// Application configuration
    config: ApplicationManagerConfig,
}

/// Application Manager configuration
pub struct ApplicationManagerConfig {
    /// Maximum number of concurrent applications
    pub max_applications: usize,
    /// Default application timeout
    pub default_timeout: Duration,
    /// Enable auto-restart
    pub auto_restart: bool,
}

impl Default for ApplicationManagerConfig {
    fn default() -> Self {
        Self {
            max_applications: 5,
            default_timeout: Duration::from_secs(30),
            auto_restart: true,
        }
    }
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

    /// Deploy a WASM application
    pub fn deploy_application(&mut self, app_id: &str, wasm_bytes: &[u8], config: ApplicationConfig, metadata: ApplicationMetadata) -> Result<(), WasmRuntimeError> {
        // Check if we can deploy more applications
        if self.applications.len() >= self.config.max_applications {
            return Err(WasmRuntimeError::OutOfMemory); // Use as "too many applications"
        }

        // Load WASM module
        self.runtime.load_module(app_id, wasm_bytes)?;

        // Create application
        let app = WasmApplication {
            app_id: String::from(app_id),
            metadata,
            config,
            status: ApplicationStatus::Loading,
            metrics: ApplicationMetrics {
                memory_usage: 0,
                cpu_usage: 0,
                uptime: 0,
                function_calls: 0,
                error_count: 0,
            },
            restart_count: 0,
            start_time: 0, // Will be set when started
        };

        // Store application
        self.applications.insert(String::from(app_id), app);

        // Start application
        self.start_application(app_id)?;

        Ok(())
    }

    /// Start an application
    pub fn start_application(&mut self, app_id: &str) -> Result<(), WasmRuntimeError> {
        let app = self.applications.get_mut(app_id)
            .ok_or(WasmRuntimeError::ApplicationNotFound(String::from(app_id)))?;

        // Update status
        app.status = ApplicationStatus::Running;
        app.start_time = 0; // Would be actual time in real implementation

        Ok(())
    }

    /// Stop an application
    pub fn stop_application(&mut self, app_id: &str) -> Result<(), WasmRuntimeError> {
        let app = self.applications.get_mut(app_id)
            .ok_or(WasmRuntimeError::ApplicationNotFound(String::from(app_id)))?;

        // Update status
        app.status = ApplicationStatus::Stopped;

        // Unload from runtime
        self.runtime.unload_application(app_id)?;

        Ok(())
    }

    /// Execute a function in an application
    pub fn execute_function(&mut self, app_id: &str, function_name: &str, params: &[crate::wasm_runtime::WasmValue]) -> Result<Vec<crate::wasm_runtime::WasmValue>, WasmRuntimeError> {
        // Execute function
        let results = self.runtime.execute_function(app_id, function_name, params)?;

        // Update metrics
        if let Some(app) = self.applications.get_mut(app_id) {
            app.metrics.function_calls += 1;
        }

        Ok(results)
    }

    /// Get application status
    pub fn get_application_status(&self, app_id: &str) -> Option<ApplicationStatus> {
        self.applications.get(app_id).map(|app| app.status.clone())
    }

    /// Get all application statuses
    pub fn get_all_application_statuses(&self) -> BTreeMap<String, ApplicationStatus> {
        self.applications.iter()
            .map(|(id, app)| (id.clone(), app.status.clone()))
            .collect()
    }

    /// Process applications (called in main loop)
    pub fn process_applications(&mut self) {
        // Update metrics for running applications
        for app in self.applications.values_mut() {
            if matches!(app.status, ApplicationStatus::Running) {
                app.metrics.uptime += 1;
                
                // Simple CPU usage simulation
                app.metrics.cpu_usage = (app.metrics.function_calls % 100) as u8;
                
                // Simple memory usage simulation
                app.metrics.memory_usage = (app.metrics.uptime * 1024) as u32; // 1KB per second
            }
        }

        // Handle auto-restart for failed applications
        if self.config.auto_restart {
            let mut to_restart = Vec::new();
            
            for (app_id, app) in &self.applications {
                if matches!(app.status, ApplicationStatus::Error(_)) && app.restart_count < app.config.max_restarts {
                    to_restart.push(app_id.clone());
                }
            }

            for app_id in to_restart {
                if let Err(_e) = self.start_application(&app_id) {
                    // Log error (in real implementation)
                    if let Some(app) = self.applications.get_mut(&app_id) {
                        app.metrics.error_count += 1;
                    }
                }
            }
        }
    }

    /// Get application metrics
    pub fn get_application_metrics(&self, app_id: &str) -> Option<&ApplicationMetrics> {
        self.applications.get(app_id).map(|app| &app.metrics)
    }

    /// Get all application metrics
    pub fn get_all_application_metrics(&self) -> BTreeMap<String, ApplicationMetrics> {
        self.applications.iter()
            .map(|(id, app)| (id.clone(), app.metrics.clone()))
            .collect()
    }
}
