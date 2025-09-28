# Wasmbed Platform - Scripts

This directory contains essential scripts for managing the Wasmbed Platform.

## Quick Start

```bash
# Make all scripts executable
chmod +x scripts/*.sh

# Main management script
./scripts/wasmbed.sh deploy
./scripts/wasmbed.sh status
./scripts/wasmbed.sh stop
```

## Essential Scripts

### `wasmbed.sh` - Main Management Console
Unified interface for all platform operations.

**Usage:**
```bash
./scripts/wasmbed.sh [COMMAND]
```

**Commands:**
- `clean` - Clean up all resources
- `build` - Build all components  
- `deploy` - Deploy complete platform
- `stop` - Stop all services
- `status` - Check system status
- `restart` - Restart all services
- `logs` - Show system logs
- `test` - Run platform tests

### `wasmbed-deploy.sh` - Platform Deployment
Deploys the complete Wasmbed platform with minimal initial setup.

**Features:**
- Creates k3d cluster
- Applies Kubernetes resources
- Starts core services (no test resources)
- Tests service endpoints
- Provides deployment summary

### `wasmbed-build.sh` - Build System
Builds all Wasmbed components.

**Features:**
- Builds Rust components
- Builds React Dashboard (if Node.js available)
- Generates certificates
- Lists built components

### `wasmbed-clean.sh` - System Cleanup
Cleans up all Wasmbed resources and processes.

**Features:**
- Stops all running processes
- Removes k3d clusters
- Cleans Docker resources
- Removes build artifacts
- Cleans certificates and logs

### `wasmbed-stop.sh` - Stop Services
Stops all Wasmbed services gracefully.

**Features:**
- Stops services using saved PIDs
- Stops remaining processes by name
- Removes k3d cluster

### `wasmbed-status.sh` - Status Check
Comprehensive status check of the platform.

**Features:**
- Checks k3d cluster status
- Verifies Kubernetes resources
- Tests service endpoints
- Shows running processes
- Provides service endpoints

## Service Endpoints

When deployed, the following endpoints are available:

- **Dashboard UI**: http://localhost:30470
- **Dashboard API**: http://localhost:30453
- **Infrastructure API**: http://localhost:30461
- **Gateway API**: http://localhost:30451

## Examples

### Basic Operations
```bash
# Deploy the platform
./scripts/wasmbed.sh deploy

# Check status
./scripts/wasmbed.sh status

# Stop everything
./scripts/wasmbed.sh stop

# Clean and rebuild
./scripts/wasmbed.sh clean
./scripts/wasmbed.sh build
./scripts/wasmbed.sh deploy
```

### Development Workflow
```bash
# Clean start
./scripts/wasmbed.sh clean
./scripts/wasmbed.sh build
./scripts/wasmbed.sh deploy

# Check status
./scripts/wasmbed.sh status

# View logs
./scripts/wasmbed.sh logs

# Test platform
./scripts/wasmbed.sh test
```

## Troubleshooting

### Common Issues

1. **Port conflicts**: Use `./scripts/wasmbed.sh stop` to clean up
2. **Build failures**: Run `./scripts/wasmbed.sh clean` then `./scripts/wasmbed.sh build`
3. **Service not responding**: Check with `./scripts/wasmbed.sh status`

### Debug Mode

Enable debug logging:
```bash
./scripts/wasmbed-logs.sh debug
./scripts/wasmbed.sh restart
```

## License

All scripts are licensed under AGPL-3.0. See the main project license for details.