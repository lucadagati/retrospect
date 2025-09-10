// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use crate::context::WasmContext;
use crate::error::{WasmResult, WasmRuntimeError};
use crate::host_functions::{HostFunctionModule, create_wasm_function_void, extract_string_from_memory, write_string_to_memory};
use rustdds::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use wasmtime::*;

/// microROS/DDS host functions for ROS 2 communication
pub struct MicroRosHostFunctions {
    context: Arc<WasmContext>,
    participant: Option<DomainParticipant>,
}

/// ROS 2 message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RosMessage {
    pub topic: String,
    pub message_type: String,
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub sequence_number: u64,
}

/// ROS 2 topic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicInfo {
    pub name: String,
    pub message_type: String,
    pub qos_profile: String,
    pub publisher_count: u32,
    pub subscriber_count: u32,
}

/// ROS 2 node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub name: String,
    pub namespace: String,
    pub publisher_count: u32,
    pub subscriber_count: u32,
    pub service_count: u32,
    pub client_count: u32,
}

impl MicroRosHostFunctions {
    /// Create new microROS host functions
    pub fn new(context: Arc<WasmContext>) -> WasmResult<Self> {
        // Initialize DDS participant (simulated for now)
        let participant = None; // Would create actual DDS participant here
        
        Ok(Self {
            context,
            participant,
        })
    }

    /// Publish a ROS 2 message
    pub fn publish_message(
        &self,
        caller: &mut Caller<'_, WasmContext>,
        args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        if args.len() < 3 {
            return Err(WasmRuntimeError::ExecutionError(
                "publish_message requires 3 parameters (topic_ptr, topic_len, data_ptr, data_len)".to_string()
            ));
        }

        let topic_ptr = match args[0] {
            wasmtime::Val::I32(ptr) => ptr,
            _ => return Err(WasmRuntimeError::ExecutionError("topic_ptr must be i32".to_string())),
        };
        let topic_len = match args[1] {
            wasmtime::Val::I32(len) => len,
            _ => return Err(WasmRuntimeError::ExecutionError("topic_len must be i32".to_string())),
        };
        let data_ptr = match args[2] {
            wasmtime::Val::I32(ptr) => ptr,
            _ => return Err(WasmRuntimeError::ExecutionError("data_ptr must be i32".to_string())),
        };
        let data_len = match args[3] {
            wasmtime::Val::I32(len) => len,
            _ => return Err(WasmRuntimeError::ExecutionError("data_len must be i32".to_string())),
        };

        // Extract topic name from WASM memory
        let topic = extract_string_from_memory(caller, topic_ptr, topic_len)?;
        
        // Extract message data from WASM memory
        let message_data = extract_string_from_memory(caller, data_ptr, data_len)?;

        // Create ROS message
        let ros_message = RosMessage {
            topic: topic.clone(),
            message_type: "std_msgs/msg/String".to_string(),
            data: message_data.into_bytes(),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            sequence_number: 0,
        };

        tracing::info!("Publishing ROS message to topic '{}': {:?}", topic, ros_message);
        
        // In a real implementation, this would publish via DDS
        // if let Some(ref participant) = self.participant {
        //     participant.publish(&ros_message)?;
        // }

        Ok(())
    }

    /// Subscribe to a ROS 2 topic
    pub fn subscribe_topic(
        &self,
        caller: &mut Caller<'_, WasmContext>,
        args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        if args.len() < 2 {
            return Err(WasmRuntimeError::ExecutionError(
                "subscribe_topic requires 2 parameters (topic_ptr, topic_len)".to_string()
            ));
        }

        let topic_ptr = match args[0] {
            wasmtime::Val::I32(ptr) => ptr,
            _ => return Err(WasmRuntimeError::ExecutionError("topic_ptr must be i32".to_string())),
        };
        let topic_len = match args[1] {
            wasmtime::Val::I32(len) => len,
            _ => return Err(WasmRuntimeError::ExecutionError("topic_len must be i32".to_string())),
        };

        // Extract topic name from WASM memory
        let topic = extract_string_from_memory(caller, topic_ptr, topic_len)?;

        tracing::info!("Subscribing to ROS topic: {}", topic);
        
        // In a real implementation, this would create a DDS subscriber
        // if let Some(ref participant) = self.participant {
        //     participant.subscribe(&topic, callback)?;
        // }

        Ok(())
    }

    /// Get available ROS 2 topics
    pub fn get_available_topics(
        &self,
        caller: &mut Caller<'_, WasmContext>,
        args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        if args.len() < 1 {
            return Err(WasmRuntimeError::ExecutionError(
                "get_available_topics requires 1 parameter (memory address)".to_string()
            ));
        }

        let memory_address = match args[0] {
            wasmtime::Val::I32(addr) => addr as usize,
            _ => return Err(WasmRuntimeError::ExecutionError("memory address must be i32".to_string())),
        };

        // Get simulated available topics
        let topics = self.simulate_available_topics()?;

        // Serialize topics to JSON
        let json_data = serde_json::to_string(&topics)
            .map_err(|e| WasmRuntimeError::ExecutionError(format!("Failed to serialize topics: {}", e)))?;

        // Write to WASM memory
        write_string_to_memory(caller, memory_address as i32, json_data.as_bytes())?;

        tracing::info!("Retrieved {} available ROS topics", topics.len());
        Ok(())
    }

    /// Get ROS 2 node information
    pub fn get_node_info(
        &self,
        caller: &mut Caller<'_, WasmContext>,
        args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        if args.len() < 1 {
            return Err(WasmRuntimeError::ExecutionError(
                "get_node_info requires 1 parameter (memory address)".to_string()
            ));
        }

        let memory_address = match args[0] {
            wasmtime::Val::I32(addr) => addr as usize,
            _ => return Err(WasmRuntimeError::ExecutionError("memory address must be i32".to_string())),
        };

        // Get simulated node information
        let node_info = self.simulate_node_info()?;

        // Serialize node info to JSON
        let json_data = serde_json::to_string(&node_info)
            .map_err(|e| WasmRuntimeError::ExecutionError(format!("Failed to serialize node info: {}", e)))?;

        // Write to WASM memory
        write_string_to_memory(caller, memory_address as i32, json_data.as_bytes())?;

        tracing::info!("Retrieved ROS node info: {:?}", node_info);
        Ok(())
    }

    /// Create a ROS 2 service client
    pub fn create_service_client(
        &self,
        caller: &mut Caller<'_, WasmContext>,
        args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        if args.len() < 2 {
            return Err(WasmRuntimeError::ExecutionError(
                "create_service_client requires 2 parameters (service_ptr, service_len)".to_string()
            ));
        }

        let service_ptr = match args[0] {
            wasmtime::Val::I32(ptr) => ptr,
            _ => return Err(WasmRuntimeError::ExecutionError("service_ptr must be i32".to_string())),
        };
        let service_len = match args[1] {
            wasmtime::Val::I32(len) => len,
            _ => return Err(WasmRuntimeError::ExecutionError("service_len must be i32".to_string())),
        };

        // Extract service name from WASM memory
        let service_name = extract_string_from_memory(caller, service_ptr, service_len)?;

        tracing::info!("Creating ROS service client for: {}", service_name);
        
        // In a real implementation, this would create a DDS service client
        // if let Some(ref participant) = self.participant {
        //     participant.create_service_client(&service_name)?;
        // }

        Ok(())
    }

    /// Call a ROS 2 service
    pub fn call_service(
        &self,
        caller: &mut Caller<'_, WasmContext>,
        args: &[wasmtime::Val],
    ) -> Result<(), WasmRuntimeError> {
        if args.len() < 4 {
            return Err(WasmRuntimeError::ExecutionError(
                "call_service requires 4 parameters (service_ptr, service_len, request_ptr, request_len)".to_string()
            ));
        }

        let service_ptr = match args[0] {
            wasmtime::Val::I32(ptr) => ptr,
            _ => return Err(WasmRuntimeError::ExecutionError("service_ptr must be i32".to_string())),
        };
        let service_len = match args[1] {
            wasmtime::Val::I32(len) => len,
            _ => return Err(WasmRuntimeError::ExecutionError("service_len must be i32".to_string())),
        };
        let request_ptr = match args[2] {
            wasmtime::Val::I32(ptr) => ptr,
            _ => return Err(WasmRuntimeError::ExecutionError("request_ptr must be i32".to_string())),
        };
        let request_len = match args[3] {
            wasmtime::Val::I32(len) => len,
            _ => return Err(WasmRuntimeError::ExecutionError("request_len must be i32".to_string())),
        };

        // Extract service name and request data from WASM memory
        let service_name = extract_string_from_memory(caller, service_ptr, service_len)?;
        let request_data = extract_string_from_memory(caller, request_ptr, request_len)?;

        tracing::info!("Calling ROS service '{}' with request: {}", service_name, request_data);
        
        // In a real implementation, this would call the service via DDS
        // if let Some(ref participant) = self.participant {
        //     let response = participant.call_service(&service_name, &request_data)?;
        //     // Write response back to WASM memory
        // }

        Ok(())
    }

    /// Simulate available ROS topics
    fn simulate_available_topics(&self) -> WasmResult<Vec<TopicInfo>> {
        Ok(vec![
            TopicInfo {
                name: "/cmd_vel".to_string(),
                message_type: "geometry_msgs/msg/Twist".to_string(),
                qos_profile: "sensor_data".to_string(),
                publisher_count: 1,
                subscriber_count: 2,
            },
            TopicInfo {
                name: "/odom".to_string(),
                message_type: "nav_msgs/msg/Odometry".to_string(),
                qos_profile: "sensor_data".to_string(),
                publisher_count: 1,
                subscriber_count: 1,
            },
            TopicInfo {
                name: "/scan".to_string(),
                message_type: "sensor_msgs/msg/LaserScan".to_string(),
                qos_profile: "sensor_data".to_string(),
                publisher_count: 1,
                subscriber_count: 1,
            },
            TopicInfo {
                name: "/camera/image_raw".to_string(),
                message_type: "sensor_msgs/msg/Image".to_string(),
                qos_profile: "sensor_data".to_string(),
                publisher_count: 1,
                subscriber_count: 0,
            },
        ])
    }

    /// Simulate node information
    fn simulate_node_info(&self) -> WasmResult<NodeInfo> {
        Ok(NodeInfo {
            name: "wasmbed_node".to_string(),
            namespace: "/".to_string(),
            publisher_count: 2,
            subscriber_count: 3,
            service_count: 1,
            client_count: 1,
        })
    }
}

impl HostFunctionModule for MicroRosHostFunctions {
    fn create_imports(&self, store: &mut Store<WasmContext>) -> WasmResult<Vec<Extern>> {
        let mut imports = Vec::new();

        // Create microROS host functions
        let publish_message = create_wasm_function_void(store, {
            let context = self.context.clone();
            let participant = self.participant.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let microros_functions = MicroRosHostFunctions { context: context.clone(), participant: participant.clone() };
                microros_functions.publish_message(caller, args)?;
                Ok(())
            }
        })?;

        let subscribe_topic = create_wasm_function_void(store, {
            let context = self.context.clone();
            let participant = self.participant.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let microros_functions = MicroRosHostFunctions { context: context.clone(), participant: participant.clone() };
                microros_functions.subscribe_topic(caller, args)?;
                Ok(())
            }
        })?;

        let get_available_topics = create_wasm_function_void(store, {
            let context = self.context.clone();
            let participant = self.participant.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let microros_functions = MicroRosHostFunctions { context: context.clone(), participant: participant.clone() };
                microros_functions.get_available_topics(caller, args)?;
                Ok(())
            }
        })?;

        let get_node_info = create_wasm_function_void(store, {
            let context = self.context.clone();
            let participant = self.participant.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let microros_functions = MicroRosHostFunctions { context: context.clone(), participant: participant.clone() };
                microros_functions.get_node_info(caller, args)?;
                Ok(())
            }
        })?;

        let create_service_client = create_wasm_function_void(store, {
            let context = self.context.clone();
            let participant = self.participant.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let microros_functions = MicroRosHostFunctions { context: context.clone(), participant: participant.clone() };
                microros_functions.create_service_client(caller, args)?;
                Ok(())
            }
        })?;

        let call_service = create_wasm_function_void(store, {
            let context = self.context.clone();
            let participant = self.participant.clone();
            move |caller: &mut Caller<'_, WasmContext>, args: &[wasmtime::Val]| {
                let microros_functions = MicroRosHostFunctions { context: context.clone(), participant: participant.clone() };
                microros_functions.call_service(caller, args)?;
                Ok(())
            }
        })?;

        // Add functions to imports
        imports.push(Extern::Func(publish_message));
        imports.push(Extern::Func(subscribe_topic));
        imports.push(Extern::Func(get_available_topics));
        imports.push(Extern::Func(get_node_info));
        imports.push(Extern::Func(create_service_client));
        imports.push(Extern::Func(call_service));

        Ok(imports)
    }
}
