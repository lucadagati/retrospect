# Wasmbed Platform

A Kubernetes-native middleware platform for deploying WebAssembly applications to edge devices.

## Quick Start

```bash
# Clone and deploy
git clone <repository-url>
cd retrospect
./scripts/wasmbed.sh deploy

# Access dashboard
open http://localhost:30470
```

## Features

- **Kubernetes-native**: Deploy WASM applications through Kubernetes manifests
- **Edge-optimized**: Designed for resource-constrained edge devices
- **Real-time Dashboard**: Web-based management interface
- **Secure Terminal**: Command execution with predefined commands
- **Device Management**: Complete device lifecycle management
- **Application Deployment**: WASM application orchestration

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CONTROL PLANE │    │   GATEWAY LAYER │    │   DEVICE LAYER  │
│                 │    │                 │    │                 │
│  Kubernetes     │◄──►│ Gateway MPU     │◄──►│ Edge Devices    │
│  Orchestrator   │    │ (WASM Runtime)  │    │ (RISC-V MCUs)   │
│                 │    │                 │    │                 │
│  - Dashboard    │    │ - Device Mgmt   │    │ - WASM Apps     │
│  - API Services │    │ - App Runtime   │    │ - Sensors       │
│  - Controllers  │    │ - Communication │    │ - Communication │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Service Endpoints

- **Dashboard UI**: http://localhost:30470
- **Dashboard API**: http://localhost:30453
- **Infrastructure API**: http://localhost:30461
- **Gateway API**: http://localhost:30451

## Management Scripts

```bash
# Main management script
./scripts/wasmbed.sh deploy    # Deploy platform
./scripts/wasmbed.sh status    # Check status
./scripts/wasmbed.sh stop      # Stop services
./scripts/wasmbed.sh clean     # Clean up
./scripts/wasmbed.sh build     # Build components
./scripts/wasmbed.sh test      # Run tests
```

## Development

### Prerequisites
- Rust 1.70+
- Kubernetes 1.25+
- Node.js 18+
- k3d (for local Kubernetes)

### Build
```bash
# Build all components
./scripts/wasmbed.sh build

# Build specific component
cargo build --package wasmbed-dashboard
```

### Run Tests
```bash
# Run all tests
./scripts/wasmbed.sh test

# Run specific tests
cargo test --package wasmbed-dashboard
```

## Documentation

- [Complete Documentation](docs/README.md)
- [API Reference](docs/api/)
- [Architecture Guide](docs/architecture/)
- [Deployment Guide](docs/deployment/)

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `./scripts/wasmbed.sh test`
5. Submit a pull request

## License

AGPL-3.0 - see [LICENSE](LICENSE) for details.

## Status

**Current Version**: 0.1.0  
**Implementation Status**: Core platform complete with dashboard, backend services, and Kubernetes integration. WASM runtime and device support are placeholders ready for implementation.