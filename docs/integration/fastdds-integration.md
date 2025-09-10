# FastDDS Integration Documentation

## Overview

This document provides comprehensive documentation for integrating FastDDS (Fast Data Distribution Service) with the Wasmbed platform, enabling real-time data distribution and communication with PX4 autopilot systems and other DDS-compatible applications.

## FastDDS Architecture

### FastDDS Components

**FastDDS Stack**:
```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Wasmbed       │    │   FastDDS       │    │   DDS           │
│   Gateway       │    │   Middleware    │    │   Applications  │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ WASM        │ │    │ │ Domain      │ │    │ │ DDS         │ │
│ │ Runtime     │ │    │ │ Participant │ │    │ │ Nodes       │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
│                 │    │                 │    │                 │
│ ┌─────────────┐ │    │ ┌─────────────┐ │    │ ┌─────────────┐ │
│ │ Application │ │    │ │ Publisher   │ │    │ │ Subscriber  │ │
│ │ Manager     │ │    │ │ Subscriber  │ │    │ │ Publisher   │ │
│ └─────────────┘ │    │ └─────────────┘ │    │ └─────────────┘ │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Integration Layers

**Application Layer**:
- WASM Application: FastDDS-enabled WebAssembly applications
- DDS Participant: DDS domain participant for communication
- DDS Interface: Standard DDS API compatibility

**Middleware Layer**:
- FastDDS: DDS middleware for real-time communication
- DDS Bridge: Bridge between embedded and DDS systems
- DDS Topics: Standard DDS topic communication

**Transport Layer**:
- UDP: FastDDS transport protocol
- TCP: Alternative transport protocol
- Shared Memory: Local communication protocol

## FastDDS Middleware Implementation

### Core Middleware Components

**FastDDS Middleware Structure**:
```rust
pub struct FastDdsMiddleware {
    domain_id: u32,
    participant: DomainParticipant,
    publishers: HashMap<String, Publisher>,
    subscribers: HashMap<String, Subscriber>,
    data_writers: HashMap<String, DataWriter>,
    data_readers: HashMap<String, DataReader>,
    transport: TransportConfig,
    qos: QosProfile,
}

impl FastDdsMiddleware {
    pub fn new(domain_id: u32) -> Result<Self, Error> {
        // Create domain participant
        let participant = DomainParticipant::new(domain_id)?;
        
        // Configure transport
        let transport = TransportConfig::default();
        
        // Configure QoS profile
        let qos = QosProfile::reliable();
        
        Ok(Self {
            domain_id,
            participant,
            publishers: HashMap::new(),
            subscribers: HashMap::new(),
            data_writers: HashMap::new(),
            data_readers: HashMap::new(),
            transport,
            qos,
        })
    }
    
    pub fn initialize(&mut self) -> Result<(), Error> {
        // Initialize participant
        self.participant.initialize()?;
        
        // Configure transport
        self.configure_transport()?;
        
        // Configure QoS profiles
        self.configure_qos_profiles()?;
        
        Ok(())
    }
}
```

### Domain Participant Management

**Domain Participant Configuration**:
```rust
pub struct DomainParticipantConfig {
    pub domain_id: u32,
    pub participant_name: String,
    pub participant_id: u32,
    pub qos_profile: QosProfile,
    pub transport_config: TransportConfig,
    pub discovery_config: DiscoveryConfig,
}

impl DomainParticipantConfig {
    pub fn new() -> Self {
        Self {
            domain_id: 0,
            participant_name: "wasmbed_gateway".to_string(),
            participant_id: 0,
            qos_profile: QosProfile::reliable(),
            transport_config: TransportConfig::default(),
            discovery_config: DiscoveryConfig::default(),
        }
    }
    
    pub fn with_domain_id(mut self, domain_id: u32) -> Self {
        self.domain_id = domain_id;
        self
    }
    
    pub fn with_participant_name(mut self, participant_name: String) -> Self {
        self.participant_name = participant_name;
        self
    }
    
    pub fn with_participant_id(mut self, participant_id: u32) -> Self {
        self.participant_id = participant_id;
        self
    }
}
```

### Publisher Management

**Publisher Implementation**:
```rust
pub struct FastDdsPublisher {
    publisher: Publisher,
    data_writer: DataWriter,
    topic_name: String,
    topic_type: String,
    qos: QosProfile,
}

impl FastDdsPublisher {
    pub fn new(
        participant: &DomainParticipant,
        topic_name: &str,
        topic_type: &str,
        qos: QosProfile,
    ) -> Result<Self, Error> {
        // Create topic
        let topic = participant.create_topic(topic_name, topic_type, &qos)?;
        
        // Create publisher
        let publisher = participant.create_publisher(&qos)?;
        
        // Create data writer
        let data_writer = publisher.create_datawriter(&topic, &qos)?;
        
        Ok(Self {
            publisher,
            data_writer,
            topic_name: topic_name.to_string(),
            topic_type: topic_type.to_string(),
            qos,
        })
    }
    
    pub fn publish<T>(&self, data: &T) -> Result<(), Error>
    where
        T: Serialize,
    {
        // Serialize data
        let serialized = serde_json::to_vec(data)?;
        
        // Write data
        self.data_writer.write(&serialized)?;
        
        Ok(())
    }
    
    pub fn get_topic_name(&self) -> &str {
        &self.topic_name
    }
    
    pub fn get_topic_type(&self) -> &str {
        &self.topic_type
    }
}
```

### Subscriber Management

**Subscriber Implementation**:
```rust
pub struct FastDdsSubscriber {
    subscriber: Subscriber,
    data_reader: DataReader,
    topic_name: String,
    topic_type: String,
    qos: QosProfile,
    callback: Option<Box<dyn Fn(&[u8]) -> Result<(), Error> + Send + Sync>>,
}

impl FastDdsSubscriber {
    pub fn new(
        participant: &DomainParticipant,
        topic_name: &str,
        topic_type: &str,
        qos: QosProfile,
    ) -> Result<Self, Error> {
        // Create topic
        let topic = participant.create_topic(topic_name, topic_type, &qos)?;
        
        // Create subscriber
        let subscriber = participant.create_subscriber(&qos)?;
        
        // Create data reader
        let data_reader = subscriber.create_datareader(&topic, &qos)?;
        
        Ok(Self {
            subscriber,
            data_reader,
            topic_name: topic_name.to_string(),
            topic_type: topic_type.to_string(),
            qos,
            callback: None,
        })
    }
    
    pub fn set_callback<F>(&mut self, callback: F)
    where
        F: Fn(&[u8]) -> Result<(), Error> + Send + Sync + 'static,
    {
        self.callback = Some(Box::new(callback));
    }
    
    pub fn receive_data(&self) -> Result<Option<Vec<u8>>, Error> {
        // Read data from data reader
        if let Some(data) = self.data_reader.read()? {
            // Call callback if set
            if let Some(callback) = &self.callback {
                callback(&data)?;
            }
            
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
}
```

## QoS Configuration

### QoS Profiles

**QoS Profile Implementation**:
```rust
pub struct QosProfile {
    pub reliability: ReliabilityKind,
    pub durability: DurabilityKind,
    pub history: HistoryKind,
    pub depth: i32,
    pub deadline: Duration,
    pub lifespan: Duration,
    pub liveliness: LivelinessKind,
    pub liveliness_lease_duration: Duration,
    pub ownership: OwnershipKind,
    pub ownership_strength: i32,
    pub destination_order: DestinationOrderKind,
    pub presentation: PresentationKind,
    pub partition: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ReliabilityKind {
    BestEffort,
    Reliable,
}

#[derive(Debug, Clone)]
pub enum DurabilityKind {
    Volatile,
    TransientLocal,
    Transient,
    Persistent,
}

#[derive(Debug, Clone)]
pub enum HistoryKind {
    KeepLast,
    KeepAll,
}

#[derive(Debug, Clone)]
pub enum LivelinessKind {
    Automatic,
    ManualByParticipant,
    ManualByTopic,
}

#[derive(Debug, Clone)]
pub enum OwnershipKind {
    Shared,
    Exclusive,
}

#[derive(Debug, Clone)]
pub enum DestinationOrderKind {
    ByReceptionTimestamp,
    BySourceTimestamp,
}

#[derive(Debug, Clone)]
pub enum PresentationKind {
    Instance,
    Topic,
    Group,
}

impl QosProfile {
    pub fn reliable() -> Self {
        Self {
            reliability: ReliabilityKind::Reliable,
            durability: DurabilityKind::Volatile,
            history: HistoryKind::KeepLast,
            depth: 10,
            deadline: Duration::from_secs(0),
            lifespan: Duration::from_secs(0),
            liveliness: LivelinessKind::Automatic,
            liveliness_lease_duration: Duration::from_secs(0),
            ownership: OwnershipKind::Shared,
            ownership_strength: 0,
            destination_order: DestinationOrderKind::ByReceptionTimestamp,
            presentation: PresentationKind::Instance,
            partition: Vec::new(),
        }
    }
    
    pub fn best_effort() -> Self {
        Self {
            reliability: ReliabilityKind::BestEffort,
            durability: DurabilityKind::Volatile,
            history: HistoryKind::KeepLast,
            depth: 10,
            deadline: Duration::from_secs(0),
            lifespan: Duration::from_secs(0),
            liveliness: LivelinessKind::Automatic,
            liveliness_lease_duration: Duration::from_secs(0),
            ownership: OwnershipKind::Shared,
            ownership_strength: 0,
            destination_order: DestinationOrderKind::ByReceptionTimestamp,
            presentation: PresentationKind::Instance,
            partition: Vec::new(),
        }
    }
    
    pub fn with_depth(mut self, depth: i32) -> Self {
        self.depth = depth;
        self
    }
    
    pub fn with_deadline(mut self, deadline: Duration) -> Self {
        self.deadline = deadline;
        self
    }
    
    pub fn with_lifespan(mut self, lifespan: Duration) -> Self {
        self.lifespan = lifespan;
        self
    }
    
    pub fn with_partition(mut self, partition: Vec<String>) -> Self {
        self.partition = partition;
        self
    }
}
```

## Transport Configuration

### UDP Transport

**UDP Transport Configuration**:
```rust
pub struct UdpTransportConfig {
    pub port: u16,
    pub interface: String,
    pub multicast: bool,
    pub multicast_group: String,
    pub send_buffer_size: usize,
    pub receive_buffer_size: usize,
    pub ttl: u8,
    pub loopback: bool,
}

impl UdpTransportConfig {
    pub fn new() -> Self {
        Self {
            port: 7400,
            interface: "0.0.0.0".to_string(),
            multicast: false,
            multicast_group: "239.255.0.1".to_string(),
            send_buffer_size: 65536,
            receive_buffer_size: 65536,
            ttl: 1,
            loopback: false,
        }
    }
    
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    
    pub fn with_interface(mut self, interface: String) -> Self {
        self.interface = interface;
        self
    }
    
    pub fn with_multicast(mut self, multicast: bool) -> Self {
        self.multicast = multicast;
        self
    }
    
    pub fn with_multicast_group(mut self, multicast_group: String) -> Self {
        self.multicast_group = multicast_group;
        self
    }
    
    pub fn with_buffer_sizes(mut self, send_size: usize, receive_size: usize) -> Self {
        self.send_buffer_size = send_size;
        self.receive_buffer_size = receive_size;
        self
    }
}
```

### TCP Transport

**TCP Transport Configuration**:
```rust
pub struct TcpTransportConfig {
    pub port: u16,
    pub interface: String,
    pub keepalive: bool,
    pub keepalive_time: Duration,
    pub keepalive_interval: Duration,
    pub keepalive_probes: u32,
    pub send_buffer_size: usize,
    pub receive_buffer_size: usize,
    pub tcp_nodelay: bool,
    pub tcp_cork: bool,
}

impl TcpTransportConfig {
    pub fn new() -> Self {
        Self {
            port: 7400,
            interface: "0.0.0.0".to_string(),
            keepalive: true,
            keepalive_time: Duration::from_secs(600),
            keepalive_interval: Duration::from_secs(60),
            keepalive_probes: 3,
            send_buffer_size: 65536,
            receive_buffer_size: 65536,
            tcp_nodelay: true,
            tcp_cork: false,
        }
    }
    
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
    
    pub fn with_interface(mut self, interface: String) -> Self {
        self.interface = interface;
        self
    }
    
    pub fn with_keepalive(mut self, keepalive: bool) -> Self {
        self.keepalive = keepalive;
        self
    }
    
    pub fn with_buffer_sizes(mut self, send_size: usize, receive_size: usize) -> Self {
        self.send_buffer_size = send_size;
        self.receive_buffer_size = receive_size;
        self
    }
}
```

### Shared Memory Transport

**Shared Memory Transport Configuration**:
```rust
pub struct SharedMemoryTransportConfig {
    pub segment_size: usize,
    pub port_queue_size: usize,
    pub healthy_check_timeout: Duration,
    pub rtps_dump_file: Option<String>,
    pub use_rtps_dump: bool,
}

impl SharedMemoryTransportConfig {
    pub fn new() -> Self {
        Self {
            segment_size: 1024 * 1024, // 1MB
            port_queue_size: 512,
            healthy_check_timeout: Duration::from_secs(5),
            rtps_dump_file: None,
            use_rtps_dump: false,
        }
    }
    
    pub fn with_segment_size(mut self, segment_size: usize) -> Self {
        self.segment_size = segment_size;
        self
    }
    
    pub fn with_port_queue_size(mut self, port_queue_size: usize) -> Self {
        self.port_queue_size = port_queue_size;
        self
    }
    
    pub fn with_rtps_dump(mut self, rtps_dump_file: String) -> Self {
        self.rtps_dump_file = Some(rtps_dump_file);
        self.use_rtps_dump = true;
        self
    }
}
```

## Discovery Configuration

### Discovery Settings

**Discovery Configuration**:
```rust
pub struct DiscoveryConfig {
    pub discovery_protocol: DiscoveryProtocol,
    pub initial_peers: Vec<String>,
    pub metatraffic_multicast_locator: String,
    pub metatraffic_unicast_locator: String,
    pub initial_announcements: Duration,
    pub lease_duration: Duration,
    pub lease_duration_announcement: Duration,
    pub simple_edp_use_pubsub_writer_datawriter_history: bool,
    pub simple_edp_use_pubsub_reader_datareader_history: bool,
    pub use_simple_participant_proxy_detection: bool,
    pub use_simple_endpoint_proxy_detection: bool,
}

#[derive(Debug, Clone)]
pub enum DiscoveryProtocol {
    Simple,
    Backup,
    Static,
    Client,
    Server,
}

impl DiscoveryConfig {
    pub fn new() -> Self {
        Self {
            discovery_protocol: DiscoveryProtocol::Simple,
            initial_peers: Vec::new(),
            metatraffic_multicast_locator: "239.255.0.1:7400".to_string(),
            metatraffic_unicast_locator: "0.0.0.0:0".to_string(),
            initial_announcements: Duration::from_secs(1),
            lease_duration: Duration::from_secs(100),
            lease_duration_announcement: Duration::from_secs(10),
            simple_edp_use_pubsub_writer_datawriter_history: true,
            simple_edp_use_pubsub_reader_datareader_history: true,
            use_simple_participant_proxy_detection: true,
            use_simple_endpoint_proxy_detection: true,
        }
    }
    
    pub fn with_discovery_protocol(mut self, protocol: DiscoveryProtocol) -> Self {
        self.discovery_protocol = protocol;
        self
    }
    
    pub fn with_initial_peers(mut self, peers: Vec<String>) -> Self {
        self.initial_peers = peers;
        self
    }
    
    pub fn with_multicast_locator(mut self, locator: String) -> Self {
        self.metatraffic_multicast_locator = locator;
        self
    }
    
    pub fn with_unicast_locator(mut self, locator: String) -> Self {
        self.metatraffic_unicast_locator = locator;
        self
    }
}
```

## Data Types and Serialization

### DDS Data Types

**Standard DDS Data Types**:
```rust
// Standard DDS data types
pub mod dds_types {
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Time {
        pub sec: i32,
        pub nanosec: u32,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Duration {
        pub sec: i32,
        pub nanosec: u32,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Locator {
        pub kind: i32,
        pub port: u32,
        pub address: [u8; 16],
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Guid {
        pub guid_prefix: [u8; 12],
        pub entity_id: [u8; 4],
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SequenceNumber {
        pub high: i32,
        pub low: u32,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SampleIdentity {
        pub writer_guid: Guid,
        pub sequence_number: SequenceNumber,
    }
}
```

### Custom Data Types

**Custom Data Type Definition**:
```rust
pub mod wasmbed_dds_types {
    use serde::{Deserialize, Serialize};
    use super::dds_types::Time;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DeviceStatus {
        pub device_id: String,
        pub device_type: String,
        pub status: String,
        pub battery_level: f32,
        pub cpu_usage: f32,
        pub memory_usage: f32,
        pub temperature: f32,
        pub timestamp: Time,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ApplicationStatus {
        pub app_id: String,
        pub app_name: String,
        pub status: String,
        pub cpu_usage: f32,
        pub memory_usage: f32,
        pub execution_time: f32,
        pub error_count: u32,
        pub timestamp: Time,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Command {
        pub command_id: String,
        pub command_type: String,
        pub parameters: std::collections::HashMap<String, String>,
        pub timestamp: Time,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Response {
        pub command_id: String,
        pub success: bool,
        pub message: String,
        pub data: std::collections::HashMap<String, String>,
        pub timestamp: Time,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SensorData {
        pub sensor_id: String,
        pub sensor_type: String,
        pub data: Vec<f32>,
        pub timestamp: Time,
    }
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ControlCommand {
        pub control_id: String,
        pub control_type: String,
        pub values: Vec<f32>,
        pub timestamp: Time,
    }
}
```

## WASM Application Integration

### FastDDS WASM Application

**WASM Application Structure**:
```rust
pub struct FastDdsWasmApplication {
    pub app_id: String,
    pub fastdds_middleware: FastDdsMiddleware,
    pub publishers: HashMap<String, FastDdsPublisher>,
    pub subscribers: HashMap<String, FastDdsSubscriber>,
    pub data_writers: HashMap<String, DataWriter>,
    pub data_readers: HashMap<String, DataReader>,
}

impl FastDdsWasmApplication {
    pub fn new(app_id: String) -> Result<Self, Error> {
        // Initialize FastDDS middleware
        let fastdds_middleware = FastDdsMiddleware::new(0)?;
        
        Ok(Self {
            app_id,
            fastdds_middleware,
            publishers: HashMap::new(),
            subscribers: HashMap::new(),
            data_writers: HashMap::new(),
            data_readers: HashMap::new(),
        })
    }
    
    pub fn initialize(&mut self) -> Result<(), Error> {
        // Initialize FastDDS middleware
        self.fastdds_middleware.initialize()?;
        
        // Create default publishers
        self.create_default_publishers()?;
        
        // Create default subscribers
        self.create_default_subscribers()?;
        
        // Create default data writers
        self.create_default_data_writers()?;
        
        // Create default data readers
        self.create_default_data_readers()?;
        
        Ok(())
    }
    
    pub fn create_publisher(
        &mut self,
        topic_name: &str,
        topic_type: &str,
        qos: QosProfile,
    ) -> Result<(), Error> {
        let publisher = FastDdsPublisher::new(
            &self.fastdds_middleware.participant,
            topic_name,
            topic_type,
            qos,
        )?;
        
        self.publishers.insert(topic_name.to_string(), publisher);
        
        Ok(())
    }
    
    pub fn create_subscriber(
        &mut self,
        topic_name: &str,
        topic_type: &str,
        qos: QosProfile,
    ) -> Result<(), Error> {
        let subscriber = FastDdsSubscriber::new(
            &self.fastdds_middleware.participant,
            topic_name,
            topic_type,
            qos,
        )?;
        
        self.subscribers.insert(topic_name.to_string(), subscriber);
        
        Ok(())
    }
}
```

### Application Lifecycle Management

**Lifecycle Manager**:
```rust
pub struct FastDdsLifecycleManager {
    applications: HashMap<String, FastDdsWasmApplication>,
    lifecycle_callbacks: HashMap<String, Box<dyn Fn() -> Result<(), Error> + Send + Sync>>,
}

impl FastDdsLifecycleManager {
    pub fn new() -> Self {
        Self {
            applications: HashMap::new(),
            lifecycle_callbacks: HashMap::new(),
        }
    }
    
    pub fn register_application(
        &mut self,
        app_id: String,
        application: FastDdsWasmApplication,
    ) -> Result<(), Error> {
        self.applications.insert(app_id.clone(), application);
        Ok(())
    }
    
    pub fn start_application(&mut self, app_id: &str) -> Result<(), Error> {
        if let Some(application) = self.applications.get_mut(app_id) {
            application.initialize()?;
            
            // Call lifecycle callback if registered
            if let Some(callback) = self.lifecycle_callbacks.get(app_id) {
                callback()?;
            }
        }
        
        Ok(())
    }
    
    pub fn stop_application(&mut self, app_id: &str) -> Result<(), Error> {
        if let Some(application) = self.applications.get_mut(app_id) {
            // Cleanup resources
            application.publishers.clear();
            application.subscribers.clear();
            application.data_writers.clear();
            application.data_readers.clear();
        }
        
        Ok(())
    }
}
```

## Performance Optimization

### Communication Optimization

**Performance Optimizer**:
```rust
pub struct FastDdsPerformanceOptimizer {
    message_batching: bool,
    compression_enabled: bool,
    priority_queue: PriorityQueue<Message>,
    rate_limiter: RateLimiter,
    connection_pool: ConnectionPool,
    memory_pool: MemoryPool,
}

impl FastDdsPerformanceOptimizer {
    pub fn new() -> Self {
        Self {
            message_batching: true,
            compression_enabled: true,
            priority_queue: PriorityQueue::new(),
            rate_limiter: RateLimiter::new(1000), // 1000 messages per second
            connection_pool: ConnectionPool::new(10), // 10 connections
            memory_pool: MemoryPool::new(1024 * 1024), // 1MB pool
        }
    }
    
    pub fn optimize_communication(&mut self) -> Result<(), Error> {
        // Apply message batching
        if self.message_batching {
            self.batch_messages()?;
        }
        
        // Apply compression
        if self.compression_enabled {
            self.compress_messages()?;
        }
        
        // Apply rate limiting
        self.apply_rate_limiting()?;
        
        // Optimize connections
        self.optimize_connections()?;
        
        // Optimize memory usage
        self.optimize_memory_usage()?;
        
        Ok(())
    }
    
    pub fn batch_messages(&mut self) -> Result<(), Error> {
        // Implement message batching logic
        Ok(())
    }
    
    pub fn compress_messages(&mut self) -> Result<(), Error> {
        // Implement message compression logic
        Ok(())
    }
    
    pub fn apply_rate_limiting(&mut self) -> Result<(), Error> {
        // Implement rate limiting logic
        Ok(())
    }
    
    pub fn optimize_connections(&mut self) -> Result<(), Error> {
        // Implement connection optimization logic
        Ok(())
    }
    
    pub fn optimize_memory_usage(&mut self) -> Result<(), Error> {
        // Implement memory optimization logic
        Ok(())
    }
}
```

## Testing and Validation

### FastDDS Integration Tests

**Integration Test Suite**:
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_fastdds_middleware_initialization() {
        // Setup
        let middleware = FastDdsMiddleware::new(0).await.unwrap();
        
        // Test initialization
        let result = middleware.initialize().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_publisher_subscriber_communication() {
        // Setup
        let mut middleware = FastDdsMiddleware::new(0).await.unwrap();
        middleware.initialize().await.unwrap();
        
        // Create publisher
        let publisher = FastDdsPublisher::new(
            &middleware.participant,
            "/test_topic",
            "wasmbed_dds_types::DeviceStatus",
            QosProfile::reliable(),
        ).unwrap();
        
        // Create subscriber
        let mut subscriber = FastDdsSubscriber::new(
            &middleware.participant,
            "/test_topic",
            "wasmbed_dds_types::DeviceStatus",
            QosProfile::reliable(),
        ).unwrap();
        
        // Set callback
        let received = Arc::new(Mutex::new(false));
        let received_clone = received.clone();
        subscriber.set_callback(move |_| {
            *received_clone.lock().unwrap() = true;
            Ok(())
        });
        
        // Publish message
        let message = wasmbed_dds_types::DeviceStatus {
            device_id: "test_device".to_string(),
            device_type: "test_type".to_string(),
            status: "active".to_string(),
            battery_level: 100.0,
            cpu_usage: 50.0,
            memory_usage: 75.0,
            temperature: 25.0,
            timestamp: Time { sec: 0, nanosec: 0 },
        };
        publisher.publish(&message).unwrap();
        
        // Wait for message
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Check if message was received
        assert!(*received.lock().unwrap());
    }
    
    #[tokio::test]
    async fn test_qos_profiles() {
        // Test reliable QoS
        let reliable_qos = QosProfile::reliable();
        assert_eq!(reliable_qos.reliability, ReliabilityKind::Reliable);
        
        // Test best effort QoS
        let best_effort_qos = QosProfile::best_effort();
        assert_eq!(best_effort_qos.reliability, ReliabilityKind::BestEffort);
        
        // Test custom QoS
        let custom_qos = QosProfile::reliable()
            .with_depth(20)
            .with_deadline(Duration::from_secs(1))
            .with_lifespan(Duration::from_secs(10));
        
        assert_eq!(custom_qos.depth, 20);
        assert_eq!(custom_qos.deadline, Duration::from_secs(1));
        assert_eq!(custom_qos.lifespan, Duration::from_secs(10));
    }
}
```

## Deployment and Configuration

### FastDDS Configuration

**Configuration File**:
```yaml
# FastDDS Configuration
fastdds:
  domain:
    id: 0
    participant_name: "wasmbed_gateway"
    participant_id: 0
  
  qos:
    reliability: "reliable"
    durability: "volatile"
    history: "keep_last"
    depth: 10
    deadline: "0s"
    lifespan: "0s"
    liveliness: "automatic"
    liveliness_lease_duration: "0s"
    ownership: "shared"
    ownership_strength: 0
    destination_order: "by_reception_timestamp"
    presentation: "instance"
    partition: []
  
  transport:
    type: "udp"
    port: 7400
    interface: "0.0.0.0"
    multicast: false
    multicast_group: "239.255.0.1"
    send_buffer_size: 65536
    receive_buffer_size: 65536
    ttl: 1
    loopback: false
  
  discovery:
    protocol: "simple"
    initial_peers: []
    metatraffic_multicast_locator: "239.255.0.1:7400"
    metatraffic_unicast_locator: "0.0.0.0:0"
    initial_announcements: "1s"
    lease_duration: "100s"
    lease_duration_announcement: "10s"
    simple_edp_use_pubsub_writer_datawriter_history: true
    simple_edp_use_pubsub_reader_datareader_history: true
    use_simple_participant_proxy_detection: true
    use_simple_endpoint_proxy_detection: true
  
  performance:
    message_batching: true
    compression: true
    rate_limiting: true
    connection_pooling: true
    memory_pooling: true
    max_connections: 100
    memory_pool_size: "1MB"
  
  topics:
    publishers:
    - name: "/wasmbed/device_status"
      type: "wasmbed_dds_types::DeviceStatus"
      qos: "reliable"
    - name: "/wasmbed/application_status"
      type: "wasmbed_dds_types::ApplicationStatus"
      qos: "reliable"
    - name: "/wasmbed/sensor_data"
      type: "wasmbed_dds_types::SensorData"
      qos: "best_effort"
    
    subscribers:
    - name: "/wasmbed/command"
      type: "wasmbed_dds_types::Command"
      qos: "reliable"
    - name: "/wasmbed/response"
      type: "wasmbed_dds_types::Response"
      qos: "reliable"
    - name: "/wasmbed/control_command"
      type: "wasmbed_dds_types::ControlCommand"
      qos: "reliable"
```

### Application Deployment

**FastDDS Application Manifest**:
```yaml
apiVersion: wasmbed.github.io/v1alpha1
kind: Application
metadata:
  name: fastdds-app
  namespace: wasmbed
spec:
  name: "FastDDS Application"
  wasmBinary: <base64-encoded-wasm-binary>
  targetDevices:
  - deviceType: "fastdds"
    count: 1
  config:
    fastdds:
      domain_id: 0
      participant_name: "fastdds_app"
      participant_id: 0
      qos_profile: "reliable"
      transport: "udp"
      port: 7400
    topics:
      publishers:
      - "/app/status"
      - "/app/data"
      subscribers:
      - "/app/command"
      - "/app/config"
    performance:
      message_batching: true
      compression: true
      rate_limiting: true
      max_connections: 50
```

## Troubleshooting

### Common Issues

**Connection Issues**:
- Check domain ID configuration
- Verify network connectivity
- Check firewall settings
- Validate QoS settings

**Message Issues**:
- Check message type compatibility
- Verify serialization format
- Check message size limits
- Validate QoS settings

**Performance Issues**:
- Check rate limiting settings
- Verify connection pooling
- Check message batching
- Validate compression settings

### Debugging Tools

**Debug Configuration**:
```yaml
# Debug Configuration
debug:
  fastdds:
    log_level: "debug"
    verbose: true
    trace: true
    performance_monitoring: true
  
  topics:
    log_publishers: true
    log_subscribers: true
    log_messages: true
  
  transport:
    log_connections: true
    log_packets: true
    log_errors: true
  
  discovery:
    log_discovery: true
    log_peers: true
    log_endpoints: true
```

**Monitoring and Logging**:
```rust
pub struct FastDdsDebugger {
    logger: Logger,
    metrics_collector: MetricsCollector,
    trace_collector: TraceCollector,
}

impl FastDdsDebugger {
    pub fn new() -> Self {
        Self {
            logger: Logger::new("fastdds_debugger"),
            metrics_collector: MetricsCollector::new(),
            trace_collector: TraceCollector::new(),
        }
    }
    
    pub fn log_message(&self, topic: &str, message: &[u8]) {
        self.logger.debug(&format!("Topic: {}, Message: {:?}", topic, message));
    }
    
    pub fn log_connection(&self, connection: &str, status: &str) {
        self.logger.debug(&format!("Connection: {}, Status: {}", connection, status));
    }
    
    pub fn collect_metrics(&mut self) -> FastDdsMetrics {
        self.metrics_collector.collect()
    }
    
    pub fn collect_traces(&mut self) -> Vec<TraceEvent> {
        self.trace_collector.collect()
    }
}
```
