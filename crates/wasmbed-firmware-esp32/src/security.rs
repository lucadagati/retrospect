// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;

use crate::application_manager::ApplicationConfig;

/// Security and isolation module for WASM applications
pub struct SecurityManager {
    /// Application permissions
    permissions: BTreeMap<String, ApplicationPermissions>,
    /// Sandbox configuration
    sandbox_config: SandboxConfig,
    /// Security policies
    policies: SecurityPolicies,
}

/// Application permissions
#[derive(Debug, Clone)]
pub struct ApplicationPermissions {
    /// Network access
    pub network_access: bool,
    /// Filesystem access
    pub filesystem_access: bool,
    /// Device access
    pub device_access: bool,
    /// Memory access limits
    pub memory_limit: usize,
    /// CPU time limit
    pub cpu_time_limit: u32,
}

impl Default for ApplicationPermissions {
    fn default() -> Self {
        Self {
            network_access: false,
            filesystem_access: false,
            device_access: false,
            memory_limit: 1024 * 1024, // 1MB
            cpu_time_limit: 1000,      // 1 second
        }
    }
}

/// Sandbox configuration
pub struct SandboxConfig {
    /// Enable memory isolation
    pub memory_isolation: bool,
    /// Enable CPU time limiting
    pub cpu_time_limiting: bool,
    /// Enable network isolation
    pub network_isolation: bool,
    /// Enable device isolation
    pub device_isolation: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            memory_isolation: true,
            cpu_time_limiting: true,
            network_isolation: true,
            device_isolation: true,
        }
    }
}

/// Security policies
pub struct SecurityPolicies {
    /// Default permissions for new applications
    pub default_permissions: ApplicationPermissions,
    /// Maximum memory per application
    pub max_memory_per_app: usize,
    /// Maximum CPU time per application
    pub max_cpu_time_per_app: u32,
    /// Enable auto-restart on security violations
    pub auto_restart_on_violation: bool,
}

impl Default for SecurityPolicies {
    fn default() -> Self {
        Self {
            default_permissions: ApplicationPermissions::default(),
            max_memory_per_app: 1024 * 1024, // 1MB
            max_cpu_time_per_app: 1000,      // 1 second
            auto_restart_on_violation: false,
        }
    }
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Self {
        Self {
            permissions: BTreeMap::new(),
            sandbox_config: SandboxConfig::default(),
            policies: SecurityPolicies::default(),
        }
    }

    /// Set permissions for an application
    pub fn set_permissions(&mut self, app_id: &str, permissions: ApplicationPermissions) {
        self.permissions.insert(String::from(app_id), permissions);
    }

    /// Get permissions for an application
    pub fn get_permissions(&self, app_id: &str) -> ApplicationPermissions {
        self.permissions.get(app_id)
            .cloned()
            .unwrap_or(self.policies.default_permissions.clone())
    }

    /// Validate application deployment
    pub fn validate_deployment(&self, app_id: &str, config: &ApplicationConfig) -> Result<(), SecurityError> {
        // Check memory limit
        if config.memory_limit as usize > self.policies.max_memory_per_app {
            return Err(SecurityError::MemoryLimitExceeded(config.memory_limit as usize));
        }

        // Check CPU time limit
        if config.cpu_time_limit > self.policies.max_cpu_time_per_app {
            return Err(SecurityError::CpuTimeLimitExceeded(config.cpu_time_limit));
        }

        Ok(())
    }

    /// Validate function execution
    pub fn validate_execution(&self, app_id: &str, function_name: &str) -> Result<(), SecurityError> {
        let permissions = self.get_permissions(app_id);

        // Check if function requires special permissions
        if function_name.contains("network") && !permissions.network_access {
            return Err(SecurityError::PermissionDenied(String::from("network access")));
        }

        if function_name.contains("filesystem") && !permissions.filesystem_access {
            return Err(SecurityError::PermissionDenied(String::from("filesystem access")));
        }

        if function_name.contains("device") && !permissions.device_access {
            return Err(SecurityError::PermissionDenied(String::from("device access")));
        }

        Ok(())
    }

    /// Check memory usage
    pub fn check_memory_usage(&self, app_id: &str, current_usage: usize) -> Result<(), SecurityError> {
        let permissions = self.get_permissions(app_id);

        if current_usage > permissions.memory_limit {
            return Err(SecurityError::MemoryLimitExceeded(current_usage));
        }

        Ok(())
    }

    /// Check CPU time usage
    pub fn check_cpu_time_usage(&self, app_id: &str, current_usage: u32) -> Result<(), SecurityError> {
        let permissions = self.get_permissions(app_id);

        if current_usage > permissions.cpu_time_limit {
            return Err(SecurityError::CpuTimeLimitExceeded(current_usage));
        }

        Ok(())
    }
}

/// Security errors
#[derive(Debug)]
pub enum SecurityError {
    /// Permission denied
    PermissionDenied(String),
    /// Memory limit exceeded
    MemoryLimitExceeded(usize),
    /// CPU time limit exceeded
    CpuTimeLimitExceeded(u32),
    /// Security violation
    SecurityViolation(String),
}

impl core::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SecurityError::PermissionDenied(permission) => write!(f, "Permission denied: {}", permission),
            SecurityError::MemoryLimitExceeded(limit) => write!(f, "Memory limit exceeded: {} bytes", limit),
            SecurityError::CpuTimeLimitExceeded(limit) => write!(f, "CPU time limit exceeded: {} ms", limit),
            SecurityError::SecurityViolation(violation) => write!(f, "Security violation: {}", violation),
        }
    }
}
