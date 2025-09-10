# Wasmbed Platform

A secure middleware platform for deploying WebAssembly applications to industrial robotic systems using Kubernetes orchestration.

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Quick Start](#quick-start)
4. [Documentation](#documentation)
5. [Current Status](#current-status)
6. [Known Issues](#known-issues)
7. [Missing Implementations](#missing-implementations)
8. [Contributing](#contributing)
9. [License](#license)

## Overview

Wasmbed is a comprehensive 3-layer architecture (cloud-fog-edge) designed to enable secure deployment and execution of WebAssembly applications on heterogeneous edge devices including MPU, MCU, and RISC-V systems. The platform provides real-time communication capabilities for industrial robotic applications, particularly PX4 autopilot integration.

### Key Features

- **Kubernetes Integration**: Custom CRDs and controllers for application lifecycle management
- **TLS Security**: Mutual authentication with custom TLS implementation
- **QEMU Emulation**: Realistic emulation of edge devices for development and testing
- **WebAssembly Runtime**: no_std compatible runtime for embedded systems
- **Real-time Communication**: microROS and FastDDS integration for PX4 systems
- **Device Management**: Secure enrollment and connection workflows

## Architecture

The platform follows a 3-layer architecture:

```mermaid
graph TB
    subgraph "Cloud Layer"
        K8S[Kubernetes Cluster]
        CTRL[Custom Controllers]
        CRD[Application/Device CRDs]
        RBAC[RBAC & Security Policies]
        ETCD[etcd State Store]
    end
    
    subgraph "Fog Layer"
        GW[Gateway MPU]
        TLS[TLS Bridge]
        MR[microROS Bridge]
        FDDS[FastDDS Middleware]
        AUTH[Device Authentication]
    end
    
    subgraph "Edge Layer"
        MPU[MPU Devices]
        MCU[MCU Devices]
        RISCV[RISC-V Devices]
        WASM[WebAssembly Runtime]
        TLS_CLIENT[TLS Client]
    end
    
    K8S --> GW
    CTRL --> GW
    CRD --> GW
    RBAC --> GW
    ETCD --> K8S
    
    GW --> TLS
    GW --> MR
    GW --> FDDS
    GW --> AUTH
    
    TLS --> MPU
    TLS --> MCU
    TLS --> RISCV
    MR --> MPU
    FDDS --> MCU
    AUTH --> RISCV
    
    MPU --> WASM
    MCU --> WASM
    RISCV --> WASM
    MPU --> TLS_CLIENT
    MCU --> TLS_CLIENT
    RISCV --> TLS_CLIENT
```

### Cloud Layer
- Kubernetes cluster with custom controllers
- Application and Device CRDs
- RBAC and security policies
- etcd for state persistence

### Fog Layer
- Gateway MPU providing TLS-secured communication bridge
- microROS bridge for PX4 integration
- FastDDS middleware for real-time communication
- Device enrollment and authentication

### Edge Layer
- Heterogeneous devices (MPU, MCU, RISC-V)
- WebAssembly runtime for no_std environments
- TLS client implementation
- Real-time application execution

## Workflows

### Device Enrollment Workflow

```mermaid
sequenceDiagram
    participant D as Device
    participant G as Gateway
    participant K as Kubernetes
    participant C as Controller
    
    D->>G: 1. Connection Request
    G->>D: 2. Certificate Request
    D->>G: 3. Device Certificate + Public Key
    G->>G: 4. Validate Certificate
    G->>K: 5. Create Device CRD
    K->>C: 6. Device Created Event
    C->>C: 7. Reconcile Device
    C->>K: 8. Update Device Status
    K->>G: 9. Device Status Update
    G->>D: 10. Enrollment Success
    D->>G: 11. Start Heartbeat
```

### Application Deployment Workflow

```mermaid
sequenceDiagram
    participant U as User
    participant K as Kubernetes
    participant C as Controller
    participant G as Gateway
    participant D as Device
    
    U->>K: 1. Deploy Application CRD
    K->>C: 2. Application Created Event
    C->>C: 3. Validate Application
    C->>G: 4. Deploy to Gateway
    G->>G: 5. Load WASM Binary
    G->>D: 6. Send Application
    D->>D: 7. Load WASM Runtime
    D->>G: 8. Application Ready
    G->>K: 9. Update Application Status
    K->>C: 10. Status Update Event
    C->>K: 11. Update Application CRD
```

### PX4 Communication Workflow

```mermaid
sequenceDiagram
    participant PX4 as PX4 Autopilot
    participant MR as microROS Bridge
    participant FDDS as FastDDS
    participant G as Gateway
    participant APP as WASM App
    
    PX4->>MR: 1. UORB Topics
    MR->>FDDS: 2. Convert to DDS
    FDDS->>G: 3. DDS Messages
    G->>APP: 4. Process Commands
    APP->>G: 5. Control Commands
    G->>FDDS: 6. Send Commands
    FDDS->>MR: 7. DDS to microROS
    MR->>PX4: 8. MAVLink Commands
    PX4->>MR: 9. Status Updates
    MR->>FDDS: 10. Status to DDS
    FDDS->>G: 11. Status Messages
    G->>APP: 12. Update Status
```

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Kubernetes cluster (k3d recommended for local development)
- QEMU system emulators (qemu-system-riscv32, qemu-system-arm, qemu-system-xtensa)
- Rust toolchain

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd retrospect
```

2. Deploy the complete platform:
```bash
./scripts/deploy.sh
```

3. Test the system:
```bash
./scripts/app.sh test
```

4. Monitor the platform:
```bash
./scripts/monitor.sh status
```

5. Clean up when done:
```bash
./scripts/cleanup.sh
```

## Documentation

### Architecture Documentation
- [System Overview](docs/architecture/system-overview.md) - Complete system architecture
- [Communication Protocols](docs/architecture/communication-protocols.md) - TLS, CBOR, and DDS protocols
- [Security Architecture](docs/architecture/security-architecture.md) - Security design and implementation

### Implementation Documentation
- [Core Components](docs/implementation/core-components.md) - Detailed component implementation
- [Workflows](docs/workflows/) - Complete workflow documentation
- [API Reference](docs/api/) - API specifications and CRD documentation

### Deployment Documentation
- [Deployment Guide](docs/deployment/deployment-guide.md) - Step-by-step deployment instructions
- [Troubleshooting](docs/deployment/troubleshooting.md) - Common issues and solutions
- [Configuration](docs/deployment/configuration.md) - Platform configuration options

### Integration Documentation
- [PX4 Integration](docs/integration/px4-integration.md) - PX4 autopilot integration
- [microROS Integration](docs/integration/microros-integration.md) - microROS bridge implementation
- [FastDDS Integration](docs/integration/fastdds-integration.md) - FastDDS middleware integration

### Development Documentation
- [Development Setup](docs/development/setup.md) - Development environment setup
- [Contributing Guidelines](docs/development/contributing.md) - Contribution guidelines
- [Testing](docs/development/testing.md) - Testing procedures and guidelines

### Problems and Solutions
- [Known Issues](docs/problems/known-issues.md) - Current known issues and workarounds
- [Missing Implementations](docs/problems/missing-implementations.md) - Critical missing features
- [Technical Debt](docs/problems/technical-debt.md) - Areas requiring refactoring

## Current Status

### Implemented Features

#### Core Platform (18 Rust Crates)
- **wasmbed-gateway**: TLS + HTTP API server with heartbeat monitoring
- **wasmbed-k8s-controller**: Kubernetes reconciliation and application lifecycle management
- **wasmbed-qemu-serial-bridge**: Real QEMU communication bridge
- **wasmbed-firmware-hifive1-qemu**: RISC-V firmware with WebAssembly runtime
- **wasmbed-firmware-esp32**: ESP32 firmware with WiFi management
- **wasmbed-mcu-simulator**: MCU testing and simulation
- **wasmbed-protocol**: CBOR communication protocol
- **wasmbed-tls-utils**: Custom TLS implementation with RustCrypto
- **wasmbed-k8s-resource**: Kubernetes CRDs and resource management
- **wasmbed-types**: Common types and data structures

#### Kubernetes Integration
- Custom Resource Definitions for Applications and Devices
- Controller with comprehensive reconciliation logic
- RBAC policies and service accounts
- StatefulSet and Service configurations
- ConfigMap and Secret management

#### Security Implementation
- TLS 1.3 with mutual authentication
- Certificate generation and management
- Custom TLS implementation using RustCrypto
- Device enrollment with public key authentication
- Secure communication channels

#### QEMU Emulation
- RISC-V emulation with HiFive1 firmware
- ARM emulation with STM32 firmware
- ESP32 emulation with XTensa firmware
- Real serial communication via TCP
- WebAssembly runtime integration

#### Device Management
- Device enrollment workflow
- Connection establishment and maintenance
- Heartbeat monitoring and status tracking
- Application deployment and execution
- Error handling and recovery

## Known Issues

### Critical Issues
1. **microROS Bridge Missing**: Gateway lacks microROS bridge implementation
2. **FastDDS Middleware Missing**: No FastDDS integration in gateway
3. **PX4 Communication Non-functional**: Real-time PX4 communication not working
4. **Real-time Application Deployment**: Missing real-time deployment capabilities

### High Priority Issues
1. **Device Capability Discovery**: Not implemented
2. **Application Lifecycle Management**: Incomplete
3. **WASM Application Validation**: Missing validation
4. **Connection Quality Monitoring**: Not implemented

### Medium Priority Issues
1. **Certificate Revocation Lists**: Not implemented
2. **Advanced Threat Detection**: Missing
3. **Security Monitoring**: Incomplete
4. **Performance Optimization**: Needs improvement

## Missing Implementations

### Critical Missing Features

#### microROS Bridge in Gateway
```rust
// Required implementation
pub struct MicroRosBridge {
    node: rcl::Node,
    participant: fastdds::DomainParticipant,
    publishers: HashMap<String, Publisher>,
    subscribers: HashMap<String, Subscriber>,
}
```

#### FastDDS Middleware
```rust
// Required implementation
pub struct FastDdsMiddleware {
    domain_id: u32,
    participant: DomainParticipant,
    transport: UdpTransport,
    qos: QosProfile,
}
```

#### PX4 Communication Bridge
```rust
// Required implementation
pub struct Px4CommunicationBridge {
    microros_bridge: MicroRosBridge,
    fastdds: FastDdsMiddleware,
    px4_topics: Px4TopicManager,
}
```

### High Priority Missing Features
1. **Real-time Application Deployment**: Complete real-time deployment system
2. **Device Capability Discovery**: Automatic device capability detection
3. **Application Performance Monitoring**: Comprehensive performance metrics
4. **Dynamic Scaling**: Automatic scaling based on load

### Medium Priority Missing Features
1. **Advanced Security Features**: Certificate revocation, threat detection
2. **Monitoring and Observability**: Comprehensive monitoring system
3. **Performance Optimization**: Runtime and communication optimization
4. **Compliance and Auditing**: Security compliance and audit trails

## Contributing

Please refer to [Contributing Guidelines](docs/development/contributing.md) for detailed contribution instructions.

### Development Setup
1. Follow [Development Setup](docs/development/setup.md) for environment configuration
2. Review [Testing Guidelines](docs/development/testing.md) for testing procedures
3. Check [Known Issues](docs/problems/known-issues.md) for current limitations

### Code Style
- Follow Rust standard formatting with `cargo fmt`
- Use `cargo clippy` for linting
- Write comprehensive tests for all new features
- Document all public APIs

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

For technical support and questions:
- Check [Troubleshooting Guide](docs/deployment/troubleshooting.md) for common issues
- Review [Known Issues](docs/problems/known-issues.md) for current limitations
- Create an issue in the project repository for bug reports
- Refer to [Missing Implementations](docs/problems/missing-implementations.md) for planned features