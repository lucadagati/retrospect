# RETROSPECT - Wasmbed Platform

**RETROSPECT** (secuRE inTegration middlewaRe fOr cpS comPutE ConTinuum) is a Kubernetes-native platform for deploying and managing WebAssembly applications on embedded devices using Renode emulation and Zephyr RTOS.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Repository Structure](#repository-structure)
- [Deployment Locations](#deployment-locations)
- [Quick Start](#quick-start)
- [Documentation](#documentation)
- [Components](#components)
- [Technologies](#technologies)
- [Development Status](#development-status)
- [License](#license)

## Overview

RETROSPECT is a complete platform that enables:

- **Device Emulation**: Full hardware emulation of ARM Cortex-M embedded devices using Renode
- **WebAssembly Deployment**: Compile and deploy WebAssembly applications to emulated devices
- **Secure Communication**: TLS 1.3 with mutual authentication between devices and gateway
- **Kubernetes Orchestration**: Complete lifecycle management using Kubernetes Custom Resources
- **Real-time Monitoring**: Web dashboard for system monitoring and device management

The platform is designed for the **Cloud-Fog-Edge** computing continuum, with components deployed across different layers:

- **Cloud Layer**: Kubernetes cluster hosting control plane services (API Server, Gateway, Dashboard, Controllers)
- **Fog Layer**: Renode emulation containers running on cluster nodes (device emulation)
- **Edge Layer**: Zephyr RTOS firmware running inside emulated devices (application execution)

## Architecture

### High-Level Architecture

```mermaid
graph TB
    subgraph "Cloud Layer - Kubernetes Cluster"
        subgraph "User Interface"
            Dashboard["Dashboard<br/>(React Web UI)<br/>Port: 3000"]
        end

        subgraph "API Layer"
            APIServer["API Server<br/>(REST API + Controllers)<br/>Port: 3001"]
        end

        subgraph "Controller Layer"
            DeviceCtrl["Device Controller"]
            AppCtrl["Application Controller"]
            GatewayCtrl["Gateway Controller"]
        end

        subgraph "Kubernetes Resources"
            DeviceCRD["Device CRD"]
            AppCRD["Application CRD"]
            GatewayCRD["Gateway CRD"]
        end
    end

    subgraph "Fog Layer - Host Runtime"
        Gateway["Gateway<br/>(Fog Gateway / Teleserver)<br/>Ports: 8080 HTTP, 8081 TLS"]
        RenodeMgr["Renode Manager<br/>(orchestrated by API Server)"]
        DevProxy["Renode Device Proxy<br/>(one per device)<br/>(container/process/VM)"]
        FirmwareVol["Firmware Volumes<br/>(Zephyr ELF files)"]
    end

    subgraph "Edge Layer - Emulated Devices"
        Zephyr["Zephyr RTOS<br/>(Network Stack + TLS)"]
        WAMR["WAMR Runtime<br/>(WebAssembly)"]
        WASMApp["WASM Applications"]
    end

    %% UI -> API
    Dashboard -->|HTTP| APIServer

    %% API -> CRDs
    APIServer -->|Manages| DeviceCRD
    APIServer -->|Manages| AppCRD
    APIServer -->|Manages| GatewayCRD

    %% Controllers watch CRDs
    DeviceCtrl -->|Watches| DeviceCRD
    AppCtrl -->|Watches| AppCRD
    GatewayCtrl -->|Watches| GatewayCRD

    %% Control plane: Gateway <-> API Server
    Gateway <--> |Control API| APIServer

    %% Emulation chain
    APIServer -->|Orchestrates| RenodeMgr
    RenodeMgr -->|Creates| DevProxy
    DevProxy -->|Loads| FirmwareVol
    DevProxy -->|Runs| Zephyr

    %% Runtime on device
    Zephyr -->|Executes| WAMR
    WAMR -->|Runs| WASMApp

    %% Data plane: Device <-> Gateway
    Zephyr <--> |TLS| Gateway
```

### Detailed Component Architecture

```mermaid
graph LR
    subgraph "Cloud Components (Kubernetes)"
        subgraph "API Server"
            API1["REST API<br/>Device & App Mgmt"]
            API2["Renode Manager API<br/>Proxy Orchestration"]
            API3["Kubernetes Client<br/>CRD Operations"]
        end

        subgraph "Controllers"
            DC["Device Controller<br/>Watches Device CRD"]
            AC["Application Controller<br/>Watches Application CRD"]
            GC["Gateway Controller<br/>Watches Gateway CRD"]
        end

        subgraph "CRDs"
            DCRD["Device CRD"]
            ACRD["Application CRD"]
            GCRD["Gateway CRD"]
        end
    end

    subgraph "Fog Components"
        subgraph "Gateway (Fog)"
            GW1["TLS Server<br/>:8081"]
            GW2["Control API<br/>:8080"]
            GW3["Enrollment<br/>(Auth/Identity)"]
            GW4["WASM Deployment<br/>(Module Delivery)"]
            GW5["Telemetry Uplink<br/>(Results/Status)"]
        end

        subgraph "Renode Device Proxy"
            RP1["Renode Emulator"]
            RP2["Platform File<br/>.repl"]
            RP3["Firmware Loader<br/>ELF"]
        end
    end

    subgraph "Edge Components"
        subgraph "Zephyr Firmware"
            Z1["TCP/IP + TLS"]
            Z2["WAMR Runtime"]
            Z3["Wasmbed Protocol<br/>CBOR"]
        end
    end

    %% API Server internals
    API1 --> API2
    API1 --> API3

    %% CRD operations and watches
    API3 --> DCRD
    API3 --> ACRD
    API3 --> GCRD
    DC --> DCRD
    AC --> ACRD
    GC --> GCRD

    %% Control plane: API Server <-> Gateway
    API1 <--> GW2

    %% Emulation chain
    API2 --> RP1
    RP1 --> RP2
    RP1 --> RP3
    RP3 --> Z1
    Z1 --> Z2
    Z2 --> Z3

    %% Data plane: Zephyr <-> Gateway
    Z3 <--> |TLS| GW1
```

### Communication Flow Architecture

```mermaid
sequenceDiagram
    participant User
    participant Dashboard
    participant APIServer as API Server
    participant K8sAPI as K8s API (CRDs)
    participant Gateway as Fog Gateway
    participant RenodeMgr as Renode Manager
    participant DevProxy as Renode Device Proxy
    participant Zephyr
    participant WAMR

    User->>Dashboard: Access Web UI
    Dashboard->>APIServer: REST API Calls

    APIServer->>K8sAPI: Create/Update Device CRD
    APIServer->>RenodeMgr: Start emulation (per device)
    RenodeMgr->>DevProxy: Create proxy instance
    DevProxy->>Zephyr: Load firmware (ELF) and boot

    Zephyr->>Gateway: TLS connect
    Gateway->>Zephyr: Enrollment flow
    Zephyr->>Gateway: Enrollment response

    Gateway->>APIServer: Notify enrollment/status
    APIServer->>K8sAPI: Update Device CRD status/identity

    User->>Dashboard: Deploy application
    Dashboard->>APIServer: Create Application CRD
    APIServer->>K8sAPI: Store Application CRD

    APIServer->>Gateway: Instruct deployment/targets
    Gateway->>Zephyr: Deliver WASM module (TLS)
    Zephyr->>WAMR: Load WASM
    WAMR->>WAMR: Execute app
    WAMR->>Gateway: Results/telemetry
    Gateway->>APIServer: Forward results/telemetry
    APIServer->>K8sAPI: Update status (optional)
```

## Repository Structure

```
retrospect/
├── crates/                          # Rust workspace components
│   ├── wasmbed-api-server/          # REST API server and orchestrator
│   ├── wasmbed-gateway/             # TLS gateway for device communication
│   ├── wasmbed-qemu-manager/        # Renode container management
│   ├── wasmbed-device-controller/   # Kubernetes Device CRD controller
│   ├── wasmbed-application-controller/  # Kubernetes Application CRD controller
│   ├── wasmbed-gateway-controller/  # Kubernetes Gateway CRD controller
│   ├── wasmbed-protocol/            # CBOR communication protocol
│   ├── wasmbed-types/               # Shared type definitions
│   ├── wasmbed-k8s-resource/        # Kubernetes CRD definitions
│   ├── wasmbed-cert/                # TLS certificate management
│   ├── wasmbed-config/              # Configuration management
│   └── ...                          # Supporting libraries
│
├── zephyr-app/                      # Zephyr RTOS firmware
│   ├── src/                         # Firmware source code
│   │   ├── main.c                   # Entry point
│   │   ├── network_handler.c/h      # Network stack management
│   │   ├── wamr_integration.c/h     # WAMR runtime integration
│   │   └── wasmbed_protocol.c/h    # Wasmbed protocol handler
│   ├── prj.conf                     # Zephyr build configuration
│   └── CMakeLists.txt               # Build system
│
├── zephyr-workspace/                # Zephyr RTOS workspace (cloned)
│   └── build/                       # Compiled firmware binaries
│       ├── stm32f746g_disco/        # STM32F746G Discovery firmware
│       ├── frdm_k64f/               # FRDM-K64F firmware
│       └── ...                      # Other board firmware
│
├── dashboard-react/                 # React web dashboard
│   ├── src/                         # React source code
│   │   ├── components/              # React components
│   │   │   ├── Dashboard.js         # Main dashboard
│   │   │   ├── DeviceManagement.js  # Device management UI
│   │   │   ├── ApplicationManagement.js  # Application management UI
│   │   │   ├── GatewayManagement.js # Gateway management UI
│   │   │   ├── Monitoring.js        # System monitoring
│   │   │   ├── NetworkTopology.js   # Network visualization
│   │   │   └── Terminal.js          # System terminal
│   │   └── App.js                   # Main application
│   └── build/                       # Built static files
│
├── k8s/                             # Kubernetes manifests
│   ├── crds/                        # Custom Resource Definitions
│   │   ├── device-crd.yaml         # Device CRD schema
│   │   ├── application-crd.yaml    # Application CRD schema
│   │   └── gateway-crd.yaml        # Gateway CRD schema
│   ├── deployments/                 # Component deployments
│   │   ├── api-server-deployment.yaml
│   │   ├── dashboard-deployment.yaml
│   │   └── wasmbed-deployments.yaml
│   ├── rbac/                        # Role-Based Access Control
│   │   ├── api-server-rbac.yaml
│   │   ├── device-controller-rbac.yaml
│   │   ├── application-controller-rbac.yaml
│   │   └── gateway-controller-rbac.yaml
│   └── namespace.yaml               # Wasmbed namespace
│
├── scripts/                         # Deployment and utility scripts
│   ├── deploy-k3s.sh               # Complete K3S deployment
│   ├── cleanup-k3s.sh             # System cleanup
│   └── README.md                   # Scripts documentation
│
├── doc/                             # Documentation
│   ├── ARCHITECTURE.md             # Detailed architecture
│   ├── DEPLOYMENT.md               # Deployment guide
│   ├── K3S_DEPLOYMENT.md           # K3S-specific deployment
│   ├── FIRMWARE.md                 # Firmware documentation
│   ├── MCU_SUPPORT.md              # Supported MCU types
│   ├── TLS_CONNECTION.md           # TLS connection details
│   ├── SEQUENCE_DIAGRAMS.md        # Communication flows
│   ├── DEVELOPMENT_STATUS.md       # Current development status
│   └── REAL_DEVICE_INTEGRATION.md  # Real device integration guide
│
├── config/                          # Configuration files
│   └── wasmbed-config.yaml         # Main configuration
│
├── certs/                           # TLS certificates
│   ├── ca-cert.pem                 # Certificate Authority
│   ├── server-cert.pem             # Server certificate
│   └── server-key.pem              # Server private key
│
├── Dockerfile.*                     # Dockerfiles for components
│   ├── Dockerfile.api-server
│   ├── Dockerfile.gateway
│   ├── Dockerfile.dashboard
│   └── ...
│
├── Cargo.toml                       # Rust workspace configuration
├── Cargo.lock                       # Rust dependency lock file
└── LICENSE                          # AGPL-3.0 license
```

## Deployment Locations

### Cloud Layer (Kubernetes Cluster)

Components deployed in the **Cloud Layer** run as Kubernetes Pods in the `wasmbed` namespace:

#### API Server (`wasmbed-api-server`)
- **Location**: Kubernetes Pod
- **Purpose**: Central orchestrator for the entire platform
- **Responsibilities**:
  - REST API for device, application, and gateway management
  - Kubernetes CRD operations
  - Renode container orchestration
  - Firmware deployment coordination
- **Why Cloud**: Centralized control plane, scalable, accessible from anywhere

#### Gateway (`wasmbed-gateway`)
- **Location**: Kubernetes Pod
- **Purpose**: Secure communication endpoint for embedded devices
- **Responsibilities**:
  - TLS 1.3 server for device connections
  - Device enrollment and authentication
  - WASM module deployment
  - Heartbeat monitoring
- **Why Cloud**: Centralized security, certificate management, scalable gateway instances

#### Dashboard (`wasmbed-dashboard`)
- **Location**: Kubernetes Pod
- **Purpose**: Web-based user interface
- **Responsibilities**:
  - Device management UI
  - Application deployment interface
  - System monitoring and visualization
  - Real-time status updates
- **Why Cloud**: Centralized access, no local installation required

#### Controllers
- **Device Controller**: Watches Device CRDs, manages device lifecycle
- **Application Controller**: Watches Application CRDs, manages application deployment
- **Gateway Controller**: Watches Gateway CRDs, manages gateway instances
- **Why Cloud**: Kubernetes-native controllers, integrated with cluster

### Fog Layer (Host Docker)

Components in the **Fog Layer** run as Docker containers on Kubernetes cluster nodes:

#### Renode Containers
- **Location**: Host Docker (one container per emulated device)
- **Purpose**: Hardware emulation for embedded devices
- **Responsibilities**:
  - CPU, memory, and peripheral emulation
  - Zephyr firmware execution
  - Network interface emulation (Ethernet/WiFi)
  - UART analyzer for logs
- **Why Fog**: Close to edge devices, efficient resource usage, isolated per device

#### Renode Manager
- **Location**: Runs inside API Server Pod, manages Docker containers
- **Purpose**: Lifecycle management of Renode containers
- **Responsibilities**:
  - Create/start/stop Renode containers
  - Configure network interfaces
  - Load firmware into containers
  - Generate Renode scripts
- **Why Fog**: Direct Docker API access, efficient container management

### Edge Layer (Emulated Devices)

Components in the **Edge Layer** run inside Renode emulated devices:

#### Zephyr RTOS Firmware
- **Location**: Runs inside Renode container, on emulated ARM Cortex-M MCU
- **Purpose**: Real-time operating system for embedded device
- **Responsibilities**:
  - Network stack initialization (TCP/IP, TLS)
  - Gateway endpoint reading from memory
  - TLS connection establishment
  - Device enrollment and heartbeat
  - WASM module reception and execution coordination
- **Why Edge**: Represents actual embedded device behavior, constrained resources

#### WAMR Runtime
- **Location**: Runs inside Zephyr firmware, on emulated MCU
- **Purpose**: WebAssembly execution engine
- **Responsibilities**:
  - WASM module loading and validation
  - WASM function execution
  - Memory management
  - System call interface
- **Why Edge**: Executes applications in constrained environment, isolated execution

#### WASM Applications
- **Location**: Loaded into WAMR runtime, executed on emulated MCU
- **Purpose**: User-defined application logic
- **Responsibilities**:
  - Application-specific functionality
  - Communication with gateway via protocol
  - Resource-constrained execution
- **Why Edge**: Actual application execution, represents edge computing workload

## Quick Start

### Prerequisites

- **K3S** Kubernetes cluster (or compatible Kubernetes 1.24+)
- **Docker** installed and running
- **kubectl** configured
- **Rust** toolchain 1.70+ (for building components)
- **Zephyr SDK** 0.16.5+ (for firmware compilation, optional)

### Installation

1. **Install K3S** (if not already installed):
```bash
curl -sfL https://get.k3s.io | sh -s - --write-kubeconfig-mode 644
mkdir -p ~/.kube
sudo cp /etc/rancher/k3s/k3s.yaml ~/.kube/config
sudo chown $(id -u):$(id -g) ~/.kube/config
```

2. **Clone Repository**:
```bash
git clone <repository-url>
cd retrospect
```

3. **Deploy System**:
```bash
./scripts/deploy-k3s.sh
```

This script will:
- Build all Docker images
- Set up local Docker registry
- Deploy all Kubernetes components
- Generate TLS certificates
- Create initial Gateway CRD

4. **Access Dashboard**:
```bash
kubectl port-forward -n wasmbed svc/wasmbed-dashboard 3000:3000
# Open http://localhost:3000 in browser
```

5. **Access API Server**:
```bash
kubectl port-forward -n wasmbed svc/wasmbed-api-server 3001:3001
# API available at http://localhost:3001
```

### Create Your First Device

1. **Via Dashboard**:
   - Navigate to "Device Management"
   - Click "Create Device"
   - Select MCU type (e.g., `Stm32F746gDisco`)
   - Select target gateway
   - Click "Create"

2. **Via API**:
```bash
curl -X POST http://localhost:3001/api/v1/devices \
  -H "Content-Type: application/json" \
  -d '{
    "name": "my-device",
    "deviceType": "MCU",
    "mcuType": "Stm32F746gDisco",
    "gatewayId": "gateway-1"
  }'
```

3. **Start Emulation**:
```bash
curl -X POST http://localhost:3001/api/v1/devices/my-device/renode/start
```

## Documentation

Complete documentation is available in the [`doc/`](doc/) directory:

### Core Documentation

- **[Architecture](doc/ARCHITECTURE.md)**: Detailed system architecture, component descriptions, and deployment locations (Cloud-Fog-Edge)
- **[Deployment](doc/DEPLOYMENT.md)**: Deployment guide and configuration instructions
- **[K3S Deployment](doc/K3S_DEPLOYMENT.md)**: K3S-specific deployment instructions and troubleshooting
- **[Development Status](doc/DEVELOPMENT_STATUS.md)**: Current development status, known issues, and next steps

### Component Documentation

- **[Firmware](doc/FIRMWARE.md)**: Zephyr RTOS firmware documentation, build instructions, and component details
- **[MCU Support](doc/MCU_SUPPORT.md)**: Complete list of supported MCU types, network capabilities, and compilation instructions
- **[TLS Connection](doc/TLS_CONNECTION.md)**: TLS connection flow, implementation details, and troubleshooting

### Integration Documentation

- **[Real Device Integration](doc/REAL_DEVICE_INTEGRATION.md)**: Step-by-step guide for integrating real hardware devices with the platform
- **[Sequence Diagrams](doc/SEQUENCE_DIAGRAMS.md)**: Communication flow diagrams for all major workflows

### Reference Documentation

- **[Wasmbed Capabilities](doc/WASMBED_CAPABILITIES.md)**: Detailed capabilities and features of the platform

### Quick Navigation

**For New Users:**
1. Start with this README for overview
2. Read [Architecture](doc/ARCHITECTURE.md) to understand the system
3. Follow [K3S Deployment](doc/K3S_DEPLOYMENT.md) for deployment
4. Check [Development Status](doc/DEVELOPMENT_STATUS.md) for current status

**For Developers:**
1. [Architecture](doc/ARCHITECTURE.md) - System architecture
2. [Firmware](doc/FIRMWARE.md) - Firmware development
3. [Sequence Diagrams](doc/SEQUENCE_DIAGRAMS.md) - Communication flows
4. [Development Status](doc/DEVELOPMENT_STATUS.md) - Current issues and next steps

**For Hardware Integration:**
1. [Real Device Integration](doc/REAL_DEVICE_INTEGRATION.md) - Complete integration guide
2. [MCU Support](doc/MCU_SUPPORT.md) - Supported hardware
3. [TLS Connection](doc/TLS_CONNECTION.md) - Network configuration

## Components

### Core Services

#### API Server (`wasmbed-api-server`)
- **Language**: Rust
- **Deployment**: Kubernetes Pod
- **Port**: 3001
- **Source**: `crates/wasmbed-api-server/`
- **Dockerfile**: `Dockerfile.api-server`
- **Responsibilities**:
  - REST API endpoints (45+ endpoints)
  - Kubernetes CRD management
  - Renode container orchestration
  - Application compilation (Rust to WASM)

#### Gateway (`wasmbed-gateway`)
- **Language**: Rust
- **Deployment**: Kubernetes Pod
- **Ports**: 8080 (HTTP), 8081 (TLS)
- **Source**: `crates/wasmbed-gateway/`
- **Dockerfile**: `Dockerfile.gateway`
- **Responsibilities**:
  - TLS 1.3 server for device connections
  - Device enrollment and authentication
  - WASM module deployment
  - Heartbeat monitoring

#### Dashboard (`dashboard-react`)
- **Language**: JavaScript (React)
- **Deployment**: Kubernetes Pod (serves static files)
- **Port**: 3000
- **Source**: `dashboard-react/`
- **Dockerfile**: `Dockerfile.dashboard`
- **Responsibilities**:
  - Web UI for system management
  - Device and application management
  - Real-time monitoring
  - Network topology visualization

### Controllers

#### Device Controller (`wasmbed-device-controller`)
- **Language**: Rust
- **Deployment**: Kubernetes Pod
- **Source**: `crates/wasmbed-device-controller/`
- **Responsibilities**: Watches Device CRDs, manages device lifecycle

#### Application Controller (`wasmbed-application-controller`)
- **Language**: Rust
- **Deployment**: Kubernetes Pod
- **Source**: `crates/wasmbed-application-controller/`
- **Responsibilities**: Watches Application CRDs, manages application deployment

#### Gateway Controller (`wasmbed-gateway-controller`)
- **Language**: Rust
- **Deployment**: Kubernetes Pod
- **Source**: `crates/wasmbed-gateway-controller/`
- **Responsibilities**: Watches Gateway CRDs, manages gateway instances

### Supporting Libraries

- **wasmbed-protocol**: CBOR-based communication protocol
- **wasmbed-types**: Shared type definitions
- **wasmbed-k8s-resource**: Kubernetes CRD definitions
- **wasmbed-cert**: TLS certificate management
- **wasmbed-config**: Configuration management
- **wasmbed-qemu-manager**: Renode container management (library)

### Firmware

#### Zephyr RTOS Firmware (`zephyr-app`)
- **Language**: C
- **RTOS**: Zephyr RTOS v4.3.0
- **Source**: `zephyr-app/`
- **Build Output**: `zephyr-workspace/build/<board>/zephyr/zephyr.elf`
- **Components**:
  - Network stack (TCP/IP, TLS 1.3)
  - WAMR runtime integration
  - Wasmbed protocol handler
  - Gateway endpoint reader

## Technologies

### Core Technologies

- **Kubernetes**: Container orchestration and lifecycle management
- **Renode**: Hardware emulation for ARM Cortex-M devices
- **Zephyr RTOS**: Real-time operating system for embedded devices
- **WAMR**: WebAssembly Micro Runtime for WASM execution
- **Rust**: Primary language for cloud components
- **React**: Frontend framework for dashboard
- **TLS 1.3**: Secure communication protocol
- **CBOR**: Compact Binary Object Representation for message serialization

### Build Tools

- **Cargo**: Rust package manager and build system
- **West**: Zephyr meta-tool for project management
- **CMake**: Build system for Zephyr firmware
- **Docker**: Containerization for all components
- **Ninja**: Build system (used by Zephyr)

### Network Protocols

- **TCP/IP**: Network transport
- **TLS 1.3**: Secure transport layer
- **CBOR**: Message serialization
- **HTTP/REST**: API communication
- **WebSocket**: Real-time updates

## Development Status

For detailed information about the current development status, known issues, and planned features, see [DEVELOPMENT_STATUS.md](doc/DEVELOPMENT_STATUS.md).

### Quick Status Summary

**Working**:
- Kubernetes deployment on K3S
- All core services (API Server, Gateway, Dashboard, Controllers)
- Device CRD creation and management
- Renode container orchestration
- MCU type support (13 MCU types)
- Dashboard UI and API integration

**In Progress**:
- End-to-end TLS connection verification
- Application deployment workflow
- Real hardware device integration

**Known Issues**:
- See [DEVELOPMENT_STATUS.md](doc/DEVELOPMENT_STATUS.md) for detailed list

## License

AGPL-3.0

See [LICENSE](LICENSE) file for details.

## Contributing

This project is part of the RETROSPECT research project. For contributions, please contact the project maintainers.

## Contact

For questions or issues:
- Check documentation in [`doc/`](doc/)
- Review [DEVELOPMENT_STATUS.md](doc/DEVELOPMENT_STATUS.md) for known issues
- Check component logs: `kubectl logs -n wasmbed <pod-name>`
