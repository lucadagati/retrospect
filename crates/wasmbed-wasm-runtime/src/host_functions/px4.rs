// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use crate::context::WasmContext;
use crate::error::{WasmResult, WasmRuntimeError};
use crate::host_functions::{HostFunctionModule, create_wasm_function_void, write_string_to_memory};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use wasmtime::*;

/// PX4 host functions for MAVLink communication
pub struct Px4HostFunctions {
    context: Arc<WasmContext>,
    /// MAVLink connection (simulated for now)
    connection: Arc<Mutex<Option<()>>>,
    /// Cached vehicle status
    vehicle_status: Arc<Mutex<Option<VehicleStatus>>>,
    /// Cached battery status
    battery_status: Arc<Mutex<Option<BatteryStatus>>>,
    /// Cached local position
    local_position: Arc<Mutex<Option<VehicleLocalPosition>>>,
}

/// PX4 vehicle command message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleCommand {
    pub timestamp: u64,
    pub param1: f32,
    pub param2: f32,
    pub param3: f32,
    pub param4: f32,
    pub param5: f32,
    pub param6: f32,
    pub param7: f32,
    pub command: u16,
    pub target_system: u8,
    pub target_component: u8,
    pub source_system: u8,
    pub source_component: u8,
    pub confirmation: u8,
}

/// PX4 vehicle status message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleStatus {
    pub timestamp: u64,
    pub armed: bool,
    pub armed_time: u64,
    pub takeoff_time: u64,
    pub nav_state: u8,
    pub nav_state_timestamp: u64,
    pub failure_detector_status: u8,
    pub hil_state: u8,
    pub vehicle_type: u8,
    pub is_vtol: bool,
    pub is_vtol_tailsitter: bool,
    pub vtol_fw_permanent_stab: bool,
    pub in_transition_mode: bool,
    pub in_transition_to_fw: bool,
    pub rc_signal_lost: bool,
    pub rc_input_mode: u8,
    pub data_link_lost: bool,
    pub data_link_lost_counter: u8,
    pub high_latency_data_link_lost: bool,
    pub engine_failure: bool,
    pub mission_failure: bool,
}

/// PX4 battery status message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryStatus {
    pub timestamp: u64,
    pub connected: bool,
    pub voltage_v: f32,
    pub voltage_filtered_v: f32,
    pub current_a: f32,
    pub current_filtered_a: f32,
    pub current_average_a: f32,
    pub discharged_mah: f32,
    pub remaining: f32,
    pub scale: f32,
    pub temperature: f32,
    pub cell_count: u8,
    pub source: u8,
    pub priority: u8,
    pub capacity: u16,
    pub cycle_count: u16,
    pub average_time_to_empty_min: u16,
    pub serial_number: u16,
    pub manufacture_date: u16,
    pub state_of_health: u8,
    pub max_error: u8,
    pub id: u8,
    pub interface_error: u8,
}

/// PX4 vehicle local position message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleLocalPosition {
    pub timestamp: u64,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub delta_xy: [f32; 2],
    pub delta_z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub delta_vxy: [f32; 2],
    pub delta_vz: f32,
    pub ax: f32,
    pub ay: f32,
    pub az: f32,
    pub yaw: f32,
    pub delta_yaw: f32,
    pub xy_valid: bool,
    pub z_valid: bool,
    pub v_xy_valid: bool,
    pub v_z_valid: bool,
    pub yaw_valid: bool,
    pub xy_reset_counter: u8,
    pub z_reset_counter: u8,
    pub vxy_reset_counter: u8,
    pub vz_reset_counter: u8,
    pub yaw_reset_counter: u8,
    pub delta_xy_reset_counter: u8,
    pub delta_z_reset_counter: u8,
    pub delta_vxy_reset_counter: u8,
    pub delta_vz_reset_counter: u8,
    pub delta_yaw_reset_counter: u8,
    pub dist_bottom: f32,
    pub dist_bottom_valid: bool,
    pub dist_bottom_sensor_bitfield: u8,
    pub eph: f32,
    pub epv: f32,
    pub evh: f32,
    pub evv: f32,
    pub dead_reckoning: bool,
    pub vxy_max: f32,
    pub vz_max: f32,
    pub hagl_min: f32,
    pub hagl_max: f32,
    pub heading: f32,
    pub heading_var: f32,
    pub heading_good_for_control: bool,
    pub heading_reset_counter: u8,
}

impl Px4HostFunctions {
    /// Create new PX4 host functions
    pub fn new(context: Arc<WasmContext>) -> WasmResult<Self> {
        Ok(Self { 
            context,
            connection: Arc::new(Mutex::new(None)),
            vehicle_status: Arc::new(Mutex::new(None)),
            battery_status: Arc::new(Mutex::new(None)),
            local_position: Arc::new(Mutex::new(None)),
        })
    }

    /// Send vehicle command to PX4
    pub fn send_vehicle_command(
        &self,
        caller: &mut Caller<'_, WasmContext>,
        args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        if args.len() < 8 {
            return Err(WasmRuntimeError::ExecutionError(
                "send_vehicle_command requires 8 parameters".to_string()
            ));
        }

        // Extract command parameters from WASM memory
        let param1 = match args[0] {
            wasmtime::Val::I32(i) => i as f32,
            wasmtime::Val::F32(f) => f32::from_bits(f),
            _ => return Err(WasmRuntimeError::ExecutionError("param1 must be i32 or f32".to_string())),
        };
        let param2 = match args[1] {
            wasmtime::Val::I32(i) => i as f32,
            wasmtime::Val::F32(f) => f32::from_bits(f),
            _ => return Err(WasmRuntimeError::ExecutionError("param2 must be i32 or f32".to_string())),
        };
        let param3 = match args[2] {
            wasmtime::Val::I32(i) => i as f32,
            wasmtime::Val::F32(f) => f32::from_bits(f),
            _ => return Err(WasmRuntimeError::ExecutionError("param3 must be i32 or f32".to_string())),
        };
        let param4 = match args[3] {
            wasmtime::Val::I32(i) => i as f32,
            wasmtime::Val::F32(f) => f32::from_bits(f),
            _ => return Err(WasmRuntimeError::ExecutionError("param4 must be i32 or f32".to_string())),
        };
        let param5 = match args[4] {
            wasmtime::Val::I32(i) => i as f32,
            wasmtime::Val::F32(f) => f32::from_bits(f),
            _ => return Err(WasmRuntimeError::ExecutionError("param5 must be i32 or f32".to_string())),
        };
        let param6 = match args[5] {
            wasmtime::Val::I32(i) => i as f32,
            wasmtime::Val::F32(f) => f32::from_bits(f),
            _ => return Err(WasmRuntimeError::ExecutionError("param6 must be i32 or f32".to_string())),
        };
        let param7 = match args[6] {
            wasmtime::Val::I32(i) => i as f32,
            wasmtime::Val::F32(f) => f32::from_bits(f),
            _ => return Err(WasmRuntimeError::ExecutionError("param7 must be i32 or f32".to_string())),
        };
        let command = match args[7] {
            wasmtime::Val::I32(i) => i as u16,
            _ => return Err(WasmRuntimeError::ExecutionError("command must be i32".to_string())),
        };

        // Create vehicle command
        let vehicle_command = VehicleCommand {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            param1,
            param2,
            param3,
            param4,
            param5,
            param6,
            param7,
            command,
            target_system: 1,
            target_component: 1,
            source_system: 255,
            source_component: 190,
            confirmation: 0,
        };

        // For now, simulate sending the command
        tracing::info!("Sending vehicle command to PX4: {:?}", vehicle_command);
        
        // In a real implementation, this would send the command via MAVLink
        // let mut conn = self.connection.lock().unwrap();
        // if let Some(ref mut connection) = *conn {
        //     connection.send(&MavMessage::COMMAND_LONG(vehicle_command))?;
        // }

        Ok(())
    }

    /// Get vehicle status from PX4
    pub fn get_vehicle_status(
        &self,
        caller: &mut Caller<'_, WasmContext>,
        args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        if args.len() < 1 {
            return Err(WasmRuntimeError::ExecutionError(
                "get_vehicle_status requires 1 parameter (memory address)".to_string()
            ));
        }

        let memory_address = match args[0] {
            wasmtime::Val::I32(addr) => addr as usize,
            _ => return Err(WasmRuntimeError::ExecutionError("memory address must be i32".to_string())),
        };

        // Get or create simulated vehicle status
        let vehicle_status = {
            let mut status = self.vehicle_status.lock().unwrap();
            if status.is_none() {
                *status = Some(VehicleStatus {
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    armed: false,
                    armed_time: 0,
                    takeoff_time: 0,
                    nav_state: 0, // MANUAL
                    nav_state_timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    failure_detector_status: 0,
                    hil_state: 0,
                    vehicle_type: 2, // MAV_TYPE_QUADROTOR
                    is_vtol: false,
                    is_vtol_tailsitter: false,
                    vtol_fw_permanent_stab: false,
                    in_transition_mode: false,
                    in_transition_to_fw: false,
                    rc_signal_lost: false,
                    rc_input_mode: 0,
                    data_link_lost: false,
                    data_link_lost_counter: 0,
                    high_latency_data_link_lost: false,
                    engine_failure: false,
                    mission_failure: false,
                });
            }
            status.clone().unwrap()
        };

        // Serialize vehicle status to JSON
        let json_data = serde_json::to_string(&vehicle_status)
            .map_err(|e| WasmRuntimeError::ExecutionError(format!("Failed to serialize vehicle status: {}", e)))?;

        // Write to WASM memory
        write_string_to_memory(caller, memory_address as i32, json_data.as_bytes())?;

        tracing::info!("Retrieved vehicle status: {:?}", vehicle_status);
        Ok(())
    }

    /// Get battery status from PX4
    pub fn get_battery_status(
        &self,
        caller: &mut Caller<'_, WasmContext>,
        args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        if args.len() < 1 {
            return Err(WasmRuntimeError::ExecutionError(
                "get_battery_status requires 1 parameter (memory address)".to_string()
            ));
        }

        let memory_address = match args[0] {
            wasmtime::Val::I32(addr) => addr as usize,
            _ => return Err(WasmRuntimeError::ExecutionError("memory address must be i32".to_string())),
        };

        // Get or create simulated battery status
        let battery_status = {
            let mut status = self.battery_status.lock().unwrap();
            if status.is_none() {
                *status = Some(BatteryStatus {
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    connected: true,
                    voltage_v: 12.6,
                    voltage_filtered_v: 12.5,
                    current_a: 2.3,
                    current_filtered_a: 2.2,
                    current_average_a: 2.1,
                    discharged_mah: 150.0,
                    remaining: 0.85,
                    scale: 1.0,
                    temperature: 25.0,
                    cell_count: 3,
                    source: 0,
                    priority: 0,
                    capacity: 5000,
                    cycle_count: 10,
                    average_time_to_empty_min: 120,
                    serial_number: 12345,
                    manufacture_date: 2024,
                    state_of_health: 95,
                    max_error: 0,
                    id: 0,
                    interface_error: 0,
                });
            }
            status.clone().unwrap()
        };

        // Serialize battery status to JSON
        let json_data = serde_json::to_string(&battery_status)
            .map_err(|e| WasmRuntimeError::ExecutionError(format!("Failed to serialize battery status: {}", e)))?;

        // Write to WASM memory
        write_string_to_memory(caller, memory_address as i32, json_data.as_bytes())?;

        tracing::info!("Retrieved battery status: {:?}", battery_status);
        Ok(())
    }

    /// Get vehicle local position from PX4
    pub fn get_vehicle_local_position(
        &self,
        caller: &mut Caller<'_, WasmContext>,
        args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        if args.len() < 1 {
            return Err(WasmRuntimeError::ExecutionError(
                "get_vehicle_local_position requires 1 parameter (memory address)".to_string()
            ));
        }

        let memory_address = match args[0] {
            wasmtime::Val::I32(addr) => addr as usize,
            _ => return Err(WasmRuntimeError::ExecutionError("memory address must be i32".to_string())),
        };

        // Get or create simulated local position
        let local_position = {
            let mut position = self.local_position.lock().unwrap();
            if position.is_none() {
                *position = Some(VehicleLocalPosition {
                    timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    x: 0.0,
                    y: 0.0,
                    z: -5.0, // 5 meters altitude
                    delta_xy: [0.0, 0.0],
                    delta_z: 0.0,
                    vx: 0.0,
                    vy: 0.0,
                    vz: 0.0,
                    delta_vxy: [0.0, 0.0],
                    delta_vz: 0.0,
                    ax: 0.0,
                    ay: 0.0,
                    az: 0.0,
                    yaw: 0.0,
                    delta_yaw: 0.0,
                    xy_valid: true,
                    z_valid: true,
                    v_xy_valid: true,
                    v_z_valid: true,
                    yaw_valid: true,
                    xy_reset_counter: 0,
                    z_reset_counter: 0,
                    vxy_reset_counter: 0,
                    vz_reset_counter: 0,
                    yaw_reset_counter: 0,
                    delta_xy_reset_counter: 0,
                    delta_z_reset_counter: 0,
                    delta_vxy_reset_counter: 0,
                    delta_vz_reset_counter: 0,
                    delta_yaw_reset_counter: 0,
                    dist_bottom: 5.0,
                    dist_bottom_valid: true,
                    dist_bottom_sensor_bitfield: 1,
                    eph: 0.1,
                    epv: 0.1,
                    evh: 0.1,
                    evv: 0.1,
                    dead_reckoning: false,
                    vxy_max: 10.0,
                    vz_max: 5.0,
                    hagl_min: 0.0,
                    hagl_max: 100.0,
                    heading: 0.0,
                    heading_var: 0.1,
                    heading_good_for_control: true,
                    heading_reset_counter: 0,
                });
            }
            position.clone().unwrap()
        };

        // Serialize local position to JSON
        let json_data = serde_json::to_string(&local_position)
            .map_err(|e| WasmRuntimeError::ExecutionError(format!("Failed to serialize local position: {}", e)))?;

        // Write to WASM memory
        write_string_to_memory(caller, memory_address as i32, json_data.as_bytes())?;

        tracing::info!("Retrieved vehicle local position: {:?}", local_position);
        Ok(())
    }

    /// Arm the vehicle
    pub fn arm_vehicle(
        &self,
        caller: &mut Caller<'_, WasmContext>,
        args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Arming vehicle via PX4");
        
        // Create arm command
        let arm_command = VehicleCommand {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            param1: 1.0, // ARM
            param2: 0.0,
            param3: 0.0,
            param4: 0.0,
            param5: 0.0,
            param6: 0.0,
            param7: 0.0,
            command: 400, // MAV_CMD_COMPONENT_ARM_DISARM
            target_system: 1,
            target_component: 1,
            source_system: 255,
            source_component: 190,
            confirmation: 0,
        };

        // Update vehicle status to armed
        {
            let mut status = self.vehicle_status.lock().unwrap();
            if let Some(ref mut vehicle_status) = *status {
                vehicle_status.armed = true;
                vehicle_status.armed_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            }
        }

        tracing::info!("Vehicle armed successfully");
        Ok(())
    }

    /// Disarm the vehicle
    pub fn disarm_vehicle(
        &self,
        caller: &mut Caller<'_, WasmContext>,
        args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Disarming vehicle via PX4");
        
        // Create disarm command
        let disarm_command = VehicleCommand {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            param1: 0.0, // DISARM
            param2: 0.0,
            param3: 0.0,
            param4: 0.0,
            param5: 0.0,
            param6: 0.0,
            param7: 0.0,
            command: 400, // MAV_CMD_COMPONENT_ARM_DISARM
            target_system: 1,
            target_component: 1,
            source_system: 255,
            source_component: 190,
            confirmation: 0,
        };

        // Update vehicle status to disarmed
        {
            let mut status = self.vehicle_status.lock().unwrap();
            if let Some(ref mut vehicle_status) = *status {
                vehicle_status.armed = false;
                vehicle_status.armed_time = 0;
            }
        }

        tracing::info!("Vehicle disarmed successfully");
        Ok(())
    }

    /// Takeoff command
    pub fn takeoff(
        &self,
        caller: &mut Caller<'_, WasmContext>,
        args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        if args.len() < 1 {
            return Err(WasmRuntimeError::ExecutionError(
                "takeoff requires 1 parameter (altitude)".to_string()
            ));
        }

        let altitude = match args[0] {
            wasmtime::Val::I32(i) => i as f32,
            wasmtime::Val::F32(f) => f32::from_bits(f),
            _ => return Err(WasmRuntimeError::ExecutionError("altitude must be i32 or f32".to_string())),
        };

        tracing::info!("Sending takeoff command to altitude: {}m", altitude);
        
        // Create takeoff command
        let takeoff_command = VehicleCommand {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            param1: 0.0, // Pitch
            param2: 0.0, // Empty
            param3: 0.0, // Empty
            param4: 0.0, // Yaw
            param5: 0.0, // Latitude
            param6: 0.0, // Longitude
            param7: altitude, // Altitude
            command: 22, // MAV_CMD_NAV_TAKEOFF
            target_system: 1,
            target_component: 1,
            source_system: 255,
            source_component: 190,
            confirmation: 0,
        };

        // Update vehicle status
        {
            let mut status = self.vehicle_status.lock().unwrap();
            if let Some(ref mut vehicle_status) = *status {
                vehicle_status.takeoff_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                vehicle_status.nav_state = 4; // AUTO_TAKEOFF
            }
        }

        tracing::info!("Takeoff command sent successfully");
        Ok(())
    }

    /// Land command
    pub fn land(
        &self,
        caller: &mut Caller<'_, WasmContext>,
        args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        tracing::info!("Sending land command");
        
        // Create land command
        let land_command = VehicleCommand {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            param1: 0.0, // Abort altitude
            param2: 0.0, // Land mode
            param3: 0.0, // Empty
            param4: 0.0, // Yaw angle
            param5: 0.0, // Latitude
            param6: 0.0, // Longitude
            param7: 0.0, // Altitude
            command: 21, // MAV_CMD_NAV_LAND
            target_system: 1,
            target_component: 1,
            source_system: 255,
            source_component: 190,
            confirmation: 0,
        };

        // Update vehicle status
        {
            let mut status = self.vehicle_status.lock().unwrap();
            if let Some(ref mut vehicle_status) = *status {
                vehicle_status.nav_state = 5; // AUTO_LAND
            }
        }

        tracing::info!("Land command sent successfully");
        Ok(())
    }

}

impl HostFunctionModule for Px4HostFunctions {
    fn create_imports(&self, store: &mut Store<WasmContext>) -> WasmResult<Vec<Extern>> {
        let mut imports = Vec::new();

        // Create PX4 host functions
        let send_vehicle_command = create_wasm_function_void(store, {
            let context = self.context.clone();
            let connection = self.connection.clone();
            let vehicle_status = self.vehicle_status.clone();
            let battery_status = self.battery_status.clone();
            let local_position = self.local_position.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let px4_functions = Px4HostFunctions { 
                    context: context.clone(),
                    connection: connection.clone(),
                    vehicle_status: vehicle_status.clone(),
                    battery_status: battery_status.clone(),
                    local_position: local_position.clone(),
                };
                px4_functions.send_vehicle_command(caller, args)?;
                Ok(())
            }
        })?;

        let get_vehicle_status = create_wasm_function_void(store, {
            let context = self.context.clone();
            let connection = self.connection.clone();
            let vehicle_status = self.vehicle_status.clone();
            let battery_status = self.battery_status.clone();
            let local_position = self.local_position.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let px4_functions = Px4HostFunctions { 
                    context: context.clone(),
                    connection: connection.clone(),
                    vehicle_status: vehicle_status.clone(),
                    battery_status: battery_status.clone(),
                    local_position: local_position.clone(),
                };
                px4_functions.get_vehicle_status(caller, args)?;
                Ok(())
            }
        })?;

        let get_battery_status = create_wasm_function_void(store, {
            let context = self.context.clone();
            let connection = self.connection.clone();
            let vehicle_status = self.vehicle_status.clone();
            let battery_status = self.battery_status.clone();
            let local_position = self.local_position.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let px4_functions = Px4HostFunctions { 
                    context: context.clone(),
                    connection: connection.clone(),
                    vehicle_status: vehicle_status.clone(),
                    battery_status: battery_status.clone(),
                    local_position: local_position.clone(),
                };
                px4_functions.get_battery_status(caller, args)?;
                Ok(())
            }
        })?;

        let get_vehicle_local_position = create_wasm_function_void(store, {
            let context = self.context.clone();
            let connection = self.connection.clone();
            let vehicle_status = self.vehicle_status.clone();
            let battery_status = self.battery_status.clone();
            let local_position = self.local_position.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let px4_functions = Px4HostFunctions { 
                    context: context.clone(),
                    connection: connection.clone(),
                    vehicle_status: vehicle_status.clone(),
                    battery_status: battery_status.clone(),
                    local_position: local_position.clone(),
                };
                px4_functions.get_vehicle_local_position(caller, args)?;
                Ok(())
            }
        })?;

        let arm_vehicle = create_wasm_function_void(store, {
            let context = self.context.clone();
            let connection = self.connection.clone();
            let vehicle_status = self.vehicle_status.clone();
            let battery_status = self.battery_status.clone();
            let local_position = self.local_position.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let px4_functions = Px4HostFunctions { 
                    context: context.clone(),
                    connection: connection.clone(),
                    vehicle_status: vehicle_status.clone(),
                    battery_status: battery_status.clone(),
                    local_position: local_position.clone(),
                };
                px4_functions.arm_vehicle(caller, args)?;
                Ok(())
            }
        })?;

        let disarm_vehicle = create_wasm_function_void(store, {
            let context = self.context.clone();
            let connection = self.connection.clone();
            let vehicle_status = self.vehicle_status.clone();
            let battery_status = self.battery_status.clone();
            let local_position = self.local_position.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let px4_functions = Px4HostFunctions { 
                    context: context.clone(),
                    connection: connection.clone(),
                    vehicle_status: vehicle_status.clone(),
                    battery_status: battery_status.clone(),
                    local_position: local_position.clone(),
                };
                px4_functions.disarm_vehicle(caller, args)?;
                Ok(())
            }
        })?;

        let takeoff = create_wasm_function_void(store, {
            let context = self.context.clone();
            let connection = self.connection.clone();
            let vehicle_status = self.vehicle_status.clone();
            let battery_status = self.battery_status.clone();
            let local_position = self.local_position.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let px4_functions = Px4HostFunctions { 
                    context: context.clone(),
                    connection: connection.clone(),
                    vehicle_status: vehicle_status.clone(),
                    battery_status: battery_status.clone(),
                    local_position: local_position.clone(),
                };
                px4_functions.takeoff(caller, args)?;
                Ok(())
            }
        })?;

        let land = create_wasm_function_void(store, {
            let context = self.context.clone();
            let connection = self.connection.clone();
            let vehicle_status = self.vehicle_status.clone();
            let battery_status = self.battery_status.clone();
            let local_position = self.local_position.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let px4_functions = Px4HostFunctions { 
                    context: context.clone(),
                    connection: connection.clone(),
                    vehicle_status: vehicle_status.clone(),
                    battery_status: battery_status.clone(),
                    local_position: local_position.clone(),
                };
                px4_functions.land(caller, args)?;
                Ok(())
            }
        })?;

        // Add functions to imports
        imports.push(Extern::Func(send_vehicle_command));
        imports.push(Extern::Func(get_vehicle_status));
        imports.push(Extern::Func(get_battery_status));
        imports.push(Extern::Func(get_vehicle_local_position));
        imports.push(Extern::Func(arm_vehicle));
        imports.push(Extern::Func(disarm_vehicle));
        imports.push(Extern::Func(takeoff));
        imports.push(Extern::Func(land));

        Ok(imports)
    }
}
