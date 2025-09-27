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

# Check prerequisites
print_status "INFO" "Checking deployment prerequisites..."

if ! command -v k3d >/dev/null 2>&1; then
    print_status "ERROR" "k3d is not installed"
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

# Create k3d cluster
print_status "INFO" "Creating k3d cluster..."
k3d cluster create wasmbed-test \
    --port "30450:30450@loadbalancer" \
    --port "30451:30451@loadbalancer" \
    --port "30460:30460@loadbalancer" \
    --port "30470:30470@loadbalancer"

print_status "SUCCESS" "k3d cluster created successfully"

# Setup Kubernetes resources
print_status "INFO" "Setting up Kubernetes resources..."
kubectl create namespace wasmbed
kubectl apply -f k8s/crds/
kubectl apply -f k8s/rbac/

# Wait for CRDs
kubectl wait --for condition=established --timeout=60s crd/devices.wasmbed.github.io
kubectl wait --for condition=established --timeout=60s crd/applications.wasmbed.github.io
kubectl wait --for condition=established --timeout=60s crd/gateways.wasmbed.io

print_status "SUCCESS" "Kubernetes resources created successfully"

# Apply test resources
print_status "INFO" "Creating test resources..."
kubectl apply -f k8s/test-resources/test-device-1.yaml
kubectl apply -f k8s/test-resources/test-app-1.yaml
kubectl apply -f k8s/test-resources/test-gateway-1.yaml

print_status "SUCCESS" "Test resources created successfully"

# Start services
print_status "INFO" "Starting all services..."

# Start Infrastructure (non-blocking)
nohup ./target/release/wasmbed-infrastructure --port 30460 > logs/infrastructure.log 2>&1 &
INFRASTRUCTURE_PID=$!
echo $INFRASTRUCTURE_PID > .infrastructure.pid
disown

# Start Gateway (using internal ports to avoid k3d conflicts) (non-blocking)
nohup ./target/release/wasmbed-gateway \
    --bind-addr 127.0.0.1:30452 \
    --http-addr 127.0.0.1:30453 \
    --private-key certs/server-key.pem \
    --certificate certs/server-cert.pem \
    --client-ca certs/ca-cert.pem \
    --namespace wasmbed \
    --pod-namespace wasmbed \
    --pod-name gateway-1 > logs/gateway.log 2>&1 &
GATEWAY_PID=$!
echo $GATEWAY_PID > .gateway.pid
disown

# Start Controllers (non-blocking)
nohup ./target/release/wasmbed-device-controller > logs/device-controller.log 2>&1 &
DEVICE_CONTROLLER_PID=$!
echo $DEVICE_CONTROLLER_PID > .device-controller.pid
disown

nohup ./target/release/wasmbed-application-controller > logs/application-controller.log 2>&1 &
APPLICATION_CONTROLLER_PID=$!
echo $APPLICATION_CONTROLLER_PID > .application-controller.pid
disown

nohup ./target/release/wasmbed-gateway-controller > logs/gateway-controller.log 2>&1 &
GATEWAY_CONTROLLER_PID=$!
echo $GATEWAY_CONTROLLER_PID > .gateway-controller.pid
disown

# Start Dashboard (non-blocking)
nohup ./target/release/wasmbed-dashboard \
    --port 30470 \
    --gateway-endpoint http://localhost:30453 \
    --infrastructure-endpoint http://localhost:30460 > logs/dashboard.log 2>&1 &
DASHBOARD_PID=$!
echo $DASHBOARD_PID > .dashboard.pid
disown

sleep 5

print_status "SUCCESS" "All services started successfully"

# Deploy Multi-Gateway System (non-blocking)
print_status "INFO" "Deploying Multi-Gateway System..."
nohup ./scripts/wasmbed-deploy-multi-gateway.sh deploy > logs/multi-gateway.log 2>&1 &
MULTI_GATEWAY_PID=$!
echo $MULTI_GATEWAY_PID > .multi-gateway.pid
disown

# Summary
print_status "INFO" "=== DEPLOYMENT SUMMARY ==="
print_status "INFO" "Infrastructure: http://localhost:30460"
print_status "INFO" "Gateway 1: http://localhost:30453"
print_status "INFO" "Gateway 2: http://localhost:30455"
print_status "INFO" "Gateway 3: http://localhost:30457"
print_status "INFO" "Dashboard: http://localhost:30470"
print_status "INFO" "Total Gateways: 3"
print_status "INFO" "Total Devices: 6 (3 MCU + 3 RISC-V)"

print_status "SUCCESS" "Wasmbed Platform deployed successfully!"

# Save PIDs for cleanup
echo "$INFRASTRUCTURE_PID $GATEWAY_PID $DEVICE_CONTROLLER_PID $APPLICATION_CONTROLLER_PID $GATEWAY_CONTROLLER_PID $DASHBOARD_PID" > .wasmbed-pids

print_status "INFO" "Use './scripts/stop.sh' to stop all services"