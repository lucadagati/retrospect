// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use crate::config::HostFunctionConfig;
use crate::context::WasmContext;
use crate::error::{WasmResult, WasmRuntimeError};
use std::sync::Arc;
use wasmtime::*;

pub mod px4;
pub mod microros;
pub mod sensors;
pub mod security;
pub mod gpio;
pub mod i2c_spi;

use px4::Px4HostFunctions;
use microros::MicroRosHostFunctions;
use sensors::SensorHostFunctions;
use security::SecurityHostFunctions;
use gpio::GpioHostFunctions;
use i2c_spi::I2cSpiHostFunctions;

/// Manager for host functions available to WASM modules
pub struct HostFunctionManager {
    /// WASM context
    context: Arc<WasmContext>,
    /// PX4 host functions
    px4_functions: Option<Px4HostFunctions>,
    /// microROS host functions
    microros_functions: Option<MicroRosHostFunctions>,
    /// Sensor host functions
    sensor_functions: Option<SensorHostFunctions>,
    /// Security host functions
    security_functions: Option<SecurityHostFunctions>,
    /// GPIO host functions
    gpio_functions: Option<GpioHostFunctions>,
    /// I2C/SPI host functions
    i2c_spi_functions: Option<I2cSpiHostFunctions>,
}

impl HostFunctionManager {
    /// Create a new host function manager
    pub fn new(config: &HostFunctionConfig, context: Arc<WasmContext>) -> WasmResult<Self> {
        let mut manager = Self {
            context: context.clone(),
            px4_functions: None,
            microros_functions: None,
            sensor_functions: None,
            security_functions: None,
            gpio_functions: None,
            i2c_spi_functions: None,
        };

        // Initialize enabled host function modules
        if config.enable_px4 {
            manager.px4_functions = Some(Px4HostFunctions::new(context.clone())?);
        }

        if config.enable_microros {
            manager.microros_functions = Some(MicroRosHostFunctions::new(context.clone())?);
        }

        if config.enable_sensors {
            manager.sensor_functions = Some(SensorHostFunctions::new(context.clone())?);
        }

        if config.enable_secure {
            manager.security_functions = Some(SecurityHostFunctions::new(context.clone())?);
        }

        if config.enable_gpio {
            manager.gpio_functions = Some(GpioHostFunctions::new(context.clone())?);
        }

        if config.enable_i2c_spi {
            manager.i2c_spi_functions = Some(I2cSpiHostFunctions::new(context.clone())?);
        }

        Ok(manager)
    }

    /// Get the PX4 functions (for testing purposes)
    pub fn px4_functions(&self) -> &Option<Px4HostFunctions> {
        &self.px4_functions
    }

    /// Get the microROS functions (for testing purposes)
    pub fn microros_functions(&self) -> &Option<MicroRosHostFunctions> {
        &self.microros_functions
    }

    /// Get the sensor functions (for testing purposes)
    pub fn sensor_functions(&self) -> &Option<SensorHostFunctions> {
        &self.sensor_functions
    }

    /// Get the security functions (for testing purposes)
    pub fn security_functions(&self) -> &Option<SecurityHostFunctions> {
        &self.security_functions
    }

    /// Get the GPIO functions (for testing purposes)
    pub fn gpio_functions(&self) -> &Option<GpioHostFunctions> {
        &self.gpio_functions
    }

    /// Get the I2C/SPI functions (for testing purposes)
    pub fn i2c_spi_functions(&self) -> &Option<I2cSpiHostFunctions> {
        &self.i2c_spi_functions
    }

    /// Create WASM time imports for all enabled host functions
    pub fn create_imports(&self, store: &mut Store<WasmContext>) -> WasmResult<Vec<Extern>> {
        let mut imports = Vec::new();

        // Add PX4 functions if enabled
        if let Some(ref px4_functions) = self.px4_functions {
            imports.extend(px4_functions.create_imports(store)?);
        }

        // Add microROS functions if enabled
        if let Some(ref microros_functions) = self.microros_functions {
            imports.extend(microros_functions.create_imports(store)?);
        }

        // Add sensor functions if enabled
        if let Some(ref sensor_functions) = self.sensor_functions {
            imports.extend(sensor_functions.create_imports(store)?);
        }

        // Add security functions if enabled
        if let Some(ref security_functions) = self.security_functions {
            imports.extend(security_functions.create_imports(store)?);
        }

        // Add GPIO functions if enabled
        if let Some(ref gpio_functions) = self.gpio_functions {
            imports.extend(gpio_functions.create_imports(store)?);
        }

        // Add I2C/SPI functions if enabled
        if let Some(ref i2c_spi_functions) = self.i2c_spi_functions {
            imports.extend(i2c_spi_functions.create_imports(store)?);
        }

        Ok(imports)
    }
}

/// Base trait for host function modules
pub trait HostFunctionModule {
    /// Create WASM time imports for this module
    fn create_imports(&self, store: &mut Store<WasmContext>) -> WasmResult<Vec<Extern>>;
}

/// Helper function to create a WASM time function that returns nothing
pub fn create_wasm_function_void<F>(
    store: &mut Store<WasmContext>,
    func: F,
) -> WasmResult<Func>
where
    F: Fn(&mut Caller<'_, WasmContext>, &[wasmtime::Val]) -> Result<(), WasmRuntimeError> + Send + Sync + 'static,
{
    let func_type = wasmtime::FuncType::new(
        std::iter::empty::<wasmtime::ValType>(),  // No parameters for now - this should be configurable
        std::iter::empty::<wasmtime::ValType>(),  // No results for now - this should be configurable
    );
    
    let func = Func::new(store, func_type, move |mut caller, args, results| {
        // Convert wasmtime::Val to our expected format
        let vals: Vec<wasmtime::Val> = args.iter().cloned().collect();
        
        if let Err(e) = func(&mut caller, &vals) {
            eprintln!("Host function error: {}", e);
        }
        Ok(())
    });
    Ok(func)
}

/// Helper function to extract string from WASM memory
pub fn extract_string_from_memory(
    caller: &mut Caller<'_, WasmContext>,
    ptr: i32,
    len: i32,
) -> WasmResult<String> {
    let memory = caller.get_export("memory")
        .ok_or_else(|| WasmRuntimeError::HostFunctionError("Memory not found".to_string()))?
        .into_memory()
        .ok_or_else(|| WasmRuntimeError::HostFunctionError("Export is not memory".to_string()))?;

    let mut data = vec![0u8; len as usize];
    memory.read(caller, ptr as usize, &mut data)
        .map_err(|e| WasmRuntimeError::HostFunctionError(format!("Failed to read memory: {}", e)))?;

    String::from_utf8(data)
        .map_err(|e| WasmRuntimeError::HostFunctionError(format!("Invalid UTF-8: {}", e)))
}

/// Helper function to write string to WASM memory
pub fn write_string_to_memory(
    caller: &mut Caller<'_, WasmContext>,
    ptr: i32,
    data: &[u8],
) -> WasmResult<()> {
    let memory = caller.get_export("memory")
        .ok_or_else(|| WasmRuntimeError::HostFunctionError("Memory not found".to_string()))?
        .into_memory()
        .ok_or_else(|| WasmRuntimeError::HostFunctionError("Export is not memory".to_string()))?;

    memory.write(caller, ptr as usize, data)
        .map_err(|e| WasmRuntimeError::HostFunctionError(format!("Failed to write memory: {}", e)))?;

    Ok(())
}

