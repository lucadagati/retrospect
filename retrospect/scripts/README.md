# Wasmbed Platform Scripts

This directory contains all the scripts needed to deploy, manage, and test the Wasmbed Platform.

## 🚀 Quick Start

### Full Deployment (Recommended)
```bash
# Complete deployment with all tests
./scripts/99-full-deployment.sh
```

### Manual Step-by-Step Deployment
```bash
# 1. Clean environment
./scripts/00-cleanup-environment.sh

# 2. Build components
./scripts/01-build-components.sh

# 3. Deploy infrastructure
./scripts/02-deploy-infrastructure.sh

# 4. Check system status
./scripts/03-check-system-status.sh

# 5. Test ARM Cortex-M
./scripts/04-test-arm-cortex-m.sh

# 6. Test all workflows
./scripts/07-test-workflows.sh
```

## 📋 Script Descriptions

### Core Deployment Scripts

| Script | Description | Usage |
|--------|-------------|-------|
| `00-cleanup-environment.sh` | Complete environment cleanup | `./scripts/00-cleanup-environment.sh` |
| `01-build-components.sh` | Build all Rust and React components | `./scripts/01-build-components.sh` |
| `02-deploy-infrastructure.sh` | Deploy all services and Kubernetes | `./scripts/02-deploy-infrastructure.sh` |
| `03-check-system-status.sh` | Check system health and status | `./scripts/03-check-system-status.sh` |
| `05-stop-services.sh` | Stop all running services | `./scripts/05-stop-services.sh` |

### Testing Scripts

| Script | Description | Usage |
|--------|-------------|-------|
| `04-test-arm-cortex-m.sh` | Test Renode ARM Cortex-M emulation | `./scripts/04-test-arm-cortex-m.sh` |
| `07-test-workflows.sh` | Test all workflows comprehensively | `./scripts/07-test-workflows.sh` |
| `08-test-3-workflows.sh` | Test the 3 main workflows only | `./scripts/08-test-3-workflows.sh` |
| `09-test-dashboard.sh` | Test complete dashboard functionality | `./scripts/09-test-dashboard.sh` |
| `10-test-renode-dashboard.sh` | Test Renode-Dashboard integration | `./scripts/10-test-renode-dashboard.sh` |

### Master Scripts

| Script | Description | Usage |
|--------|-------------|-------|
| `99-full-deployment.sh` | Complete deployment sequence | `./scripts/99-full-deployment.sh` |

## 🔄 Workflow Testing

### The 3 Main Workflows

1. **Device Enrollment Workflow**
   - Creates device via Kubernetes CRD
   - Device Controller processes the CRD
   - Device becomes "Enrolled" status
   - API reflects real device state

2. **Application Deployment Workflow**
   - Creates application via Kubernetes CRD
   - Application Controller processes the CRD
   - Gateway receives deployment commands
   - WASM applications deployed to connected devices
   - API reflects real application state

3. **System Monitoring Workflow**
   - Real-time API endpoints provide live data
   - Dashboard React displays current state
   - Controllers continuously reconcile resources
   - Heartbeat and status updates work

### Testing Commands

```bash
# Test all workflows
./scripts/07-test-workflows.sh

# Test only the 3 main workflows
./scripts/08-test-3-workflows.sh

# Test complete dashboard functionality
./scripts/09-test-dashboard.sh

# Manual verification
curl -s http://localhost:3001/api/v1/devices | jq
curl -s http://localhost:3001/api/v1/applications | jq
curl -s http://localhost:3001/api/v1/gateways | jq
```

## 🌐 Service Endpoints

After deployment, these endpoints will be available:

| Service | Endpoint | Description |
|---------|----------|-------------|
| Infrastructure API | http://localhost:30460 | Core infrastructure services |
| API Server (Backend) | http://localhost:3001 | REST API for dashboard |
| Dashboard UI (Frontend) | http://localhost:3000 | React web interface |
| Gateway HTTP API | http://localhost:8080 | Gateway management API |
| Gateway TLS | 127.0.0.1:8081 | Device communication (TLS) |
| Kubernetes API | `kubectl cluster-info` | Kubernetes cluster |

## 🔧 Management Commands

```bash
# Check system status
./scripts/03-check-system-status.sh

# Stop all services
./scripts/05-stop-services.sh

# Complete cleanup
./scripts/00-cleanup-environment.sh

# View logs
tail -f device-controller.log
tail -f application-controller.log
tail -f gateway-controller.log
tail -f gateway.log
tail -f api-server.log
tail -f infrastructure.log
```

## ✅ Verification

### No Mocks Used
All scripts use real data:
- ✅ Real Kubernetes CRDs and controllers
- ✅ Real API endpoints with live data
- ✅ Real device and application states
- ✅ Real-time monitoring and heartbeat data
- ✅ Real Renode ARM Cortex-M emulation
- ✅ Real TLS handshake and communication
- ✅ Real WASM runtime execution

### Manual Verification
```bash
# Check Kubernetes resources
kubectl get devices,applications,gateways -n wasmbed

# Check API responses
curl -s http://localhost:3001/api/v1/devices | jq
curl -s http://localhost:3001/api/v1/applications | jq
curl -s http://localhost:3001/api/v1/gateways | jq

# Check service health
curl -s http://localhost:30460/health
curl -s http://localhost:3001/health

# Access dashboard
open http://localhost:3000
```

## 🚨 Troubleshooting

### Common Issues

1. **Port conflicts**: Run `./scripts/00-cleanup-environment.sh`
2. **Build failures**: Ensure Rust toolchain is installed
3. **Kubernetes issues**: Check k3d installation
4. **API not responding**: Check service logs

### Log Files
- `device-controller.log` - Device controller activity
- `application-controller.log` - Application controller activity
- `gateway-controller.log` - Gateway controller activity
- `api-server.log` - API server activity
- `infrastructure.log` - Infrastructure service activity
- `dashboard.log` - Dashboard React activity

## 📝 Notes

- All scripts are designed to be run from the project root directory
- Scripts include comprehensive error checking and status reporting
- All workflows are tested with real data (no mocks)
- The system is fully functional and ready for production use