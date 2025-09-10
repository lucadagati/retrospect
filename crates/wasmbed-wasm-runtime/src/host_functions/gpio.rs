// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use crate::context::{GpioPinState, GpioMode, GpioPull, WasmContext};
use crate::error::{WasmResult, WasmRuntimeError};
use crate::host_functions::{HostFunctionModule, create_wasm_function_void, extract_string_from_memory, write_string_to_memory};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use wasmtime::*;

/// GPIO host functions for GPIO pin control
pub struct GpioHostFunctions {
    context: Arc<WasmContext>,
}

/// GPIO pin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpioConfig {
    pub pin: u32,
    pub mode: String,
    pub pull: String,
    pub initial_value: bool,
}

/// GPIO pin state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpioState {
    pub pin: u32,
    pub mode: String,
    pub pull: String,
    pub value: bool,
    pub configured: bool,
}

/// GPIO operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpioResult {
    pub success: bool,
    pub value: Option<bool>,
    pub error_message: Option<String>,
}

impl GpioHostFunctions {
    /// Create new GPIO host functions
    pub fn new(context: Arc<WasmContext>) -> WasmResult<Self> {
        Ok(Self { context })
    }

    /// Configure GPIO pin
    pub fn configure_pin(
        &self,
        _caller: &mut Caller<'_, WasmContext>,
        _args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Configuring GPIO pin");
        Ok(())
    }

    /// Read GPIO pin
    pub fn read_pin(
        &self,
        _caller: &mut Caller<'_, WasmContext>,
        _args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Reading GPIO pin");
        Ok(())
    }

    /// Write GPIO pin
    pub fn write_pin(
        &self,
        _caller: &mut Caller<'_, WasmContext>,
        _args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Writing GPIO pin");
        Ok(())
    }

    /// Get GPIO pin state
    pub fn get_pin_state(
        &self,
        _caller: &mut Caller<'_, WasmContext>,
        _args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Getting GPIO pin state");
        Ok(())
    }

    /// Get all GPIO pins
    pub fn get_all_pins(
        &self,
        _caller: &mut Caller<'_, WasmContext>,
        _args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Getting all GPIO pins");
        Ok(())
    }

    /// Perform pin configuration
    fn perform_pin_configuration(&self, config: &GpioConfig) -> WasmResult<GpioResult> {
        // Parse mode
        let mode = match config.mode.as_str() {
            "input" => GpioMode::Input,
            "output" => GpioMode::Output,
            "alternate" => GpioMode::Alternate,
            "analog" => GpioMode::Analog,
            _ => return Ok(GpioResult {
                success: false,
                value: None,
                error_message: Some(format!("Invalid GPIO mode: {}", config.mode)),
            }),
        };

        // Parse pull
        let pull = match config.pull.as_str() {
            "none" => GpioPull::None,
            "up" => GpioPull::Up,
            "down" => GpioPull::Down,
            _ => return Ok(GpioResult {
                success: false,
                value: None,
                error_message: Some(format!("Invalid GPIO pull: {}", config.pull)),
            }),
        };

        // Create pin state
        let pin_state = GpioPinState {
            pin: config.pin,
            mode,
            value: config.initial_value,
            pull,
        };

        // Store pin state
        self.context.device_state.gpio_pins.insert(config.pin, pin_state);

        tracing::info!("Configured GPIO pin {} as {} with pull {}", 
                      config.pin, config.mode, config.pull);

        Ok(GpioResult {
            success: true,
            value: Some(config.initial_value),
            error_message: None,
        })
    }

    /// Perform pin read
    fn perform_pin_read(&self, pin: u32) -> WasmResult<GpioResult> {
        // Get pin state
        let pin_state = self.context.device_state.gpio_pins.get(&pin)
            .ok_or_else(|| WasmRuntimeError::HostFunctionError(
                format!("GPIO pin {} not configured", pin)
            ))?;

        // Simulate pin read (in real implementation, read from hardware)
        let value = pin_state.value;

        tracing::info!("Read GPIO pin {}: {}", pin, value);

        Ok(GpioResult {
            success: true,
            value: Some(value),
            error_message: None,
        })
    }

    /// Perform pin write
    fn perform_pin_write(&self, pin: u32, value: bool) -> WasmResult<GpioResult> {
        // Get pin state
        let mut pin_state = self.context.device_state.gpio_pins.get_mut(&pin)
            .ok_or_else(|| WasmRuntimeError::HostFunctionError(
                format!("GPIO pin {} not configured", pin)
            ))?;

        // Check if pin is configured as output
        if pin_state.mode != GpioMode::Output {
            return Ok(GpioResult {
                success: false,
                value: None,
                error_message: Some(format!("GPIO pin {} is not configured as output", pin)),
            });
        }

        // Update pin value
        pin_state.value = value;

        tracing::info!("Wrote GPIO pin {}: {}", pin, value);

        Ok(GpioResult {
            success: true,
            value: Some(value),
            error_message: None,
        })
    }

    /// Get pin state
    fn perform_get_pin_state(&self, pin: u32) -> WasmResult<GpioState> {
        // Get pin state
        let pin_state = self.context.device_state.gpio_pins.get(&pin)
            .ok_or_else(|| WasmRuntimeError::HostFunctionError(
                format!("GPIO pin {} not configured", pin)
            ))?;

        Ok(GpioState {
            pin: pin_state.pin,
            mode: format!("{:?}", pin_state.mode).to_lowercase(),
            pull: format!("{:?}", pin_state.pull).to_lowercase(),
            value: pin_state.value,
            configured: true,
        })
    }

    /// Get all pins
    fn perform_get_all_pins(&self) -> WasmResult<Vec<GpioState>> {
        let mut pins = Vec::new();

        for entry in self.context.device_state.gpio_pins.iter() {
            let pin_state = entry.value();
            pins.push(GpioState {
                pin: pin_state.pin,
                mode: format!("{:?}", pin_state.mode).to_lowercase(),
                pull: format!("{:?}", pin_state.pull).to_lowercase(),
                value: pin_state.value,
                configured: true,
            });
        }

        Ok(pins)
    }
}

impl HostFunctionModule for GpioHostFunctions {
    fn create_imports(&self, store: &mut Store<WasmContext>) -> WasmResult<Vec<Extern>> {
        let mut imports = Vec::new();

        // Create GPIO host functions
        let configure_pin = create_wasm_function_void(store, {
            let context = self.context.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let gpio_functions = GpioHostFunctions { context: context.clone() };
                gpio_functions.configure_pin(caller, args)?;
                Ok(())
            }
        })?;

        let read_pin = create_wasm_function_void(store, {
            let context = self.context.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let gpio_functions = GpioHostFunctions { context: context.clone() };
                gpio_functions.read_pin(caller, args)?;
                Ok(())
            }
        })?;

        let write_pin = create_wasm_function_void(store, {
            let context = self.context.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let gpio_functions = GpioHostFunctions { context: context.clone() };
                gpio_functions.write_pin(caller, args)?;
                Ok(())
            }
        })?;

        let get_pin_state = create_wasm_function_void(store, {
            let context = self.context.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let gpio_functions = GpioHostFunctions { context: context.clone() };
                gpio_functions.get_pin_state(caller, args)?;
                Ok(())
            }
        })?;

        let get_all_pins = create_wasm_function_void(store, {
            let context = self.context.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let gpio_functions = GpioHostFunctions { context: context.clone() };
                gpio_functions.get_all_pins(caller, args)?;
                Ok(())
            }
        })?;

        // Add functions to imports
        imports.push(Extern::Func(configure_pin));
        imports.push(Extern::Func(read_pin));
        imports.push(Extern::Func(write_pin));
        imports.push(Extern::Func(get_pin_state));
        imports.push(Extern::Func(get_all_pins));

        Ok(imports)
    }
}
