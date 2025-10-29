#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

# Wasmbed Platform - Build Script
# This script builds all Wasmbed components

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    local status=$1
    local message=$2
    case $status in
        "SUCCESS")
            echo -e "${GREEN}✓ $message${NC}"
            ;;
        "ERROR")
            echo -e "${RED}✗ $message${NC}"
            ;;
        "WARNING")
            echo -e "${YELLOW}⚠ $message${NC}"
            ;;
        "INFO")
            echo -e "${BLUE}ℹ $message${NC}"
            ;;
    esac
}

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

print_status "INFO" "Starting Wasmbed Platform Build..."

# Check prerequisites
print_status "INFO" "Checking build prerequisites..."

if ! command_exists cargo; then
    print_status "ERROR" "cargo is not installed. Please install Rust toolchain."
    exit 1
fi

if ! command_exists node; then
    print_status "WARNING" "node is not installed. React dashboard will not be built."
    BUILD_DASHBOARD=false
else
    BUILD_DASHBOARD=true
fi

print_status "SUCCESS" "Prerequisites check completed"

# Build Rust components
print_status "INFO" "Building Rust components..."
cargo build --release

if [ $? -ne 0 ]; then
    print_status "ERROR" "Rust build failed"
    exit 1
fi

print_status "SUCCESS" "Rust components built successfully"

# Build React Dashboard
if [ "$BUILD_DASHBOARD" = true ]; then
    print_status "INFO" "Building React Dashboard..."
    
    if [ -d "dashboard-react" ]; then
        cd dashboard-react
        
        # Install dependencies
        print_status "INFO" "Installing React dependencies..."
        npm install
        
        # Build dashboard
        print_status "INFO" "Building React dashboard..."
        npm run build
        
        cd ..
        print_status "SUCCESS" "React Dashboard built successfully"
    else
        print_status "WARNING" "React dashboard directory not found"
    fi
else
    print_status "INFO" "Skipping React Dashboard build (node not available)"
fi

# Install React Dashboard dependencies for development (even if build was skipped)
if [ -d "dashboard-react" ] && [ ! -d "dashboard-react/node_modules" ]; then
    print_status "INFO" "Installing React Dashboard dependencies for development..."
    cd dashboard-react
    npm install
    cd ..
    print_status "SUCCESS" "React Dashboard dependencies installed for development"
fi

# Generate certificates if they don't exist
print_status "INFO" "Checking certificates..."
if [ ! -d "certs" ]; then
    print_status "INFO" "Generating certificates..."
    mkdir -p certs
    
    # Generate CA certificate (v3)
    openssl req -x509 -newkey rsa:4096 -keyout certs/ca-key.pem -out certs/ca-cert.pem -days 365 -nodes -subj "/C=US/ST=State/L=City/O=Wasmbed/CN=Wasmbed-CA"
    
    # Generate server certificate (v3)
    openssl req -newkey rsa:4096 -keyout certs/server-key.pem -out certs/server.csr -nodes -subj "/C=US/ST=State/L=City/O=Wasmbed/CN=localhost"
    # Create a config file for v3 extensions
    cat > certs/server-ext.cnf <<EOF
basicConstraints=CA:FALSE
keyUsage=digitalSignature,keyEncipherment
extendedKeyUsage=serverAuth
subjectAltName=DNS:localhost,IP:127.0.0.1
EOF
    openssl x509 -req -in certs/server.csr -CA certs/ca-cert.pem -CAkey certs/ca-key.pem -out certs/server-cert.pem -days 365 -CAcreateserial -extfile certs/server-ext.cnf
    rm certs/server.csr certs/server-ext.cnf
    
    print_status "SUCCESS" "Certificates generated"
else
    print_status "SUCCESS" "Certificates already exist"
fi

# List built components
print_status "INFO" "Built components:"
echo "  - wasmbed-device-controller"
echo "  - wasmbed-application-controller" 
echo "  - wasmbed-gateway-controller"
echo "  - wasmbed-gateway"
echo "  - wasmbed-infrastructure"
echo "  - wasmbed-dashboard"
echo "  - wasmbed-device-runtime"
echo "  - wasmbed-qemu-manager"
echo "  - wasmbed-qemu-deployment"

if [ "$BUILD_DASHBOARD" = true ]; then
    echo "  - React Dashboard (dashboard-react/build/)"
fi

print_status "SUCCESS" "Wasmbed Platform build completed successfully!"
print_status "INFO" "All components are ready for deployment"
