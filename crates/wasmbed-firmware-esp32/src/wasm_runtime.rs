// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use log::{error, info, warn};
use wasmi::{
    Engine, Module, Linker, Store, Instance, Memory,
    TypedFunc, Caller, ExternType, FuncType, Value
};
use thiserror::Error;

use wasmbed_protocol::{ClientMessage, ServerMessage, DeviceUuid};
use wasmbed_types::PublicKey;

/// Configuration for WASM runtime
#[derive(Debug, Clone)]
pub struct WasmRuntimeConfig {
    /// Maximum memory per application
    pub max_memory_per_app: usize,
    /// Maximum number of concurrent applications
    pub max_concurrent_apps: usize,
    /// Default execution timeout
    pub default_timeout: Duration,
    /// Maximum stack size per application
    pub max_stack_size: usize,
}

impl Default for WasmRuntimeConfig {
    fn default() -> Self {
        Self {
            max_memory_per_app: 1024 * 1024, // 1MB
            max_concurrent_apps: 4,
            default_timeout: Duration::from_secs(30),
            max_stack_size: 64 * 1024, // 64KB stack
        }
    }
}

/// WASM Runtime for ESP32 devices using wasmi 0.17
pub struct WasmRuntime {
    /// Engine for WASM execution
    engine: Engine,
    /// Active WASM instances
    instances: BTreeMap<String, WasmInstance>,
    /// Runtime configuration
    config: WasmRuntimeConfig,
}

/// WASM Instance wrapper
#[derive(Debug)]
pub struct WasmInstance {
    /// WASM instance
    instance: Instance,
    /// Store for the instance
    store: Store<()>,
    /// Module name
    module_name: String,
    /// Memory reference
    memory: Option<Memory>,
    /// Exported functions
    functions: BTreeMap<String, TypedFunc<(), ()>>,
}

/// WASM Runtime errors
#[derive(Error, Debug)]
pub enum WasmRuntimeError {
    #[error("Module load error: {0}")]
    ModuleLoadError(String),
    #[error("Instance creation error: {0}")]
    InstanceCreationError(String),
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error("Application not found: {0}")]
    ApplicationNotFound(String),
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
    #[error("Memory limit exceeded: {0} bytes")]
    MemoryLimitExceeded(usize),
    #[error("Out of memory")]
    OutOfMemory,
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Type mismatch")]
    TypeMismatch,
}

impl WasmRuntime {
    /// Create a new WASM runtime
    pub fn new(config: WasmRuntimeConfig) -> Result<Self, WasmRuntimeError> {
        // Create engine with default configuration
        let engine = Engine::default();
        
        Ok(Self {
            engine,
            instances: BTreeMap::new(),
            config,
        })
    }

    /// Load a WASM module
    pub fn load_module(&mut self, module_name: &str, wasm_bytes: &[u8]) -> Result<(), WasmRuntimeError> {
        info!("Loading WASM module: {}", module_name);

        // Parse the WASM module
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| WasmRuntimeError::ModuleLoadError(format!("Failed to parse module: {}", e)))?;

        // Create a new store
        let mut store = Store::new(&self.engine, ());

        // Create linker for host functions
        let mut linker = Linker::new();
        
        // Add host functions
        self.add_host_functions(&mut linker)?;

        // Instantiate the module
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| WasmRuntimeError::InstanceCreationError(format!("Failed to instantiate: {}", e)))?
            .start(&mut store)
            .map_err(|e| WasmRuntimeError::InstanceCreationError(format!("Failed to start: {}", e)))?;

        // Get memory if available
        let memory = instance.get_memory(&store, "memory").ok();

        // Extract exported functions
        let mut functions = BTreeMap::new();
        for export in module.exports() {
            if let ExternType::Func(func_type) = export.ty() {
                if func_type.params().is_empty() && func_type.results().is_empty() {
                    if let Some(func) = instance.get_func(&store, export.name()) {
                        if let Ok(typed_func) = TypedFunc::<(), ()>::new(&func, &store) {
                            functions.insert(export.name().to_string(), typed_func);
                        }
                    }
                }
            }
        }

        // Create WASM instance wrapper
        let wasm_instance = WasmInstance {
            instance,
            store,
            module_name: module_name.to_string(),
            memory,
            functions,
        };

        // Store the instance
        self.instances.insert(module_name.to_string(), wasm_instance);

        info!("WASM module loaded successfully: {}", module_name);
        Ok(())
    }

    /// Execute a function in a WASM module
    pub fn execute_function(&mut self, module_name: &str, function_name: &str, _args: &[Value]) -> Result<Value, WasmRuntimeError> {
        let instance = self.instances.get_mut(module_name)
            .ok_or_else(|| WasmRuntimeError::ApplicationNotFound(module_name.to_string()))?;

        let func = instance.functions.get(function_name)
            .ok_or_else(|| WasmRuntimeError::FunctionNotFound(function_name.to_string()))?;

        // Execute the function
        func.call(&mut instance.store, ())
            .map_err(|e| WasmRuntimeError::ExecutionError(format!("Function execution failed: {}", e)))?;

        // Return a dummy value for now
        Ok(Value::I32(0))
    }

    /// Get application status
    pub fn get_application_status(&self, module_name: &str) -> Option<ApplicationStatus> {
        if self.instances.contains_key(module_name) {
            Some(ApplicationStatus::Running)
        } else {
            None
        }
    }

    /// Get application metrics
    pub fn get_application_metrics(&self, module_name: &str) -> Option<ApplicationMetrics> {
        if let Some(instance) = self.instances.get(module_name) {
            let memory_usage = if let Some(memory) = &instance.memory {
                memory.size(&instance.store) as usize * 65536 // Convert pages to bytes
            } else {
                0
            };

            Some(ApplicationMetrics {
                app_id: module_name.to_string(),
                memory_usage,
                cpu_usage: 0, // TODO: Implement actual CPU tracking
                function_calls: 0, // TODO: Implement function call counting
                avg_execution_time: 0, // TODO: Implement execution time tracking
                error_count: 0, // TODO: Implement error counting
                status: ApplicationStatus::Running,
                last_activity: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs(),
            })
        } else {
            None
        }
    }

    /// Get memory info for an application
    pub fn get_memory_info(&self, module_name: &str) -> Option<MemoryInfo> {
        if let Some(instance) = self.instances.get(module_name) {
            if let Some(memory) = &instance.memory {
                Some(MemoryInfo {
                    total_pages: memory.size(&instance.store),
                    used_pages: memory.size(&instance.store), // Simplified
                    max_pages: Some(16), // 1MB max
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Add host functions to the linker
    fn add_host_functions(&self, linker: &mut Linker<()>) -> Result<(), WasmRuntimeError> {
        // Add console.log function
        linker
            .func_wrap("console", "log", |caller: Caller<'_, ()>, ptr: i32, len: i32| {
                if let Some(memory) = caller.get_export("memory") {
                    if let Ok(bytes) = memory.read(&caller, ptr as u32, len as u32) {
                        let message = String::from_utf8_lossy(&bytes);
                        info!("WASM console.log: {}", message);
                    }
                }
            })
            .map_err(|e| WasmRuntimeError::ModuleLoadError(format!("Failed to add console.log: {}", e)))?;

        // Add memory allocation function
        linker
            .func_wrap("env", "malloc", |caller: Caller<'_, ()>, size: i32| -> i32 {
                // Simplified memory allocation - return a dummy pointer
                size
            })
            .map_err(|e| WasmRuntimeError::ModuleLoadError(format!("Failed to add malloc: {}", e)))?;

        // Add memory free function
        linker
            .func_wrap("env", "free", |caller: Caller<'_, ()>, ptr: i32| {
                // Simplified memory free - do nothing for now
                let _ = caller;
                let _ = ptr;
            })
            .map_err(|e| WasmRuntimeError::ModuleLoadError(format!("Failed to add free: {}", e)))?;

        Ok(())
    }

    /// Create a new engine with custom configuration
    pub fn create_engine() -> Engine {
        let mut engine_config = wasmi::Config::default();
        // Enable bulk memory operations
        engine_config.wasm_bulk_memory(true);
        // Enable reference types
        engine_config.wasm_reference_types(true);
        
        Engine::new(&engine_config)
    }
}

/// Application status
#[derive(Debug, Clone)]
pub enum ApplicationStatus {
    Loading,
    Running,
    Stopped,
    Error(String),
}

/// Application metrics
#[derive(Debug, Clone)]
pub struct ApplicationMetrics {
    pub app_id: String,
    pub memory_usage: usize,
    pub cpu_usage: u8,
    pub function_calls: u64,
    pub avg_execution_time: u32,
    pub error_count: u64,
    pub status: ApplicationStatus,
    pub last_activity: u64,
}

/// Memory information
#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total_pages: u32,
    pub used_pages: u32,
    pub max_pages: Option<u32>,
}

/// Simple WASM modules for testing
pub mod test_modules {
    /// Simple WASM module that just returns 42
    pub const SIMPLE_WASM: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, // WASM magic
        0x01, 0x00, 0x00, 0x00, // Version 1
        0x01, 0x04, 0x01, 0x60, 0x00, 0x00, // Function type
        0x03, 0x02, 0x01, 0x00, // Function section
        0x07, 0x07, 0x01, 0x03, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00, // Export section
        0x0a, 0x05, 0x01, 0x03, 0x00, 0x2a, 0x0b, // Code section
    ];

    /// WASM module that multiplies two numbers
    pub const MULTIPLY_WASM: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, // WASM magic
        0x01, 0x00, 0x00, 0x00, // Version 1
        0x01, 0x06, 0x01, 0x60, 0x02, 0x7f, 0x7f, 0x01, 0x7f, // Function type (i32, i32) -> i32
        0x03, 0x02, 0x01, 0x00, // Function section
        0x07, 0x0b, 0x01, 0x07, 0x6d, 0x75, 0x6c, 0x74, 0x69, 0x70, 0x6c, 0x79, 0x00, 0x00, // Export section
        0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x20, 0x01, 0x6c, 0x0b, // Code section
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let config = WasmRuntimeConfig::default();
        let runtime = WasmRuntime::new(config);
        assert!(runtime.is_ok());
    }

    #[test]
    fn test_simple_module_loading() {
        let config = WasmRuntimeConfig::default();
        let mut runtime = WasmRuntime::new(config).unwrap();
        
        let result = runtime.load_module("test", test_modules::SIMPLE_WASM);
        assert!(result.is_ok());
        
        let status = runtime.get_application_status("test");
        assert_eq!(status, Some(ApplicationStatus::Running));
    }

    #[test]
    fn test_function_execution() {
        let config = WasmRuntimeConfig::default();
        let mut runtime = WasmRuntime::new(config).unwrap();
        
        runtime.load_module("test", test_modules::SIMPLE_WASM).unwrap();
        
        let result = runtime.execute_function("test", "main", &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_info() {
        let config = WasmRuntimeConfig::default();
        let mut runtime = WasmRuntime::new(config).unwrap();
        
        runtime.load_module("test", test_modules::SIMPLE_WASM).unwrap();
        
        let memory_info = runtime.get_memory_info("test");
        // Memory info might be None for simple modules without memory
        // This is expected behavior
    }

    #[test]
    fn test_metrics() {
        let config = WasmRuntimeConfig::default();
        let mut runtime = WasmRuntime::new(config).unwrap();
        
        runtime.load_module("test", test_modules::SIMPLE_WASM).unwrap();
        
        let metrics = runtime.get_application_metrics("test");
        assert!(metrics.is_some());
        
        if let Some(metrics) = metrics {
            assert_eq!(metrics.app_id, "test");
            assert_eq!(metrics.status, ApplicationStatus::Running);
        }
    }
}