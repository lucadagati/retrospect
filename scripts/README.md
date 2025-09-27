# Wasmbed Platform - Scripts Documentation

This directory contains all the management scripts for the Wasmbed Platform. These scripts provide a complete command-line interface for managing the platform.

## Quick Start

```bash
# Make all scripts executable
chmod +x scripts/*.sh

# Load environment (add to ~/.bashrc or ~/.zshrc)
source scripts/env.sh

# Deploy the complete platform
wasmbed deploy

# Check status
wasmbed status

# Stop all services
wasmbed stop
```

## Main Scripts

### `wasmbed.sh` - Main Management Console
The primary script that provides access to all platform operations.

**Usage:**
```bash
wasmbed [COMMAND] [OPTIONS]
```

**Commands:**
- `clean` - Clean up all resources
- `build` - Build all components
- `deploy` - Deploy complete platform
- `stop` - Stop all services
- `status` - Check system status
- `restart` - Restart all services
- `devices [cmd]` - Manage devices
- `applications [cmd]` - Manage applications
- `monitor [cmd]` - Monitor system
- `test` - Run workflow tests

### `clean.sh` - System Cleanup
Cleans up all Wasmbed resources and processes.

**Features:**
- Stops all running processes
- Removes k3d clusters
- Cleans Docker resources
- Removes build artifacts
- Cleans certificates and logs

### `build.sh` - Build System
Builds all Wasmbed components.

**Features:**
- Builds Rust components
- Builds React Dashboard (if Node.js available)
- Generates certificates
- Lists built components

### `deploy.sh` - Platform Deployment
Deploys the complete Wasmbed platform.

**Features:**
- Creates k3d cluster
- Applies Kubernetes resources
- Starts all services
- Tests service endpoints
- Provides deployment summary

### `stop.sh` - Stop Services
Stops all Wasmbed services gracefully.

**Features:**
- Stops services using saved PIDs
- Stops remaining processes by name
- Removes k3d cluster

### `status.sh` - Status Check
Comprehensive status check of the platform.

**Features:**
- Checks k3d cluster status
- Verifies Kubernetes resources
- Tests service endpoints
- Shows running processes
- Provides service endpoints

## Resource Management Scripts

### `devices.sh` - Device Management
Manages devices in the platform.

**Commands:**
- `list` - List all devices
- `create <name>` - Create new device
- `delete <name>` - Delete device
- `status <name>` - Show device status
- `enroll <name>` - Force enrollment
- `connect <name>` - Simulate connection
- `disconnect <name>` - Simulate disconnection
- `logs <name>` - Show device logs

### `applications.sh` - Application Management
Manages applications in the platform.

**Commands:**
- `list` - List all applications
- `create <name>` - Create new application
- `delete <name>` - Delete application
- `status <name>` - Show application status
- `deploy <name>` - Force deployment
- `stop <name>` - Stop application
- `restart <name>` - Restart application
- `logs <name>` - Show application logs

### `monitor.sh` - Monitoring & Observability
Provides monitoring and observability features.

**Commands:**
- `overview` - System overview
- `devices` - Device metrics
- `applications` - Application metrics
- `gateways` - Gateway metrics
- `infrastructure` - Infrastructure metrics
- `logs` - System logs
- `health` - Health check
- `watch` - Real-time resource watching

## Utility Scripts

### `certs.sh` - Certificate Management
Manages TLS certificates for the platform.

**Commands:**
- `generate` - Generate new certificates
- `renew` - Renew existing certificates
- `validate` - Validate certificate chain
- `info` - Show certificate information
- `clean` - Clean up certificates

### `logs.sh` - Log Management
Manages logs and debugging.

**Commands:**
- `show` - Show recent logs
- `follow` - Follow logs in real-time
- `errors` - Show only error logs
- `events` - Show Kubernetes events
- `controller <name>` - Show controller logs
- `service <name>` - Show service logs
- `debug` - Enable debug logging
- `clean` - Clean up log files

### `test-complete-workflows.sh` - End-to-End Testing
Comprehensive testing of all workflows.

**Features:**
- Tests complete deployment
- Verifies all services
- Checks resource status
- Provides detailed test results

## Environment Setup

### `env.sh` - Environment Configuration
Sets up environment variables and aliases.

**Features:**
- Sets up aliases for quick access
- Exports service URLs
- Provides helpful messages

**Aliases:**
- `wb-clean` - Quick clean
- `wb-build` - Quick build
- `wb-deploy` - Quick deploy
- `wb-stop` - Quick stop
- `wb-status` - Quick status
- `wb-devices` - Quick device management
- `wb-apps` - Quick application management
- `wb-monitor` - Quick monitoring

## Service Endpoints

When the platform is deployed, the following endpoints are available:

- **Infrastructure API**: http://localhost:30460
- **Gateway API**: http://localhost:30451
- **Dashboard UI**: http://localhost:30470
- **Monitoring**: http://localhost:9090
- **Logging**: http://localhost:8080

## Examples

### Basic Operations
```bash
# Deploy the platform
wasmbed deploy

# Check status
wasmbed status

# Stop everything
wasmbed stop
```

### Device Management
```bash
# List devices
wasmbed devices list

# Create a device
wasmbed devices create my-device

# Check device status
wasmbed devices status my-device

# Force enrollment
wasmbed devices enroll my-device
```

### Application Management
```bash
# List applications
wasmbed applications list

# Create an application
wasmbed applications create my-app

# Deploy application
wasmbed applications deploy my-app

# Check status
wasmbed applications status my-app
```

### Monitoring
```bash
# System overview
wasmbed monitor overview

# Check health
wasmbed monitor health

# Watch resources
wasmbed monitor watch

# Show device metrics
wasmbed monitor devices
```

### Testing
```bash
# Run complete tests
wasmbed test

# Test specific components
wasmbed test-devices
wasmbed test-applications
wasmbed test-gateways
```

## Troubleshooting

### Common Issues

1. **Port conflicts**: Use `wasmbed stop` to clean up, then `wasmbed deploy`
2. **Certificate issues**: Run `./scripts/certs.sh generate`
3. **Build failures**: Run `wasmbed clean` then `wasmbed build`
4. **Service not responding**: Check with `wasmbed status`

### Debug Mode

Enable debug logging:
```bash
./scripts/logs.sh debug
wasmbed restart
```

### Log Analysis

View logs:
```bash
# Show recent logs
./scripts/logs.sh show

# Follow logs
./scripts/logs.sh follow

# Show errors only
./scripts/logs.sh errors
```

## Contributing

When adding new scripts:

1. Follow the existing naming convention
2. Include proper help documentation
3. Use the standard color scheme
4. Add error handling
5. Make scripts executable
6. Update this documentation

## License

All scripts are licensed under AGPL-3.0. See the main project license for details.
