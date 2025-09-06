# Wasmbed Platform API Documentation

## Overview

Wasmbed è una piattaforma completa per il deployment e l'esecuzione di applicazioni WebAssembly su dispositivi IoT edge, utilizzando Kubernetes come control plane e Gateway come intermediari per la comunicazione con i dispositivi MCU.

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Kubernetes    │    │     Gateway     │    │   MCU Devices   │
│   Control Plane │◄──►│      (MPU)      │◄──►│   (ESP32/RISC-V)│
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Core Components

### 1. Kubernetes Custom Resources

#### Device Resource
```yaml
apiVersion: wasmbed.github.io/v1
kind: Device
metadata:
  name: device-name
  namespace: wasmbed
spec:
  deviceId: "unique-device-id"
  publicKey: "base64-encoded-public-key"
  deviceType: "hifive1" | "esp32"
  capabilities: ["wasm", "tls", "wifi"]
  state: "online" | "offline" | "error"
  lastSeen: "2025-01-27T10:00:00Z"
  applications: ["app1", "app2"]
```

#### Application Resource
```yaml
apiVersion: wasmbed.github.io/v1
kind: Application
metadata:
  name: app-name
  namespace: wasmbed
spec:
  name: "application-name"
  wasmBinary: "base64-encoded-wasm-binary"
  targetDevices: ["device1", "device2"]
  config:
    memoryLimit: 1048576
    executionTimeout: "30s"
    maxStackSize: 65536
  state: "deployed" | "running" | "stopped" | "error"
  deployedDevices: ["device1"]
  lastDeployed: "2025-01-27T10:00:00Z"
```

### 2. Gateway HTTP API

#### Endpoints

##### GET /health
Health check del Gateway.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-01-27T10:00:00Z",
  "version": "0.1.0"
}
```

##### POST /api/v1/devices
Registra un nuovo dispositivo.

**Request:**
```json
{
  "deviceId": "device-123",
  "publicKey": "base64-encoded-public-key",
  "deviceType": "hifive1",
  "capabilities": ["wasm", "tls"]
}
```

**Response:**
```json
{
  "success": true,
  "deviceId": "device-123",
  "enrollmentStatus": "pending"
}
```

##### GET /api/v1/devices/{deviceId}
Ottieni informazioni su un dispositivo.

**Response:**
```json
{
  "deviceId": "device-123",
  "publicKey": "base64-encoded-public-key",
  "deviceType": "hifive1",
  "capabilities": ["wasm", "tls"],
  "state": "online",
  "lastSeen": "2025-01-27T10:00:00Z",
  "applications": ["app1", "app2"]
}
```

##### POST /api/v1/applications
Deploya una nuova applicazione.

**Request:**
```json
{
  "name": "my-app",
  "wasmBinary": "base64-encoded-wasm-binary",
  "targetDevices": ["device-123"],
  "config": {
    "memoryLimit": 1048576,
    "executionTimeout": "30s",
    "maxStackSize": 65536
  }
}
```

**Response:**
```json
{
  "success": true,
  "applicationId": "app-456",
  "deploymentStatus": "deploying"
}
```

##### GET /api/v1/applications/{appId}
Ottieni informazioni su un'applicazione.

**Response:**
```json
{
  "applicationId": "app-456",
  "name": "my-app",
  "state": "running",
  "deployedDevices": ["device-123"],
  "lastDeployed": "2025-01-27T10:00:00Z",
  "metrics": {
    "memoryUsage": 512000,
    "executionTime": 150,
    "functionCalls": 1000
  }
}
```

##### POST /api/v1/applications/{appId}/start
Avvia un'applicazione.

**Response:**
```json
{
  "success": true,
  "applicationId": "app-456",
  "status": "starting"
}
```

##### POST /api/v1/applications/{appId}/stop
Ferma un'applicazione.

**Response:**
```json
{
  "success": true,
  "applicationId": "app-456",
  "status": "stopping"
}
```

##### DELETE /api/v1/applications/{appId}
Rimuovi un'applicazione.

**Response:**
```json
{
  "success": true,
  "applicationId": "app-456",
  "status": "removed"
}
```

##### GET /api/v1/metrics
Ottieni metriche del sistema.

**Response:**
```json
{
  "timestamp": "2025-01-27T10:00:00Z",
  "devices": {
    "total": 10,
    "online": 8,
    "offline": 2
  },
  "applications": {
    "total": 25,
    "running": 20,
    "stopped": 5
  },
  "performance": {
    "avgResponseTime": 150,
    "throughput": 1000
  }
}
```

### 3. Protocol Messages

#### Client Messages

##### Device Registration
```rust
ClientMessage::DeviceRegistration {
    device_id: DeviceId,
    public_key: PublicKey,
    device_type: DeviceType,
    capabilities: Vec<Capability>,
}
```

##### Heartbeat
```rust
ClientMessage::Heartbeat {
    timestamp: SystemTime,
    metrics: DeviceMetrics,
}
```

##### Application Status
```rust
ClientMessage::ApplicationStatus {
    application_id: ApplicationId,
    status: ApplicationStatus,
    metrics: ApplicationMetrics,
}
```

##### Error Report
```rust
ClientMessage::ErrorReport {
    error_type: ErrorType,
    message: String,
    timestamp: SystemTime,
}
```

#### Server Messages

##### Application Deployment
```rust
ServerMessage::ApplicationDeployment {
    application_id: ApplicationId,
    wasm_binary: Vec<u8>,
    config: ApplicationConfig,
}
```

##### Application Control
```rust
ServerMessage::ApplicationControl {
    application_id: ApplicationId,
    command: ApplicationCommand,
}
```

##### Configuration Update
```rust
ServerMessage::ConfigurationUpdate {
    device_id: DeviceId,
    config: DeviceConfig,
}
```

##### Error Response
```rust
ServerMessage::ErrorResponse {
    error_code: ErrorCode,
    message: String,
}
```

### 4. WASM Runtime API

#### RISC-V Runtime (HiFive1 QEMU)

```rust
// Creazione runtime
let config = WasmRuntimeConfig {
    max_memory_per_app: 1024 * 1024,
    max_concurrent_apps: 4,
    default_timeout: Duration::from_secs(30),
    max_stack_size: 64 * 1024,
};
let mut runtime = WasmRuntime::new(config)?;

// Caricamento modulo
runtime.load_module("app-1", &wasm_binary)?;

// Esecuzione funzione
let result = runtime.execute_function("app-1", "main", &[])?;

// Gestione memoria
let memory_info = runtime.get_memory_info("app-1");
```

#### ESP32 Runtime (wasmi)

```rust
// Creazione runtime
let mut runtime = WasmRuntime::new(WasmRuntimeConfig::default())?;

// Caricamento modulo
let module = Module::new(&engine, &wasm_binary)?;
let instance = runtime.instantiate(&module)?;

// Esecuzione funzione
let result = instance.call_exported_function("main", &[])?;
```

### 5. Security

#### TLS Configuration
- **CA Certificate**: `/etc/wasmbed/ca-cert.pem`
- **Server Certificate**: `/etc/wasmbed/server-cert.pem`
- **Server Private Key**: `/etc/wasmbed/server-key.pem`
- **Client Certificate**: `/etc/wasmbed/client-cert.pem`

#### Message Signing
Tutti i messaggi sono firmati usando Ed25519:
```rust
let signature = signing_key.sign(&message_bytes);
let signed_message = SignedMessage {
    data: message_bytes,
    signature: signature.to_bytes(),
};
```

#### Message Encryption
I messaggi sensibili sono crittografati usando AES-256-GCM:
```rust
let cipher = Aes256Gcm::new(&key);
let ciphertext = cipher.encrypt(&nonce, &plaintext)?;
```

### 6. Error Handling

#### Error Codes
- `E001`: Invalid device ID
- `E002`: Authentication failed
- `E003`: Application not found
- `E004`: Device offline
- `E005`: WASM execution error
- `E006`: Memory limit exceeded
- `E007`: Timeout exceeded
- `E008`: Invalid configuration
- `E009`: Network error
- `E010`: Internal server error

#### Error Response Format
```json
{
  "error": {
    "code": "E005",
    "message": "WASM execution error",
    "details": "Function 'main' not found",
    "timestamp": "2025-01-27T10:00:00Z"
  }
}
```

### 7. Monitoring and Metrics

#### Device Metrics
```rust
struct DeviceMetrics {
    cpu_usage: f64,        // 0.0 - 1.0
    memory_usage: usize,   // bytes
    network_rx: u64,       // bytes received
    network_tx: u64,       // bytes transmitted
}
```

#### Application Metrics
```rust
struct ApplicationMetrics {
    memory_usage: usize,      // bytes
    execution_count: u64,     // function calls
    avg_execution_time: u64,  // microseconds
    error_count: u64,         // errors
}
```

#### System Metrics
```rust
struct SystemMetrics {
    devices_total: u32,
    devices_online: u32,
    applications_total: u32,
    applications_running: u32,
    avg_response_time: u64,    // milliseconds
    throughput: u64,          // requests per second
}
```

### 8. Configuration

#### Gateway Configuration
```yaml
# gateway-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: wasmbed-gateway
  namespace: wasmbed
spec:
  replicas: 1
  selector:
    matchLabels:
      app: wasmbed-gateway
  template:
    metadata:
      labels:
        app: wasmbed-gateway
    spec:
      containers:
      - name: gateway
        image: wasmbed-gateway:latest
        ports:
        - containerPort: 8080
        - containerPort: 8443
        env:
        - name: PRIVATE_KEY
          value: "/etc/wasmbed/server-key.pem"
        - name: CERTIFICATE
          value: "/etc/wasmbed/server-cert.pem"
        - name: CLIENT_CA
          value: "/etc/wasmbed/ca-cert.pem"
        - name: POD_NAMESPACE
          valueFrom:
            fieldRef:
              fieldPath: metadata.namespace
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        volumeMounts:
        - name: certs
          mountPath: /etc/wasmbed
      volumes:
      - name: certs
        secret:
          secretName: wasmbed-certs
```

#### Device Configuration
```yaml
# test-device.yaml
apiVersion: wasmbed.github.io/v1
kind: Device
metadata:
  name: test-hifive1-device
  namespace: wasmbed
spec:
  deviceId: "hifive1-test-001"
  publicKey: "dGVzdC1wdWJsaWMta2V5LWZvci1oaWZpdmUx"
  deviceType: "hifive1"
  capabilities: ["wasm"]
  state: "online"
  lastSeen: "2025-01-27T10:00:00Z"
  applications: []
```

### 9. Testing

#### Unit Tests
```bash
cargo test --workspace --lib
```

#### Integration Tests
```bash
cargo test --manifest-path tests/Cargo.toml
```

#### End-to-End Tests
```bash
./scripts/run-all-tests.sh
```

#### Manual Testing
```bash
# Avvia cluster Kubernetes
k3d cluster create wasmbed-test

# Deploy Gateway
kubectl apply -f gateway-deployment.yaml

# Deploy test resources
kubectl apply -f test-device.yaml
kubectl apply -f test-wasm-app.yaml

# Compila e testa firmware RISC-V
cargo build -p wasmbed-firmware-hifive1-qemu --target riscv32imac-unknown-none-elf
qemu-system-riscv32 -machine sifive_e -kernel target/riscv32imac-unknown-none-elf/debug/wasmbed-firmware-hifive1-qemu -nographic
```

### 10. Troubleshooting

#### Common Issues

##### Gateway TLS Errors
```bash
# Verifica certificati
openssl x509 -in certs/server-cert.pem -text -noout

# Rigenera certificati
./scripts/generate-certs.sh
```

##### Kubernetes Connection Issues
```bash
# Verifica cluster
kubectl cluster-info

# Ricrea cluster
k3d cluster delete wasmbed-test
k3d cluster create wasmbed-test
```

##### WASM Runtime Errors
```bash
# Verifica compilazione firmware
cargo check -p wasmbed-firmware-hifive1-qemu --target riscv32imac-unknown-none-elf

# Test runtime
cargo test --manifest-path tests/Cargo.toml riscv_runtime_tests
```

#### Logs
```bash
# Gateway logs
kubectl logs -l app=wasmbed-gateway -n wasmbed

# Device logs (QEMU)
qemu-system-riscv32 -machine sifive_e -kernel firmware.bin -nographic -serial stdio
```

### 11. Performance Tuning

#### Gateway Performance
- Aumenta replicas del Gateway per carichi elevati
- Configura resource limits appropriati
- Usa persistent volumes per certificati

#### WASM Runtime Performance
- Ottimizza `max_memory_per_app` in base alle esigenze
- Configura `execution_timeout` appropriato
- Monitora metriche di performance

#### Kubernetes Performance
- Usa nodi con risorse sufficienti
- Configura resource quotas per namespace
- Monitora utilizzo CPU/memoria

### 12. Security Best Practices

#### Certificate Management
- Rinnova certificati prima della scadenza
- Usa certificati con validità appropriata
- Implementa rotation automatica

#### Network Security
- Usa TLS per tutte le comunicazioni
- Implementa firewall rules appropriate
- Monitora traffico di rete

#### Access Control
- Implementa RBAC per Kubernetes
- Usa service accounts appropriati
- Monitora accessi e modifiche

---

## Support

Per supporto e domande:
- **Issues**: GitHub Issues
- **Documentation**: Questo documento
- **Examples**: Cartella `examples/`
- **Tests**: Cartella `tests/`
