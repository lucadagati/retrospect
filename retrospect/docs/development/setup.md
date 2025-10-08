# Development Setup Guide

## Overview

This document provides step-by-step instructions for setting up a complete development environment for the Wasmbed platform.

## System Requirements

### Minimum Requirements

**Hardware**:
- CPU: 4 cores (2.0 GHz)
- RAM: 8GB
- Storage: 50GB free space
- Network: Internet connection

**Software**:
- Linux (Ubuntu 20.04+ recommended)
- macOS (10.15+)
- Windows 10+ (with WSL2)

### Recommended Requirements

**Hardware**:
- CPU: 8 cores (3.0 GHz)
- RAM: 16GB
- Storage: 100GB free space
- Network: Stable internet connection

**Software**:
- Linux (Ubuntu 22.04+)
- macOS (12+)
- Windows 11+ (with WSL2)

## Installation Steps

### 1. Install System Dependencies

#### Ubuntu/Debian

```bash
# Update package list
sudo apt-get update

# Install essential packages
sudo apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    cmake \
    ninja-build \
    clang \
    llvm-dev \
    libclang-dev \
    curl \
    wget \
    git \
    make \
    unzip \
    software-properties-common

# Install QEMU emulators
sudo apt-get install -y \
    qemu-system-riscv32 \
    qemu-system-arm \
    qemu-system-xtensa \
    qemu-utils \
    qemu-user-static

# Install additional tools
sudo apt-get install -y \
    netcat-openbsd \
    jq \
    tree \
    htop \
    vim \
    nano
```

#### macOS

```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install essential packages
brew install \
    cmake \
    ninja \
    llvm \
    curl \
    wget \
    git \
    make \
    unzip \
    jq \
    tree \
    htop \
    vim \
    nano

# Install QEMU
brew install qemu

# Install additional tools
brew install --cask docker
```

#### Windows (WSL2)

```bash
# Install WSL2 if not already installed
wsl --install

# Install Ubuntu in WSL2
wsl --install -d Ubuntu

# Follow Ubuntu installation steps above
```

### 2. Install Rust Toolchain

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install required components
rustup component add rustfmt clippy
rustup target add riscv32imac-unknown-none-elf
rustup target add thumbv7em-none-eabihf
rustup target add xtensa-esp32-espidf

# Install additional Rust tools
cargo install cargo-watch
cargo install cargo-expand
cargo install cargo-audit
cargo install cargo-deny
cargo install cargo-outdated
cargo install cargo-tree

# Verify installation
rustc --version
cargo --version
rustup show
```

### 3. Install Docker

#### Ubuntu/Debian

```bash
# Install Docker
sudo apt-get update
sudo apt-get install -y \
    ca-certificates \
    curl \
    gnupg \
    lsb-release

# Add Docker's official GPG key
sudo mkdir -p /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /etc/apt/keyrings/docker.gpg

# Set up repository
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# Install Docker Engine
sudo apt-get update
sudo apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# Start Docker service
sudo systemctl start docker
sudo systemctl enable docker

# Add user to docker group
sudo usermod -aG docker $USER
newgrp docker

# Verify installation
docker --version
docker compose version
```

#### macOS

```bash
# Install Docker Desktop
brew install --cask docker

# Start Docker Desktop
open /Applications/Docker.app

# Verify installation
docker --version
docker compose version
```

#### Windows

```bash
# Install Docker Desktop
# Download from https://www.docker.com/products/docker-desktop
# Follow installation wizard

# Verify installation
docker --version
docker compose version
```

### 4. Install Kubernetes (k3d)

```bash
# Install k3d
curl -s https://raw.githubusercontent.com/k3d-io/k3d/main/install.sh | bash

# Verify installation
k3d version

# Test k3d
k3d cluster create test --port "8080:80@loadbalancer"
k3d cluster delete test
```

### 5. Install Additional Development Tools

#### Git Configuration

```bash
# Configure Git
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
git config --global init.defaultBranch main
git config --global pull.rebase false
```

#### VS Code (Optional)

```bash
# Install VS Code
# Download from https://code.visualstudio.com/

# Install Rust extension
code --install-extension rust-lang.rust-analyzer
code --install-extension tamasfe.even-better-toml
code --install-extension serayuzgur.crates
code --install-extension vadimcn.vscode-lldb
```

#### Additional Tools

```bash
# Install additional development tools
sudo apt-get install -y \
    tmux \
    screen \
    rsync \
    ssh \
    openssh-client \
    openssh-server

# Install network tools
sudo apt-get install -y \
    netcat-openbsd \
    tcpdump \
    wireshark \
    nmap
```

## Project Setup

### 1. Clone Repository

```bash
# Clone the repository
git clone <repository-url>
cd retrospect

# Verify project structure
ls -la
tree -L 2
```

### 2. Build Project

```bash
# Build all components
cargo build --release

# Build specific components
cargo build --release -p wasmbed-gateway
cargo build --release -p wasmbed-k8s-controller
cargo build --release -p wasmbed-qemu-serial-bridge

# Build firmware components
cargo build --release --target riscv32imac-unknown-none-elf
cargo build --release --target thumbv7em-none-eabihf
cargo build --release --target xtensa-esp32-espidf
```

### 3. Run Tests

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test -p wasmbed-gateway
cargo test -p wasmbed-k8s-controller
cargo test -p wasmbed-qemu-serial-bridge

# Run tests with output
cargo test -- --nocapture

# Run tests in specific module
cargo test module_name
```

### 4. Setup Development Scripts

```bash
# Make scripts executable
chmod +x scripts/*.sh

# Test scripts
./scripts/deploy.sh --help
./scripts/app.sh --help
./scripts/monitor.sh --help
./scripts/cleanup.sh --help
```

### 5. Setup Pre-commit Hooks

```bash
# Install pre-commit
pip install pre-commit

# Setup pre-commit hooks
pre-commit install

# Test pre-commit hooks
pre-commit run --all-files
```

## Development Environment Configuration

### 1. Environment Variables

```bash
# Create environment file
cat > .env << EOF
# Development environment variables
RUST_LOG=debug
RUST_BACKTRACE=1
CARGO_TARGET_DIR=target
WASMBED_DEV_MODE=true
WASMBED_LOG_LEVEL=debug
EOF

# Source environment variables
source .env
```

### 2. Cargo Configuration

```bash
# Create Cargo configuration
mkdir -p .cargo
cat > .cargo/config.toml << EOF
[build]
target-dir = "target"

[target.riscv32imac-unknown-none-elf]
runner = "qemu-system-riscv32 -machine sifive_u -smp 2 -m 128M -nographic -kernel"

[target.thumbv7em-none-eabihf]
runner = "qemu-system-arm -machine stm32-p103 -smp 1 -m 64M -nographic -kernel"

[target.xtensa-esp32-espidf]
runner = "qemu-system-xtensa -machine esp32 -smp 1 -m 4M -nographic -kernel"
EOF
```

### 3. VS Code Configuration

```bash
# Create VS Code configuration
mkdir -p .vscode
cat > .vscode/settings.json << EOF
{
    "rust-analyzer.cargo.buildScripts.enable": true,
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.checkOnSave.extraArgs": ["--", "-D", "warnings"],
    "rust-analyzer.rustfmt.extraArgs": ["--edition", "2021"],
    "editor.formatOnSave": true,
    "editor.codeActionsOnSave": {
        "source.fixAll": true,
        "source.organizeImports": true
    }
}
EOF
```

### 4. Git Configuration

```bash
# Create Git configuration
cat > .gitignore << EOF
# Rust
target/
Cargo.lock
*.pem
*.key
*.crt

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log
logs/

# Temporary files
tmp/
temp/
EOF
```

## Development Workflow

### 1. Daily Development Setup

```bash
# Start development session
cd retrospect

# Pull latest changes
git pull origin main

# Build project
cargo build

# Run tests
cargo test

# Start development server
cargo run --bin wasmbed-gateway
```

### 2. Feature Development

```bash
# Create feature branch
git checkout -b feature/your-feature-name

# Make changes
# ... edit files ...

# Test changes
cargo test
cargo clippy
cargo fmt

# Commit changes
git add .
git commit -m "Add your feature description"

# Push changes
git push origin feature/your-feature-name
```

### 3. Debugging

```bash
# Debug with gdb
cargo build --release
gdb target/release/wasmbed-gateway

# Debug with lldb (macOS)
cargo build --release
lldb target/release/wasmbed-gateway

# Debug with VS Code
# Use VS Code debugger with launch.json configuration
```

### 4. Performance Profiling

```bash
# Install profiling tools
cargo install cargo-flamegraph
cargo install cargo-profdata

# Profile application
cargo flamegraph --bin wasmbed-gateway

# Profile with perf
perf record --call-graph dwarf cargo run --release --bin wasmbed-gateway
perf report
```

## Testing Environment

### 1. Unit Testing

```bash
# Run unit tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_function_name

# Run tests in specific module
cargo test module_name
```

### 2. Integration Testing

```bash
# Run integration tests
cargo test --test integration

# Run specific integration test
cargo test --test integration test_name
```

### 3. End-to-End Testing

```bash
# Deploy test environment
./scripts/deploy.sh

# Run end-to-end tests
./scripts/app.sh test

# Clean up test environment
./scripts/cleanup.sh
```

### 4. QEMU Testing

```bash
# Test QEMU emulation
./scripts/app.sh qemu

# Test QEMU communication
./scripts/app.sh qemu-comm

# Start QEMU devices
./scripts/app.sh start-qemu

# Stop QEMU devices
./scripts/app.sh stop-qemu
```

## Troubleshooting

### Common Issues

#### 1. Rust Installation Issues

**Problem**: Rust not found in PATH
**Solution**:
```bash
source ~/.cargo/env
echo 'source ~/.cargo/env' >> ~/.bashrc
```

#### 2. Docker Permission Issues

**Problem**: Permission denied for Docker
**Solution**:
```bash
sudo usermod -aG docker $USER
newgrp docker
```

#### 3. QEMU Installation Issues

**Problem**: QEMU not found
**Solution**:
```bash
sudo apt-get update
sudo apt-get install -y qemu-system-riscv32 qemu-system-arm qemu-system-xtensa
```

#### 4. Build Issues

**Problem**: Build failures
**Solution**:
```bash
cargo clean
cargo build --release
```

### Development Tools Issues

#### 1. VS Code Rust Extension

**Problem**: Rust analyzer not working
**Solution**:
```bash
# Reinstall Rust extension
code --uninstall-extension rust-lang.rust-analyzer
code --install-extension rust-lang.rust-analyzer
```

#### 2. Pre-commit Hooks

**Problem**: Pre-commit hooks failing
**Solution**:
```bash
pre-commit uninstall
pre-commit install
pre-commit run --all-files
```

### Performance Issues

#### 1. Slow Build Times

**Solution**:
```bash
# Use sccache for faster builds
cargo install sccache
export RUSTC_WRAPPER=sccache
```

#### 2. Memory Issues

**Solution**:
```bash
# Increase swap space
sudo fallocate -l 4G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

## Maintenance

### Regular Maintenance

#### 1. Update Dependencies

```bash
# Update Rust toolchain
rustup update

# Update Cargo packages
cargo update

# Check for outdated packages
cargo outdated
```

#### 2. Clean Build Artifacts

```bash
# Clean build artifacts
cargo clean

# Clean specific target
cargo clean --target riscv32imac-unknown-none-elf
```

#### 3. Security Audit

```bash
# Run security audit
cargo audit

# Check for vulnerabilities
cargo deny check
```

### Backup and Recovery

#### 1. Backup Development Environment

```bash
# Backup configuration files
tar -czf dev-env-backup.tar.gz \
    .cargo/ \
    .vscode/ \
    .env \
    .gitignore \
    Cargo.toml \
    Cargo.lock
```

#### 2. Restore Development Environment

```bash
# Restore configuration files
tar -xzf dev-env-backup.tar.gz
```
