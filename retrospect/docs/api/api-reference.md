# API Reference Documentation

## Overview

This document provides comprehensive API reference documentation for the Wasmbed platform, including REST API endpoints, Custom Resource Definitions (CRDs), and WebAssembly runtime APIs.

## REST API Reference

### Base URL

**Development Environment**:
```
http://localhost:8080
```

**Production Environment**:
```
https://wasmbed-gateway.example.com
```

### Authentication

**API Key Authentication**:
```bash
curl -H "Authorization: Bearer <api-key>" \
     -H "Content-Type: application/json" \
     https://wasmbed-gateway.example.com/api/v1/health
```

**Certificate Authentication**:
```bash
curl --cert client-cert.pem --key client-key.pem \
     -H "Content-Type: application/json" \
     https://wasmbed-gateway.example.com/api/v1/health
```

### Common Response Formats

**Success Response**:
```json
{
  "status": "success",
  "data": {
    "message": "Operation completed successfully"
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

**Error Response**:
```json
{
  "status": "error",
  "error": {
    "code": "INVALID_REQUEST",
    "message": "Invalid request parameters",
    "details": "The provided device ID is invalid"
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## Health and Status Endpoints

### Health Check

**GET /health**

Check the health status of the gateway server.

**Response**:
```json
{
  "status": "success",
  "data": {
    "status": "healthy",
    "version": "1.0.0",
    "uptime": "24h30m15s",
    "components": {
      "gateway": "healthy",
      "kubernetes": "healthy",
      "database": "healthy"
    }
  }
}
```

### System Status

**GET /api/v1/status**

Get detailed system status information.

**Response**:
```json
{
  "status": "success",
  "data": {
    "system": {
      "cpu_usage": 45.2,
      "memory_usage": 67.8,
      "disk_usage": 23.1,
      "network_usage": 12.5
    },
    "devices": {
      "total": 150,
      "active": 142,
      "inactive": 8
    },
    "applications": {
      "total": 25,
      "running": 23,
      "stopped": 2
    }
  }
}
```

## Device Management API

### List Devices

**GET /api/v1/devices**

List all registered devices.

**Query Parameters**:
- `limit`: Maximum number of devices to return (default: 100)
- `offset`: Number of devices to skip (default: 0)
- `status`: Filter by device status (active, inactive, enrolling)
- `type`: Filter by device type (mpu, mcu, riscv)

**Response**:
```json
{
  "status": "success",
  "data": {
    "devices": [
      {
        "id": "device-001",
        "name": "RISC-V Device 1",
        "type": "riscv",
        "status": "active",
        "capabilities": ["wasm", "tls", "serial"],
        "last_heartbeat": "2024-01-01T00:00:00Z",
        "created_at": "2024-01-01T00:00:00Z"
      }
    ],
    "total": 150,
    "limit": 100,
    "offset": 0
  }
}
```

### Get Device

**GET /api/v1/devices/{device_id}**

Get detailed information about a specific device.

**Response**:
```json
{
  "status": "success",
  "data": {
    "id": "device-001",
    "name": "RISC-V Device 1",
    "type": "riscv",
    "status": "active",
    "capabilities": ["wasm", "tls", "serial"],
    "public_key": "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----",
    "gateway_endpoint": "wasmbed-gateway:8080",
    "last_heartbeat": "2024-01-01T00:00:00Z",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
}
```

### Create Device

**POST /api/v1/devices**

Register a new device.

**Request Body**:
```json
{
  "name": "New Device",
  "type": "riscv",
  "capabilities": ["wasm", "tls", "serial"],
  "public_key": "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----",
  "gateway_endpoint": "wasmbed-gateway:8080"
}
```

**Response**:
```json
{
  "status": "success",
  "data": {
    "id": "device-002",
    "name": "New Device",
    "type": "riscv",
    "status": "enrolling",
    "capabilities": ["wasm", "tls", "serial"],
    "public_key": "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----",
    "gateway_endpoint": "wasmbed-gateway:8080",
    "created_at": "2024-01-01T00:00:00Z"
  }
}
```

### Update Device

**PUT /api/v1/devices/{device_id}**

Update device information.

**Request Body**:
```json
{
  "name": "Updated Device Name",
  "capabilities": ["wasm", "tls", "serial", "wifi"]
}
```

**Response**:
```json
{
  "status": "success",
  "data": {
    "id": "device-001",
    "name": "Updated Device Name",
    "type": "riscv",
    "status": "active",
    "capabilities": ["wasm", "tls", "serial", "wifi"],
    "public_key": "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----",
    "gateway_endpoint": "wasmbed-gateway:8080",
    "last_heartbeat": "2024-01-01T00:00:00Z",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
}
```

### Delete Device

**DELETE /api/v1/devices/{device_id}**

Remove a device from the system.

**Response**:
```json
{
  "status": "success",
  "data": {
    "message": "Device deleted successfully"
  }
}
```

## Application Management API

### List Applications

**GET /api/v1/applications**

List all deployed applications.

**Query Parameters**:
- `limit`: Maximum number of applications to return (default: 100)
- `offset`: Number of applications to skip (default: 0)
- `status`: Filter by application status (running, stopped, deploying)
- `device_type`: Filter by target device type

**Response**:
```json
{
  "status": "success",
  "data": {
    "applications": [
      {
        "id": "app-001",
        "name": "PX4 Drone Control",
        "version": "1.0.0",
        "status": "running",
        "target_devices": ["riscv"],
        "deployed_at": "2024-01-01T00:00:00Z",
        "last_update": "2024-01-01T00:00:00Z"
      }
    ],
    "total": 25,
    "limit": 100,
    "offset": 0
  }
}
```

### Get Application

**GET /api/v1/applications/{app_id}**

Get detailed information about a specific application.

**Response**:
```json
{
  "status": "success",
  "data": {
    "id": "app-001",
    "name": "PX4 Drone Control",
    "version": "1.0.0",
    "status": "running",
    "target_devices": ["riscv"],
    "wasm_binary": "<base64-encoded-wasm-binary>",
    "config": {
      "microros": {
        "node_name": "px4_drone_control_node",
        "domain_id": 0
      },
      "fastdds": {
        "domain_id": 0,
        "transport": "udp"
      }
    },
    "deployed_at": "2024-01-01T00:00:00Z",
    "last_update": "2024-01-01T00:00:00Z"
  }
}
```

### Deploy Application

**POST /api/v1/applications**

Deploy a new application.

**Request Body**:
```json
{
  "name": "New Application",
  "version": "1.0.0",
  "target_devices": ["riscv"],
  "wasm_binary": "<base64-encoded-wasm-binary>",
  "config": {
    "microros": {
      "node_name": "new_app_node",
      "domain_id": 0
    }
  }
}
```

**Response**:
```json
{
  "status": "success",
  "data": {
    "id": "app-002",
    "name": "New Application",
    "version": "1.0.0",
    "status": "deploying",
    "target_devices": ["riscv"],
    "wasm_binary": "<base64-encoded-wasm-binary>",
    "config": {
      "microros": {
        "node_name": "new_app_node",
        "domain_id": 0
      }
    },
    "deployed_at": "2024-01-01T00:00:00Z"
  }
}
```

### Update Application

**PUT /api/v1/applications/{app_id}**

Update application configuration.

**Request Body**:
```json
{
  "version": "1.1.0",
  "config": {
    "microros": {
      "node_name": "updated_app_node",
      "domain_id": 0
    }
  }
}
```

**Response**:
```json
{
  "status": "success",
  "data": {
    "id": "app-001",
    "name": "PX4 Drone Control",
    "version": "1.1.0",
    "status": "updating",
    "target_devices": ["riscv"],
    "wasm_binary": "<base64-encoded-wasm-binary>",
    "config": {
      "microros": {
        "node_name": "updated_app_node",
        "domain_id": 0
      }
    },
    "deployed_at": "2024-01-01T00:00:00Z",
    "last_update": "2024-01-01T00:00:00Z"
  }
}
```

### Delete Application

**DELETE /api/v1/applications/{app_id}**

Remove an application from the system.

**Response**:
```json
{
  "status": "success",
  "data": {
    "message": "Application deleted successfully"
  }
}
```

## Admin API

### Pairing Mode Management

**GET /api/v1/admin/pairing-mode**

Get current pairing mode status.

**Response**:
```json
{
  "status": "success",
  "data": {
    "pairing_mode": false,
    "pairing_timeout": 300,
    "pairing_max_attempts": 3
  }
}
```

**POST /api/v1/admin/pairing-mode**

Enable or disable pairing mode.

**Request Body**:
```json
{
  "pairing_mode": true,
  "pairing_timeout": 600,
  "pairing_max_attempts": 5
}
```

**Response**:
```json
{
  "status": "success",
  "data": {
    "pairing_mode": true,
    "pairing_timeout": 600,
    "pairing_max_attempts": 5
  }
}
```

### Heartbeat Configuration

**GET /api/v1/admin/heartbeat-timeout**

Get current heartbeat timeout configuration.

**Response**:
```json
{
  "status": "success",
  "data": {
    "heartbeat_interval": 30,
    "heartbeat_timeout": 90,
    "heartbeat_max_misses": 3
  }
}
```

**POST /api/v1/admin/heartbeat-timeout**

Update heartbeat timeout configuration.

**Request Body**:
```json
{
  "heartbeat_interval": 60,
  "heartbeat_timeout": 180,
  "heartbeat_max_misses": 5
}
```

**Response**:
```json
{
  "status": "success",
  "data": {
    "heartbeat_interval": 60,
    "heartbeat_timeout": 180,
    "heartbeat_max_misses": 5
  }
}
```

## Custom Resource Definitions (CRDs)

### Application CRD

**API Version**: `wasmbed.github.io/v1alpha1`
**Kind**: `Application`

**Schema**:
```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: applications.wasmbed.github.io
spec:
  group: wasmbed.github.io
  versions:
  - name: v1alpha1
    served: true
    storage: true
    schema:
      openAPIV3Schema:
        type: object
        properties:
          spec:
            type: object
            properties:
              name:
                type: string
                description: "Application name"
              version:
                type: string
                description: "Application version"
              targetDevices:
                type: array
                items:
                  type: string
                description: "Target device types"
              wasmBinary:
                type: string
                format: byte
                description: "Base64-encoded WASM binary"
              config:
                type: object
                description: "Application configuration"
            required:
            - name
            - version
            - targetDevices
            - wasmBinary
          status:
            type: object
            properties:
              phase:
                type: string
                enum: ["Creating", "Deploying", "Running", "Stopping", "Stopped", "Failed"]
              message:
                type: string
              deployedAt:
                type: string
                format: date-time
              lastUpdate:
                type: string
                format: date-time
```

**Example**:
```yaml
apiVersion: wasmbed.github.io/v1alpha1
kind: Application
metadata:
  name: px4-drone-control
  namespace: wasmbed
spec:
  name: "PX4 Drone Control"
  version: "1.0.0"
  targetDevices:
  - "riscv"
  wasmBinary: <base64-encoded-wasm-binary>
  config:
    microros:
      node_name: "px4_drone_control_node"
      domain_id: 0
    fastdds:
      domain_id: 0
      transport: "udp"
status:
  phase: "Running"
  message: "Application deployed successfully"
  deployedAt: "2024-01-01T00:00:00Z"
  lastUpdate: "2024-01-01T00:00:00Z"
```

### Device CRD

**API Version**: `wasmbed.github.io/v1alpha1`
**Kind**: `Device`

**Schema**:
```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: devices.wasmbed.github.io
spec:
  group: wasmbed.github.io
  versions:
  - name: v1alpha1
    served: true
    storage: true
    schema:
      openAPIV3Schema:
        type: object
        properties:
          spec:
            type: object
            properties:
              deviceType:
                type: string
                description: "Device type (mpu, mcu, riscv)"
              capabilities:
                type: array
                items:
                  type: string
                description: "Device capabilities"
              publicKey:
                type: string
                description: "Device public key"
              gatewayEndpoint:
                type: string
                description: "Gateway endpoint"
            required:
            - deviceType
            - capabilities
            - publicKey
            - gatewayEndpoint
          status:
            type: object
            properties:
              phase:
                type: string
                enum: ["Enrolling", "Enrolled", "Connected", "Disconnected", "Failed"]
              message:
                type: string
              lastHeartbeat:
                type: string
                format: date-time
              pairingMode:
                type: boolean
```

**Example**:
```yaml
apiVersion: wasmbed.github.io/v1alpha1
kind: Device
metadata:
  name: riscv-device-001
  namespace: wasmbed
spec:
  deviceType: "riscv"
  capabilities:
  - "wasm"
  - "tls"
  - "serial"
  publicKey: "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----"
  gatewayEndpoint: "wasmbed-gateway:8080"
status:
  phase: "Connected"
  message: "Device connected successfully"
  lastHeartbeat: "2024-01-01T00:00:00Z"
  pairingMode: false
```

## WebAssembly Runtime API

### WASM Runtime Interface

**Runtime Initialization**:
```rust
pub struct WasmRuntime {
    modules: HashMap<String, WasmModule>,
    instances: HashMap<String, WasmInstance>,
    memory_manager: WasmMemoryManager,
}

impl WasmRuntime {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            instances: HashMap::new(),
            memory_manager: WasmMemoryManager::new(),
        }
    }
    
    pub fn load_module(&mut self, name: &str, wasm_binary: &[u8]) -> Result<(), Error> {
        // Load WASM module
        let module = WasmModule::from_bytes(wasm_binary)?;
        self.modules.insert(name.to_string(), module);
        Ok(())
    }
    
    pub fn create_instance(&mut self, module_name: &str, instance_name: &str) -> Result<(), Error> {
        // Create WASM instance
        if let Some(module) = self.modules.get(module_name) {
            let instance = WasmInstance::new(module)?;
            self.instances.insert(instance_name.to_string(), instance);
            Ok(())
        } else {
            Err(Error::ModuleNotFound)
        }
    }
    
    pub fn call_function(&mut self, instance_name: &str, function_name: &str, args: &[Value]) -> Result<Value, Error> {
        // Call WASM function
        if let Some(instance) = self.instances.get_mut(instance_name) {
            instance.call_function(function_name, args)
        } else {
            Err(Error::InstanceNotFound)
        }
    }
}
```

### WASM Application Interface

**Application Interface**:
```rust
pub trait WasmApplication {
    fn initialize(&mut self) -> Result<(), Error>;
    fn execute(&mut self) -> Result<(), Error>;
    fn cleanup(&mut self) -> Result<(), Error>;
    fn get_status(&self) -> ApplicationStatus;
    fn get_metrics(&self) -> ApplicationMetrics;
}

pub struct ApplicationStatus {
    pub status: String,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub execution_time: f32,
    pub error_count: u32,
}

pub struct ApplicationMetrics {
    pub function_calls: u64,
    pub memory_allocations: u64,
    pub execution_time: Duration,
    pub error_count: u64,
}
```

### PX4 Application Interface

**PX4 Application Interface**:
```rust
pub trait Px4Application: WasmApplication {
    fn initialize_px4_integration(&mut self) -> Result<(), Error>;
    fn process_px4_command(&mut self, command: Px4Command) -> Result<(), Error>;
    fn get_px4_status(&self) -> Px4Status;
    fn set_flight_mode(&mut self, mode: FlightMode) -> Result<(), Error>;
    fn emergency_stop(&mut self) -> Result<(), Error>;
}

pub struct Px4Command {
    pub command_type: Px4CommandType,
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
    pub heading: f64,
    pub speed: f64,
    pub timestamp: u64,
}

pub struct Px4Status {
    pub armed: bool,
    pub flight_mode: FlightMode,
    pub battery_level: f32,
    pub position: Position,
    pub attitude: Attitude,
    pub velocity: Velocity,
}
```

## Error Codes

### HTTP Status Codes

- `200 OK`: Request successful
- `201 Created`: Resource created successfully
- `400 Bad Request`: Invalid request parameters
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Access denied
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource conflict
- `500 Internal Server Error`: Server error

### Error Codes

**Device Errors**:
- `DEVICE_NOT_FOUND`: Device does not exist
- `DEVICE_ALREADY_EXISTS`: Device already registered
- `DEVICE_INVALID_TYPE`: Invalid device type
- `DEVICE_INVALID_CAPABILITIES`: Invalid device capabilities
- `DEVICE_CONNECTION_FAILED`: Device connection failed

**Application Errors**:
- `APPLICATION_NOT_FOUND`: Application does not exist
- `APPLICATION_ALREADY_EXISTS`: Application already deployed
- `APPLICATION_INVALID_BINARY`: Invalid WASM binary
- `APPLICATION_DEPLOYMENT_FAILED`: Application deployment failed
- `APPLICATION_EXECUTION_FAILED`: Application execution failed

**System Errors**:
- `SYSTEM_UNAVAILABLE`: System temporarily unavailable
- `SYSTEM_OVERLOADED`: System overloaded
- `SYSTEM_MAINTENANCE`: System under maintenance
- `SYSTEM_ERROR`: Internal system error

## Rate Limiting

### Rate Limits

**Default Limits**:
- API requests: 1000 requests per minute
- Device registrations: 100 registrations per hour
- Application deployments: 50 deployments per hour
- File uploads: 10 uploads per hour

**Rate Limit Headers**:
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1640995200
```

**Rate Limit Exceeded Response**:
```json
{
  "status": "error",
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded",
    "details": "Too many requests. Please try again later."
  },
  "timestamp": "2024-01-01T00:00:00Z"
}
```

## SDKs and Client Libraries

### Rust Client

**Installation**:
```toml
[dependencies]
wasmbed-client = "1.0.0"
```

**Usage**:
```rust
use wasmbed_client::WasmbedClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = WasmbedClient::new("https://wasmbed-gateway.example.com")?;
    
    // List devices
    let devices = client.list_devices().await?;
    println!("Found {} devices", devices.len());
    
    // Deploy application
    let app = client.deploy_application("My App", "1.0.0", &wasm_binary).await?;
    println!("Deployed application: {}", app.id);
    
    Ok(())
}
```

### Python Client

**Installation**:
```bash
pip install wasmbed-client
```

**Usage**:
```python
from wasmbed_client import WasmbedClient

client = WasmbedClient("https://wasmbed-gateway.example.com")

# List devices
devices = client.list_devices()
print(f"Found {len(devices)} devices")

# Deploy application
app = client.deploy_application("My App", "1.0.0", wasm_binary)
print(f"Deployed application: {app.id}")
```

### JavaScript Client

**Installation**:
```bash
npm install wasmbed-client
```

**Usage**:
```javascript
const WasmbedClient = require('wasmbed-client');

const client = new WasmbedClient('https://wasmbed-gateway.example.com');

// List devices
client.listDevices().then(devices => {
    console.log(`Found ${devices.length} devices`);
});

// Deploy application
client.deployApplication('My App', '1.0.0', wasmBinary).then(app => {
    console.log(`Deployed application: ${app.id}`);
});
```
