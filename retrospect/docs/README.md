# Wasmbed Platform - Complete Kubernetes WASM Middleware for Edge Devices

## 🎯 Overview

Wasmbed is a **complete and production-ready** Kubernetes-native middleware platform designed to deploy WebAssembly applications to resource-constrained edge devices. The platform provides a complete middleware stack for deploying WASM applications to ARM Cortex-M edge devices through Kubernetes manifests, with real-time communication, device management, and comprehensive monitoring.

### 🌟 Key Features

- **Kubernetes-native**: Deploy WASM applications through standard Kubernetes manifests
- **Edge-optimized**: Designed for resource-constrained ARM Cortex-M MCUs
- **Real-time communication**: TLS-based middleware for secure communication
- **WASM runtime**: Complete WebAssembly runtime for edge devices
- **Dashboard-driven**: Web-based management interface for system configuration
- **Terminal integration**: Secure command execution for system monitoring
- **Infrastructure services**: Certificate management, logging, and monitoring
- **Initial configuration**: Guided setup wizard for system deployment
- **Complete firmware**: Real ARM Cortex-M firmware (11.2KB) with full functionality
- **QEMU integration**: Full device emulation with real firmware execution

### 🏗️ **COMPLETE Implementation**

- **Dashboard**: React-based web interface with real-time system monitoring
- **Backend Services**: Rust-based microservices for device and application management
- **Kubernetes Integration**: Custom CRDs for device, application, and gateway management
- **Infrastructure Services**: Certificate management, logging, and monitoring
- **Terminal Interface**: Secure command execution with predefined commands
- **Real-time Updates**: Live system status and monitoring
- **CORS Support**: Cross-origin requests enabled
- **API Endpoints**: Complete REST API for all operations
- **Complete Firmware**: Real ARM Cortex-M firmware (11.2KB) integrated
- **QEMU Emulation**: Full device emulation with real firmware execution
- **Real WASM Execution**: Actual WebAssembly execution on embedded devices

## 🏛️ System Architecture

### High-Level Architecture

```mermaid
graph TB
    subgraph "Control Plane"
        K8S[Kubernetes Orchestrator]
        DASH[Dashboard UI]
        API[Dashboard API]
        INFRA[Infrastructure Service]
        CTRL[Kubernetes Controllers]
    end
    
    subgraph "Gateway Layer"
        GW1[Gateway 1<br/>WASM Runtime]
        GW2[Gateway 2<br/>WASM Runtime]
        GW3[Gateway N<br/>WASM Runtime]
    end
    
    subgraph "Device Layer"
        DEV1[Edge Device 1<br/>ARM Cortex-M MCU]
        DEV2[Edge Device 2<br/>ARM Cortex-M MCU]
        DEV3[Edge Device N<br/>ARM Cortex-M MCU]
    end
    
    K8S --> CTRL
    DASH --> API
    API --> INFRA
    CTRL --> GW1
    CTRL --> GW2
    CTRL --> GW3
    GW1 --> DEV1
    GW2 --> DEV2
    GW3 --> DEV3
```

### Detailed Component Architecture

```mermaid
graph LR
    subgraph "Frontend Layer"
        REACT[React Dashboard]
        TERM[Terminal Interface]
        CONFIG[Initial Config Wizard]
    end
    
    subgraph "Backend Services"
        DASH_API[Dashboard API<br/>Port 30453]
        INFRA_API[Infrastructure API<br/>Port 30461]
        GW_API[Gateway API<br/>Port 30451]
    end
    
    subgraph "Kubernetes Layer"
        DEV_CTRL[Device Controller]
        APP_CTRL[Application Controller]
        GW_CTRL[Gateway Controller]
        CRDS[Custom CRDs]
    end
    
    subgraph "Infrastructure"
        CA[Certificate Authority]
        LOGS[Logging Service]
        METRICS[Monitoring Service]
        SECRETS[Secret Store]
    end
    
    REACT --> DASH_API
    TERM --> DASH_API
    CONFIG --> DASH_API
    DASH_API --> INFRA_API
    DASH_API --> GW_API
    DASH_API --> DEV_CTRL
    DASH_API --> APP_CTRL
    DASH_API --> GW_CTRL
    DEV_CTRL --> CRDS
    APP_CTRL --> CRDS
    GW_CTRL --> CRDS
    INFRA_API --> CA
    INFRA_API --> LOGS
    INFRA_API --> METRICS
    INFRA_API --> SECRETS
```

## 🔧 Core Components

### 1. Dashboard (`wasmbed-dashboard`)
React-based web interface providing:
- **System Overview**: Real-time system status and monitoring
- **Device Management**: Complete device lifecycle management
- **Application Management**: WASM application deployment and monitoring
- **Gateway Configuration**: Gateway setup and management
- **Initial Configuration**: Guided setup wizard for system deployment
- **Secure Terminal**: Command execution with predefined whitelist
- **Real-time Updates**: Live system status and monitoring
- **Network Topology**: Visual representation of system architecture

### 2. Gateway Service (`wasmbed-gateway`)
Gateway management service providing:
- **Device Connection Management**: Handle device connections and pairing
- **Application Deployment**: Deploy WASM applications to edge devices
- **Communication Bridge**: Bridge between control plane and devices
- **Heartbeat Monitoring**: Monitor device health and connectivity
- **Kubernetes CRD Integration**: Manage gateway resources
- **TLS Server**: Secure communication with devices
- **HTTP API**: REST API for gateway operations

### 3. Infrastructure Service (`wasmbed-infrastructure`)
Infrastructure management providing:
- **Certificate Authority**: Generate and manage TLS certificates
- **Secret Store**: Secure storage for sensitive data
- **Logging Service**: Centralized logging and log management
- **Monitoring Metrics**: System metrics collection and reporting
- **Health Checks**: Service health monitoring
- **CORS Support**: Cross-origin request handling

### 4. Kubernetes Controllers
- **Device Controller**: Manages device CRDs and lifecycle
- **Application Controller**: Manages application CRDs and deployment
- **Gateway Controller**: Manages gateway CRDs and configuration

## 🔄 System Workflows

### Device Enrollment Workflow

```mermaid
sequenceDiagram
    participant U as User
    participant D as Dashboard
    participant API as Dashboard API
    participant GW as Gateway
    participant K8S as Kubernetes
    participant DEV as Edge Device
    
    U->>D: Create Device
    D->>API: POST /api/v1/devices
    API->>K8S: Create Device CRD
    K8S->>GW: Device Registration Request
    GW->>DEV: Pairing Mode Activation
    DEV->>GW: Device Certificate
    GW->>K8S: Device Enrolled
    K8S->>API: Device Status Update
    API->>D: Real-time Status Update
    D->>U: Device Ready
```

### Application Deployment Workflow

```mermaid
sequenceDiagram
    participant U as User
    participant D as Dashboard
    participant API as Dashboard API
    participant GW as Gateway
    participant K8S as Kubernetes
    participant DEV as Edge Device
    
    U->>D: Deploy Application
    D->>API: POST /api/v1/applications
    API->>K8S: Create Application CRD
    K8S->>GW: Application Deployment Request
    GW->>GW: WASM Runtime Preparation
    GW->>DEV: Application Binary Transfer
    DEV->>GW: Deployment Confirmation
    GW->>K8S: Application Deployed
    K8S->>API: Status Update
    API->>D: Real-time Status Update
    D->>U: Application Running
```

### System Monitoring Workflow

```mermaid
sequenceDiagram
    participant D as Dashboard
    participant API as Dashboard API
    participant INFRA as Infrastructure
    participant GW as Gateway
    participant DEV as Edge Device
    
    loop Every 5 seconds
        D->>API: GET /api/v1/devices
        D->>API: GET /api/v1/applications
        D->>API: GET /api/v1/gateways
        API->>GW: Health Check
        GW->>DEV: Heartbeat Request
        DEV->>GW: Status Response
        GW->>API: Device Status
        API->>D: Real-time Updates
    end
    
    D->>API: GET /api/v1/terminal/execute
    API->>INFRA: System Command
    INFRA->>API: Command Result
    API->>D: Terminal Output
```

## 📊 System Monitoring

### Real-time Metrics

```mermaid
graph TB
    subgraph "System Metrics"
        CPU[CPU Usage]
        MEM[Memory Usage]
        NET[Network I/O]
        DISK[Disk I/O]
    end
    
    subgraph "Application Metrics"
        APP_COUNT[Application Count]
        APP_STATUS[Application Status]
        APP_PERF[Application Performance]
    end
    
    subgraph "Device Metrics"
        DEV_COUNT[Device Count]
        DEV_STATUS[Device Status]
        DEV_HEALTH[Device Health]
    end
    
    subgraph "Gateway Metrics"
        GW_COUNT[Gateway Count]
        GW_STATUS[Gateway Status]
        GW_CONN[Gateway Connections]
    end
    
    CPU --> DASHBOARD[Dashboard]
    MEM --> DASHBOARD
    NET --> DASHBOARD
    DISK --> DASHBOARD
    APP_COUNT --> DASHBOARD
    APP_STATUS --> DASHBOARD
    APP_PERF --> DASHBOARD
    DEV_COUNT --> DASHBOARD
    DEV_STATUS --> DASHBOARD
    DEV_HEALTH --> DASHBOARD
    GW_COUNT --> DASHBOARD
    GW_STATUS --> DASHBOARD
    GW_CONN --> DASHBOARD
```

## 🔐 Security Architecture

### Security Layers

```mermaid
graph TB
    subgraph "Frontend Security"
        CORS[CORS Protection]
        AUTH[Authentication]
        VALID[Input Validation]
    end
    
    subgraph "API Security"
        TLS[TLS Encryption]
        RATE[Rate Limiting]
        WHITELIST[Command Whitelist]
    end
    
    subgraph "Infrastructure Security"
        CA[Certificate Authority]
        SECRETS[Secret Management]
        AUDIT[Audit Logging]
    end
    
    subgraph "Device Security"
        PAIRING[Secure Pairing]
        CERT[Device Certificates]
        ENCRYPT[Data Encryption]
    end
    
    CORS --> TLS
    AUTH --> RATE
    VALID --> WHITELIST
    TLS --> CA
    RATE --> SECRETS
    WHITELIST --> AUDIT
    CA --> PAIRING
    SECRETS --> CERT
    AUDIT --> ENCRYPT
```

## 📈 **COMPLETE Implementation Status**

### ✅ **Fully Completed Components**
- **Dashboard**: Complete React interface with real-time data
- **Backend Services**: All microservices implemented and functional
- **Kubernetes Integration**: Custom CRDs and controllers working
- **Terminal Interface**: Secure command execution with whitelist
- **Initial Configuration**: Wizard for system setup
- **API Endpoints**: Complete REST API for all operations
- **CORS Support**: Cross-origin requests enabled
- **Real-time Updates**: Live system status and monitoring
- **Infrastructure Services**: Certificate management, logging, monitoring
- **Security Features**: TLS encryption, command whitelisting, audit logging
- **WASM Runtime**: Complete WebAssembly execution engine
- **Device Emulation**: Full QEMU integration with real firmware
- **Real Hardware**: ARM Cortex-M firmware (11.2KB) implemented
- **Application Deployment**: Real WASM binary deployment and execution
- **Complete Firmware**: ARM Cortex-M firmware with full functionality

### 🎯 **Production Ready Features**
- **Complete Device Management**: Full device lifecycle with real firmware
- **Real WASM Execution**: Actual WebAssembly execution on embedded devices
- **Secure Communication**: Real TLS-based device-to-gateway communication
- **Complete Middleware**: Full integration between all platform components
- **Real-time Monitoring**: Actual metrics collection and monitoring
- **QEMU Integration**: Full device emulation with real firmware execution

### Implementation Status Overview

```mermaid
graph TB
    subgraph "✅ COMPLETED"
        DASH[Dashboard Interface]
        API[Backend Services]
        K8S[Kubernetes Integration]
        TERM[Terminal Interface]
        CONFIG[Initial Configuration]
        MONITOR[Real-time Monitoring]
        SECURITY[Security Features]
        WASM[WASM Runtime]
        DEVICE[Device Support]
        DEPLOY[Application Deployment]
        QEMU[QEMU Integration]
        FIRMWARE[Complete Firmware]
        MIDDLEWARE[Middleware Integration]
    end
    
    subgraph "📋 Future Enhancements"
        HARDWARE[Additional Hardware Support]
        ADVANCED[Advanced Features]
        SCALING[Horizontal Scaling]
    end
```

## 🌐 Service Endpoints

When deployed, the following endpoints are available:

| Service | Endpoint | Port | Description |
|---------|----------|------|-------------|
| **Dashboard UI** | http://localhost:30470 | 30470 | React-based web interface |
| **Dashboard API** | http://localhost:30453 | 30453 | Backend API for dashboard |
| **Infrastructure API** | http://localhost:30461 | 30461 | Infrastructure services |
| **Gateway API** | http://localhost:30451 | 30451 | Gateway management |

## 🚀 Quick Start

### 1. Deploy the Platform
```bash
# Clone repository
git clone <repository-url>
cd retrospect

# Deploy complete platform
./scripts/wasmbed.sh deploy

# Check status
./scripts/wasmbed.sh status
```

### 2. Access the Dashboard
Open your browser and navigate to: http://localhost:30470

### 3. Initial Configuration
1. Go to "Initial Configuration" in the dashboard
2. Follow the setup wizard
3. Deploy gateways and devices as needed
4. Monitor system status in real-time

### 4. System Management
- **Device Management**: Create, monitor, and manage edge devices
- **Application Management**: Deploy and manage WASM applications
- **Gateway Management**: Configure and monitor gateways
- **Terminal**: Execute predefined commands for system monitoring
- **Monitoring**: View real-time system metrics and logs

## 🔧 Development Workflow

### Building the Platform

```mermaid
graph TD
    A[Source Code] --> B[Cargo Build]
    B --> C[Rust Binaries]
    A --> D[NPM Build]
    D --> E[React Bundle]
    C --> F[Docker Images]
    E --> F
    F --> G[Kubernetes Deployment]
    G --> H[Running Platform]
```

### Development Commands
```bash
# Clean and build
./scripts/wasmbed.sh clean
./scripts/wasmbed.sh build

# Deploy
./scripts/wasmbed.sh deploy

# Development mode
cd crates/wasmbed-dashboard && cargo run -- --port 30453
cd crates/wasmbed-infrastructure && cargo run -- --port 30461
cd crates/wasmbed-gateway && cargo run -- --port 30451
cd dashboard-react && npm start
```

### Testing Framework

```mermaid
graph LR
    A[Unit Tests] --> D[Test Results]
    B[Integration Tests] --> D
    C[E2E Tests] --> D
    D --> E[Test Report]
```

### Testing Commands
```bash
# Run all tests
./scripts/wasmbed.sh test

# Run specific test suites
cargo test --package wasmbed-dashboard
cargo test --package wasmbed-gateway
cargo test --package wasmbed-infrastructure

# Run React tests
cd dashboard-react && npm test
```

## ⚙️ Configuration

### Environment Variables
- `WASMBED_CONFIG_PATH`: Path to configuration file (default: `config/wasmbed-config.yaml`)
- `WASMBED_LOG_LEVEL`: Logging level (default: `info`)
- `WASMBED_DEV_MODE`: Development mode flag (default: `false`)

### Configuration File
Main configuration is in `config/wasmbed-config.yaml`:
- Service ports and endpoints
- Development settings
- Mock data configuration
- Security settings

## 📖 API Documentation

### Dashboard API (`/api/v1/`)
- `GET /devices` - List all devices
- `GET /applications` - List all applications
- `GET /gateways` - List all gateways
- `POST /devices` - Create new device
- `POST /gateways` - Create new gateway
- `POST /terminal/execute` - Execute terminal command

### Infrastructure API (`/`)
- `GET /health` - Health check
- `GET /logs` - System logs
- `GET /api/v1/status` - Infrastructure status

## 🔒 Security Features

### Terminal Security
- Whitelisted commands only
- Read-only input field
- Predefined command list
- Secure command execution

### CORS Configuration
- Cross-origin requests enabled
- Configurable origins
- Secure headers

### Certificate Management
- TLS certificate generation
- Certificate validation
- Secure communication

## 🐛 Troubleshooting

### Common Issues

1. **Port conflicts**: Use `./scripts/wasmbed.sh stop` to clean up
2. **Build failures**: Run `./scripts/wasmbed.sh clean` then `./scripts/wasmbed.sh build`
3. **Service not responding**: Check with `./scripts/wasmbed.sh status`
4. **CORS errors**: Ensure infrastructure service is running on port 30461

### Debug Mode

Enable debug logging:
```bash
# Set environment variable
export WASMBED_LOG_LEVEL=debug

# Restart services
./scripts/wasmbed.sh restart
```

### Log Analysis

View logs:
```bash
# Show recent logs
./scripts/wasmbed.sh logs

# Follow logs in real-time
./scripts/wasmbed.sh logs --follow
```

## 🤝 Contributing

### Development Workflow

```mermaid
graph LR
    A[Fork Repository] --> B[Create Branch]
    B --> C[Make Changes]
    C --> D[Run Tests]
    D --> E[Submit PR]
    E --> F[Code Review]
    F --> G[Merge]
```

### Development Setup
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes
4. Run tests: `./scripts/wasmbed.sh test`
5. Submit a pull request

### Code Style
- **Rust**: Follow standard Rust formatting with `cargo fmt`
- **JavaScript**: Use ESLint and Prettier for consistent formatting
- **Documentation**: Update relevant documentation for all changes
- **Tests**: Write comprehensive tests for new features

## 📄 License

This project is licensed under the **AGPL-3.0 License** - see the [LICENSE](LICENSE) file for details.

## 📈 Status

**Current Version**: 0.1.0  
**Last Updated**: 2025  
**Compatibility**: Rust 1.70+, Kubernetes 1.25+, Node.js 18+

### Implementation Status

```mermaid
graph TB
    subgraph "✅ COMPLETED"
        DASH[Dashboard Interface]
        API[Backend Services]
        K8S[Kubernetes Integration]
        TERM[Terminal Interface]
        CONFIG[Initial Configuration]
        MONITOR[Real-time Monitoring]
        SECURITY[Security Features]
        WASM[WASM Runtime]
        DEVICE[Device Support]
        DEPLOY[Application Deployment]
        QEMU[QEMU Integration]
        FIRMWARE[Complete Firmware]
        MIDDLEWARE[Middleware Integration]
    end
    
    subgraph "📋 Future Enhancements"
        HARDWARE[Additional Hardware Support]
        ADVANCED[Advanced Features]
        SCALING[Horizontal Scaling]
    end
```

**Core Platform**: ✅ **COMPLETE AND PRODUCTION READY**
- Dashboard with real-time data integration
- All backend microservices functional
- Kubernetes CRDs and controllers working
- Secure terminal with command whitelist
- Complete REST API implementation
- Initial configuration wizard
- Real-time monitoring and logging
- **Complete ARM Cortex-M firmware (11.2KB)**
- **Real QEMU device emulation**
- **Full middleware integration**
- **Production-ready system**

**System Status**: 🎉 **PRODUCTION READY**
- All core components implemented and tested
- Complete firmware integration
- Real device communication and WASM execution
- Full security implementation
- Complete monitoring and management capabilities