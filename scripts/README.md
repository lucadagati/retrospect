# Wasmbed Platform Scripts

This directory contains all the scripts needed to deploy, manage, and test the Wasmbed Platform.

## 🚀 Quick Start

### Full CI/CD Pipeline (Recommended for first-time setup)
```bash
# Complete deployment with all tests (ideal for CI/CD)
./scripts/99-ci-cd-pipeline.sh
```

### Quick Deployment (Recommended for development)
```bash
# Fast deployment without extensive testing
./scripts/06-master-control.sh deploy
```

### Manual Step-by-Step Deployment
```bash
# 1. Clean environment
./scripts/01-cleanup-environment.sh

# 2. Build components
./scripts/03-build-components.sh

# 3. Deploy infrastructure
./scripts/04-deploy-infrastructure.sh

# 4. Check system status
./scripts/05-check-system-status.sh

# 5. Setup demo environment
./scripts/08-setup-complete-demo.sh

# 6. Test ARM Cortex-M
./scripts/09-test-arm-cortex-m.sh

# 7. Test all workflows
./scripts/10-test-workflows.sh
```

## 📋 Script Descriptions

### Core Deployment Scripts

| Script | Description | Usage |
|--------|-------------|-------|
| `01-cleanup-environment.sh` | Complete environment cleanup | `./scripts/01-cleanup-environment.sh` |
| `02-fix-kubectl-config.sh` | Fix kubectl configuration | `./scripts/02-fix-kubectl-config.sh` |
| `03-build-components.sh` | Build all Rust and React components | `./scripts/03-build-components.sh` |
| `04-deploy-infrastructure.sh` | Deploy all services and Kubernetes | `./scripts/04-deploy-infrastructure.sh` |
| `05-check-system-status.sh` | Check system health and status | `./scripts/05-check-system-status.sh` |
| `06-stop-services.sh` | Stop all running services | `./scripts/06-stop-services.sh` |
| `07-master-control.sh` | Master control script (clean/build/deploy/status/stop) | `./scripts/07-master-control.sh [command]` |

### Testing Scripts

| Script | Description | Usage |
|--------|-------------|-------|
| `08-setup-complete-demo.sh` | Setup complete demo with Renode devices and WASM apps | `./scripts/08-setup-complete-demo.sh` |
| `09-test-arm-cortex-m.sh` | Test Renode ARM Cortex-M emulation | `./scripts/09-test-arm-cortex-m.sh` |
| `10-test-workflows.sh` | Test all workflows comprehensively | `./scripts/10-test-workflows.sh` |
| `11-test-dashboard.sh` | Test complete dashboard functionality | `./scripts/11-test-dashboard.sh` |
| `12-test-renode-dashboard.sh` | Test Renode-Dashboard integration | `./scripts/12-test-renode-dashboard.sh` |

### Master Scripts

| Script | Description | Usage |
|--------|-------------|-------|
| `99-ci-cd-pipeline.sh` | Complete CI/CD pipeline (clean→build→deploy→test) | `./scripts/99-ci-cd-pipeline.sh` |

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
# Test all workflows (comprehensive)
./scripts/10-test-workflows.sh

# Test Renode ARM Cortex-M emulation
./scripts/09-test-arm-cortex-m.sh

# Test complete dashboard functionality
./scripts/11-test-dashboard.sh

# Test Renode-Dashboard integration
./scripts/12-test-renode-dashboard.sh

# Manual API verification
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
./scripts/01-cleanup-environment.sh

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

1. **Port conflicts**: Run `./scripts/01-cleanup-environment.sh`
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