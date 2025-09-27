# Wasmbed Platform Configuration

## Platform Information
- **Name**: Wasmbed Platform
- **Version**: 1.0.0
- **License**: AGPL-3.0
- **Description**: WebAssembly runtime platform for edge devices

## Architecture Overview
The Wasmbed Platform implements a complete edge computing solution with:
- **Device Layer**: QEMU-emulated MCU/MPU/RISC-V devices with no_std runtime
- **Gateway Layer**: Edge gateway with TLS, enrollment, and deployment services
- **Control Plane**: Kubernetes-based orchestration with custom controllers
- **Infrastructure**: Certificate Authority, monitoring, logging, and secret management
- **Dashboard**: React-based web interface for platform management

## Service Ports
- **Infrastructure API**: 30460
- **Gateway TLS**: 30450
- **Gateway HTTP**: 30451
- **Dashboard**: 30470
- **Monitoring**: 9090
- **Logging**: 8080

## Kubernetes Resources
- **Namespace**: wasmbed
- **CRDs**: 
  - devices.wasmbed.github.io/v0
  - applications.wasmbed.github.io/v1alpha1
  - gateways.wasmbed.io/v1

## Components
### Core Services
- **wasmbed-device-controller**: Manages device lifecycle
- **wasmbed-application-controller**: Manages application deployment
- **wasmbed-gateway-controller**: Manages gateway operations
- **wasmbed-gateway**: Edge gateway service
- **wasmbed-infrastructure**: Infrastructure services
- **wasmbed-dashboard**: Web dashboard

### Device Runtime
- **wasmbed-device-runtime**: no_std device runtime
- **wasmbed-wasm-runtime**: WebAssembly runtime
- **wasmbed-qemu-manager**: QEMU device emulation
- **wasmbed-qemu-deployment**: QEMU deployment service

### Supporting Libraries
- **wasmbed-k8s-resource**: Kubernetes resource definitions
- **wasmbed-tls-utils**: TLS utilities
- **wasmbed-protocol-server**: Communication protocols

## Workflows
### Device Enrollment
1. Device generates keypair
2. Device connects to Gateway via TLS
3. Gateway validates device and creates Device CRD
4. Device receives UUID and enrollment confirmation

### Application Deployment
1. Application CRD created with WASM bytecode
2. Application Controller finds target devices
3. Gateway receives deployment request
4. WASM bytecode deployed to target devices
5. Status updated in Application CRD

### Device Connection
1. Device establishes TLS connection to Gateway
2. Gateway authenticates device using stored public key
3. Heartbeat system maintains connection
4. Status updated in Device CRD

## Management Scripts
- **wasmbed.sh**: Main management console
- **clean.sh**: System cleanup
- **build.sh**: Build all components
- **deploy.sh**: Deploy complete platform
- **stop.sh**: Stop all services
- **status.sh**: Check system status
- **devices.sh**: Device management
- **applications.sh**: Application management
- **monitor.sh**: Monitoring and observability
- **certs.sh**: Certificate management
- **logs.sh**: Log management

## Quick Start
```bash
# Deploy the platform
wasmbed deploy

# Check status
wasmbed status

# Manage devices
wasmbed devices list

# Manage applications
wasmbed applications list

# Monitor system
wasmbed monitor health

# Stop platform
wasmbed stop
```

## Development
### Prerequisites
- Rust toolchain
- k3d (Kubernetes in Docker)
- kubectl
- Node.js (for React Dashboard)
- OpenSSL (for certificates)

### Building
```bash
cargo build --release
```

### Testing
```bash
wasmbed test
```

### Debugging
```bash
./scripts/logs.sh debug
wasmbed restart
```

## Security
- TLS encryption for all communications
- Certificate-based device authentication
- RBAC for Kubernetes resources
- Secure key management

## Monitoring
- Prometheus-compatible metrics
- Structured logging
- Real-time resource monitoring
- Health checks and alerts

## Documentation
- Architecture diagrams in PlantUML format
- API documentation
- Deployment guides
- Troubleshooting guides
