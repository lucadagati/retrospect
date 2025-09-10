// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use crate::context::{SensorReading, SensorType, SensorQuality, WasmContext};
use crate::error::{WasmResult, WasmRuntimeError};
use crate::host_functions::{HostFunctionModule, create_wasm_function_void, extract_string_from_memory, write_string_to_memory};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::SystemTime;
use wasmtime::*;

/// Sensor host functions for sensor data access
pub struct SensorHostFunctions {
    context: Arc<WasmContext>,
}

/// Sensor reading request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReadingRequest {
    pub sensor_id: String,
    pub sensor_type: SensorType,
}

/// Sensor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConfig {
    pub sensor_id: String,
    pub sensor_type: SensorType,
    pub sample_rate: f32, // Hz
    pub enabled: bool,
    pub calibration_data: Vec<f32>,
}

/// Sensor calibration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationData {
    pub sensor_id: String,
    pub offset: Vec<f32>,
    pub scale: Vec<f32>,
    pub temperature_coefficient: Vec<f32>,
    pub calibrated: bool,
}

impl SensorHostFunctions {
    /// Create new sensor host functions
    pub fn new(context: Arc<WasmContext>) -> WasmResult<Self> {
        Ok(Self { context })
    }

    /// Read sensor data
    pub fn read_sensor(
        &self,
        _caller: &mut Caller<'_, WasmContext>,
        _args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Reading sensor data");
        Ok(())
    }

    /// Configure sensor
    pub fn configure_sensor(
        &self,
        _caller: &mut Caller<'_, WasmContext>,
        _args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Configuring sensor");
        Ok(())
    }

    /// Calibrate sensor
    pub fn calibrate_sensor(
        &self,
        _caller: &mut Caller<'_, WasmContext>,
        _args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Calibrating sensor");
        Ok(())
    }

    /// Get sensor list
    pub fn get_sensor_list(
        &self,
        _caller: &mut Caller<'_, WasmContext>,
        _args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Getting sensor list");
        Ok(())
    }

    /// Get sensor status
    pub fn get_sensor_status(
        &self,
        _caller: &mut Caller<'_, WasmContext>,
        _args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Getting sensor status");
        Ok(())
    }

    /// Simulate sensor reading
    fn simulate_sensor_reading(&self, request: &SensorReadingRequest) -> WasmResult<SensorReading> {
        let value = match request.sensor_type {
            SensorType::Accelerometer => 9.81, // m/s²
            SensorType::Gyroscope => 0.1,     // rad/s
            SensorType::Magnetometer => 25.0, // μT
            SensorType::Barometer => 101325.0, // Pa
            SensorType::Temperature => 25.0,   // °C
            SensorType::Humidity => 60.0,     // %
            SensorType::Pressure => 1013.25,   // hPa
            SensorType::Light => 500.0,       // lux
            SensorType::Proximity => 10.0,    // cm
            SensorType::Camera => 0.0,        // placeholder
            SensorType::Gps => 0.0,            // placeholder
            SensorType::Lidar => 5.0,         // m
            SensorType::Ultrasonic => 15.0,   // cm
            SensorType::Infrared => 20.0,     // °C
        };

        let unit = match request.sensor_type {
            SensorType::Accelerometer => "m/s²".to_string(),
            SensorType::Gyroscope => "rad/s".to_string(),
            SensorType::Magnetometer => "μT".to_string(),
            SensorType::Barometer => "Pa".to_string(),
            SensorType::Temperature => "°C".to_string(),
            SensorType::Humidity => "%".to_string(),
            SensorType::Pressure => "hPa".to_string(),
            SensorType::Light => "lux".to_string(),
            SensorType::Proximity => "cm".to_string(),
            SensorType::Camera => "pixels".to_string(),
            SensorType::Gps => "degrees".to_string(),
            SensorType::Lidar => "m".to_string(),
            SensorType::Ultrasonic => "cm".to_string(),
            SensorType::Infrared => "°C".to_string(),
        };

        Ok(SensorReading {
            sensor_id: request.sensor_id.clone(),
            sensor_type: request.sensor_type,
            value,
            unit,
            timestamp: SystemTime::now(),
            quality: SensorQuality::Good,
        })
    }

    /// Simulate available sensors
    fn simulate_available_sensors(&self) -> WasmResult<Vec<SensorConfig>> {
        Ok(vec![
            SensorConfig {
                sensor_id: "accel_001".to_string(),
                sensor_type: SensorType::Accelerometer,
                sample_rate: 100.0,
                enabled: true,
                calibration_data: vec![0.0, 0.0, 0.0],
            },
            SensorConfig {
                sensor_id: "gyro_001".to_string(),
                sensor_type: SensorType::Gyroscope,
                sample_rate: 100.0,
                enabled: true,
                calibration_data: vec![0.0, 0.0, 0.0],
            },
            SensorConfig {
                sensor_id: "mag_001".to_string(),
                sensor_type: SensorType::Magnetometer,
                sample_rate: 50.0,
                enabled: true,
                calibration_data: vec![0.0, 0.0, 0.0],
            },
            SensorConfig {
                sensor_id: "baro_001".to_string(),
                sensor_type: SensorType::Barometer,
                sample_rate: 20.0,
                enabled: true,
                calibration_data: vec![0.0],
            },
            SensorConfig {
                sensor_id: "temp_001".to_string(),
                sensor_type: SensorType::Temperature,
                sample_rate: 1.0,
                enabled: true,
                calibration_data: vec![0.0],
            },
            SensorConfig {
                sensor_id: "lidar_001".to_string(),
                sensor_type: SensorType::Lidar,
                sample_rate: 10.0,
                enabled: true,
                calibration_data: vec![0.0],
            },
        ])
    }

    /// Simulate sensor status
    fn simulate_sensor_status(&self, sensor_id: &str) -> WasmResult<SensorConfig> {
        // Find sensor in simulated list
        let sensors = self.simulate_available_sensors()?;
        sensors.into_iter()
            .find(|s| s.sensor_id == sensor_id)
            .ok_or_else(|| WasmRuntimeError::SensorError(
                format!("Sensor {} not found", sensor_id)
            ))
    }
}

impl HostFunctionModule for SensorHostFunctions {
    fn create_imports(&self, store: &mut Store<WasmContext>) -> WasmResult<Vec<Extern>> {
        let mut imports = Vec::new();

        // Create sensor host functions
        let read_sensor = create_wasm_function_void(store, {
            let context = self.context.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let sensor_functions = SensorHostFunctions { context: context.clone() };
                sensor_functions.read_sensor(caller, args)?;
                Ok(())
            }
        })?;

        let configure_sensor = create_wasm_function_void(store, {
            let context = self.context.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let sensor_functions = SensorHostFunctions { context: context.clone() };
                sensor_functions.configure_sensor(caller, args)?;
                Ok(())
            }
        })?;

        let calibrate_sensor = create_wasm_function_void(store, {
            let context = self.context.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let sensor_functions = SensorHostFunctions { context: context.clone() };
                sensor_functions.calibrate_sensor(caller, args)?;
                Ok(())
            }
        })?;

        let get_sensor_list = create_wasm_function_void(store, {
            let context = self.context.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let sensor_functions = SensorHostFunctions { context: context.clone() };
                sensor_functions.get_sensor_list(caller, args)?;
                Ok(())
            }
        })?;

        let get_sensor_status = create_wasm_function_void(store, {
            let context = self.context.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let sensor_functions = SensorHostFunctions { context: context.clone() };
                sensor_functions.get_sensor_status(caller, args)?;
                Ok(())
            }
        })?;

        // Add functions to imports
        imports.push(Extern::Func(read_sensor));
        imports.push(Extern::Func(configure_sensor));
        imports.push(Extern::Func(calibrate_sensor));
        imports.push(Extern::Func(get_sensor_list));
        imports.push(Extern::Func(get_sensor_status));

        Ok(imports)
    }
}
