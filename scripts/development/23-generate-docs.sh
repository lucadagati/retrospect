#!/bin/bash
# Generate documentation for Wasmbed platform
# This script generates comprehensive documentation

set -e

echo "📚 Generating Wasmbed documentation..."

# Check if cargo is available
if ! command -v cargo >/dev/null 2>&1; then
    echo "❌ Cargo not found"
    echo "Please install Rust and Cargo"
    exit 1
fi

echo "✅ Cargo is available"

# Create docs directory
mkdir -p docs

# Generate Rust documentation
echo "📋 Generating Rust documentation..."
cargo doc --no-deps --all-features --open

if [ $? -eq 0 ]; then
    echo "✅ Rust documentation generated"
else
    echo "❌ Rust documentation generation failed"
    exit 1
fi

# Generate API documentation
echo "📋 Generating API documentation..."
cargo doc --document-private-items --no-deps --all-features

if [ $? -eq 0 ]; then
    echo "✅ API documentation generated"
else
    echo "❌ API documentation generation failed"
    exit 1
fi

# Generate project documentation
echo "📋 Generating project documentation..."

# Create comprehensive README
cat > docs/README.md << 'EOF'
# Wasmbed Platform Documentation

## Overview
Wasmbed is an IoT device management platform using Kubernetes and WebAssembly.

## Architecture
- **Kubernetes Control Plane**: Orchestration and management
- **Gateway MPU**: Proxy between Kubernetes and MCUs
- **MCU Firmware**: WASM runtime for edge devices

## Components
- `wasmbed-gateway`: Gateway service
- `wasmbed-k8s-controller`: Kubernetes controller
- `wasmbed-protocol`: Communication protocol
- `wasmbed-firmware-hifive1-qemu`: MCU firmware

## Usage
```bash
./wasmbed.sh setup      # First time setup
./wasmbed.sh deploy     # Deploy to Kubernetes
./wasmbed.sh test       # Run tests
./wasmbed.sh status     # Check status
```

## Development
```bash
./wasmbed.sh dev        # Development environment
./wasmbed.sh build      # Build all components
./wasmbed.sh lint       # Run linting
```

## Security
```bash
./wasmbed.sh security-scan        # Security validation
./wasmbed.sh security-hardening   # Apply hardening
```

## Operations
```bash
./wasmbed.sh monitor              # Start monitoring
./wasmbed.sh backup               # Create backup
./wasmbed.sh disaster-recovery    # Recovery procedures
```
EOF

echo "✅ Project documentation generated"

# Generate component documentation
echo "📋 Generating component documentation..."

# Gateway documentation
cat > docs/gateway.md << 'EOF'
# Wasmbed Gateway

## Purpose
The Gateway acts as a proxy between Kubernetes and MCU devices.

## Features
- TLS-secured communication
- Device management
- Application deployment
- Heartbeat monitoring

## Configuration
- Port: 4423
- TLS certificates required
- Kubernetes integration

## Deployment
```bash
./wasmbed.sh deploy
```
EOF

# Controller documentation
cat > docs/controller.md << 'EOF'
# Wasmbed Kubernetes Controller

## Purpose
Manages Device and Application CRDs in Kubernetes.

## Features
- Continuous reconciliation
- Status updates
- Event recording
- Health endpoints

## CRDs
- `Device`: MCU device registration
- `Application`: WASM application deployment

## Health Endpoints
- `/health`: Liveness probe
- `/ready`: Readiness probe
- `/metrics`: Prometheus metrics
EOF

# Protocol documentation
cat > docs/protocol.md << 'EOF'
# Wasmbed Protocol

## Overview
CBOR-based communication protocol between Gateway and MCUs.

## Message Types
- Enrollment: Device registration
- Heartbeat: Connection monitoring
- Application: WASM deployment
- Status: Health reporting

## Security
- TLS 1.3 encryption
- Mutual authentication
- Certificate-based identity
EOF

echo "✅ Component documentation generated"

# Generate deployment guide
echo "📋 Generating deployment guide..."
cat > docs/deployment.md << 'EOF'
# Deployment Guide

## Prerequisites
- Kubernetes cluster (k3d, minikube, etc.)
- Docker
- Rust toolchain

## Quick Start
```bash
# Setup
./wasmbed.sh setup

# Deploy
./wasmbed.sh deploy

# Test
./wasmbed.sh test
```

## Production Deployment
```bash
# Security hardening
./wasmbed.sh security-hardening

# Monitoring
./wasmbed.sh monitor

# Backup
./wasmbed.sh backup
```

## Troubleshooting
```bash
# Check status
./wasmbed.sh status

# Health checks
./wasmbed.sh health-check

# View logs
./wasmbed.sh logs
```
EOF

echo "✅ Deployment guide generated"

# Generate API reference
echo "📋 Generating API reference..."
cat > docs/api-reference.md << 'EOF'
# API Reference

## Device CRD
```yaml
apiVersion: wasmbed.github.io/v0
kind: Device
metadata:
  name: my-device
spec:
  publicKey: "base64-encoded-key"
```

## Application CRD
```yaml
apiVersion: wasmbed.github.io/v1alpha1
kind: Application
metadata:
  name: my-app
spec:
  name: "My App"
  wasm_bytes: "base64-encoded-wasm"
  target_devices:
    device_names: ["my-device"]
```

## Gateway API
- `POST /enroll`: Device enrollment
- `POST /heartbeat`: Heartbeat update
- `POST /deploy`: Application deployment
- `GET /status`: Device status
EOF

echo "✅ API reference generated"

echo ""
echo "🎉 Documentation generation completed!"
echo ""
echo "📊 Documentation Summary:"
echo "========================="
echo "✅ Rust documentation: Generated"
echo "✅ API documentation: Generated"
echo "✅ Project documentation: Generated"
echo "✅ Component documentation: Generated"
echo "✅ Deployment guide: Generated"
echo "✅ API reference: Generated"
echo ""
echo "📁 Documentation location:"
echo "  - Rust docs: target/doc/"
echo "  - Project docs: docs/"
echo ""
echo "Next steps:"
echo "  open target/doc/index.html              # View Rust documentation"
echo "  ./wasmbed.sh test                       # Run tests"
echo "  ./wasmbed.sh deploy                     # Deploy platform"

