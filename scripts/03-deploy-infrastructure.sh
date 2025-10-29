#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

# Wasmbed Platform - Deploy Script
# This script deploys the complete Wasmbed platform

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

print_status "INFO" "Starting Wasmbed Platform Deployment..."

# Build Docker images
print_status "INFO" "Building Docker images..."
if [ -f "Dockerfile.gateway" ]; then
    print_status "INFO" "Building wasmbed/gateway:latest..."
    docker build -f Dockerfile.gateway -t wasmbed/gateway:latest . || {
        print_status "WARNING" "Failed to build wasmbed/gateway:latest"
    }
else
    print_status "WARNING" "Dockerfile.gateway not found, skipping gateway image build"
fi

if [ -f "Dockerfile.gateway-controller" ]; then
    print_status "INFO" "Building wasmbed/gateway-controller:latest..."
    docker build -f Dockerfile.gateway-controller -t wasmbed/gateway-controller:latest . || {
        print_status "WARNING" "Failed to build wasmbed/gateway-controller:latest"
    }
else
    print_status "WARNING" "Dockerfile.gateway-controller not found, skipping gateway-controller image build"
fi

# Check prerequisites
print_status "INFO" "Checking deployment prerequisites..."

if ! command -v kind >/dev/null 2>&1; then
    print_status "ERROR" "kind is not installed"
    exit 1
fi

if ! command -v kubectl >/dev/null 2>&1; then
    print_status "ERROR" "kubectl is not installed"
    exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
    print_status "ERROR" "cargo is not installed"
    exit 1
fi

print_status "SUCCESS" "All prerequisites are available"

# Clean up any existing processes
print_status "INFO" "Cleaning up existing processes..."
pkill -f "wasmbed-" 2>/dev/null || true
# Port cleanup already done by cleanup script
sleep 2

# Create logs directory
print_status "INFO" "Creating logs directory..."
mkdir -p logs

# Build all components
print_status "INFO" "Building all components..."
cargo build --release

if [ $? -ne 0 ]; then
    print_status "ERROR" "Build failed"
    exit 1
fi

print_status "SUCCESS" "All components built successfully"

# Check if Kind cluster exists, create if not
print_status "INFO" "Checking Kind cluster..."
if ! kind get clusters | grep -q "wasmbed"; then
    print_status "INFO" "Creating Kind cluster..."
    kind create cluster --name wasmbed
    print_status "SUCCESS" "Kind cluster created successfully"
else
    print_status "SUCCESS" "Kind cluster 'wasmbed' already exists"
fi

# Import images to Kind cluster (AFTER cluster creation)
print_status "INFO" "Importing Docker images to Kind cluster..."
kind load docker-image wasmbed/gateway:latest --name wasmbed 2>/dev/null || {
    print_status "WARNING" "Failed to import wasmbed/gateway:latest to Kind"
}
kind load docker-image wasmbed/gateway-controller:latest --name wasmbed 2>/dev/null || {
    print_status "WARNING" "Failed to import wasmbed/gateway-controller:latest to Kind"
}

# Configure kubectl context
print_status "INFO" "Configuring kubectl context..."
kubectl config use-context kind-wasmbed
print_status "SUCCESS" "kubectl context configured"

# Setup Kubernetes resources
print_status "INFO" "Setting up Kubernetes resources..."
kubectl create namespace wasmbed 2>/dev/null || {
    print_status "INFO" "Namespace wasmbed already exists, continuing..."
}
kubectl apply -f k8s/crds/
kubectl apply -f k8s/rbac/

# Wait for CRDs
kubectl wait --for condition=established --timeout=60s crd/devices.wasmbed.github.io
kubectl wait --for condition=established --timeout=60s crd/applications.wasmbed.github.io
kubectl wait --for condition=established --timeout=60s crd/gateways.wasmbed.io

print_status "SUCCESS" "Kubernetes resources created successfully"

# Verify kubectl connection
print_status "INFO" "Verifying kubectl connection..."
if kubectl cluster-info >/dev/null 2>&1; then
    print_status "SUCCESS" "kubectl connection verified"
else
    print_status "ERROR" "kubectl connection failed"
    exit 1
fi

# Skip test resources - will be created via dashboard
print_status "INFO" "Skipping test resources - will be created via dashboard configuration"

# Start services
print_status "INFO" "Starting all services..."

# Start Infrastructure (non-blocking)
print_status "INFO" "Starting Infrastructure Service..."
nohup ./target/release/wasmbed-infrastructure --port 30460 > infrastructure.log 2>&1 &
INFRASTRUCTURE_PID=$!
echo $INFRASTRUCTURE_PID > .infrastructure.pid
disown
sleep 3

# Verify Infrastructure is running
if ! kill -0 $INFRASTRUCTURE_PID 2>/dev/null; then
    print_status "ERROR" "Infrastructure service failed to start"
    exit 1
fi
print_status "SUCCESS" "Infrastructure Service started (PID: $INFRASTRUCTURE_PID)"

# Generate TLS certificates if they don't exist
print_status "INFO" "Checking TLS certificates..."
if [ ! -d "certs" ] || [ ! -f "certs/ca-cert.pem" ] || [ ! -f "certs/server-cert.pem" ] || [ ! -f "certs/server-key.pem" ]; then
    print_status "INFO" "Generating TLS certificates..."
    mkdir -p certs
    
    # Generate CA certificate
    openssl genrsa -out certs/ca-key.pem 2048
    cat > certs/ca.conf << 'EOF'
[ ca ]
default_ca = CA_default

[ CA_default ]
dir = ./certs
certs = $dir
crl_dir = $dir/crl
database = $dir/index.txt
new_certs_dir = $dir/newcerts
serial = $dir/serial
crlnumber = $dir/crlnumber
crl = $dir/crl.pem
private_key = $dir/ca-key.pem
certificate = $dir/ca-cert.pem
RANDFILE = $dir/.rand
x509_extensions = v3_ca
name_opt = ca_default
cert_opt = ca_default
default_days = 365
default_crl_days = 30
default_md = sha256
preserve = no
policy = policy_strict

[ policy_strict ]
countryName = match
stateOrProvinceName = match
organizationName = match
organizationalUnitName = optional
commonName = supplied
emailAddress = optional

[ usr_cert ]
basicConstraints = CA:FALSE
nsCertType = client, email
nsComment = "OpenSSL Generated Certificate"
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid,issuer
keyUsage = critical, nonRepudiation, digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth, clientAuth

[ v3_ca ]
basicConstraints = critical,CA:TRUE
keyUsage = critical, keyCertSign, cRLSign
subjectKeyIdentifier = hash
authorityKeyIdentifier = keyid:always,issuer
EOF
    openssl req -new -x509 -key certs/ca-key.pem -out certs/ca-cert.pem -days 365 -subj "/C=IT/ST=Italy/L=Italy/O=Wasmbed/OU=Development/CN=Wasmbed CA" -extensions v3_ca -config certs/ca.conf
    
    # Generate server certificate
    openssl genrsa -out certs/server-key.pem 2048
    openssl req -new -key certs/server-key.pem -out certs/server.csr -subj "/C=IT/ST=Italy/L=Italy/O=Wasmbed/OU=Development/CN=localhost"
    touch certs/index.txt && echo 1000 > certs/serial && mkdir -p certs/newcerts
    openssl ca -config certs/ca.conf -in certs/server.csr -out certs/server-cert.pem -days 365 -extensions usr_cert -batch
    rm certs/server.csr
    
    print_status "SUCCESS" "TLS certificates generated successfully"
else
    print_status "SUCCESS" "TLS certificates already exist"
fi

# Create Kubernetes secret for certificates
print_status "INFO" "Creating Kubernetes secret for certificates..."
kubectl create secret generic gateway-certificates \
    --from-file=ca-cert.pem=certs/ca-cert.pem \
    --from-file=ca-key.pem=certs/ca-key.pem \
    --from-file=server-cert.pem=certs/server-cert.pem \
    --from-file=server-key.pem=certs/server-key.pem \
    -n wasmbed 2>/dev/null || {
    print_status "WARNING" "Certificate secret already exists or failed to create"
}

# Start Gateway (non-blocking)
print_status "INFO" "Starting Gateway..."
nohup ./target/release/wasmbed-gateway \
    --bind-addr 127.0.0.1:8081 \
    --http-addr 127.0.0.1:8080 \
    --private-key certs/server-key.pem \
    --certificate certs/server-cert.pem \
    --client-ca certs/ca-cert.pem \
    --namespace wasmbed \
    --pod-namespace wasmbed \
    --pod-name gateway-1 > logs/gateway.log 2>&1 &
GATEWAY_PID=$!
echo $GATEWAY_PID > .gateway.pid
disown
sleep 3

# Verify Gateway is running
if ! kill -0 $GATEWAY_PID 2>/dev/null; then
    print_status "ERROR" "Gateway service failed to start"
    exit 1
fi
print_status "SUCCESS" "Gateway started (PID: $GATEWAY_PID)"

# Start Controllers (non-blocking)
print_status "INFO" "Starting Controllers..."
nohup ./target/release/wasmbed-device-controller > device-controller.log 2>&1 &
DEVICE_CONTROLLER_PID=$!
echo $DEVICE_CONTROLLER_PID > .device-controller.pid
disown

nohup ./target/release/wasmbed-application-controller > application-controller.log 2>&1 &
APPLICATION_CONTROLLER_PID=$!
echo $APPLICATION_CONTROLLER_PID > .application-controller.pid
disown

nohup ./target/release/wasmbed-gateway-controller > gateway-controller.log 2>&1 &
GATEWAY_CONTROLLER_PID=$!
echo $GATEWAY_CONTROLLER_PID > .gateway-controller.pid
disown

print_status "SUCCESS" "Controllers started successfully"

# Start API Server (non-blocking)
print_status "INFO" "Starting API Server..."
nohup ./target/release/wasmbed-api-server \
            --port 3001 \
            --gateway-endpoint http://localhost:8080 \
            --infrastructure-endpoint http://localhost:30460 > api-server.log 2>&1 &
API_SERVER_PID=$!
echo $API_SERVER_PID > .api-server.pid
disown
sleep 3

# Verify API Server is running
if ! kill -0 $API_SERVER_PID 2>/dev/null; then
    print_status "ERROR" "API Server failed to start"
    exit 1
fi
print_status "SUCCESS" "API Server started (PID: $API_SERVER_PID)"

# Wait for API Server to initialize and verify Kubernetes connection
print_status "INFO" "Waiting for API Server to initialize..."
sleep 5

# Test API Server Kubernetes connection
print_status "INFO" "Testing API Server Kubernetes connection..."
if curl -4 -s http://localhost:3001/api/v1/devices >/dev/null 2>&1; then
    print_status "SUCCESS" "API Server connected to Kubernetes successfully"
else
    print_status "WARNING" "API Server may have issues connecting to Kubernetes"
fi

# Start Dashboard React (non-blocking)
print_status "INFO" "Starting Dashboard React..."
cd dashboard-react

# Install dependencies if node_modules doesn't exist
if [ ! -d "node_modules" ]; then
    print_status "INFO" "Installing React dashboard dependencies..."
    npm install
    print_status "SUCCESS" "React dashboard dependencies installed"
fi

nohup npm start > ../dashboard.log 2>&1 &
DASHBOARD_PID=$!
echo $DASHBOARD_PID > ../.dashboard.pid
cd ..
disown
sleep 10

# Verify Dashboard is running
if ! kill -0 $DASHBOARD_PID 2>/dev/null; then
    print_status "ERROR" "Dashboard React failed to start"
    exit 1
fi
print_status "SUCCESS" "Dashboard React started (PID: $DASHBOARD_PID)"

sleep 5

print_status "SUCCESS" "All services started successfully"

# Skip Multi-Gateway deployment - will be deployed via dashboard
print_status "INFO" "Skipping Multi-Gateway deployment - will be deployed via dashboard configuration"

# Test services
print_status "INFO" "Testing deployed services..."

# Test Infrastructure API
print_status "INFO" "Testing Infrastructure API..."
if curl -4 -s http://localhost:30460/health >/dev/null 2>&1; then
    print_status "SUCCESS" "Infrastructure API is responding"
else
    print_status "WARNING" "Infrastructure API is not responding"
fi

# Test Gateway
print_status "INFO" "Testing Gateway..."
if curl -4 -s http://localhost:8080/health >/dev/null 2>&1; then
    print_status "SUCCESS" "Gateway is responding"
else
    print_status "WARNING" "Gateway is not responding"
fi

# Test API Server
print_status "INFO" "Testing API Server..."
if curl -4 -s http://localhost:3001/health >/dev/null 2>&1; then
    print_status "SUCCESS" "API Server is responding"
else
    print_status "WARNING" "API Server is not responding"
fi

# Test API endpoints
print_status "INFO" "Testing API endpoints..."
if curl -4 -s http://localhost:3001/api/v1/devices >/dev/null 2>&1; then
    print_status "SUCCESS" "Devices API endpoint is working"
else
    print_status "WARNING" "Devices API endpoint is not responding"
fi

if curl -4 -s http://localhost:3001/api/v1/gateways >/dev/null 2>&1; then
    print_status "SUCCESS" "Gateways API endpoint is working"
else
    print_status "WARNING" "Gateways API endpoint is not responding"
fi

# Summary
print_status "INFO" "=== DEPLOYMENT SUMMARY ==="
print_status "INFO" "Infrastructure API: http://localhost:30460"
print_status "INFO" "API Server: http://localhost:3001"
print_status "INFO" "Dashboard UI: http://localhost:3000 (React frontend)"
print_status "INFO" "Gateway HTTP API: http://localhost:8080"
print_status "INFO" "Gateway TLS: 127.0.0.1:8081 (Device communication)"
print_status "INFO" "Controllers: Running (Device, Application, Gateway)"
print_status "INFO" "TLS Certificates: Generated and configured"
print_status "INFO" "Next Steps: Use dashboard to deploy WASM applications to devices"

print_status "SUCCESS" "Wasmbed Platform deployed successfully!"

# Save PIDs for cleanup
echo "$INFRASTRUCTURE_PID $GATEWAY_PID $DEVICE_CONTROLLER_PID $APPLICATION_CONTROLLER_PID $GATEWAY_CONTROLLER_PID $API_SERVER_PID $DASHBOARD_PID" > .wasmbed-pids

print_status "INFO" "Use './scripts/05-stop-services.sh' to stop all services"
print_status "INFO" "Use './scripts/00-cleanup-environment.sh' for complete cleanup"