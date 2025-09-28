# Wasmbed Platform - Kubernetes WASM Middleware for Edge Devices

## Overview

Wasmbed is a Kubernetes-native middleware platform designed to deploy WebAssembly applications to resource-constrained edge devices. The platform provides a complete middleware stack for deploying WASM applications to edge devices through Kubernetes manifests, with a focus on real-time communication and device management.

**Key Features**:
- **Kubernetes-native**: Deploy WASM applications through standard Kubernetes manifests
- **Edge-optimized**: Designed for resource-constrained edge devices
- **Real-time communication**: DDS-based middleware for low-latency communication
- **WASM runtime**: Optimized WebAssembly runtime for edge devices
- **Dashboard-driven**: Web-based management interface for system configuration
- **Terminal integration**: Secure command execution for system monitoring

**Current Implementation**:
- **Dashboard**: React-based web interface with real-time system monitoring
- **Backend Services**: Rust-based microservices for device and application management
- **Kubernetes Integration**: Custom CRDs for device, application, and gateway management
- **Infrastructure Services**: Certificate management, logging, and monitoring
- **Terminal Interface**: Secure command execution with predefined commands

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

## Core Components

### 1. Dashboard (`wasmbed-dashboard`)
React-based web interface providing:
- System overview and monitoring
- Device and application management
- Gateway configuration
- Initial system setup wizard
- Secure terminal interface
- Real-time status updates

### 2. Gateway Service (`wasmbed-gateway`)
Gateway management service providing:
- Device connection management
- Application deployment
- Communication bridge
- Heartbeat monitoring
- Kubernetes CRD integration

### 3. Infrastructure Service (`wasmbed-infrastructure`)
Infrastructure management providing:
- Certificate authority
- Secret store
- Logging service
- Monitoring metrics
- Health checks

### 4. Kubernetes Controllers
- **Device Controller**: Manages device CRDs and lifecycle
- **Application Controller**: Manages application CRDs and deployment
- **Gateway Controller**: Manages gateway CRDs and configuration

## Current Implementation Status

### ✅ Completed Components
- **Dashboard**: Complete React interface with real-time data
- **Backend Services**: All microservices implemented and functional
- **Kubernetes Integration**: Custom CRDs and controllers working
- **Terminal Interface**: Secure command execution with whitelist
- **Initial Configuration**: Wizard for system setup
- **API Endpoints**: Complete REST API for all operations
- **CORS Support**: Cross-origin requests enabled
- **Real-time Updates**: Live system status and monitoring

### ⚠️ Current Limitations
- **WASM Runtime**: Placeholder implementation (needs real WASM execution)
- **Device Emulation**: QEMU integration not yet implemented
- **Real Hardware**: No physical device support yet
- **Application Deployment**: Mock responses (needs real deployment)

## Service Endpoints

When deployed, the following endpoints are available:

- **Dashboard UI**: http://localhost:30470
- **Dashboard API**: http://localhost:30453
- **Infrastructure API**: http://localhost:30461
- **Gateway API**: http://localhost:30451

## Quick Start

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

## Development Workflow

### Building the Platform
```bash
# Clean and build
./scripts/wasmbed.sh clean
./scripts/wasmbed.sh build

# Deploy
./scripts/wasmbed.sh deploy
```

### Testing
```bash
# Run tests
./scripts/wasmbed.sh test

# Check logs
./scripts/wasmbed.sh logs
```

### Development Mode
```bash
# Start individual services
cd crates/wasmbed-dashboard && cargo run -- --port 30453
cd crates/wasmbed-infrastructure && cargo run -- --port 30461
cd crates/wasmbed-gateway && cargo run -- --port 30451

# Start React dashboard
cd dashboard-react && npm start
```

## Configuration

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

## API Documentation

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

## Security Features

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

## Troubleshooting

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

## Contributing

### Development Setup
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `./scripts/wasmbed.sh test`
5. Submit a pull request

### Code Style
- Rust: Follow standard Rust formatting
- JavaScript: Use ESLint and Prettier
- Documentation: Update relevant docs

## License

This project is licensed under the AGPL-3.0 License - see the LICENSE file for details.

## Status

**Current Version**: 0.1.0  
**Last Updated**: 2025  
**Compatibility**: Rust 1.70+, Kubernetes 1.25+, Node.js 18+

**Implementation Status**: ✅ **CORE PLATFORM COMPLETE**
- **Dashboard**: ✅ Complete with real-time data
- **Backend Services**: ✅ All microservices functional
- **Kubernetes Integration**: ✅ CRDs and controllers working
- **Terminal Interface**: ✅ Secure command execution
- **API Endpoints**: ✅ Complete REST API
- **WASM Runtime**: ⚠️ Placeholder (needs real implementation)
- **Device Support**: ⚠️ Mock data (needs real hardware)

**Next Steps**:
1. Implement real WASM runtime for edge devices
2. Add QEMU integration for device emulation
3. Implement real hardware device support
4. Add real application deployment mechanisms