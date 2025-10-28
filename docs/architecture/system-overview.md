# System Architecture Overview

## Introduction

The Wasmbed platform implements a comprehensive 3-layer architecture designed for secure deployment and execution of WebAssembly applications on constrained devices. The architecture follows cloud-fog-edge principles with Kubernetes orchestration and real-time communication capabilities using Renode emulation.

## High-Level Architecture

```mermaid
graph TB
    subgraph "Cloud Layer"
        K8S[Kubernetes Cluster]
        CTRL[Custom Controllers]
        CRD[Application/Device CRDs]
        RBAC[RBAC & Security Policies]
        ETCD[etcd State Store]
        API[Kubernetes API Server]
    end
    
    subgraph "Fog Layer"
        GW[Gateway Server]
        TLS[TLS Bridge]
        HTTP[HTTP API Server]
        HB[Heartbeat Monitor]
        AUTH[Device Authentication]
        WASM_RT[WASM Runtime]
    end
    
    subgraph "Edge Layer"
        RENODE[Renode Emulation]
        ARM[ARM Cortex-M4 Devices]
        FIRMWARE[Real Firmware]
        TLS_CLIENT[TLS Client]
        WASM[WebAssembly Runtime]
    end
    
    K8S --> GW
    CTRL --> GW
    CRD --> GW
    RBAC --> GW
    ETCD --> K8S
    API --> CTRL
    
    GW --> TLS
    GW --> HTTP
    GW --> HB
    GW --> AUTH
    GW --> WASM_RT
    
    TLS --> RENODE
    AUTH --> RENODE
    WASM_RT --> RENODE
    
    RENODE --> ARM
    ARM --> FIRMWARE
    FIRMWARE --> TLS_CLIENT
    FIRMWARE --> WASM
```

## Architectural Layers

### Cloud Layer

The cloud layer provides centralized orchestration and management capabilities through Kubernetes.

```mermaid
graph LR
    subgraph "Kubernetes Cluster"
        API[API Server]
        CTRL[Controller Manager]
        SCHED[Scheduler]
        ETCD[etcd]
    end
    
    subgraph "Custom Resources"
        APP[Application CRD]
        DEV[Device CRD]
        GW[Gateway CRD]
    end
    
    subgraph "Custom Controllers"
        APP_CTRL[Application Controller]
        DEV_CTRL[Device Controller]
        GW_CTRL[Gateway Controller]
    end
    
    subgraph "RBAC"
        SA[Service Accounts]
        ROLE[Roles]
        RB[Role Bindings]
    end
    
    API --> CTRL
    CTRL --> APP_CTRL
    CTRL --> DEV_CTRL
    CTRL --> GW_CTRL
    
    APP_CTRL --> APP
    DEV_CTRL --> DEV
    GW_CTRL --> GW
    
    SA --> ROLE
    ROLE --> RB
```

#### Components

**Kubernetes Cluster**
- Custom Resource Definitions (CRDs) for Applications, Devices, and Gateways
- Controllers for application lifecycle management
- RBAC for secure access control
- etcd for state persistence

**Custom Controllers**
- Application Controller: Manages WASM application deployment
- Device Controller: Handles device registration and status
- Gateway Controller: Manages gateway instances
- Reconciliation loops for maintaining desired state

**API Server**
- RESTful API for platform management
- CRD validation and admission control
- Resource versioning and conflict resolution

### Fog Layer

The fog layer consists of gateway servers that bridge cloud orchestration with edge device communication.

```mermaid
graph TB
    subgraph "Gateway Server"
        HTTP[HTTP API Server :8080]
        TLS_SRV[TLS Server :8081]
        HB_MON[Heartbeat Monitor]
        APP_MGR[Application Manager]
        AUTH_SRV[Authentication Service]
        WASM_RT[WASM Runtime]
    end
    
    subgraph "Communication Protocols"
        TLS_PROTO[TLS 1.3]
        CBOR_PROTO[CBOR Protocol]
        HTTP_PROTO[HTTP/2]
    end
    
    subgraph "Security"
        CERT_MGR[Certificate Manager]
        KEY_STORE[Key Store]
        AUTH_MGR[Authentication Manager]
    end
    
    HTTP --> HTTP_PROTO
    TLS_SRV --> TLS_PROTO
    AUTH_SRV --> CBOR_PROTO
    
    CERT_MGR --> TLS_SRV
    KEY_STORE --> AUTH_SRV
    AUTH_MGR --> AUTH_SRV
```

#### Gateway Server

**Core Functionality**
- TLS-secured communication hub
- Device enrollment and authentication
- Application deployment coordination
- Heartbeat monitoring and device status tracking
- Real WASM runtime execution

**Communication Protocols**
- TLS 1.3 with mutual authentication
- CBOR message encoding for efficiency
- HTTP API for management operations
- Real-time device communication

**Security Features**
- Certificate-based device authentication
- Encrypted communication channels
- Access control and authorization
- Audit logging and monitoring

### Edge Layer

The edge layer encompasses constrained devices running WebAssembly applications through Renode emulation.

```mermaid
graph TB
    subgraph "Renode Emulation"
        RENODE[Renode Platform]
        ARDUINO[Arduino Nano 33 BLE]
        STM32[STM32F4 Discovery]
        UNO[Arduino Uno R4]
    end
    
    subgraph "Real Firmware"
        FIRMWARE[Rust Firmware]
        TLS_CLI[TLS Client]
        APP_LOADER[Application Loader]
        HB_CLIENT[Heartbeat Client]
        WASM_RT[WASM Runtime]
    end
    
    subgraph "Communication"
        SERIAL_BRIDGE[Serial Bridge]
        TCP_BRIDGE[TCP Bridge]
        TLS_CONN[TLS Connection]
    end
    
    RENODE --> ARDUINO
    RENODE --> STM32
    RENODE --> UNO
    
    ARDUINO --> FIRMWARE
    STM32 --> FIRMWARE
    UNO --> FIRMWARE
    
    FIRMWARE --> TLS_CLI
    FIRMWARE --> APP_LOADER
    FIRMWARE --> HB_CLIENT
    FIRMWARE --> WASM_RT
    
    FIRMWARE --> SERIAL_BRIDGE
    SERIAL_BRIDGE --> TCP_BRIDGE
    TCP_BRIDGE --> TLS_CONN
```

#### Device Types

**Arduino Nano 33 BLE**
- ARM Cortex-M4 processor
- 256KB RAM, 1MB Flash
- Bluetooth Low Energy support
- Real-time capabilities

**STM32F4 Discovery**
- ARM Cortex-M4 processor
- 192KB RAM, 1MB Flash
- Rich peripheral set
- High-performance processing

**Arduino Uno R4**
- ARM Cortex-M4 processor
- 32KB RAM, 256KB Flash
- Cost-effective solution
- Educational platform

#### Real Firmware Implementation

**Rust no_std Firmware**
- Minimal standard library dependencies
- Custom allocator implementations
- Embedded system optimizations
- Memory management for constrained environments

**Runtime Features**
- Real TLS client implementation
- CBOR message serialization
- WASM runtime integration
- Heartbeat and status reporting
- Application lifecycle management

## Communication Architecture

### Inter-Layer Communication

**Cloud to Fog**
- Kubernetes API calls
- Resource updates and notifications
- Configuration management
- Monitoring and telemetry

**Fog to Edge**
- TLS-secured device connections
- Application deployment commands
- Heartbeat and status updates
- Real-time data exchange

### Intra-Layer Communication

**Cloud Layer**
- Kubernetes internal communication
- Controller coordination
- Resource synchronization
- Event propagation

**Fog Layer**
- Gateway clustering and load balancing
- Inter-gateway communication
- Shared state management
- Failover and redundancy

**Edge Layer**
- Device-to-device communication
- Local network protocols
- Real-time data sharing
- Collaborative processing

## Security Architecture

### Authentication and Authorization

**Device Authentication**
- X.509 certificate-based authentication
- Public key infrastructure (PKI)
- Certificate validation and revocation
- Mutual TLS authentication

**User Authentication**
- Role-based access control (RBAC)
- Kubernetes service accounts
- API key management
- Multi-factor authentication support

### Communication Security

**Transport Security**
- TLS 1.3 encryption
- Perfect forward secrecy
- Certificate pinning
- Secure key exchange

**Message Security**
- CBOR message signing
- Integrity verification
- Replay attack prevention
- Confidentiality protection

### Runtime Security

**WebAssembly Sandboxing**
- Memory isolation
- Function call restrictions
- Resource access control
- Execution time limits

**Device Security**
- Secure boot verification
- Firmware integrity checks
- Runtime protection mechanisms
- Hardware security module (HSM) support

## Deployment Architecture

### Kubernetes Integration

**Custom Resource Definitions**
- Application CRD for WASM application management
- Device CRD for edge device registration
- Gateway CRD for gateway management
- Status CRDs for runtime monitoring

**Controller Architecture**
- Event-driven reconciliation
- State machine implementation
- Error handling and recovery
- Scalability and performance optimization

### Container Orchestration

**Gateway Deployment**
- StatefulSet for persistent state
- ConfigMap for configuration management
- Secret management for certificates
- Service exposure and load balancing

**Application Deployment**
- DaemonSet for edge device deployment
- Job and CronJob for batch processing
- Pod security policies
- Resource quotas and limits

## Scalability and Performance

### Horizontal Scaling

**Gateway Scaling**
- Multiple gateway instances
- Load balancing and distribution
- Geographic distribution
- Auto-scaling based on load

**Device Scaling**
- Dynamic device registration
- Load distribution across devices
- Resource-aware scheduling
- Performance monitoring

### Vertical Scaling

**Resource Optimization**
- Memory usage optimization
- CPU utilization monitoring
- Storage efficiency
- Network bandwidth management

**Performance Tuning**
- WebAssembly execution optimization
- Communication protocol efficiency
- Caching and buffering strategies
- Latency reduction techniques

## Monitoring and Observability

### Metrics Collection

**System Metrics**
- Resource utilization monitoring
- Performance indicators
- Error rates and success rates
- Latency and throughput measurements

**Application Metrics**
- WebAssembly execution metrics
- Function call statistics
- Memory usage patterns
- Error tracking and analysis

### Logging and Tracing

**Distributed Tracing**
- Request flow tracking
- Performance bottleneck identification
- Error propagation analysis
- Cross-service communication monitoring

**Centralized Logging**
- Structured logging format
- Log aggregation and analysis
- Real-time log streaming
- Historical log retention

## Fault Tolerance and Reliability

### Error Handling

**Graceful Degradation**
- Service availability maintenance
- Partial functionality preservation
- User experience continuity
- Recovery procedures

**Failure Recovery**
- Automatic retry mechanisms
- Circuit breaker patterns
- Fallback strategies
- Disaster recovery procedures

### High Availability

**Redundancy**
- Multiple instance deployment
- Geographic distribution
- Data replication
- Backup and restore procedures

**Health Monitoring**
- Continuous health checks
- Proactive failure detection
- Automated recovery actions
- Alert and notification systems

## Production Status

### ✅ Production Ready Features

- **Real TLS Communication**: Complete TLS 1.3 implementation
- **Renode Emulation**: Full constrained device emulation
- **WASM Runtime**: Complete WebAssembly execution engine
- **Kubernetes Integration**: Full CRD and controller implementation
- **Real-time Dashboard**: Live monitoring and management interface
- **Certificate Management**: Complete TLS certificate infrastructure
- **Device Lifecycle**: Create, deploy, monitor, stop devices
- **Application Deployment**: Full WASM application orchestration

### 🚀 Advanced Capabilities

- **Multi-Architecture Support**: ARM Cortex-M4, RISC-V
- **Real Device Communication**: Actual device-to-gateway protocols
- **Secure Enrollment**: Certificate-based device authentication
- **Real-time Monitoring**: Live device status and health
- **Scalable Architecture**: Kubernetes-native orchestration
- **Production Security**: Complete security architecture