# Protocol Specification

## 🎯 Overview

This document provides the complete technical specification for the Wasmbed communication protocol, including message formats, state machines, and implementation requirements.

## 📋 Protocol Version

- **Current Version**: 1.0.0
- **Release Date**: September 2024
- **Status**: Stable
- **Compatibility**: Backward compatible with future 1.x versions

## 🔧 Message Format

### Envelope Structure

All messages are wrapped in a standardized envelope:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Envelope {
    /// Unique message identifier (UUID v4)
    pub message_id: MessageId,
    
    /// Protocol version (major.minor.patch)
    pub version: ProtocolVersion,
    
    /// Unix timestamp in seconds
    pub timestamp: u64,
    
    /// Actual message payload
    pub message: Message,
    
    /// Optional correlation ID for request-response
    pub correlation_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageId(pub String);

#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}
```

### Message Types

#### Client Messages (MCU → Gateway)

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    /// Device enrollment request
    Enrollment(EnrollmentRequest),
    
    /// Heartbeat message
    Heartbeat(HeartbeatMessage),
    
    /// Status update
    StatusUpdate(StatusUpdateMessage),
    
    /// Application status report
    ApplicationStatus(ApplicationStatusMessage),
    
    /// Application error report
    ApplicationError(ApplicationErrorMessage),
    
    /// Ping message
    Ping(PingMessage),
    
    /// Error message
    Error(ErrorMessage),
}
```

#### Server Messages (Gateway → MCU)

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    /// Enrollment response
    EnrollmentResponse(EnrollmentResponse),
    
    /// Deploy application command
    DeployApplication(DeployApplicationMessage),
    
    /// Stop application command
    StopApplication(StopApplicationMessage),
    
    /// Pong response
    Pong(PongMessage),
    
    /// Error message
    Error(ErrorMessage),
}
```

## 🔄 Message Flow

### 1. Device Enrollment

#### Enrollment Request

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct EnrollmentRequest {
    /// Device type identifier
    pub device_type: String,
    
    /// Device capabilities
    pub capabilities: Vec<String>,
    
    /// Base64 encoded public key
    pub public_key: String,
    
    /// Firmware version
    pub firmware_version: String,
    
    /// Unique hardware identifier
    pub hardware_id: String,
    
    /// Optional device metadata
    pub metadata: Option<HashMap<String, String>>,
}
```

#### Enrollment Response

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct EnrollmentResponse {
    /// Success status
    pub success: bool,
    
    /// Assigned device ID
    pub device_id: Option<String>,
    
    /// Error message if failed
    pub error_message: Option<String>,
    
    /// Device configuration
    pub config: Option<DeviceConfig>,
    
    /// Gateway information
    pub gateway_info: Option<GatewayInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceConfig {
    /// Heartbeat interval in seconds
    pub heartbeat_interval: u32,
    
    /// Connection timeout in seconds
    pub connection_timeout: u32,
    
    /// Maximum message size
    pub max_message_size: u32,
    
    /// Supported protocol features
    pub features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayInfo {
    /// Gateway version
    pub version: String,
    
    /// Gateway capabilities
    pub capabilities: Vec<String>,
    
    /// Gateway endpoint
    pub endpoint: String,
}
```

### 2. Heartbeat Communication

#### Heartbeat Message

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    /// Device ID
    pub device_id: String,
    
    /// Device uptime in seconds
    pub uptime: u64,
    
    /// Current memory usage in bytes
    pub memory_usage: u32,
    
    /// CPU usage percentage (0-100)
    pub cpu_usage: u8,
    
    /// Number of running applications
    pub application_count: u8,
    
    /// Optional device temperature in Celsius
    pub temperature: Option<i16>,
    
    /// Optional battery level (0-100)
    pub battery_level: Option<u8>,
    
    /// Optional signal strength
    pub signal_strength: Option<i8>,
}
```

#### Pong Response

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct PongMessage {
    /// Response timestamp
    pub timestamp: u64,
    
    /// Optional server status
    pub server_status: Option<String>,
}
```

### 3. Application Management

#### Deploy Application

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct DeployApplicationMessage {
    /// Application ID
    pub application_id: String,
    
    /// Application name
    pub name: String,
    
    /// WASM binary data
    pub wasm_bytes: Vec<u8>,
    
    /// Application configuration
    pub config: ApplicationConfig,
    
    /// Environment variables
    pub environment_vars: HashMap<String, String>,
    
    /// Application arguments
    pub args: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationConfig {
    /// Memory limit in bytes
    pub memory_limit: u32,
    
    /// CPU time limit in milliseconds
    pub cpu_time_limit: u32,
    
    /// Auto-restart on failure
    pub auto_restart: bool,
    
    /// Maximum restart attempts
    pub max_restarts: u8,
    
    /// Application timeout in milliseconds
    pub timeout: u32,
    
    /// Priority level (0-255)
    pub priority: u8,
}
```

#### Application Status

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationStatusMessage {
    /// Application ID
    pub application_id: String,
    
    /// Application status
    pub status: ApplicationStatus,
    
    /// Runtime metrics
    pub metrics: Option<ApplicationMetrics>,
    
    /// Error message if failed
    pub error: Option<String>,
    
    /// Timestamp
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ApplicationStatus {
    Deploying,
    Running,
    Stopped,
    Failed,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApplicationMetrics {
    /// Memory usage in bytes
    pub memory_usage: u32,
    
    /// CPU usage percentage
    pub cpu_usage: u8,
    
    /// Uptime in seconds
    pub uptime: u64,
    
    /// Number of function calls
    pub function_calls: u64,
}
```

## 🔐 Security Model

### TLS Configuration

```rust
pub struct TlsConfig {
    /// Certificate file path
    pub cert_file: String,
    
    /// Private key file path
    pub key_file: String,
    
    /// CA certificate file path
    pub ca_file: String,
    
    /// Verify peer certificates
    pub verify_peer: bool,
    
    /// Minimum TLS version
    pub min_tls_version: TlsVersion,
    
    /// Cipher suites
    pub cipher_suites: Vec<u16>,
    
    /// Certificate pinning
    pub cert_pinning: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy)]
pub enum TlsVersion {
    V1_2,
    V1_3,
}
```

### Certificate Validation

1. **Server Certificate**: Gateway presents certificate to MCU
2. **Client Certificate**: MCU presents certificate to Gateway
3. **CA Validation**: Both sides validate against trusted CA
4. **Certificate Pinning**: Optional certificate fingerprint validation
5. **Revocation**: Check certificate revocation lists (CRL/OCSP)

### Message Integrity

- **TLS Encryption**: All messages encrypted in transit
- **Message Signing**: Optional message signing for integrity
- **Replay Protection**: Timestamp-based replay protection
- **Sequence Numbers**: Optional sequence numbers for ordering

## 📊 Performance Requirements

### Latency Requirements

| Operation | Maximum Latency | Target Latency |
|-----------|----------------|----------------|
| Enrollment | 5 seconds | 2 seconds |
| Heartbeat | 100ms | 50ms |
| Application Deploy | 10 seconds | 5 seconds |
| Status Update | 500ms | 200ms |

### Throughput Requirements

| Metric | Minimum | Target |
|--------|---------|--------|
| Messages per second | 100 | 1000 |
| Concurrent connections | 100 | 1000 |
| Application deployments | 10/min | 100/min |

### Resource Usage

| Component | Memory | CPU | Network |
|-----------|--------|-----|---------|
| Protocol processing | < 1MB | < 1% | < 1KB/msg |
| TLS handshake | < 2MB | < 5% | < 10KB |
| Message serialization | < 512KB | < 1% | Variable |

## 🔧 Implementation Requirements

### CBOR Serialization

```rust
// Example CBOR encoding
let enrollment = EnrollmentRequest {
    device_type: "hifive1".to_string(),
    capabilities: vec!["wasm".to_string(), "tls".to_string()],
    public_key: "base64_encoded_key".to_string(),
    firmware_version: "1.0.0".to_string(),
    hardware_id: "hifive1-001".to_string(),
    metadata: Some(HashMap::new()),
};

let encoded = serde_cbor::to_vec(&enrollment)?;
```

### Message Validation

```rust
pub fn validate_message(message: &Message) -> Result<(), ProtocolError> {
    match message {
        Message::Enrollment(req) => validate_enrollment(req),
        Message::Heartbeat(hb) => validate_heartbeat(hb),
        Message::DeployApplication(deploy) => validate_deployment(deploy),
        _ => Ok(()),
    }
}

fn validate_enrollment(req: &EnrollmentRequest) -> Result<(), ProtocolError> {
    if req.device_type.is_empty() {
        return Err(ProtocolError::InvalidMessage("device_type cannot be empty"));
    }
    
    if req.capabilities.is_empty() {
        return Err(ProtocolError::InvalidMessage("capabilities cannot be empty"));
    }
    
    if req.public_key.is_empty() {
        return Err(ProtocolError::InvalidMessage("public_key cannot be empty"));
    }
    
    Ok(())
}
```

### Error Handling

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorMessage {
    /// Error code
    pub error_code: ErrorCode,
    
    /// Human-readable error message
    pub error_message: String,
    
    /// Additional error details
    pub details: Option<Value>,
    
    /// Retry after seconds
    pub retry_after: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ErrorCode {
    InvalidMessage = 1000,
    AuthenticationFailed = 1001,
    DeviceNotFound = 1002,
    ApplicationNotFound = 1003,
    ResourceExhausted = 1004,
    InternalError = 1005,
    UnsupportedFeature = 1006,
    RateLimited = 1007,
}
```

## 🚀 Version Compatibility

### Backward Compatibility

- **Minor Versions**: Backward compatible
- **Major Versions**: May include breaking changes
- **Deprecation**: 6-month deprecation period for removed features

### Version Negotiation

```rust
fn negotiate_version(client_version: ProtocolVersion, server_version: ProtocolVersion) -> ProtocolVersion {
    // Use the lower of the two versions for compatibility
    let major = min(client_version.major, server_version.major);
    let minor = min(client_version.minor, server_version.minor);
    let patch = min(client_version.patch, server_version.patch);
    
    ProtocolVersion { major, minor, patch }
}
```

## 🔍 Debugging and Monitoring

### Protocol Logging

```rust
// Enable protocol debugging
tracing::info!("Sending enrollment request: {:?}", request);
tracing::debug!("CBOR encoded size: {} bytes", encoded.len());
tracing::warn!("Message validation failed: {}", error);
```

### Message Tracing

```rust
// Add correlation ID for request tracing
let correlation_id = uuid::Uuid::new_v4().to_string();
let envelope = Envelope {
    message_id: MessageId::new(),
    version: ProtocolVersion::current(),
    timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
    message: request,
    correlation_id: Some(correlation_id),
};
```

### Performance Monitoring

```rust
#[derive(Debug)]
pub struct ProtocolMetrics {
    /// Messages sent
    pub messages_sent: u64,
    
    /// Messages received
    pub messages_received: u64,
    
    /// Errors encountered
    pub errors: u64,
    
    /// Average latency
    pub average_latency: Duration,
    
    /// Throughput (messages per second)
    pub throughput: f64,
}
```

## 🧪 Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enrollment_serialization() {
        let request = EnrollmentRequest {
            device_type: "hifive1".to_string(),
            capabilities: vec!["wasm".to_string()],
            public_key: "test_key".to_string(),
            firmware_version: "1.0.0".to_string(),
            hardware_id: "test-001".to_string(),
            metadata: None,
        };

        let encoded = serde_cbor::to_vec(&request).unwrap();
        let decoded: EnrollmentRequest = serde_cbor::from_slice(&encoded).unwrap();
        
        assert_eq!(request.device_type, decoded.device_type);
    }

    #[test]
    fn test_message_validation() {
        let request = EnrollmentRequest {
            device_type: "".to_string(), // Invalid empty string
            capabilities: vec![],
            public_key: "".to_string(),
            firmware_version: "1.0.0".to_string(),
            hardware_id: "test-001".to_string(),
            metadata: None,
        };

        let result = validate_enrollment(&request);
        assert!(result.is_err());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_enrollment_flow() {
    let (client, server) = create_test_connection().await;
    
    // Send enrollment request
    let request = create_test_enrollment_request();
    client.send_enrollment(request).await.unwrap();
    
    // Verify enrollment response
    let response = server.receive_enrollment_response().await.unwrap();
    assert!(response.success);
}

#[tokio::test]
async fn test_heartbeat_performance() {
    let (client, server) = create_test_connection().await;
    
    let start = Instant::now();
    for _ in 0..1000 {
        client.send_heartbeat().await.unwrap();
    }
    let duration = start.elapsed();
    
    let throughput = 1000.0 / duration.as_secs_f64();
    assert!(throughput > 100.0); // 100+ messages per second
}
```

## 📋 Compliance

### Standards Compliance

- **TLS**: RFC 8446 (TLS 1.3)
- **CBOR**: RFC 7049
- **UUID**: RFC 4122
- **Base64**: RFC 4648

### Security Compliance

- **Certificate Validation**: RFC 5280
- **TLS Cipher Suites**: RFC 8446 Appendix B
- **Certificate Pinning**: RFC 7469

---

**Last Updated**: September 2024  
**Version**: Protocol v1.0.0  
**Maintainer**: Wasmbed Development Team
