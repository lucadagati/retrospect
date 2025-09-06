# Wasmbed Platform

[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](https://opensource.org/licenses/AGPL-3.0)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Kubernetes](https://img.shields.io/badge/kubernetes-1.28+-blue.svg)](https://kubernetes.io/)
[![WebAssembly](https://img.shields.io/badge/webassembly-wasm-purple.svg)](https://webassembly.org/)

A complete platform for deploying and executing WebAssembly applications on IoT edge devices, using Kubernetes as control plane and Gateways as intermediaries for communication with MCU devices.

## Key Features

- **Kubernetes Integration**: Complete orchestration through Custom Resource Definitions (CRDs)
- **Security First**: TLS 1.3, Ed25519 signatures, AES-256-GCM encryption
- **High Performance**: Optimized WASM runtime for MCU devices
- **Multi-Platform**: Support for ESP32 and RISC-V (HiFive1)
- **Easy Deployment**: Automated scripts for setup and testing
- **Comprehensive Monitoring**: Detailed metrics and alerting
- **Extensive Testing**: Complete end-to-end tests

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Kubernetes    │    │     Gateway     │    │   MCU Devices   │
│   Control Plane │◄──►│      (MPU)      │◄──►│   (ESP32/RISC-V)│
│                 │    │                 │    │                 │
│ • Device CRDs   │    │ • HTTP API      │    │ • WASM Runtime  │
│ • App CRDs      │    │ • TLS/CBOR      │    │ • Firmware      │
│ • Controller    │    │ • Security      │    │ • Hardware      │
│ • Monitoring    │    │ • Monitoring    │    │ • Communication│
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Prerequisites

- **Rust** 1.75+
- **Kubernetes** 1.28+ (k3d recommended)
- **QEMU** for RISC-V emulation
- **Docker** for containerization
- **OpenSSL** for certificate generation

## Quick Start

### 1. Clone the repository
```bash
git clone https://github.com/lucadagati/retrospect.git
cd retrospect
```

### 2. Deploy complete platform
```bash
# Deploy entire Wasmbed platform with 4 MCU devices
./deploy-complete.sh
```

### 3. Run microROS application
```bash
# Deploy and run microROS application for PX4 DDS communication
./run-microROS-app.sh

# Monitor application in real-time
./run-microROS-app.sh --monitor

# Check application status
./run-microROS-app.sh --status
```

### 4. Clean up everything
```bash
# Complete cleanup (removes all components)
./cleanup-all.sh

# Force cleanup without confirmation
./cleanup-all.sh --force
```

## Documentation

- **[API Documentation](docs/API_DOCUMENTATION.md)**: Complete API documentation
- **[Architecture](docs/ARCHITECTURE.md)**: Detailed platform architecture
- **[Examples](apps/)**: Usage examples and configuration
- **[Scripts](scripts/README.md)**: Automation scripts documentation

## Components

### Kubernetes Control Plane
- **Device CRD**: IoT device management
- **Application CRD**: WASM application management
- **Controller**: Automatic orchestration
- **Monitoring**: Metrics and alerting

### Gateway (MPU)
- **HTTP API**: RESTful API for management
- **TLS/CBOR**: Secure and efficient communication
- **Security**: Authentication and authorization
- **Monitoring**: System metrics collection

### MCU Devices
- **RISC-V (HiFive1)**: Custom WASM runtime for `no_std`
- **ESP32**: WASM runtime based on `wasmi`
- **Firmware**: Application management and communication
- **Hardware**: Peripheral interface

## Testing Status

The platform has been comprehensively tested with the following results:

### ✅ Core Components
- **Compilation**: All core components compile successfully
- **Unit Tests**: 6 tests passed (certificate serialization, protocol messages, device UUID)
- **Dependencies**: All Rust dependencies resolved correctly

### ✅ Kubernetes Deployment
- **Cluster Creation**: k3d cluster created successfully
- **CRDs**: Device and Application CRDs deployed and functional
- **RBAC**: Service accounts, roles, and bindings configured correctly
- **Namespace**: Wasmbed namespace created and isolated

### ✅ Gateway Functionality
- **Docker Image**: Gateway image built and imported to k3d
- **TLS Secrets**: Certificate secrets created and mounted
- **StatefulSet**: Gateway StatefulSet deployed (3 replicas)
- **Service**: Gateway service exposed on ports 8080/8443

### ✅ CRDs and Controller
- **Device CRD**: Successfully created test device with proper schema
- **Application CRD**: Successfully created test application
- **RBAC**: Controller permissions configured correctly
- **Resource Management**: CRUD operations working as expected

### ✅ Security and Certificates
- **Certificate Generation**: RSA certificates generated successfully
- **Certificate Validation**: CA-signed certificates validated correctly
- **TLS Configuration**: TLS 1.3 with proper key formats
- **Security Scan**: Basic security checks passed (RBAC, network policies, secrets)

### ⚠️ Known Issues
- **Gateway Certificate Parsing**: Gateway has issues parsing private keys (format compatibility)
- **Firmware Compilation**: RISC-V firmware has linking issues (missing libc functions)
- **Certificate Rotation**: Script has issues with private key conversion

### 🔧 Recommendations
1. **Gateway**: Fix private key parsing to support multiple formats
2. **Firmware**: Add proper libc linking for RISC-V target
3. **Certificates**: Improve certificate rotation script error handling
4. **Testing**: Add integration tests for Gateway TLS functionality

## Testing

### Automated Testing
```bash
# Complete platform deployment test
./deploy-complete.sh

# microROS application test
./run-microROS-app.sh --monitor

# Complete cleanup test
./cleanup-all.sh --force
```

### Manual Testing
```bash
# Unit tests
cargo test --workspace --lib

# Build verification
cargo build --workspace

# Component testing
kubectl get pods -n wasmbed
kubectl get devices -n wasmbed
kubectl get applications -n wasmbed
```

### Component Verification
```bash
# Check Gateway health
curl -k https://localhost:8443/health

# Verify DDS communication
kubectl logs -l app=wasmbed-gateway -n wasmbed --tail=20

# Test device connectivity
kubectl describe devices -n wasmbed
```

## Security

### TLS Certificates
- **CA Certificate**: `/etc/wasmbed/ca-cert.pem`
- **Server Certificate**: `/etc/wasmbed/server-cert.pem`
- **Server Private Key**: `/etc/wasmbed/server-key.pem`

### Encryption
- **TLS 1.3**: Secure communication
- **Ed25519**: Digital message signing
- **AES-256-GCM**: Sensitive data encryption

### Authentication
- **Certificate-based**: Certificate-based authentication
- **Public Key**: Device identity verification
- **RBAC**: Role-based access control

## Monitoring

### System Metrics
- **Devices**: Total, online, offline
- **Applications**: Total, running, stopped
- **Performance**: Latency, throughput, errors

### Device Metrics
- **CPU Usage**: Processor utilization
- **Memory Usage**: Memory utilization
- **Network**: Network traffic
- **Power**: Energy consumption

### Alerting
- **Health Checks**: Component status verification
- **Error Tracking**: Error tracking
- **Performance**: Performance degradation
- **Security**: Security events

## Deployment

### Automated Deployment
```bash
# Complete platform deployment
./deploy-complete.sh

# microROS application deployment
./run-microROS-app.sh

# Complete cleanup
./cleanup-all.sh
```

### Manual Deployment
```bash
# Create cluster
k3d cluster create wasmbed-platform --port "8080:80@loadbalancer" --port "8443:443@loadbalancer"

# Deploy CRDs
kubectl apply -f resources/k8s/crds/

# Deploy Gateway
kubectl apply -f resources/k8s/

# Create devices
kubectl apply -f - <<EOF
apiVersion: wasmbed.github.io/v1
kind: Device
metadata:
  name: test-device
  namespace: wasmbed
spec:
  deviceId: "test-device-001"
  deviceType: "riscv-hifive1"
  capabilities: ["wasm-execution", "tls-client"]
EOF
```

## Contributing

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **Push** to the branch (`git push origin feature/amazing-feature`)
5. **Open** a Pull Request

### Guidelines
- Follow Rust code conventions
- Add tests for new features
- Update documentation
- Maintain compatibility with existing versions

## License

This project is released under the [AGPL-3.0](LICENSE) license.

## Support

- **Issues**: [GitHub Issues](https://github.com/your-org/wasmbed/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/wasmbed/discussions)
- **Documentation**: [docs/](docs/)
- **Examples**: [apps/](apps/)

## Roadmap

### v0.2.0 (Next)
- [ ] Complete ESP32 support with wasmi
- [ ] Web dashboard for monitoring
- [ ] GraphQL API for advanced queries
- [ ] Support for standard IoT protocols

### v0.3.0 (Future)
- [ ] Multi-cloud deployment
- [ ] Edge-to-edge communication
- [ ] Machine learning integration
- [ ] 5G network support

## Acknowledgments

- **Rust Community** for the excellent ecosystem
- **Kubernetes** for orchestration
- **WebAssembly** for the universal runtime
- **Contributors** for support and feedback

---

**Wasmbed** - Bringing WebAssembly to edge computing