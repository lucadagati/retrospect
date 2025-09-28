# Wasmbed Platform - Scripts

This directory contains essential scripts for managing the Wasmbed Platform.

## Quick Start

```bash
# Make all scripts executable
chmod +x scripts/*.sh

# Deploy the complete platform
./scripts/wasmbed.sh deploy

# Check status
./scripts/wasmbed.sh status

# Stop all services
./scripts/wasmbed.sh stop
```

## Essential Scripts

### `wasmbed.sh` - Main Management Console
**Unified interface for all platform operations.**

**Usage:**
```bash
./scripts/wasmbed.sh [COMMAND]
```

**Commands:**
- `clean` - Clean up all resources (k3d cluster, processes, build artifacts)
- `build` - Build all Rust components
- `deploy` - Deploy complete platform (k3d cluster + services)
- `stop` - Stop all services gracefully
- `status` - Check system status and health
- `restart` - Stop and restart all services

**Examples:**
```bash
./scripts/wasmbed.sh deploy    # Deploy the complete platform
./scripts/wasmbed.sh status    # Check if everything is running
./scripts/wasmbed.sh stop      # Stop all services
./scripts/wasmbed.sh clean     # Clean everything and start fresh
```

### `wasmbed-deploy.sh` - Platform Deployment
**Deploys the complete Wasmbed platform with minimal initial setup.**

**Features:**
- Creates k3d cluster with proper port mappings
- Applies Kubernetes resources (CRDs, RBAC)
- Starts core services (Infrastructure, Controllers, Dashboard)
- No test resources (gateways/devices deployed via dashboard)
- Provides deployment summary

**Port Mappings:**
- Infrastructure API: 30461
- Dashboard API: 30453
- Dashboard UI: 30470
- Gateway API: 30451 (when deployed via dashboard)

### `wasmbed-build.sh` - Build System
**Builds all Wasmbed components.**

**Features:**
- Builds Rust components with `cargo build --release`
- Generates certificates if needed
- Lists built components
- Error handling and validation

### `wasmbed-clean.sh` - System Cleanup
**Cleans up all Wasmbed resources and processes.**

**Features:**
- Stops all running processes
- Removes k3d clusters
- Cleans Docker resources
- Removes build artifacts
- Cleans certificates and logs

### `wasmbed-stop.sh` - Stop Services
**Stops all Wasmbed services gracefully.**

**Features:**
- Stops services using saved PIDs
- Stops remaining processes by name
- Removes k3d cluster
- Clean shutdown

### `wasmbed-status.sh` - Status Check
**Comprehensive status check of the platform.**

**Features:**
- Checks k3d cluster status
- Verifies Kubernetes resources
- Tests service endpoints
- Shows running processes
- Provides service endpoints

## Service Endpoints

When deployed, the following endpoints are available:

- **Dashboard UI**: http://localhost:30453
- **Dashboard API**: http://localhost:30453/api/v1/*
- **Infrastructure API**: http://localhost:30461
- **Gateway API**: http://localhost:30451 (when deployed via dashboard)

## Deployment Workflow

### 1. Initial Deployment
```bash
# Clean start
./scripts/wasmbed.sh clean

# Deploy platform
./scripts/wasmbed.sh deploy

# Verify deployment
./scripts/wasmbed.sh status
```

### 2. Access Dashboard
```bash
# Open browser to dashboard
open http://localhost:30453
```

### 3. Configure System
- Use "Initial Configuration" wizard in dashboard
- Deploy gateways and devices via dashboard
- Monitor system status

### 4. Development Workflow
```bash
# Make changes to code
# Rebuild
./scripts/wasmbed.sh build

# Restart services
./scripts/wasmbed.sh restart

# Check status
./scripts/wasmbed.sh status
```

## Troubleshooting

### Common Issues

1. **Port conflicts**: Use `./scripts/wasmbed.sh stop` to clean up
2. **Build failures**: Run `./scripts/wasmbed.sh clean` then `./scripts/wasmbed.sh build`
3. **Service not responding**: Check with `./scripts/wasmbed.sh status`

### Debug Steps

1. **Check status**:
   ```bash
   ./scripts/wasmbed.sh status
   ```

2. **Check logs**:
   ```bash
   tail -f logs/infrastructure.log
   tail -f logs/dashboard.log
   ```

3. **Clean restart**:
   ```bash
   ./scripts/wasmbed.sh clean
   ./scripts/wasmbed.sh deploy
   ```

### Service Health Checks

- **Infrastructure**: `curl http://localhost:30461/health`
- **Dashboard**: `curl http://localhost:30453/api/v1/status`
- **Kubernetes**: `kubectl get pods -n wasmbed`

## Architecture Overview

The Wasmbed platform consists of:

1. **Control Plane**: Kubernetes cluster with custom controllers
2. **Infrastructure Service**: Certificate authority, monitoring, logging
3. **Gateway Services**: Device communication and management
4. **Dashboard**: Web-based management interface
5. **Controllers**: Device, Application, and Gateway controllers

## Security

- TLS certificates for secure communication
- Kubernetes RBAC for access control
- Isolated network namespaces
- Secure device enrollment process

## License

All scripts are licensed under AGPL-3.0. See the main project license for details.