# Wasmbed Platform - UML Architecture Diagrams

This directory contains comprehensive UML diagrams for the Wasmbed Platform architecture, documenting the complete system design, component interactions, and implementation details.

## Diagram Overview

### System Architecture Diagrams

#### 1. System Architecture (`system-architecture.puml`)
- **Purpose**: Complete system overview with all layers and components
- **Scope**: Frontend, API, Gateway, Device Management, Kubernetes, and Device layers
- **Key Components**: React Dashboard, API Server, Gateway, Renode Manager, ARM Cortex-M4 devices
- **Implementation**: Reflects current Renode integration and real TLS implementation
- **Layout**: Vertical orientation optimized for A4 format

#### 2. Security Architecture (`security-architecture.puml`)
- **Purpose**: Comprehensive security model and TLS implementation
- **Scope**: Certificate Authority, TLS communication, device authentication, infrastructure security
- **Key Components**: X.509 v3 certificates, TLS 1.3, rustls implementation, WASM sandbox
- **Implementation**: Real certificate management and mutual TLS authentication
- **Layout**: Vertical orientation optimized for A4 format

#### 3. Renode Device Architecture (`renode-device-architecture.puml`)
- **Purpose**: Detailed device emulation and firmware architecture
- **Scope**: Renode emulation layer, ARM Cortex-M4 devices, firmware components
- **Key Components**: Arduino Nano 33 BLE, STM32F4 Discovery, Arduino Uno R4, real firmware
- **Implementation**: Complete constrained device emulation with Renode
- **Layout**: Vertical orientation optimized for A4 format

#### 4. Application Deployment Architecture (`application-deployment-architecture.puml`)
- **Purpose**: WASM application deployment pipeline and runtime management
- **Scope**: Compilation, deployment, execution, monitoring, and management
- **Key Components**: Rust compiler, WASM runtime, device execution, Kubernetes integration
- **Implementation**: Real WASM compilation and deployment to constrained devices
- **Layout**: Vertical orientation optimized for A4 format

### Sequence Diagrams

#### 1. Device Enrollment (`../sequence-diagrams/device-enrollment-detailed.puml`)
- **Purpose**: Complete device enrollment workflow with Renode integration
- **Scope**: Device initialization, TLS handshake, enrollment, heartbeat setup
- **Key Components**: Arduino Nano 33 BLE, TLS client, Gateway, Renode Manager, Kubernetes
- **Implementation**: Real TLS 1.3 handshake and X.509 v3 certificate validation
- **Layout**: Vertical orientation optimized for A4 format

#### 2. Application Deployment (`../sequence-diagrams/application-deployment-detailed.puml`)
- **Purpose**: Complete application deployment workflow from compilation to execution
- **Scope**: Rust compilation, WASM deployment, device execution, monitoring
- **Key Components**: User, Dashboard, API Server, Gateway, WASM Runtime, Arduino devices
- **Implementation**: Real WASM compilation and deployment to Renode devices
- **Layout**: Vertical orientation optimized for A4 format

## Technical Implementation Details

### Current Implementation Status

All diagrams reflect the **production-ready implementation** with:

- ✅ **Real Components**: No mocks or simulations
- ✅ **Renode Integration**: Complete ARM Cortex-M4 device emulation
- ✅ **TLS 1.3**: Real mutual TLS authentication with X.509 v3 certificates
- ✅ **WASM Runtime**: Complete WebAssembly execution engine
- ✅ **Kubernetes Integration**: Full CRD and controller implementation
- ✅ **Real Firmware**: Actual embedded firmware with TLS client

### Key Technologies

- **Renode**: ARM Cortex-M4 device emulation
- **rustls**: TLS 1.3 implementation with Ring crypto provider
- **wasmtime**: WebAssembly runtime for constrained devices
- **Kubernetes**: Container orchestration and resource management
- **CBOR**: Efficient binary serialization protocol
- **X.509 v3**: Certificate infrastructure for device authentication

### Device Support

- **Arduino Nano 33 BLE**: ARM Cortex-M4, 1MB RAM, 256KB Flash
- **STM32F4 Discovery**: ARM Cortex-M4, 1MB RAM, 1MB Flash
- **Arduino Uno R4**: ARM Cortex-M4, 512KB RAM, 256KB Flash

## Usage Instructions

### Viewing Diagrams

1. **PlantUML Online**: Use [PlantUML Online Server](http://www.plantuml.com/plantuml/uml/)
2. **VS Code Extension**: Install PlantUML extension for VS Code
3. **Local Rendering**: Use PlantUML JAR file for local rendering

### Generating Images

```bash
# Install PlantUML
sudo apt-get install plantuml

# Generate PNG images (A4 optimized)
plantuml -tpng *.puml

# Generate SVG images (A4 optimized)
plantuml -tsvg *.puml

# Generate PDF images (A4 optimized)
plantuml -tpdf *.puml

# Generate high-resolution images for A4 printing
plantuml -tpng -Sdpi=300 *.puml
```

### A4 Format Optimization

All diagrams are optimized for A4 format with:

- **Vertical Layout**: `skinparam direction top to bottom` for better A4 fit
- **Compact Design**: Optimized component spacing and sizing
- **Clear Hierarchy**: Logical top-to-bottom flow
- **Readable Text**: Appropriate font sizes for A4 printing
- **Professional Appearance**: Clean, professional styling

### Editing Diagrams

1. Open `.puml` files in any text editor
2. Modify components, connections, and notes as needed
3. Use PlantUML syntax for additional elements
4. Test changes using PlantUML online server
5. Maintain vertical layout for A4 compatibility

## Architecture Principles

### Design Patterns

- **Controller Pattern**: Kubernetes controllers for resource reconciliation
- **Gateway Pattern**: Centralized communication hub for devices
- **Microservices**: Modular service architecture
- **Event-Driven**: Asynchronous event processing
- **Layered Architecture**: Clear separation of concerns

### Security Principles

- **Defense in Depth**: Multiple security layers
- **Zero Trust**: Verify all communications
- **Certificate-Based**: Strong authentication
- **Encryption**: End-to-end encrypted communication
- **Sandboxing**: Isolated execution environments

### Scalability Principles

- **Horizontal Scaling**: Multiple gateway instances
- **Load Balancing**: Kubernetes service load balancing
- **Resource Management**: Proper resource limits
- **Auto-scaling**: Kubernetes HPA integration
- **Stateless Services**: Scalable service design

## Maintenance and Updates

### Keeping Diagrams Current

- Update diagrams when implementing new features
- Reflect actual implementation changes
- Maintain consistency with codebase
- Document architectural decisions
- Update component relationships
- Maintain A4 format optimization

### Version Control

- All diagrams are version controlled
- Changes tracked with git
- Documentation updated with implementation
- Regular review and validation

## Related Documentation

- **[API Reference](../api/api-reference.md)**: Complete API documentation
- **[Deployment Guide](../deployment/deployment-guide.md)**: Step-by-step deployment
- **[Security Overview](../security/security-overview.md)**: Security implementation
- **[Complete Implementation](../implementation/complete-implementation.md)**: Implementation status

---

**Last Updated**: 2025  
**Version**: 0.1.0  
**Status**: Production Ready  
**Format**: A4 Optimized
