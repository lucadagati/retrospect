# Wasmbed Platform API Reference

This document provides comprehensive documentation for the Wasmbed Platform REST API. The API enables management of devices, applications, gateways, and system monitoring through HTTP endpoints.

## Base URL

The API is available at: `http://localhost:3001`

## Authentication

Currently, the API does not require authentication for local development. In production environments, authentication will be implemented using JWT tokens or API keys.

## Content Type

All API requests and responses use JSON format with the `Content-Type: application/json` header.

## Endpoints

### Health Check

#### GET /health

Check the health status of the API server.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": 1761156001
}
```

### Devices

#### GET /api/v1/devices

Retrieve all devices managed by the platform.

**Response:**
```json
{
  "devices": [
    {
      "id": "device-001",
      "name": "Arduino Nano 33 BLE",
      "architecture": "ARM_CORTEX_M",
      "device_type": "MCU",
      "mcu_type": "RenodeArduinoNano33Ble",
      "status": "Running",
      "endpoint": "127.0.0.1:30001",
      "wasm_runtime": null
    }
  ]
}
```

#### POST /api/v1/devices

Create a new device.

**Request Body:**
```json
{
  "id": "device-001",
  "name": "Arduino Nano 33 BLE",
  "architecture": "ARM_CORTEX_M",
  "device_type": "MCU",
  "mcu_type": "RenodeArduinoNano33Ble"
}
```

**Response:**
```json
{
  "id": "device-001",
  "name": "Arduino Nano 33 BLE",
  "architecture": "ARM_CORTEX_M",
  "device_type": "MCU",
  "mcu_type": "RenodeArduinoNano33Ble",
  "status": "Stopped",
  "endpoint": "127.0.0.1:30001",
  "wasm_runtime": null
}
```

#### GET /api/v1/devices/{id}

Retrieve a specific device by ID.

**Response:**
```json
{
  "id": "device-001",
  "name": "Arduino Nano 33 BLE",
  "architecture": "ARM_CORTEX_M",
  "device_type": "MCU",
  "mcu_type": "RenodeArduinoNano33Ble",
  "status": "Running",
  "endpoint": "127.0.0.1:30001",
  "wasm_runtime": null
}
```

#### POST /api/v1/devices/{id}/renode/start

Start a Renode device emulation.

**Response:**
```json
{
  "status": "success",
  "message": "Device started successfully"
}
```

#### POST /api/v1/devices/{id}/renode/stop

Stop a Renode device emulation.

**Response:**
```json
{
  "status": "success",
  "message": "Device stopped successfully"
}
```

### Applications

#### GET /api/v1/applications

Retrieve all applications deployed on the platform.

**Response:**
```json
{
  "applications": [
    {
      "id": "app-001",
      "name": "Hello World",
      "wasm_binary": "base64-encoded-wasm",
      "target_devices": ["device-001"],
      "status": "Deployed",
      "deployment_progress": {
        "target_count": 1,
        "deployed_count": 1,
        "progress_percentage": 100
      }
    }
  ]
}
```

#### POST /api/v1/applications

Deploy a new WASM application.

**Request Body:**
```json
{
  "id": "app-001",
  "name": "Hello World",
  "wasm_binary": "base64-encoded-wasm",
  "target_devices": ["device-001"]
}
```

**Response:**
```json
{
  "id": "app-001",
  "name": "Hello World",
  "wasm_binary": "base64-encoded-wasm",
  "target_devices": ["device-001"],
  "status": "Deploying",
  "deployment_progress": {
    "target_count": 1,
    "deployed_count": 0,
    "progress_percentage": 0
  }
}
```

### Gateways

#### GET /api/v1/gateways

Retrieve all gateways in the system.

**Response:**
```json
{
  "gateways": [
    {
      "id": "gateway-001",
      "name": "Main Gateway",
      "endpoint": "127.0.0.1:8081",
      "status": "Running",
      "connected_devices": ["device-001"],
      "wasm_runtime": "wasmtime"
    }
  ]
}
```

### Compilation

#### POST /api/v1/compile

Compile Rust code to WebAssembly.

**Request Body:**
```json
{
  "rust_code": "fn main() { println!(\"Hello, world!\"); }"
}
```

**Response:**
```json
{
  "wasm_binary": "base64-encoded-wasm",
  "compilation_successful": true,
  "errors": null
}
```

## Error Responses

All endpoints may return error responses in the following format:

```json
{
  "error": "Error message",
  "code": "ERROR_CODE",
  "details": "Additional error details"
}
```

### Common Error Codes

- `400` - Bad Request: Invalid request format or parameters
- `404` - Not Found: Resource not found
- `500` - Internal Server Error: Server-side error
- `503` - Service Unavailable: Service temporarily unavailable

## Rate Limiting

Currently, no rate limiting is implemented. In production, rate limiting will be added to prevent abuse.

## Examples

### Create and Start a Device

```bash
# Create device
curl -X POST http://localhost:3001/api/v1/devices \
  -H "Content-Type: application/json" \
  -d '{
    "id": "test-device",
    "name": "Test Device",
    "architecture": "ARM_CORTEX_M",
    "device_type": "MCU",
    "mcu_type": "RenodeArduinoNano33Ble"
  }'

# Start device
curl -X POST http://localhost:3001/api/v1/devices/test-device/renode/start
```

### Deploy an Application

```bash
# Compile Rust to WASM
curl -X POST http://localhost:3001/api/v1/compile \
  -H "Content-Type: application/json" \
  -d '{
    "rust_code": "fn main() { println!(\"Hello from WASM!\"); }"
  }'

# Deploy application
curl -X POST http://localhost:3001/api/v1/applications \
  -H "Content-Type: application/json" \
  -d '{
    "id": "hello-app",
    "name": "Hello Application",
    "wasm_binary": "base64-encoded-wasm",
    "target_devices": ["test-device"]
  }'
```

## Service Endpoints

| Service | Endpoint | Port | Description |
|---------|----------|------|-------------|
| Dashboard UI | http://localhost:3000 | 3000 | React web interface |
| Dashboard API | http://localhost:3001 | 3001 | REST API (this document) |
| Infrastructure API | http://localhost:30460 | 30460 | Infrastructure services |
| Gateway HTTP API | http://localhost:8080 | 8080 | Gateway management |
| Gateway TLS | 127.0.0.1:8081 | 8081 | Device communication (TLS) |

## Implementation Notes

- All API endpoints use real implementations (no mocks)
- Device management integrates with Renode for constrained device emulation
- WASM compilation uses real Rust toolchain
- TLS communication uses real certificates and rustls
- All responses reflect actual system state