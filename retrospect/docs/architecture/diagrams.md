# Architecture Diagrams

## Overview

This document contains comprehensive Mermaid diagrams for the Wasmbed platform architecture, including system overview, component interactions, data flows, and deployment scenarios.

## System Architecture Overview

### High-Level 3-Layer Architecture

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
        GW[Gateway MPU]
        TLS[TLS Bridge]
        MR[microROS Bridge]
        FDDS[FastDDS Middleware]
        AUTH[Device Authentication]
        HTTP[HTTP API Server]
        HB[Heartbeat Monitor]
    end
    
    subgraph "Edge Layer"
        MPU[MPU Devices]
        MCU[MCU Devices]
        RISCV[RISC-V Devices]
        WASM[WebAssembly Runtime]
        TLS_CLIENT[TLS Client]
        QEMU[QEMU Emulation]
    end
    
    subgraph "PX4 Integration"
        PX4[PX4 Autopilot]
        UORB[UORB Topics]
        MAV[MAVLink Protocol]
    end
    
    K8S --> GW
    CTRL --> GW
    CRD --> GW
    RBAC --> GW
    ETCD --> K8S
    API --> CTRL
    
    GW --> TLS
    GW --> MR
    GW --> FDDS
    GW --> AUTH
    GW --> HTTP
    GW --> HB
    
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
    
    RISCV --> QEMU
    MCU --> QEMU
    MPU --> QEMU
    
    MR --> PX4
    FDDS --> PX4
    PX4 --> UORB
    PX4 --> MAV
```

## Component Architecture

### Cloud Layer Components

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
        CFG[Configuration CRD]
    end
    
    subgraph "Custom Controllers"
        APP_CTRL[Application Controller]
        DEV_CTRL[Device Controller]
        CFG_CTRL[Configuration Controller]
    end
    
    subgraph "RBAC"
        SA[Service Accounts]
        ROLE[Roles]
        RB[Role Bindings]
    end
    
    API --> CTRL
    CTRL --> APP_CTRL
    CTRL --> DEV_CTRL
    CTRL --> CFG_CTRL
    
    APP_CTRL --> APP
    DEV_CTRL --> DEV
    CFG_CTRL --> CFG
    
    SA --> ROLE
    ROLE --> RB
```

### Fog Layer Components

```mermaid
graph TB
    subgraph "Gateway MPU"
        HTTP[HTTP API Server]
        TLS_SRV[TLS Server]
        MR_BRIDGE[microROS Bridge]
        FDDS_MW[FastDDS Middleware]
        AUTH_SRV[Authentication Service]
        HB_MON[Heartbeat Monitor]
        APP_MGR[Application Manager]
    end
    
    subgraph "Communication Protocols"
        TLS_PROTO[TLS 1.3]
        CBOR_PROTO[CBOR Protocol]
        DDS_PROTO[DDS Protocol]
        HTTP_PROTO[HTTP/2]
    end
    
    subgraph "Security"
        CERT_MGR[Certificate Manager]
        KEY_STORE[Key Store]
        AUTH_MGR[Authentication Manager]
    end
    
    HTTP --> HTTP_PROTO
    TLS_SRV --> TLS_PROTO
    MR_BRIDGE --> DDS_PROTO
    FDDS_MW --> DDS_PROTO
    AUTH_SRV --> CBOR_PROTO
    
    CERT_MGR --> TLS_SRV
    KEY_STORE --> AUTH_SRV
    AUTH_MGR --> AUTH_SRV
```

### Edge Layer Components

```mermaid
graph TB
    subgraph "Device Types"
        MPU[MPU Devices]
        MCU[MCU Devices]
        RISCV[RISC-V Devices]
    end
    
    subgraph "Runtime Components"
        WASM_RT[WebAssembly Runtime]
        TLS_CLI[TLS Client]
        APP_LOADER[Application Loader]
        HB_CLIENT[Heartbeat Client]
    end
    
    subgraph "QEMU Emulation"
        QEMU_RISCV[qemu-system-riscv32]
        QEMU_ARM[qemu-system-arm]
        QEMU_XTENSA[qemu-system-xtensa]
        SERIAL_BRIDGE[Serial Bridge]
    end
    
    MPU --> WASM_RT
    MCU --> WASM_RT
    RISCV --> WASM_RT
    
    MPU --> TLS_CLI
    MCU --> TLS_CLI
    RISCV --> TLS_CLI
    
    MPU --> APP_LOADER
    MCU --> APP_LOADER
    RISCV --> APP_LOADER
    
    RISCV --> QEMU_RISCV
    MCU --> QEMU_ARM
    MPU --> QEMU_XTENSA
    
    QEMU_RISCV --> SERIAL_BRIDGE
    QEMU_ARM --> SERIAL_BRIDGE
    QEMU_XTENSA --> SERIAL_BRIDGE
```

## Communication Flow Architecture

### Complete System Communication Flow

```mermaid
sequenceDiagram
    participant U as User
    participant K as Kubernetes
    participant C as Controller
    participant G as Gateway
    participant D as Device
    participant PX4 as PX4
    
    Note over U,PX4: Complete System Communication Flow
    
    U->>K: 1. Deploy Application
    K->>C: 2. Application Event
    C->>G: 3. Deploy to Gateway
    G->>D: 4. Send WASM Binary
    D->>D: 5. Load WASM Runtime
    D->>G: 6. Application Ready
    
    Note over G,PX4: PX4 Integration Flow
    PX4->>G: 7. UORB Topics
    G->>D: 8. Process Commands
    D->>G: 9. Control Commands
    G->>PX4: 10. MAVLink Commands
    PX4->>G: 11. Status Updates
    G->>D: 12. Update Status
    
    Note over D,K: Status Reporting
    D->>G: 13. Heartbeat
    G->>K: 14. Update Status
    K->>C: 15. Status Event
    C->>K: 16. Update CRD
```

## Security Architecture

### Multi-Layer Security Model

```mermaid
graph TB
    subgraph "Security Layers"
        APP_SEC[Application Security]
        TRANS_SEC[Transport Security]
        NET_SEC[Network Security]
        INFRA_SEC[Infrastructure Security]
    end
    
    subgraph "Application Security"
        WASM_SANDBOX[WASM Sandboxing]
        MEM_PROT[Memory Protection]
        RES_LIMIT[Resource Limits]
        CODE_SIGN[Code Signing]
    end
    
    subgraph "Transport Security"
        TLS_MUTUAL[Mutual TLS]
        CERT_MGMT[Certificate Management]
        KEY_EXCH[Key Exchange]
        MSG_ENC[Message Encryption]
    end
    
    subgraph "Network Security"
        FIREWALL[Firewall Rules]
        NET_POL[Network Policies]
        VPN[VPN Tunnels]
        IDS[Intrusion Detection]
    end
    
    subgraph "Infrastructure Security"
        RBAC_K8S[Kubernetes RBAC]
        POD_SEC[Pod Security]
        SECRET_MGMT[Secret Management]
        AUDIT_LOG[Audit Logging]
    end
    
    APP_SEC --> WASM_SANDBOX
    APP_SEC --> MEM_PROT
    APP_SEC --> RES_LIMIT
    APP_SEC --> CODE_SIGN
    
    TRANS_SEC --> TLS_MUTUAL
    TRANS_SEC --> CERT_MGMT
    TRANS_SEC --> KEY_EXCH
    TRANS_SEC --> MSG_ENC
    
    NET_SEC --> FIREWALL
    NET_SEC --> NET_POL
    NET_SEC --> VPN
    NET_SEC --> IDS
    
    INFRA_SEC --> RBAC_K8S
    INFRA_SEC --> POD_SEC
    INFRA_SEC --> SECRET_MGMT
    INFRA_SEC --> AUDIT_LOG
```

## Data Flow Architecture

### Data Processing Pipeline

```mermaid
graph LR
    subgraph "Data Sources"
        PX4_DATA[PX4 Data]
        SENSOR_DATA[Sensor Data]
        APP_DATA[Application Data]
        SYS_DATA[System Data]
    end
    
    subgraph "Data Processing"
        COLLECT[Data Collection]
        TRANSFORM[Data Transformation]
        VALIDATE[Data Validation]
        AGGREGATE[Data Aggregation]
    end
    
    subgraph "Data Storage"
        K8S_STORE[Kubernetes State]
        CACHE[Memory Cache]
        LOGS[Audit Logs]
        METRICS[Metrics Store]
    end
    
    subgraph "Data Consumption"
        DASHBOARD[Dashboard]
        ALERTS[Alerting]
        API_CONSUMERS[API Consumers]
        ANALYTICS[Analytics]
    end
    
    PX4_DATA --> COLLECT
    SENSOR_DATA --> COLLECT
    APP_DATA --> COLLECT
    SYS_DATA --> COLLECT
    
    COLLECT --> TRANSFORM
    TRANSFORM --> VALIDATE
    VALIDATE --> AGGREGATE
    
    AGGREGATE --> K8S_STORE
    AGGREGATE --> CACHE
    AGGREGATE --> LOGS
    AGGREGATE --> METRICS
    
    K8S_STORE --> DASHBOARD
    CACHE --> ALERTS
    LOGS --> API_CONSUMERS
    METRICS --> ANALYTICS
```

## Component Interaction Matrix

### Component Relationships

```mermaid
graph TB
    subgraph "Component Interactions"
        K8S_API[Kubernetes API]
        CTRL[Controller]
        GW[Gateway]
        DEV[Device]
        PX4[PX4]
    end
    
    K8S_API <--> CTRL
    CTRL <--> GW
    GW <--> DEV
    GW <--> PX4
    DEV <--> PX4
    
    Note over K8S_API,CTRL: CRD Management
    Note over CTRL,GW: Application Deployment
    Note over GW,DEV: Device Communication
    Note over GW,PX4: PX4 Integration
    Note over DEV,PX4: Direct Control
```

## Scalability Architecture

### Horizontal and Vertical Scaling

```mermaid
graph TB
    subgraph "Horizontal Scaling"
        LB[Load Balancer]
        GW1[Gateway 1]
        GW2[Gateway 2]
        GW3[Gateway 3]
        GW_N[Gateway N]
    end
    
    subgraph "Vertical Scaling"
        CPU[CPU Scaling]
        MEM[Memory Scaling]
        STORAGE[Storage Scaling]
        NET[Network Scaling]
    end
    
    subgraph "Auto Scaling"
        HPA[Horizontal Pod Autoscaler]
        VPA[Vertical Pod Autoscaler]
        CA[Cluster Autoscaler]
        METRICS[Metrics Server]
    end
    
    LB --> GW1
    LB --> GW2
    LB --> GW3
    LB --> GW_N
    
    HPA --> GW1
    HPA --> GW2
    HPA --> GW3
    HPA --> GW_N
    
    VPA --> CPU
    VPA --> MEM
    VPA --> STORAGE
    VPA --> NET
    
    METRICS --> HPA
    METRICS --> VPA
    METRICS --> CA
```

## Deployment Architecture

### Environment Progression

```mermaid
graph TB
    subgraph "Development Environment"
        DEV_K8S[k3d Cluster]
        DEV_GW[Gateway Dev]
        DEV_QEMU[QEMU Devices]
    end
    
    subgraph "Staging Environment"
        STAGE_K8S[Staging K8s]
        STAGE_GW[Gateway Staging]
        STAGE_DEV[Staging Devices]
    end
    
    subgraph "Production Environment"
        PROD_K8S[Production K8s]
        PROD_GW[Gateway Prod]
        PROD_DEV[Production Devices]
    end
    
    DEV_K8S --> STAGE_K8S
    STAGE_K8S --> PROD_K8S
    
    DEV_GW --> STAGE_GW
    STAGE_GW --> PROD_GW
    
    DEV_QEMU --> STAGE_DEV
    STAGE_DEV --> PROD_DEV
```

## Monitoring and Observability Architecture

### Complete Monitoring Stack

```mermaid
graph TB
    subgraph "Data Collection"
        METRICS_COLL[Metrics Collection]
        LOGS_COLL[Logs Collection]
        TRACES_COLL[Traces Collection]
        EVENTS_COLL[Events Collection]
    end
    
    subgraph "Data Processing"
        METRICS_PROC[Metrics Processing]
        LOGS_PROC[Logs Processing]
        TRACES_PROC[Traces Processing]
        EVENTS_PROC[Events Processing]
    end
    
    subgraph "Data Storage"
        PROMETHEUS[Prometheus]
        LOKI[Loki]
        JAEGER[Jaeger]
        ETCD_STORE[etcd Store]
    end
    
    subgraph "Visualization"
        GRAFANA[Grafana]
        KIBANA[Kibana]
        JAEGER_UI[Jaeger UI]
        K8S_DASH[K8s Dashboard]
    end
    
    subgraph "Alerting"
        ALERT_MGR[Alert Manager]
        NOTIFICATION[Notifications]
        ESCALATION[Escalation]
    end
    
    METRICS_COLL --> METRICS_PROC
    LOGS_COLL --> LOGS_PROC
    TRACES_COLL --> TRACES_PROC
    EVENTS_COLL --> EVENTS_PROC
    
    METRICS_PROC --> PROMETHEUS
    LOGS_PROC --> LOKI
    TRACES_PROC --> JAEGER
    EVENTS_PROC --> ETCD_STORE
    
    PROMETHEUS --> GRAFANA
    LOKI --> KIBANA
    JAEGER --> JAEGER_UI
    ETCD_STORE --> K8S_DASH
    
    PROMETHEUS --> ALERT_MGR
    ALERT_MGR --> NOTIFICATION
    NOTIFICATION --> ESCALATION
```

## PX4 Integration Architecture

### PX4 Communication Stack

```mermaid
graph TB
    subgraph "PX4 Autopilot"
        PX4_CORE[PX4 Core]
        UORB[UORB Topics]
        MAV[MAVLink Protocol]
        FLIGHT_MODES[Flight Modes]
    end
    
    subgraph "microROS Bridge"
        MR_NODE[microROS Node]
        MR_PUB[microROS Publishers]
        MR_SUB[microROS Subscribers]
        MR_SRV[microROS Services]
    end
    
    subgraph "FastDDS Middleware"
        FDDS_PARTICIPANT[Domain Participant]
        FDDS_PUB[Data Writers]
        FDDS_SUB[Data Readers]
        FDDS_QOS[QoS Profiles]
    end
    
    subgraph "Gateway Integration"
        GW_BRIDGE[PX4 Bridge]
        GW_PROCESS[Message Processing]
        GW_ROUTE[Message Routing]
    end
    
    subgraph "WASM Application"
        WASM_APP[WASM App]
        WASM_LOGIC[Control Logic]
        WASM_STATE[State Management]
    end
    
    PX4_CORE --> UORB
    UORB --> MAV
    MAV --> FLIGHT_MODES
    
    MR_NODE --> MR_PUB
    MR_NODE --> MR_SUB
    MR_NODE --> MR_SRV
    
    FDDS_PARTICIPANT --> FDDS_PUB
    FDDS_PARTICIPANT --> FDDS_SUB
    FDDS_PARTICIPANT --> FDDS_QOS
    
    GW_BRIDGE --> GW_PROCESS
    GW_PROCESS --> GW_ROUTE
    
    WASM_APP --> WASM_LOGIC
    WASM_LOGIC --> WASM_STATE
    
    UORB --> MR_NODE
    MR_PUB --> FDDS_PUB
    MR_SUB --> FDDS_SUB
    FDDS_PUB --> GW_BRIDGE
    FDDS_SUB --> GW_BRIDGE
    GW_ROUTE --> WASM_APP
    WASM_STATE --> GW_ROUTE
```

## QEMU Emulation Architecture

### Device Emulation Stack

```mermaid
graph TB
    subgraph "QEMU Emulators"
        QEMU_RISCV[qemu-system-riscv32]
        QEMU_ARM[qemu-system-arm]
        QEMU_XTENSA[qemu-system-xtensa]
    end
    
    subgraph "Firmware Images"
        RISCV_FW[RISC-V Firmware]
        ARM_FW[ARM Firmware]
        ESP32_FW[ESP32 Firmware]
    end
    
    subgraph "Serial Communication"
        SERIAL_TCP[TCP Serial Bridge]
        SERIAL_UART[UART Interface]
        SERIAL_PROTOCOL[Serial Protocol]
    end
    
    subgraph "Device Simulation"
        DEV_SIM[Device Simulator]
        NET_SIM[Network Simulator]
        SENSOR_SIM[Sensor Simulator]
    end
    
    subgraph "Gateway Integration"
        GW_QEMU[QEMU Bridge]
        GW_SERIAL[Serial Handler]
        GW_DEVICE[Device Manager]
    end
    
    QEMU_RISCV --> RISCV_FW
    QEMU_ARM --> ARM_FW
    QEMU_XTENSA --> ESP32_FW
    
    RISCV_FW --> SERIAL_UART
    ARM_FW --> SERIAL_UART
    ESP32_FW --> SERIAL_UART
    
    SERIAL_UART --> SERIAL_TCP
    SERIAL_TCP --> SERIAL_PROTOCOL
    
    DEV_SIM --> NET_SIM
    NET_SIM --> SENSOR_SIM
    
    SERIAL_PROTOCOL --> GW_QEMU
    GW_QEMU --> GW_SERIAL
    GW_SERIAL --> GW_DEVICE
    
    SENSOR_SIM --> GW_QEMU
```

## Network Architecture

### Network Topology

```mermaid
graph TB
    subgraph "Cloud Network"
        CLOUD_LB[Cloud Load Balancer]
        CLOUD_GW[Cloud Gateway]
        CLOUD_K8S[Kubernetes Cluster]
    end
    
    subgraph "Fog Network"
        FOG_GW1[Fog Gateway 1]
        FOG_GW2[Fog Gateway 2]
        FOG_GW3[Fog Gateway 3]
        FOG_SWITCH[Fog Switch]
    end
    
    subgraph "Edge Network"
        EDGE_SWITCH[Edge Switch]
        EDGE_DEV1[Edge Device 1]
        EDGE_DEV2[Edge Device 2]
        EDGE_DEV3[Edge Device 3]
    end
    
    subgraph "PX4 Network"
        PX4_NET[PX4 Network]
        PX4_DEV1[PX4 Device 1]
        PX4_DEV2[PX4 Device 2]
    end
    
    CLOUD_LB --> CLOUD_GW
    CLOUD_GW --> CLOUD_K8S
    
    CLOUD_GW --> FOG_SWITCH
    FOG_SWITCH --> FOG_GW1
    FOG_SWITCH --> FOG_GW2
    FOG_SWITCH --> FOG_GW3
    
    FOG_GW1 --> EDGE_SWITCH
    FOG_GW2 --> EDGE_SWITCH
    FOG_GW3 --> EDGE_SWITCH
    
    EDGE_SWITCH --> EDGE_DEV1
    EDGE_SWITCH --> EDGE_DEV2
    EDGE_SWITCH --> EDGE_DEV3
    
    FOG_GW1 --> PX4_NET
    FOG_GW2 --> PX4_NET
    FOG_GW3 --> PX4_NET
    
    PX4_NET --> PX4_DEV1
    PX4_NET --> PX4_DEV2
```

## Performance Architecture

### Performance Optimization Stack

```mermaid
graph TB
    subgraph "Application Performance"
        WASM_OPT[WASM Optimization]
        MEM_MGMT[Memory Management]
        CPU_OPT[CPU Optimization]
        CACHE_OPT[Cache Optimization]
    end
    
    subgraph "Communication Performance"
        PROTO_OPT[Protocol Optimization]
        COMPRESSION[Message Compression]
        BATCHING[Message Batching]
        RATE_LIMIT[Rate Limiting]
    end
    
    subgraph "Network Performance"
        BANDWIDTH[Bandwidth Management]
        LATENCY[Latency Optimization]
        THROUGHPUT[Throughput Optimization]
        RELIABILITY[Reliability Optimization]
    end
    
    subgraph "System Performance"
        RESOURCE_MGMT[Resource Management]
        LOAD_BALANCE[Load Balancing]
        SCALING[Auto Scaling]
        MONITORING[Performance Monitoring]
    end
    
    WASM_OPT --> MEM_MGMT
    MEM_MGMT --> CPU_OPT
    CPU_OPT --> CACHE_OPT
    
    PROTO_OPT --> COMPRESSION
    COMPRESSION --> BATCHING
    BATCHING --> RATE_LIMIT
    
    BANDWIDTH --> LATENCY
    LATENCY --> THROUGHPUT
    THROUGHPUT --> RELIABILITY
    
    RESOURCE_MGMT --> LOAD_BALANCE
    LOAD_BALANCE --> SCALING
    SCALING --> MONITORING
    
    CACHE_OPT --> PROTO_OPT
    RATE_LIMIT --> BANDWIDTH
    RELIABILITY --> RESOURCE_MGMT
    MONITORING --> WASM_OPT
```
