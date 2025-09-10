// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use crate::context::{I2cDevice, SpiDevice, SpiMode, WasmContext};
use crate::error::{WasmResult, WasmRuntimeError};
use crate::host_functions::{HostFunctionModule, create_wasm_function_void, extract_string_from_memory, write_string_to_memory};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use wasmtime::*;

/// I2C/SPI host functions for communication with external devices
pub struct I2cSpiHostFunctions {
    context: Arc<WasmContext>,
}

/// I2C device configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I2cConfig {
    pub address: u8,
    pub bus: u8,
    pub speed: u32, // Hz
}

/// SPI device configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiConfig {
    pub cs_pin: u32,
    pub bus: u8,
    pub speed: u32, // Hz
    pub mode: String,
}

/// I2C transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I2cTransaction {
    pub device_address: u8,
    pub write_data: Vec<u8>,
    pub read_length: u32,
}

/// SPI transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiTransaction {
    pub device_cs: u32,
    pub write_data: Vec<u8>,
    pub read_length: u32,
}

/// Communication result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunicationResult {
    pub success: bool,
    pub data: Option<Vec<u8>>,
    pub error_message: Option<String>,
}

impl I2cSpiHostFunctions {
    /// Create new I2C/SPI host functions
    pub fn new(context: Arc<WasmContext>) -> WasmResult<Self> {
        Ok(Self { context })
    }

    /// Configure I2C device
    pub fn configure_i2c(
        &self,
        _caller: &mut Caller<'_, WasmContext>,
        _args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Configuring I2C device");
        Ok(())
    }

    /// Configure SPI device
    pub fn configure_spi(
        &self,
        _caller: &mut Caller<'_, WasmContext>,
        _args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Configuring SPI device");
        Ok(())
    }

    /// Perform I2C transaction
    pub fn i2c_transaction(
        &self,
        _caller: &mut Caller<'_, WasmContext>,
        _args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Performing I2C transaction");
        Ok(())
    }

    /// Perform SPI transaction
    pub fn spi_transaction(
        &self,
        _caller: &mut Caller<'_, WasmContext>,
        _args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Performing SPI transaction");
        Ok(())
    }

    /// Get I2C device list
    pub fn get_i2c_devices(
        &self,
        _caller: &mut Caller<'_, WasmContext>,
        _args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Getting I2C devices");
        Ok(())
    }

    /// Get SPI device list
    pub fn get_spi_devices(
        &self,
        _caller: &mut Caller<'_, WasmContext>,
        _args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Getting SPI devices");
        Ok(())
    }

    /// Perform I2C configuration
    fn perform_i2c_configuration(&self, config: &I2cConfig) -> WasmResult<CommunicationResult> {
        // Create I2C device
        let device = I2cDevice {
            address: config.address,
            bus: config.bus,
            speed: config.speed,
            connected: true,
        };

        // Store device
        self.context.device_state.i2c_devices.insert(config.address, device);

        tracing::info!("Configured I2C device at address 0x{:02X} on bus {}", 
                      config.address, config.bus);

        Ok(CommunicationResult {
            success: true,
            data: None,
            error_message: None,
        })
    }

    /// Perform SPI configuration
    fn perform_spi_configuration(&self, config: &SpiConfig) -> WasmResult<CommunicationResult> {
        // Parse SPI mode
        let mode = match config.mode.as_str() {
            "mode0" => SpiMode::Mode0,
            "mode1" => SpiMode::Mode1,
            "mode2" => SpiMode::Mode2,
            "mode3" => SpiMode::Mode3,
            _ => return Ok(CommunicationResult {
                success: false,
                data: None,
                error_message: Some(format!("Invalid SPI mode: {}", config.mode)),
            }),
        };

        // Create SPI device
        let device = SpiDevice {
            cs_pin: config.cs_pin,
            bus: config.bus,
            speed: config.speed,
            mode,
            connected: true,
        };

        // Store device
        self.context.device_state.spi_devices.insert(config.cs_pin as u8, device);

        tracing::info!("Configured SPI device with CS pin {} on bus {}", 
                      config.cs_pin, config.bus);

        Ok(CommunicationResult {
            success: true,
            data: None,
            error_message: None,
        })
    }

    /// Perform I2C transaction
    fn perform_i2c_transaction(&self, transaction: &I2cTransaction) -> WasmResult<CommunicationResult> {
        // Check if device is configured
        let device = self.context.device_state.i2c_devices.get(&transaction.device_address)
            .ok_or_else(|| WasmRuntimeError::HostFunctionError(
                format!("I2C device at address 0x{:02X} not configured", transaction.device_address)
            ))?;

        if !device.connected {
            return Ok(CommunicationResult {
                success: false,
                data: None,
                error_message: Some(format!("I2C device at address 0x{:02X} not connected", transaction.device_address)),
            });
        }

        // Simulate I2C transaction (in real implementation, perform actual I2C communication)
        let mut read_data = Vec::new();
        for i in 0..transaction.read_length {
            read_data.push((i as u8) + 0x10); // Simulate data
        }

        tracing::info!("I2C transaction: address=0x{:02X}, write_len={}, read_len={}", 
                      transaction.device_address, transaction.write_data.len(), transaction.read_length);

        Ok(CommunicationResult {
            success: true,
            data: Some(read_data),
            error_message: None,
        })
    }

    /// Perform SPI transaction
    fn perform_spi_transaction(&self, transaction: &SpiTransaction) -> WasmResult<CommunicationResult> {
        // Check if device is configured
        let device = self.context.device_state.spi_devices.get(&(transaction.device_cs as u8))
            .ok_or_else(|| WasmRuntimeError::HostFunctionError(
                format!("SPI device with CS pin {} not configured", transaction.device_cs)
            ))?;

        if !device.connected {
            return Ok(CommunicationResult {
                success: false,
                data: None,
                error_message: Some(format!("SPI device with CS pin {} not connected", transaction.device_cs)),
            });
        }

        // Simulate SPI transaction (in real implementation, perform actual SPI communication)
        let mut read_data = Vec::new();
        for i in 0..transaction.read_length {
            read_data.push((i as u8) + 0x20); // Simulate data
        }

        tracing::info!("SPI transaction: cs_pin={}, write_len={}, read_len={}", 
                      transaction.device_cs, transaction.write_data.len(), transaction.read_length);

        Ok(CommunicationResult {
            success: true,
            data: Some(read_data),
            error_message: None,
        })
    }

    /// Get I2C devices
    fn perform_get_i2c_devices(&self) -> WasmResult<Vec<I2cConfig>> {
        let mut devices = Vec::new();

        for entry in self.context.device_state.i2c_devices.iter() {
            let device = entry.value();
            devices.push(I2cConfig {
                address: device.address,
                bus: device.bus,
                speed: device.speed,
            });
        }

        Ok(devices)
    }

    /// Get SPI devices
    fn perform_get_spi_devices(&self) -> WasmResult<Vec<SpiConfig>> {
        let mut devices = Vec::new();

        for entry in self.context.device_state.spi_devices.iter() {
            let device = entry.value();
            devices.push(SpiConfig {
                cs_pin: device.cs_pin,
                bus: device.bus,
                speed: device.speed,
                mode: format!("{:?}", device.mode).to_lowercase(),
            });
        }

        Ok(devices)
    }
}

impl HostFunctionModule for I2cSpiHostFunctions {
    fn create_imports(&self, store: &mut Store<WasmContext>) -> WasmResult<Vec<Extern>> {
        let mut imports = Vec::new();

        // Create I2C/SPI host functions
        let configure_i2c = create_wasm_function_void(store, {
            let context = self.context.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let i2c_spi_functions = I2cSpiHostFunctions { context: context.clone() };
                i2c_spi_functions.configure_i2c(caller, args)?;
                Ok(())
            }
        })?;

        let configure_spi = create_wasm_function_void(store, {
            let context = self.context.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let i2c_spi_functions = I2cSpiHostFunctions { context: context.clone() };
                i2c_spi_functions.configure_spi(caller, args)?;
                Ok(())
            }
        })?;

        let i2c_transaction = create_wasm_function_void(store, {
            let context = self.context.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let i2c_spi_functions = I2cSpiHostFunctions { context: context.clone() };
                i2c_spi_functions.i2c_transaction(caller, args)?;
                Ok(())
            }
        })?;

        let spi_transaction = create_wasm_function_void(store, {
            let context = self.context.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let i2c_spi_functions = I2cSpiHostFunctions { context: context.clone() };
                i2c_spi_functions.spi_transaction(caller, args)?;
                Ok(())
            }
        })?;

        let get_i2c_devices = create_wasm_function_void(store, {
            let context = self.context.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let i2c_spi_functions = I2cSpiHostFunctions { context: context.clone() };
                i2c_spi_functions.get_i2c_devices(caller, args)?;
                Ok(())
            }
        })?;

        let get_spi_devices = create_wasm_function_void(store, {
            let context = self.context.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let i2c_spi_functions = I2cSpiHostFunctions { context: context.clone() };
                i2c_spi_functions.get_spi_devices(caller, args)?;
                Ok(())
            }
        })?;

        // Add functions to imports
        imports.push(Extern::Func(configure_i2c));
        imports.push(Extern::Func(configure_spi));
        imports.push(Extern::Func(i2c_transaction));
        imports.push(Extern::Func(spi_transaction));
        imports.push(Extern::Func(get_i2c_devices));
        imports.push(Extern::Func(get_spi_devices));

        Ok(imports)
    }
}
