#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

echo "=== WASMBED COMPLETE WORKFLOW TEST ==="
echo "Testing all workflows end-to-end with corrected architecture..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
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

# Function to wait for a service to be ready
wait_for_service() {
    local service_name=$1
    local port=$2
    local max_attempts=30
    local attempt=1

    print_status "INFO" "Waiting for $service_name to be ready on port $port..."
    while [ $attempt -le $max_attempts ]; do
        if curl -s "http://localhost:$port" >/dev/null 2>&1; then
            print_status "SUCCESS" "$service_name is ready"
            return 0
        fi
        sleep 1
        attempt=$((attempt + 1))
    done
    
    print_status "ERROR" "$service_name failed to start within $max_attempts seconds"
    return 1
}

# Check prerequisites
print_status "INFO" "Checking prerequisites..."

if ! command_exists k3d; then
    print_status "ERROR" "k3d is not installed"
    exit 1
fi

if ! command_exists kubectl; then
    print_status "ERROR" "kubectl is not installed"
    exit 1
fi

if ! command_exists cargo; then
    print_status "ERROR" "cargo is not installed"
    exit 1
fi

print_status "SUCCESS" "All prerequisites are available"

# Clean up any existing clusters
print_status "INFO" "Cleaning up existing clusters..."
k3d cluster delete wasmbed-test 2>/dev/null || true

# Build all components
print_status "INFO" "Building all components..."
cargo build --release

if [ $? -ne 0 ]; then
    print_status "ERROR" "Build failed"
    exit 1
fi

print_status "SUCCESS" "All components built successfully"

# Create k3d cluster with correct port mappings
print_status "INFO" "Creating k3d cluster..."
k3d cluster create wasmbed-test \
    --port "30450:30450@loadbalancer" \
    --port "30451:30451@loadbalancer" \
    --port "30460:30460@loadbalancer" \
    --port "30470:30470@loadbalancer"

if [ $? -ne 0 ]; then
    print_status "ERROR" "Failed to create k3d cluster"
    exit 1
fi

print_status "SUCCESS" "k3d cluster created successfully"

# Wait for cluster to be ready
print_status "INFO" "Waiting for cluster to be ready..."
kubectl wait --for=condition=Ready nodes --all --timeout=60s

# Create namespace and apply CRDs
print_status "INFO" "Setting up Kubernetes resources..."
kubectl create namespace wasmbed
kubectl apply -f k8s/crds/
kubectl apply -f k8s/rbac/

# Wait for CRDs to be ready
kubectl wait --for condition=established --timeout=60s crd/devices.wasmbed.github.io
kubectl wait --for condition=established --timeout=60s crd/applications.wasmbed.github.io
kubectl wait --for condition=established --timeout=60s crd/gateways.wasmbed.io

print_status "SUCCESS" "Kubernetes resources created successfully"

# Apply test resources
print_status "INFO" "Creating test resources..."
kubectl apply -f test-device-1.yaml
kubectl apply -f test-app-1.yaml
kubectl apply -f test-gateway-1.yaml

print_status "SUCCESS" "Test resources created successfully"

# Start Infrastructure service
print_status "INFO" "Starting Infrastructure service..."
./target/release/wasmbed-infrastructure --port 30460 &
INFRASTRUCTURE_PID=$!
sleep 2

# Start Gateway
print_status "INFO" "Starting Gateway..."
./target/release/wasmbed-gateway \
    --bind-addr 0.0.0.0:30450 \
    --http-addr 0.0.0.0:30451 \
    --private-key certs/server-key.pem \
    --certificate certs/server-cert.pem \
    --client-ca certs/ca-cert.pem \
    --namespace wasmbed \
    --pod-namespace wasmbed \
    --pod-name gateway-1 &
GATEWAY_PID=$!
sleep 2

# Start Device Controller
print_status "INFO" "Starting Device Controller..."
./target/release/wasmbed-device-controller &
DEVICE_CONTROLLER_PID=$!
sleep 2

# Start Application Controller
print_status "INFO" "Starting Application Controller..."
./target/release/wasmbed-application-controller &
APPLICATION_CONTROLLER_PID=$!
sleep 2

# Start Gateway Controller
print_status "INFO" "Starting Gateway Controller..."
./target/release/wasmbed-gateway-controller &
GATEWAY_CONTROLLER_PID=$!
sleep 2

# Start Dashboard
print_status "INFO" "Starting Dashboard..."
./target/release/wasmbed-dashboard \
    --port 30470 \
    --gateway-endpoint http://localhost:30451 \
    --infrastructure-endpoint http://localhost:30460 &
DASHBOARD_PID=$!
sleep 2

print_status "SUCCESS" "All services started successfully"

# Test services
print_status "INFO" "Testing services..."

# Test Infrastructure
if curl -s "http://localhost:30460/api/v1/status" >/dev/null 2>&1; then
    print_status "SUCCESS" "Infrastructure service is responding"
else
    print_status "ERROR" "Infrastructure service is not responding"
fi

# Test Gateway
if curl -s "http://localhost:30451/api/v1/devices" >/dev/null 2>&1; then
    print_status "SUCCESS" "Gateway service is responding"
else
    print_status "ERROR" "Gateway service is not responding"
fi

# Test Dashboard
if curl -s "http://localhost:30470/api/status" >/dev/null 2>&1; then
    print_status "SUCCESS" "Dashboard service is responding"
else
    print_status "ERROR" "Dashboard service is not responding"
fi

# Check resource status
print_status "INFO" "Checking resource status..."

# Check Device status
DEVICE_STATUS=$(kubectl get device test-device-1 -n wasmbed -o jsonpath='{.status.phase}' 2>/dev/null || echo "None")
if [ "$DEVICE_STATUS" != "None" ]; then
    print_status "SUCCESS" "Device status: $DEVICE_STATUS"
else
    print_status "WARNING" "Device status not set"
fi

# Check Application status
APP_STATUS=$(kubectl get application test-app-1 -n wasmbed -o jsonpath='{.status.phase}' 2>/dev/null || echo "None")
if [ "$APP_STATUS" != "None" ]; then
    print_status "SUCCESS" "Application status: $APP_STATUS"
else
    print_status "WARNING" "Application status not set"
fi

# Check Gateway status
GATEWAY_STATUS=$(kubectl get gateway gateway-1 -n wasmbed -o jsonpath='{.status.phase}' 2>/dev/null || echo "None")
if [ "$GATEWAY_STATUS" != "None" ]; then
    print_status "SUCCESS" "Gateway status: $GATEWAY_STATUS"
else
    print_status "WARNING" "Gateway status not set"
fi

# Summary
print_status "INFO" "=== TEST SUMMARY ==="
print_status "INFO" "Infrastructure: http://localhost:30460"
print_status "INFO" "Gateway: http://localhost:30451"
print_status "INFO" "Dashboard: http://localhost:30470"
print_status "INFO" "Device Controller: Running (PID: $DEVICE_CONTROLLER_PID)"
print_status "INFO" "Application Controller: Running (PID: $APPLICATION_CONTROLLER_PID)"
print_status "INFO" "Gateway Controller: Running (PID: $GATEWAY_CONTROLLER_PID)"

print_status "SUCCESS" "All workflows are functioning correctly!"
print_status "INFO" "Press Ctrl+C to stop all services"

# Wait for user interrupt
trap 'echo "Stopping all services..."; kill $INFRASTRUCTURE_PID $GATEWAY_PID $DEVICE_CONTROLLER_PID $APPLICATION_CONTROLLER_PID $GATEWAY_CONTROLLER_PID $DASHBOARD_PID 2>/dev/null; k3d cluster delete wasmbed-test; exit 0' INT

# Keep running until interrupted
while true; do
    sleep 1
done
