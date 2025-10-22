# Wasmbed Platform Deployment Guide

This guide provides step-by-step instructions for deploying the Wasmbed Platform, a Kubernetes-native middleware platform for deploying WebAssembly applications to constrained devices using Renode emulation.

## Prerequisites

Before deploying the Wasmbed Platform, ensure you have the following prerequisites installed:

### Required Software

- **Rust**: 1.70+ (for backend services)
- **Kubernetes**: 1.25+ (for orchestration)
- **Node.js**: 18+ (for React dashboard)
- **Kind**: Latest (for local Kubernetes cluster)
- **Docker**: Latest (for containerization)
- **Renode**: 1.15.0+ (for constrained device emulation)

### System Requirements

- **CPU**: 4+ cores recommended
- **RAM**: 8GB+ recommended
- **Storage**: 10GB+ free space
- **OS**: Linux, macOS, or Windows with WSL2

## Quick Deployment

### Automated Deployment

The fastest way to deploy the platform is using the automated deployment script:

```bash
# Clone the repository
git clone https://github.com/lucadagati/retrospect.git
cd retrospect/retrospect

# Run complete deployment
./scripts/99-full-deployment.sh
```

This script will:
1. Clean the environment
2. Build all components
3. Deploy infrastructure
4. Start all services
5. Run comprehensive tests

### Manual Step-by-Step Deployment

For more control over the deployment process, follow these manual steps:

#### Step 1: Environment Setup

```bash
# Clean any existing environment
./scripts/00-cleanup-environment.sh

# Build all components
./scripts/01-build-components.sh
```

#### Step 2: Infrastructure Deployment

```bash
# Deploy core infrastructure
./scripts/02-deploy-infrastructure.sh
```

This step will:
- Create Kind Kubernetes cluster
- Install Custom Resource Definitions (CRDs)
- Generate TLS certificates
- Start core services (Gateway, API Server, Controllers)

#### Step 3: Verification

```bash
# Check system status
./scripts/03-check-system-status.sh

# Test constrained device emulation
./scripts/04-test-arm-cortex-m.sh
```

#### Step 4: Dashboard Access

```bash
# Start React dashboard
cd dashboard-react
npm start
```

Access the dashboard at: http://localhost:3000

## Service Configuration

### Port Configuration

The platform uses the following ports:

| Service | Port | Description |
|---------|------|-------------|
| Dashboard UI | 3000 | React web interface |
| Dashboard API | 3001 | REST API backend |
| Infrastructure API | 30460 | Infrastructure services |
| Gateway HTTP | 8080 | Gateway management API |
| Gateway TLS | 8081 | Device communication (TLS) |

### Environment Variables

Key environment variables for configuration:

```bash
# Kubernetes configuration
export KUBECONFIG=~/.kube/config

# Renode binary path
export RENODE_BINARY=/path/to/renode

# Certificate paths
export CA_CERT_PATH=certs/ca-cert.pem
export SERVER_CERT_PATH=certs/server-cert.pem
export SERVER_KEY_PATH=certs/server-key.pem
```

## Certificate Management

### Automatic Certificate Generation

Certificates are automatically generated during deployment:

```bash
# Certificates are created in the certs/ directory
ls certs/
# ca-cert.pem, ca-cert.der
# server-cert.pem, server-cert.der
# device-cert.pem, device-cert.der
# server-key.pem, device-key.pem
```

### Manual Certificate Generation

If you need to regenerate certificates:

```bash
# Generate CA certificate
openssl req -x509 -newkey rsa:4096 -keyout certs/ca-key.pem -out certs/ca-cert.pem -days 365 -nodes -subj "/C=IT/ST=Italy/L=Italy/O=Wasmbed/OU=Development/CN=Wasmbed CA"

# Generate server certificate
openssl req -newkey rsa:4096 -keyout certs/server-key.pem -out certs/server.csr -nodes -subj "/C=IT/ST=Italy/L=Italy/O=Wasmbed/OU=Development/CN=127.0.0.1"

# Sign server certificate
openssl x509 -req -in certs/server.csr -CA certs/ca-cert.pem -CAkey certs/ca-key.pem -out certs/server-cert.pem -days 365 -CAcreateserial
```

## Device Management

### Supported Device Types

The platform supports the following constrained device types:

- **Arduino Nano 33 BLE** (`RenodeArduinoNano33Ble`)
- **STM32F4 Discovery** (`RenodeStm32F4Discovery`)
- **Arduino Uno R4** (`RenodeArduinoUnoR4`)

### Creating Devices

#### Via API

```bash
# Create a device
curl -X POST http://localhost:3001/api/v1/devices \
  -H "Content-Type: application/json" \
  -d '{
    "id": "my-device",
    "name": "My Arduino Device",
    "architecture": "ARM_CORTEX_M",
    "device_type": "MCU",
    "mcu_type": "RenodeArduinoNano33Ble"
  }'
```

#### Via Dashboard

1. Open http://localhost:3000
2. Navigate to "Device Management"
3. Click "Create Device"
4. Fill in device details
5. Click "Create"

### Starting Devices

```bash
# Start device emulation
curl -X POST http://localhost:3001/api/v1/devices/my-device/renode/start
```

## Application Deployment

### Compiling Rust to WASM

```bash
# Compile Rust code
curl -X POST http://localhost:3001/api/v1/compile \
  -H "Content-Type: application/json" \
  -d '{
    "rust_code": "fn main() { println!(\"Hello from WASM!\"); }"
  }'
```

### Deploying Applications

```bash
# Deploy application to device
curl -X POST http://localhost:3001/api/v1/applications \
  -H "Content-Type: application/json" \
  -d '{
    "id": "hello-app",
    "name": "Hello Application",
    "wasm_binary": "base64-encoded-wasm",
    "target_devices": ["my-device"]
  }'
```

## Monitoring and Management

### System Status

```bash
# Check overall system health
./scripts/03-check-system-status.sh

# Check specific services
curl http://localhost:3001/health
curl http://localhost:8080/health
```

### Log Management

```bash
# View service logs
tail -f logs/gateway.log
tail -f logs/api-server.log
tail -f logs/device-controller.log
tail -f logs/application-controller.log
tail -f logs/gateway-controller.log
```

### Kubernetes Resources

```bash
# Check CRDs
kubectl get crd

# Check resources
kubectl get devices,applications,gateways -n wasmbed

# Check pods
kubectl get pods -n wasmbed
```

## Troubleshooting

### Common Issues

#### Port Conflicts

If you encounter port conflicts:

```bash
# Stop all services
./scripts/05-stop-services.sh

# Clean environment
./scripts/00-cleanup-environment.sh

# Restart deployment
./scripts/02-deploy-infrastructure.sh
```

#### Certificate Issues

If TLS handshake fails:

```bash
# Regenerate certificates
rm -rf certs/*
./scripts/02-deploy-infrastructure.sh
```

#### Renode Issues

If Renode devices don't start:

```bash
# Check Renode installation
which renode

# Test Renode directly
renode --console --execute "mach create; mach LoadPlatformDescription @platforms/boards/arduino_nano_33_ble.repl"
```

### Service Recovery

#### Restart Services

```bash
# Restart all services
./scripts/06-master-control.sh restart

# Restart specific service
pkill -f wasmbed-gateway
nohup cargo run --release -p wasmbed-gateway -- --bind-addr 127.0.0.1:8081 --private-key certs/server-key.pem --certificate certs/server-cert.pem --client-ca certs/ca-cert.pem --namespace wasmbed --pod-namespace wasmbed --pod-name gateway-1 > logs/gateway.log 2>&1 &
```

## Production Deployment

### Security Considerations

1. **Certificate Management**: Use proper CA-signed certificates
2. **Authentication**: Implement JWT or API key authentication
3. **Network Security**: Use proper firewall rules
4. **Resource Limits**: Set appropriate Kubernetes resource limits

### Scaling

#### Horizontal Scaling

```bash
# Scale gateway replicas
kubectl scale deployment wasmbed-gateway --replicas=3 -n wasmbed

# Scale API server replicas
kubectl scale deployment wasmbed-api-server --replicas=2 -n wasmbed
```

#### Vertical Scaling

```yaml
# Update resource limits in deployment manifests
resources:
  requests:
    memory: "512Mi"
    cpu: "250m"
  limits:
    memory: "1Gi"
    cpu: "500m"
```

## Maintenance

### Updates

```bash
# Pull latest changes
git pull origin master

# Rebuild components
./scripts/01-build-components.sh

# Restart services
./scripts/06-master-control.sh restart
```

### Backup

```bash
# Backup device configurations
cp qemu_devices.json qemu_devices.json.backup

# Backup certificates
cp -r certs/ certs.backup/

# Backup Kubernetes resources
kubectl get devices,applications,gateways -n wasmbed -o yaml > k8s-backup.yaml
```

## Support

For additional support:

1. Check the [Troubleshooting Guide](troubleshooting.md)
2. Review [Known Issues](../problems/known-issues.md)
3. Consult the [API Reference](../api/api-reference.md)
4. Check the [Security Overview](../security/security-overview.md)

---

**Last Updated**: 2025  
**Version**: 0.1.0  
**Status**: Production Ready