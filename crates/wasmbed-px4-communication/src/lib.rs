// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};
use tokio::time::interval;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

use wasmbed_microros_bridge::MicroRosBridge;
use wasmbed_fastdds_middleware::FastDdsMiddleware;

/// PX4 Communication Bridge
/// 
/// This bridge integrates microROS and FastDDS middleware to provide
/// comprehensive communication capabilities for PX4 autopilot systems.
pub struct Px4CommunicationBridge {
    /// microROS bridge for ROS 2 communication
    microros_bridge: Arc<MicroRosBridge>,
    
    /// FastDDS middleware for DDS communication
    fastdds: Arc<FastDdsMiddleware>,
    
    /// PX4 topic manager for topic configuration
    px4_topics: Arc<Px4TopicManager>,
    
    /// Bridge configuration
    config: Px4BridgeConfig,
    
    /// Bridge state
    state: Arc<RwLock<Px4BridgeState>>,
}

/// PX4 topic manager
#[derive(Debug)]
pub struct Px4TopicManager {
    /// Input topics (commands to PX4)
    pub input_topics: DashMap<String, Px4TopicInfo>,
    
    /// Output topics (status from PX4)
    pub output_topics: DashMap<String, Px4TopicInfo>,
    
    /// Topic type mappings
    pub topic_types: DashMap<String, String>,
}

/// PX4 topic information
#[derive(Debug, Clone)]
pub struct Px4TopicInfo {
    /// Topic name
    pub name: String,
    
    /// Topic type
    pub topic_type: String,
    
    /// Message size estimate
    pub message_size: usize,
}

/// PX4 bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Px4BridgeConfig {
    /// DDS domain ID
    pub dds_domain_id: u32,
    
    /// Node name
    pub node_name: String,
    
    /// PX4 system ID
    pub system_id: u8,
    
    /// PX4 component ID
    pub component_id: u8,
    
    /// MAVLink protocol version
    pub mavlink_version: u8,
    
    /// Heartbeat interval
    pub heartbeat_interval: Duration,
    
    /// Command timeout
    pub command_timeout: Duration,
}

/// PX4 bridge state
#[derive(Debug, Clone)]
pub struct Px4BridgeState {
    /// Bridge initialization status
    pub initialized: bool,
    
    /// Connection status
    pub connected: bool,
    
    /// Last heartbeat time
    pub last_heartbeat: Option<SystemTime>,
    
    /// Active topics count
    pub active_topics: usize,
    
    /// Error count
    pub error_count: u64,
}

/// PX4 bridge error types
#[derive(Debug, thiserror::Error)]
pub enum Px4BridgeError {
    #[error("microROS error: {0}")]
    MicroRosError(String),
    
    #[error("FastDDS error: {0}")]
    FastDdsError(String),
    
    #[error("PX4 communication error: {0}")]
    Px4Error(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Topic not found: {0}")]
    TopicNotFound(String),
    
    #[error("Bridge not initialized")]
    NotInitialized,
    
    #[error("Bridge not connected")]
    NotConnected,
}

impl Px4CommunicationBridge {
    /// Create a new PX4 communication bridge
    pub async fn new(config: Px4BridgeConfig) -> Result<Self, Px4BridgeError> {
        info!("Creating PX4 communication bridge with config: {:?}", config);
        
        // Create microROS bridge
        let microros_config = wasmbed_microros_bridge::BridgeConfig {
            dds_domain_id: config.dds_domain_id,
            node_name: config.node_name.clone(),
            qos: wasmbed_microros_bridge::QosConfig::default(),
            px4_config: wasmbed_microros_bridge::Px4Config {
                system_id: config.system_id,
                component_id: config.component_id,
                mavlink_version: config.mavlink_version,
                heartbeat_interval: config.heartbeat_interval,
                command_timeout: config.command_timeout,
            },
        };
        
        let microros_bridge = MicroRosBridge::new(microros_config).await
            .map_err(|e| Px4BridgeError::MicroRosError(e.to_string()))?;
        
        // Create FastDDS middleware
        let fastdds_config = wasmbed_fastdds_middleware::MiddlewareConfig {
            domain_id: config.dds_domain_id,
            qos: wasmbed_fastdds_middleware::QosConfig::default(),
            transport: wasmbed_fastdds_middleware::TransportConfig::default(),
        };
        
        let fastdds = FastDdsMiddleware::new(fastdds_config).await
            .map_err(|e| Px4BridgeError::FastDdsError(e.to_string()))?;
        
        // Create PX4 topic manager
        let px4_topics = Arc::new(Px4TopicManager {
            input_topics: DashMap::new(),
            output_topics: DashMap::new(),
            topic_types: DashMap::new(),
        });
        
        // Initialize bridge state
        let state = Arc::new(RwLock::new(Px4BridgeState {
            initialized: false,
            connected: false,
            last_heartbeat: None,
            active_topics: 0,
            error_count: 0,
        }));
        
        let bridge = Self {
            microros_bridge: Arc::new(microros_bridge),
            fastdds: Arc::new(fastdds),
            px4_topics,
            config,
            state,
        };
        
        info!("PX4 communication bridge created successfully");
        Ok(bridge)
    }
    
    /// Initialize the bridge
    #[instrument(skip(self))]
    pub async fn initialize(&self) -> Result<(), Px4BridgeError> {
        info!("Initializing PX4 communication bridge");
        
        // Initialize microROS bridge
        self.microros_bridge.initialize().await
            .map_err(|e| Px4BridgeError::MicroRosError(e.to_string()))?;
        
        // Initialize FastDDS middleware
        self.fastdds.initialize().await
            .map_err(|e| Px4BridgeError::FastDdsError(e.to_string()))?;
        
        // Configure PX4 topics
        self.configure_px4_topics().await?;
        
        // Start heartbeat task
        self.start_heartbeat_task().await?;
        
        // Update bridge state
        {
            let mut state = self.state.write().await;
            state.initialized = true;
            state.connected = true;
        }
        
        info!("PX4 communication bridge initialized successfully");
        Ok(())
    }
    
    /// Configure PX4 topics
    async fn configure_px4_topics(&self) -> Result<(), Px4BridgeError> {
        info!("Configuring PX4 topics");
        
        // Configure input topics (commands to PX4)
        let input_topics = vec![
            ("/fmu/in/vehicle_command", "px4_msgs::msg::VehicleCommand"),
            ("/fmu/in/position_setpoint", "px4_msgs::msg::PositionSetpoint"),
            ("/fmu/in/attitude_setpoint", "px4_msgs::msg::AttitudeSetpoint"),
            ("/fmu/in/trajectory_setpoint", "px4_msgs::msg::TrajectorySetpoint"),
            ("/fmu/in/offboard_control_mode", "px4_msgs::msg::OffboardControlMode"),
        ];
        
        // Configure output topics (status from PX4)
        let output_topics = vec![
            ("/fmu/out/vehicle_status", "px4_msgs::msg::VehicleStatus"),
            ("/fmu/out/vehicle_local_position", "px4_msgs::msg::VehicleLocalPosition"),
            ("/fmu/out/battery_status", "px4_msgs::msg::BatteryStatus"),
            ("/fmu/out/vehicle_attitude", "px4_msgs::msg::VehicleAttitude"),
            ("/fmu/out/actuator_outputs", "px4_msgs::msg::ActuatorOutputs"),
            ("/fmu/out/sensor_combined", "px4_msgs::msg::SensorCombined"),
            ("/fmu/out/gps_position", "px4_msgs::msg::GpsPosition"),
        ];
        
        // Register input topics
        for (topic_name, topic_type) in input_topics {
            let topic_info = Px4TopicInfo {
                name: topic_name.to_string(),
                topic_type: topic_type.to_string(),
                message_size: 1024, // Default message size
            };
            
            self.px4_topics.input_topics.insert(topic_name.to_string(), topic_info);
            self.px4_topics.topic_types.insert(topic_name.to_string(), topic_type.to_string());
        }
        
        // Register output topics
        for (topic_name, topic_type) in output_topics {
            let topic_info = Px4TopicInfo {
                name: topic_name.to_string(),
                topic_type: topic_type.to_string(),
                message_size: 1024, // Default message size
            };
            
            self.px4_topics.output_topics.insert(topic_name.to_string(), topic_info);
            self.px4_topics.topic_types.insert(topic_name.to_string(), topic_type.to_string());
        }
        
        info!("PX4 topics configured: {} input, {} output", 
              self.px4_topics.input_topics.len(), 
              self.px4_topics.output_topics.len());
        
        Ok(())
    }
    
    /// Send MAVLink command to PX4
    pub async fn send_mavlink_command(&self, command_id: u16, param1: f32) -> Result<(), Px4BridgeError> {
        debug!("Sending MAVLink command: {} with param1: {}", command_id, param1);
        
        // Create command message
        let command_data = format!("{{\"command_id\":{},\"param1\":{}}}", command_id, param1);
        
        // Send via microROS bridge
        self.microros_bridge.publish_message("/fmu/in/vehicle_command", command_data.into_bytes()).await
            .map_err(|e| Px4BridgeError::MicroRosError(e.to_string()))?;
        
        debug!("MAVLink command sent successfully");
        Ok(())
    }
    
    /// Subscribe to PX4 status topic
    pub async fn subscribe_px4_status(&self, topic_name: &str) -> Result<(), Px4BridgeError> {
        debug!("Subscribing to PX4 status topic: {}", topic_name);
        
        // Subscribe via microROS bridge
        self.microros_bridge.subscribe_to_topic(topic_name).await
            .map_err(|e| Px4BridgeError::MicroRosError(e.to_string()))?;
        
        debug!("Subscribed to PX4 status topic: {}", topic_name);
        Ok(())
    }
    
    /// Publish PX4 command
    pub async fn publish_px4_command(&self, topic_name: &str, data: Vec<u8>) -> Result<(), Px4BridgeError> {
        debug!("Publishing PX4 command to topic: {}", topic_name);
        
        // Publish via microROS bridge
        self.microros_bridge.publish_message(topic_name, data).await
            .map_err(|e| Px4BridgeError::MicroRosError(e.to_string()))?;
        
        debug!("PX4 command published successfully");
        Ok(())
    }
    
    /// Start heartbeat task
    async fn start_heartbeat_task(&self) -> Result<(), Px4BridgeError> {
        info!("Starting heartbeat task");
        
        let state = self.state.clone();
        let mut interval = interval(self.config.heartbeat_interval);
        
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                
                let mut state = state.write().await;
                state.last_heartbeat = Some(SystemTime::now());
            }
        });
        
        Ok(())
    }
    
    /// Get bridge status
    pub async fn get_status(&self) -> Px4BridgeState {
        self.state.read().await.clone()
    }
    
    /// Shutdown the bridge
    pub async fn shutdown(&self) -> Result<(), Px4BridgeError> {
        info!("Shutting down PX4 communication bridge");
        
        // Update bridge state
        {
            let mut state = self.state.write().await;
            state.connected = false;
            state.initialized = false;
        }
        
        info!("PX4 communication bridge shutdown complete");
        Ok(())
    }
}

impl Default for Px4BridgeConfig {
    fn default() -> Self {
        Self {
            dds_domain_id: 0,
            node_name: "px4_bridge".to_string(),
            system_id: 1,
            component_id: 1,
            mavlink_version: 2,
            heartbeat_interval: Duration::from_secs(1),
            command_timeout: Duration::from_secs(5),
        }
    }
}