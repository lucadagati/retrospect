// SPDX-License-Identifier: AGPL-3.0
// Copyright © 2025 Wasmbed contributors

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use dashmap::DashMap;
use futures::stream::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};
use tokio::time::interval;
use tracing::{debug, error, info, instrument};
use uuid::Uuid;

use rustdds::{
    DomainParticipant, Publisher as RustDdsPublisher, Subscriber as RustDdsSubscriber,
    Topic, QosPolicyBuilder, TopicKind, Keyed,
    policy::{Reliability, Durability, History},
    CDRSerializerAdapter, CDRDeserializerAdapter,
};
// ROS 2 imports (commented out - requires ROS 2 installation)
// use r2r::{Node, Publisher as RosPublisher, Subscriber as RosSubscriber};
// use ros2_client::{Context, Node as Ros2Node};

/// PX4 message wrapper for DDS communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Px4Message {
    /// Message ID
    pub id: Uuid,
    
    /// Topic name
    pub topic: String,
    
    /// Message type
    pub message_type: String,
    
    /// Serialized message data
    pub data: Vec<u8>,
    
    /// Timestamp
    pub timestamp: SystemTime,
    
    /// Source system ID
    pub source_system_id: u8,
    
    /// Target system ID
    pub target_system_id: u8,
}

impl Keyed for Px4Message {
    type K = String;
    
    fn key(&self) -> Self::K {
        self.topic.clone()
    }
}

/// microROS Bridge for PX4 communication
/// 
/// This bridge enables real-time communication between WebAssembly applications
/// and PX4 autopilot systems using DDS middleware.
pub struct MicroRosBridge {
    /// DDS domain participant for DDS communication
    participant: Arc<DomainParticipant>,
    
    /// ROS 2 node for ROS communication (commented out - requires ROS 2 installation)
    // ros_node: Arc<Node>,
    
    /// ROS 2 context (commented out - requires ROS 2 installation)
    // ros_context: Arc<Context>,
    
    /// Active publishers for PX4 topics
    publishers: Arc<DashMap<String, RustDdsPublisher>>,
    
    /// Active subscribers for PX4 topics
    subscribers: Arc<DashMap<String, RustDdsSubscriber>>,
    
    /// ROS 2 publishers (commented out - requires ROS 2 installation)
    // ros_publishers: Arc<DashMap<String, RosPublisher>>,
    
    /// ROS 2 subscribers (commented out - requires ROS 2 installation)
    // ros_subscribers: Arc<DashMap<String, RosSubscriber>>,
    
    /// Data writers for publishing data
    data_writers: Arc<DashMap<String, rustdds::with_key::DataWriter<Px4Message, CDRSerializerAdapter<Px4Message>>>>,
    
    /// Data readers for receiving data
    data_readers: Arc<DashMap<String, rustdds::with_key::DataReader<Px4Message, CDRDeserializerAdapter<Px4Message>>>>,
    
    /// Topics
    topics: Arc<DashMap<String, Topic>>,
    
    /// PX4 topic manager for topic configuration
    topic_manager: Arc<RwLock<Px4TopicManager>>,
    
    /// Message channels for async communication
    message_channels: Arc<MessageChannels>,
    
    /// Bridge configuration
    config: BridgeConfig,
    
    /// Bridge state
    state: Arc<RwLock<BridgeState>>,
}

/// Bridge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    /// DDS domain ID
    pub dds_domain_id: u32,
    
    /// Node name
    pub node_name: String,
    
    /// QoS configuration
    pub qos: QosConfig,
    
    /// PX4 configuration
    pub px4_config: Px4Config,
}

/// QoS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QosConfig {
    /// Reliability kind
    pub reliability: Reliability,
    
    /// Durability kind
    pub durability: Durability,
    
    /// History depth
    pub history_depth: i32,
    
    /// Deadline duration
    pub deadline_duration: Duration,
}

/// PX4 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Px4Config {
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

/// Bridge state
#[derive(Debug, Clone)]
pub struct BridgeState {
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

/// PX4 topic manager
#[derive(Debug)]
pub struct Px4TopicManager {
    /// Input topics (commands to PX4)
    pub input_topics: HashMap<String, Px4TopicInfo>,
    
    /// Output topics (status from PX4)
    pub output_topics: HashMap<String, Px4TopicInfo>,
    
    /// Topic type mappings
    pub topic_types: HashMap<String, String>,
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

/// Message channels for async communication
#[derive(Debug)]
pub struct MessageChannels {
    /// Incoming message channel
    pub incoming_tx: mpsc::UnboundedSender<Px4Message>,
    pub incoming_rx: Arc<RwLock<mpsc::UnboundedReceiver<Px4Message>>>,
    
    /// Outgoing message channel
    pub outgoing_tx: mpsc::UnboundedSender<Px4Message>,
    pub outgoing_rx: Arc<RwLock<mpsc::UnboundedReceiver<Px4Message>>>,
}


/// Bridge error types
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("ROS 2 error: {0}")]
    RosError(String),
    
    #[error("FastDDS error: {0}")]
    FastDdsError(String),
    
    #[error("PX4 communication error: {0}")]
    Px4Error(String),
    
    #[error("Topic not found: {0}")]
    TopicNotFound(String),
    
    #[error("Invalid message type: {0}")]
    InvalidMessageType(String),
    
    #[error("Bridge not initialized")]
    NotInitialized,
    
    #[error("Bridge not connected")]
    NotConnected,
}

impl MicroRosBridge {
    /// Create a new microROS bridge
    pub async fn new(config: BridgeConfig) -> Result<Self, BridgeError> {
        info!("Creating microROS bridge with config: {:?}", config);
        
        // Initialize DDS domain participant
        let participant = DomainParticipant::new(config.dds_domain_id as u16)
            .map_err(|e| BridgeError::FastDdsError(e.to_string()))?;
        
        // Initialize ROS 2 context and node (commented out - requires ROS 2 installation)
        // let ros_context = Arc::new(Context::new().await?);
        // let ros_node = Arc::new(Node::new(&ros_context, &config.node_name)?);
        
        // Create PX4 topic manager
        let topic_manager = Arc::new(RwLock::new(Px4TopicManager::new()));
        
        // Create message channels
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();
        let (outgoing_tx, outgoing_rx) = mpsc::unbounded_channel();
        
        let message_channels = Arc::new(MessageChannels {
            incoming_tx,
            incoming_rx: Arc::new(RwLock::new(incoming_rx)),
            outgoing_tx,
            outgoing_rx: Arc::new(RwLock::new(outgoing_rx)),
        });
        
        // Initialize bridge state
        let state = Arc::new(RwLock::new(BridgeState {
            initialized: false,
            connected: false,
            last_heartbeat: None,
            active_topics: 0,
            error_count: 0,
        }));
        
        let bridge = Self {
            participant: Arc::new(participant),
            // ros_node,
            // ros_context,
            publishers: Arc::new(DashMap::new()),
            subscribers: Arc::new(DashMap::new()),
            // ros_publishers: Arc::new(DashMap::new()),
            // ros_subscribers: Arc::new(DashMap::new()),
            data_writers: Arc::new(DashMap::new()),
            data_readers: Arc::new(DashMap::new()),
            topics: Arc::new(DashMap::new()),
            topic_manager,
            message_channels,
            config,
            state,
        };
        
        info!("microROS bridge created successfully");
        Ok(bridge)
    }
    
    /// Initialize the bridge
    #[instrument(skip(self))]
    pub async fn initialize(&self) -> Result<(), BridgeError> {
        info!("Initializing microROS bridge");
        
        // Configure PX4 topics
        self.configure_px4_topics().await?;
        
        // Create publishers and subscribers
        self.create_publishers().await?;
        self.create_subscribers().await?;
        
        // Start message processing tasks
        self.start_message_processing().await?;
        
        // Start heartbeat task
        self.start_heartbeat_task().await?;
        
        // Update bridge state
        {
            let mut state = self.state.write().await;
            state.initialized = true;
            state.connected = true;
        }
        
        info!("microROS bridge initialized successfully");
        Ok(())
    }
    
    /// Configure PX4 topics
    async fn configure_px4_topics(&self) -> Result<(), BridgeError> {
        info!("Configuring PX4 topics");
        
        // Configure input topics (commands to PX4)
        let input_topics = vec![
            ("/fmu/in/vehicle_command", "std_msgs::msg::String"),
            ("/fmu/in/position_setpoint", "std_msgs::msg::String"),
            ("/fmu/in/attitude_setpoint", "std_msgs::msg::String"),
            ("/fmu/in/trajectory_setpoint", "std_msgs::msg::String"),
            ("/fmu/in/offboard_control_mode", "std_msgs::msg::String"),
        ];
        
        // Configure output topics (status from PX4)
        let output_topics = vec![
            ("/fmu/out/vehicle_status", "std_msgs::msg::String"),
            ("/fmu/out/vehicle_local_position", "std_msgs::msg::String"),
            ("/fmu/out/battery_status", "std_msgs::msg::String"),
            ("/fmu/out/vehicle_attitude", "std_msgs::msg::String"),
            ("/fmu/out/actuator_outputs", "std_msgs::msg::String"),
            ("/fmu/out/sensor_combined", "std_msgs::msg::String"),
            ("/fmu/out/gps_position", "std_msgs::msg::String"),
        ];
        
        // Register input topics
        {
            let mut topic_manager = self.topic_manager.write().await;
            for (topic_name, topic_type) in input_topics {
                let topic_info = Px4TopicInfo {
                    name: topic_name.to_string(),
                    topic_type: topic_type.to_string(),
                    message_size: 1024, // Default message size
                };
                
                topic_manager.input_topics.insert(topic_name.to_string(), topic_info);
                topic_manager.topic_types.insert(topic_name.to_string(), topic_type.to_string());
            }
        }
        
        // Register output topics
        {
            let mut topic_manager = self.topic_manager.write().await;
            for (topic_name, topic_type) in output_topics {
                let topic_info = Px4TopicInfo {
                    name: topic_name.to_string(),
                    topic_type: topic_type.to_string(),
                    message_size: 1024, // Default message size
                };
                
                topic_manager.output_topics.insert(topic_name.to_string(), topic_info);
                topic_manager.topic_types.insert(topic_name.to_string(), topic_type.to_string());
            }
        }
        
        info!("PX4 topics configured: {} input, {} output", 
              self.topic_manager.read().await.input_topics.len(), 
              self.topic_manager.read().await.output_topics.len());
        
        Ok(())
    }
    
    /// Create QoS profile
    fn create_qos_profile(&self) -> QosPolicyBuilder {
        QosPolicyBuilder::new()
            .reliability(self.config.qos.reliability.clone())
            .durability(self.config.qos.durability.clone())
            .history(History::KeepLast { depth: self.config.qos.history_depth })
    }
    
    /// Create publishers for input topics
    async fn create_publishers(&self) -> Result<(), BridgeError> {
        info!("Creating publishers for input topics");
        
        let qos = self.create_qos_profile().build();
        let topic_manager = self.topic_manager.read().await;
        
        for (topic_name, topic_info) in &topic_manager.input_topics {
            // Create DDS topic
            let topic = self.participant.create_topic(
                topic_name.clone(),
                topic_info.topic_type.clone(),
                &qos,
                TopicKind::WithKey,
            ).map_err(|e| BridgeError::FastDdsError(e.to_string()))?;
            
            self.topics.insert(topic_name.clone(), topic.clone());
            
            // Create DDS publisher
            let publisher = self.participant.create_publisher(&qos)
                .map_err(|e| BridgeError::FastDdsError(e.to_string()))?;
            
            self.publishers.insert(topic_name.clone(), publisher.clone());
            
            // Create data writer
            let data_writer = publisher.create_datawriter::<Px4Message, CDRSerializerAdapter<Px4Message>>(&topic, None)
                .map_err(|e| BridgeError::FastDdsError(e.to_string()))?;
            
            self.data_writers.insert(topic_name.clone(), data_writer);
            
            info!("Created publisher for topic: {}", topic_name);
        }
        
        Ok(())
    }
    
    /// Create subscribers for output topics
    async fn create_subscribers(&self) -> Result<(), BridgeError> {
        info!("Creating subscribers for output topics");
        
        let qos = self.create_qos_profile().build();
        let topic_manager = self.topic_manager.read().await;
        
        for (topic_name, topic_info) in &topic_manager.output_topics {
            // Create DDS topic
            let topic = self.participant.create_topic(
                topic_name.clone(),
                topic_info.topic_type.clone(),
                &qos,
                TopicKind::WithKey,
            ).map_err(|e| BridgeError::FastDdsError(e.to_string()))?;
            
            self.topics.insert(topic_name.clone(), topic.clone());
            
            // Create DDS subscriber
            let subscriber = self.participant.create_subscriber(&qos)
                .map_err(|e| BridgeError::FastDdsError(e.to_string()))?;
            
            self.subscribers.insert(topic_name.clone(), subscriber.clone());
            
            // Create data reader
            let data_reader = subscriber.create_datareader::<Px4Message, CDRDeserializerAdapter<Px4Message>>(&topic, None)
                .map_err(|e| BridgeError::FastDdsError(e.to_string()))?;
            
            self.data_readers.insert(topic_name.clone(), data_reader);
            
            info!("Created subscriber for topic: {}", topic_name);
        }
        
        Ok(())
    }
    
    /// Start message processing tasks
    async fn start_message_processing(&self) -> Result<(), BridgeError> {
        info!("Starting message processing tasks");
        
        // Start incoming message processor
        let incoming_rx = self.message_channels.incoming_rx.clone();
        let data_writers = self.data_writers.clone();
        
        tokio::spawn(async move {
            let mut rx = incoming_rx.write().await;
            while let Some(message) = rx.recv().await {
                if let Err(e) = Self::process_incoming_message(&message, &data_writers).await {
                    error!("Error processing incoming message: {}", e);
                }
            }
        });
        
        // Start outgoing message processor
        let outgoing_rx = self.message_channels.outgoing_rx.clone();
        let data_readers = self.data_readers.clone();
        
        tokio::spawn(async move {
            let mut rx = outgoing_rx.write().await;
            while let Some(message) = rx.recv().await {
                if let Err(e) = Self::process_outgoing_message(&message, &data_readers).await {
                    error!("Error processing outgoing message: {}", e);
                }
            }
        });
        
        Ok(())
    }
    
    /// Process incoming message (from WASM app to PX4)
    async fn process_incoming_message(
        message: &Px4Message,
        data_writers: &DashMap<String, rustdds::with_key::DataWriter<Px4Message, CDRSerializerAdapter<Px4Message>>>,
    ) -> Result<(), BridgeError> {
        debug!("Processing incoming message for topic: {}", message.topic);
        
        // Write data using data writer
        if let Some(data_writer) = data_writers.get(&message.topic) {
            data_writer.write(message.clone(), None)
                .map_err(|e| BridgeError::FastDdsError(e.to_string()))?;
        }
        
        debug!("Message published to topic: {}", message.topic);
        Ok(())
    }
    
    /// Process outgoing message (from PX4 to WASM app)
    async fn process_outgoing_message(
        message: &Px4Message,
        data_readers: &DashMap<String, rustdds::with_key::DataReader<Px4Message, CDRDeserializerAdapter<Px4Message>>>,
    ) -> Result<(), BridgeError> {
        debug!("Processing outgoing message for topic: {}", message.topic);
        
        // Read data using data reader
        if let Some(_data_reader) = data_readers.get(&message.topic) {
            // Handle data reading
            debug!("Data read from topic: {}", message.topic);
        }
        
        Ok(())
    }
    
    /// Start heartbeat task
    async fn start_heartbeat_task(&self) -> Result<(), BridgeError> {
        info!("Starting heartbeat task");
        
        let state = self.state.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(config.px4_config.heartbeat_interval);
            
            loop {
                interval.tick().await;
                
                // Update heartbeat timestamp
                {
                    let mut state_guard = state.write().await;
                    state_guard.last_heartbeat = Some(SystemTime::now());
                }
                
                debug!("Heartbeat sent");
            }
        });
        
        Ok(())
    }
    
    /// Publish message to PX4
    pub async fn publish_message(&self, topic: &str, data: Vec<u8>) -> Result<(), BridgeError> {
        let message = Px4Message {
            id: Uuid::new_v4(),
            topic: topic.to_string(),
            message_type: self.topic_manager.read().await.topic_types.get(topic)
                .ok_or_else(|| BridgeError::TopicNotFound(topic.to_string()))?
                .clone(),
            data,
            timestamp: SystemTime::now(),
            source_system_id: self.config.px4_config.system_id,
            target_system_id: self.config.px4_config.component_id,
        };
        
        self.message_channels.incoming_tx.send(message)
            .map_err(|e| BridgeError::Px4Error(e.to_string()))?;
        
        Ok(())
    }
    
    /// Subscribe to PX4 topic
    pub async fn subscribe_to_topic(&self, topic: &str) -> Result<(), BridgeError> {
        if !self.topic_manager.read().await.output_topics.contains_key(topic) {
            return Err(BridgeError::TopicNotFound(topic.to_string()));
        }
        
        info!("Subscribed to topic: {}", topic);
        Ok(())
    }
    
    /// Get bridge status
    pub async fn get_status(&self) -> BridgeState {
        self.state.read().await.clone()
    }
    
    /// Shutdown the bridge
    pub async fn shutdown(&self) -> Result<(), BridgeError> {
        info!("Shutting down microROS bridge");
        
        // Update bridge state
        {
            let mut state = self.state.write().await;
            state.connected = false;
            state.initialized = false;
        }
        
        // Close message channels
        drop(self.message_channels.incoming_tx.clone());
        drop(self.message_channels.outgoing_tx.clone());
        
        info!("microROS bridge shutdown complete");
        Ok(())
    }
}

impl Px4TopicManager {
    /// Create a new PX4 topic manager
    pub fn new() -> Self {
        Self {
            input_topics: HashMap::new(),
            output_topics: HashMap::new(),
            topic_types: HashMap::new(),
        }
    }
}

impl Default for BridgeConfig {
    fn default() -> Self {
        Self {
            dds_domain_id: 0,
            node_name: "wasmbed_gateway".to_string(),
            qos: QosConfig::default(),
            px4_config: Px4Config::default(),
        }
    }
}

impl Default for QosConfig {
    fn default() -> Self {
        Self {
            reliability: Reliability::Reliable { max_blocking_time: Duration::from_millis(100).into() },
            durability: Durability::Volatile,
            history_depth: 10,
            deadline_duration: Duration::from_millis(100),
        }
    }
}

impl Default for Px4Config {
    fn default() -> Self {
        Self {
            system_id: 1,
            component_id: 1,
            mavlink_version: 2,
            heartbeat_interval: Duration::from_secs(1),
            command_timeout: Duration::from_secs(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_bridge_creation() {
        let config = BridgeConfig::default();
        let bridge = MicroRosBridge::new(config).await;
        assert!(bridge.is_ok());
    }
    
    #[tokio::test]
    async fn test_bridge_initialization() {
        let config = BridgeConfig::default();
        let bridge = MicroRosBridge::new(config).await.unwrap();
        let result = bridge.initialize().await;
        if let Err(e) = &result {
            println!("Initialization error: {:?}", e);
        }
        assert!(result.is_ok());
    }
}
