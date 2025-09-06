# Development Setup Guide

##  Overview

This guide provides detailed instructions for setting up a development environment for Wasmbed, including all necessary tools, dependencies, and configuration.

##  Prerequisites

### System Requirements

- **Operating System**: Linux (Ubuntu 20.04+), macOS (10.15+), or Windows (WSL2)
- **CPU**: 2+ cores recommended
- **Memory**: 8GB+ RAM recommended
- **Storage**: 20GB+ free space
- **Network**: Internet connectivity for downloads

### Required Software

#### Core Dependencies

```bash
# Package Manager
- Nix 2.8+ (for reproducible development environment)

# Container Runtime
- Docker 20.10+ (for containerization)

# Kubernetes Tools
- kubectl 1.28+ (Kubernetes CLI)
- k3d 5.4+ (local Kubernetes cluster)

# Emulation
- QEMU 6.0+ (for MCU emulation)

# Build Tools
- Rust 1.88+ (compiler and toolchain)
- Cargo (package manager)
```

#### Optional Dependencies

```bash
# Development Tools
- VS Code with Rust extension
- Git (version control)
- Make (build automation)

# Monitoring
- Prometheus (metrics collection)
- Grafana (dashboards)

# Testing
- k6 (load testing)
- Postman (API testing)
```

##  Installation Steps

### 1. Install Nix

```bash
# Install Nix package manager
curl -s https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash

# Verify installation
nix --version

# Enable flakes (if not already enabled)
mkdir -p ~/.config/nix
echo "experimental-features = nix-command flakes" >> ~/.config/nix/nix.conf
```

### 2. Install Docker

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install docker.io
sudo systemctl start docker
sudo systemctl enable docker
sudo usermod -aG docker $USER

# macOS
brew install --cask docker

# Verify installation
docker --version
docker run hello-world
```

### 3. Install k3d

```bash
# Install k3d
curl -s https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash

# Verify installation
k3d --version
```

### 4. Install QEMU

```bash
# Ubuntu/Debian
sudo apt-get install qemu-system

# macOS
brew install qemu

# Verify installation
qemu-system-riscv32 --version
```

##  Project Setup

### 1. Clone Repository

```bash
# Clone Wasmbed repository
git clone https://github.com/your-org/wasmbed.git
cd wasmbed

# Verify repository
ls -la
```

### 2. Enter Development Environment

```bash
# Enter Nix development environment
nix develop

# Verify environment
rustc --version
cargo --version
kubectl version --client
```

### 3. Build Project

```bash
# Build all crates
cargo build

# Build specific crate
cargo build -p wasmbed-gateway
cargo build -p wasmbed-k8s-controller
cargo build -p wasmbed-firmware-hifive1-qemu

# Build with optimizations
cargo build --release
```

### 4. Run Tests

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p wasmbed-protocol
cargo test -p wasmbed-k8s-resource

# Run integration tests
cargo test --test integration
```

##  Development Workflow

### 1. Code Organization

```
wasmbed/
├── crates/                          # Rust crates
│   ├── wasmbed-gateway/            # Gateway implementation
│   ├── wasmbed-k8s-controller/     # Kubernetes controller
│   ├── wasmbed-k8s-resource/       # CRD definitions
│   ├── wasmbed-protocol/           # Communication protocol
│   ├── wasmbed-firmware-hifive1-qemu/ # MCU firmware
│   └── wasmbed-cert-tool/          # Certificate generation
├── resources/                       # Configuration files
│   ├── k8s/                        # Kubernetes manifests
│   ├── k3d/                        # k3d configuration
│   └── dev-certs/                  # Development certificates
├── scripts/                         # Automation scripts
├── docs/                           # Documentation
└── tests/                          # Integration tests
```

### 2. Development Commands

```bash
# Start development environment
./scripts/setup.sh

# Run complete system test
./scripts/test.sh

# Clean up environment
./scripts/cleanup.sh

# Generate certificates
./scripts/generate-certs.sh

# Deploy to local cluster
./scripts/deploy.sh
```

### 3. Code Quality

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check for security issues
cargo audit

# Run benchmarks
cargo bench
```

##  Testing

### 1. Unit Tests

```bash
# Run unit tests for specific crate
cargo test -p wasmbed-protocol

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_enrollment_flow
```

### 2. Integration Tests

```bash
# Run integration tests
cargo test --test integration

# Run with specific features
cargo test --features integration-tests

# Run with custom environment
RUST_LOG=debug cargo test --test integration
```

### 3. End-to-End Tests

```bash
# Start test environment
./scripts/setup.sh

# Run end-to-end tests
./scripts/test-e2e.sh

# Clean up
./scripts/cleanup.sh
```

##  Debugging

### 1. Logging

```bash
# Enable debug logging
export RUST_LOG=debug

# Enable specific module logging
export RUST_LOG=wasmbed_gateway=debug,wasmbed_protocol=trace

# Run with logging
cargo run -p wasmbed-gateway
```

### 2. Debugging Tools

```bash
# Use GDB for debugging
gdb target/debug/wasmbed-gateway

# Use LLDB (macOS)
lldb target/debug/wasmbed-gateway

# Debug with VS Code
# Add launch.json configuration
```

### 3. Profiling

```bash
# Profile with perf
perf record -g cargo run --release
perf report

# Profile with flamegraph
cargo install flamegraph
cargo flamegraph
```

##  Performance Testing

### 1. Load Testing

```bash
# Install k6
curl -L https://github.com/grafana/k6/releases/download/v0.45.0/k6-v0.45.0-linux-amd64.tar.gz | tar xz
sudo cp k6-v0.45.0-linux-amd64/k6 /usr/local/bin/

# Run load test
k6 run tests/load-test.js
```

### 2. Benchmarking

```bash
# Run benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench protocol_benchmarks

# Compare benchmarks
cargo bench --bench compare
```

##  Security

### 1. Certificate Management

```bash
# Generate development certificates
cargo run -p wasmbed-cert-tool -- generate-ca \
  --common-name "Wasmbed Dev CA" \
  --out-key resources/dev-certs/ca.key \
  --out-cert resources/dev-certs/ca.der

# Generate device certificates
cargo run -p wasmbed-cert-tool -- issue-cert \
  --ca-key resources/dev-certs/ca.key \
  --ca-cert resources/dev-certs/ca.der \
  --common-name "test-device" \
  --out-key resources/dev-certs/device.key \
  --out-cert resources/dev-certs/device.der \
  client
```

### 2. Security Scanning

```bash
# Run security audit
cargo audit

# Scan for vulnerabilities
cargo install cargo-audit
cargo audit

# Check dependencies
cargo tree
cargo outdated
```

##  Deployment

### 1. Local Development

```bash
# Start local cluster
k3d cluster create wasmbed --config resources/k3d/config.yaml

# Deploy Wasmbed
kubectl apply -f resources/k8s/

# Verify deployment
kubectl get all -n wasmbed
```

### 2. Production Build

```bash
# Build production images
docker build -f crates/wasmbed-gateway/Dockerfile -t wasmbed-gateway:latest .
docker build -f crates/wasmbed-k8s-controller/Dockerfile -t wasmbed-k8s-controller:latest .

# Push to registry
docker push your-registry/wasmbed-gateway:latest
docker push your-registry/wasmbed-k8s-controller:latest
```

##  Best Practices

### 1. Code Style

```rust
// Use consistent formatting
cargo fmt

// Follow Rust conventions
cargo clippy

// Document public APIs
/// This function does something important
pub fn important_function() -> Result<(), Error> {
    // Implementation
}
```

### 2. Error Handling

```rust
// Use proper error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WasmbedError {
    #[error("Network error: {0}")]
    Network(#[from] std::io::Error),
    #[error("Protocol error: {0}")]
    Protocol(String),
}
```

### 3. Testing

```rust
// Write comprehensive tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_functionality() {
        // Test implementation
        assert!(true);
    }

    #[tokio::test]
    async fn test_async_functionality() {
        // Async test implementation
        assert!(true);
    }
}
```

##  IDE Configuration

### 1. VS Code

```json
// .vscode/settings.json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.buildScripts.enable": true,
    "rust-analyzer.procMacro.enable": true,
    "editor.formatOnSave": true,
    "files.associations": {
        "*.x": "linkerscript"
    }
}
```

### 2. IntelliJ IDEA

```xml
<!-- .idea/workspace.xml -->
<component name="RustProjectSettings">
    <option name="autoReloadCargoProject" value="true" />
    <option name="useCargoCheckOnSave" value="true" />
</component>
```

## 📞 Support

### Getting Help

- **Documentation**: Check the `docs/` directory
- **Issues**: [GitHub Issues](https://github.com/your-org/wasmbed/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/wasmbed/discussions)
- **Wiki**: [Documentation Wiki](https://github.com/your-org/wasmbed/wiki)

### Common Issues

```bash
# Nix environment issues
nix develop --command bash

# Docker permission issues
sudo usermod -aG docker $USER
newgrp docker

# k3d cluster issues
k3d cluster delete wasmbed
k3d cluster create wasmbed
```

---

**Last Updated**: September 2024  
**Version**: Development Setup v1.0.0  
**Maintainer**: Wasmbed Development Team
