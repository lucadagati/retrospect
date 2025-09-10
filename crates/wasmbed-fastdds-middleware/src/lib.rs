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

use rustdds::{
    DomainParticipant, Publisher as RustDdsPublisher, Subscriber as RustDdsSubscriber,
    Topic, QosPolicyBuilder, TopicKind, Keyed,
    policy::{Reliability, Durability, History},
    CDRSerializerAdapter, CDRDeserializerAdapter,
};

/// FastDDS Middleware for real-time data distribution
/// 
/// This middleware provides high-performance, real-time data distribution
/// capabilities for industrial applications using the DDS (Data Distribution Service) standard.
pub struct FastDdsMiddleware {
    /// Domain participant for DDS communication
    participant: Arc<DomainParticipant>,
    
    /// Domain ID
    domain_id: u32,
    
    /// Active publishers
    publishers: Arc<DashMap<String, RustDdsPublisher>>,
    
    /// Active subscribers
    subscribers: Arc<DashMap<String, RustDdsSubscriber>>,
    
    /// Data writers for publishing data
    data_writers: Arc<DashMap<String, rustdds::with_key::DataWriter<DdsMessage, CDRSerializerAdapter<DdsMessage>>>>,
    
    /// Data readers for receiving data
    data_readers: Arc<DashMap<String, rustdds::with_key::DataReader<DdsMessage, CDRDeserializerAdapter<DdsMessage>>>>,
    
    /// Topics
    topics: Arc<DashMap<String, Topic>>,
    
    /// Middleware configuration
    config: MiddlewareConfig,
    
    /// Middleware state
    state: Arc<RwLock<MiddlewareState>>,
}

/// DDS message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdsMessage {
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

impl Keyed for DdsMessage {
    type K = String;
    
    fn key(&self) -> Self::K {
        self.topic.clone()
    }
}

/// Middleware configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareConfig {
    /// DDS domain ID
    pub domain_id: u32,
    
    /// QoS configuration
    pub qos: QosConfig,
    
    /// Transport configuration
    pub transport: TransportConfig,
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

/// Transport configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Use UDP transport
    pub use_udp: bool,
    
    /// Use TCP transport
    pub use_tcp: bool,
    
    /// Use shared memory transport
    pub use_shared_memory: bool,
}

/// Middleware state
#[derive(Debug, Clone)]
pub struct MiddlewareState {
    /// Middleware initialization status
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

/// Middleware error types
#[derive(Debug, thiserror::Error)]
pub enum MiddlewareError {
    #[error("DDS error: {0}")]
    DdsError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Topic not found: {0}")]
    TopicNotFound(String),
    
    #[error("Middleware not initialized")]
    NotInitialized,
    
    #[error("Middleware not connected")]
    NotConnected,
}

impl FastDdsMiddleware {
    /// Create a new FastDDS middleware
    pub async fn new(config: MiddlewareConfig) -> Result<Self, MiddlewareError> {
        info!("Creating FastDDS middleware with config: {:?}", config);
        
        // Initialize DDS domain participant
        let participant = DomainParticipant::new(config.domain_id as u16)
            .map_err(|e| MiddlewareError::DdsError(e.to_string()))?;
        
        // Initialize middleware state
        let state = Arc::new(RwLock::new(MiddlewareState {
            initialized: false,
            connected: false,
            last_heartbeat: None,
            active_topics: 0,
            error_count: 0,
        }));
        
        let middleware = Self {
            participant: Arc::new(participant),
            domain_id: config.domain_id,
            publishers: Arc::new(DashMap::new()),
            subscribers: Arc::new(DashMap::new()),
            data_writers: Arc::new(DashMap::new()),
            data_readers: Arc::new(DashMap::new()),
            topics: Arc::new(DashMap::new()),
            config,
            state,
        };
        
        info!("FastDDS middleware created successfully");
        Ok(middleware)
    }
    
    /// Initialize the middleware
    #[instrument(skip(self))]
    pub async fn initialize(&self) -> Result<(), MiddlewareError> {
        info!("Initializing FastDDS middleware");
        
        // Start heartbeat task
        self.start_heartbeat_task().await?;
        
        // Update middleware state
        {
            let mut state = self.state.write().await;
            state.initialized = true;
            state.connected = true;
        }
        
        info!("FastDDS middleware initialized successfully");
        Ok(())
    }
    
    /// Create QoS profile
    fn create_qos_profile(&self) -> QosPolicyBuilder {
        QosPolicyBuilder::new()
            .reliability(self.config.qos.reliability.clone())
            .durability(self.config.qos.durability.clone())
            .history(History::KeepLast { depth: self.config.qos.history_depth })
    }
    
    /// Create publisher for a topic
    pub async fn create_publisher(&self, topic_name: &str, topic_type: &str) -> Result<(), MiddlewareError> {
        info!("Creating publisher for topic: {}", topic_name);
        
        let qos = self.create_qos_profile().build();
        
        // Create DDS topic
        let topic = self.participant.create_topic(
            topic_name.to_string(),
            topic_type.to_string(),
            &qos,
            TopicKind::NoKey,
        ).map_err(|e| MiddlewareError::DdsError(e.to_string()))?;
        
        self.topics.insert(topic_name.to_string(), topic.clone());
        
        // Create DDS publisher
        let publisher = self.participant.create_publisher(&qos)
            .map_err(|e| MiddlewareError::DdsError(e.to_string()))?;
        
        self.publishers.insert(topic_name.to_string(), publisher.clone());
        
        // Create data writer
        let data_writer = publisher.create_datawriter::<DdsMessage, CDRSerializerAdapter<DdsMessage>>(&topic, None)
            .map_err(|e| MiddlewareError::DdsError(e.to_string()))?;
        
        self.data_writers.insert(topic_name.to_string(), data_writer);
        
        info!("Created publisher for topic: {}", topic_name);
        Ok(())
    }
    
    /// Create subscriber for a topic
    pub async fn create_subscriber(&self, topic_name: &str, topic_type: &str) -> Result<(), MiddlewareError> {
        info!("Creating subscriber for topic: {}", topic_name);
        
        let qos = self.create_qos_profile().build();
        
        // Create DDS topic
        let topic = self.participant.create_topic(
            topic_name.to_string(),
            topic_type.to_string(),
            &qos,
            TopicKind::NoKey,
        ).map_err(|e| MiddlewareError::DdsError(e.to_string()))?;
        
        self.topics.insert(topic_name.to_string(), topic.clone());
        
        // Create DDS subscriber
        let subscriber = self.participant.create_subscriber(&qos)
            .map_err(|e| MiddlewareError::DdsError(e.to_string()))?;
        
        self.subscribers.insert(topic_name.to_string(), subscriber.clone());
        
        // Create data reader
        let data_reader = subscriber.create_datareader::<DdsMessage, CDRDeserializerAdapter<DdsMessage>>(&topic, None)
            .map_err(|e| MiddlewareError::DdsError(e.to_string()))?;
        
        self.data_readers.insert(topic_name.to_string(), data_reader);
        
        info!("Created subscriber for topic: {}", topic_name);
        Ok(())
    }
    
    /// Publish message to a topic
    pub async fn publish_message(&self, topic: &str, data: Vec<u8>) -> Result<(), MiddlewareError> {
        let message = DdsMessage {
            id: Uuid::new_v4(),
            topic: topic.to_string(),
            message_type: "DdsMessage".to_string(),
            data,
            timestamp: SystemTime::now(),
            source_system_id: 1,
            target_system_id: 1,
        };
        
        if let Some(data_writer) = self.data_writers.get(topic) {
            data_writer.write(message, None)
                .map_err(|e| MiddlewareError::DdsError(e.to_string()))?;
        }
        
        debug!("Message published to topic: {}", topic);
        Ok(())
    }
    
    /// Start heartbeat task
    async fn start_heartbeat_task(&self) -> Result<(), MiddlewareError> {
        info!("Starting heartbeat task");
        
        let state = self.state.clone();
        let mut interval = interval(Duration::from_secs(1));
        
        tokio::spawn(async move {
            loop {
                interval.tick().await;
                
                let mut state = state.write().await;
                state.last_heartbeat = Some(SystemTime::now());
            }
        });
        
        Ok(())
    }
    
    /// Get middleware status
    pub async fn get_status(&self) -> MiddlewareState {
        self.state.read().await.clone()
    }
    
    /// Shutdown the middleware
    pub async fn shutdown(&self) -> Result<(), MiddlewareError> {
        info!("Shutting down FastDDS middleware");
        
        // Update middleware state
        {
            let mut state = self.state.write().await;
            state.connected = false;
            state.initialized = false;
        }
        
        info!("FastDDS middleware shutdown complete");
        Ok(())
    }
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            domain_id: 0,
            qos: QosConfig::default(),
            transport: TransportConfig::default(),
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

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            use_udp: true,
            use_tcp: false,
            use_shared_memory: false,
        }
    }
}