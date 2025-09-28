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

# Skip test resources - will be created via dashboard
print_status "INFO" "Skipping test resources - will be created via dashboard configuration"

# Start services
print_status "INFO" "Starting all services..."

# Start Infrastructure (non-blocking)
nohup ./target/release/wasmbed-infrastructure --port 30460 > logs/infrastructure.log 2>&1 &
INFRASTRUCTURE_PID=$!
echo $INFRASTRUCTURE_PID > .infrastructure.pid
disown

# Skip Gateway startup - will be deployed via dashboard
print_status "INFO" "Skipping Gateway startup - will be deployed via dashboard configuration"

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

# Skip Multi-Gateway deployment - will be deployed via dashboard
print_status "INFO" "Skipping Multi-Gateway deployment - will be deployed via dashboard configuration"

# Summary
print_status "INFO" "=== DEPLOYMENT SUMMARY ==="
print_status "INFO" "Infrastructure: http://localhost:30460"
print_status "INFO" "Dashboard: http://localhost:30470"
print_status "INFO" "Controllers: Running (Device, Application, Gateway)"
print_status "INFO" "Next Steps: Use dashboard to configure gateways and devices"

print_status "SUCCESS" "Wasmbed Platform deployed successfully!"

# Save PIDs for cleanup
echo "$INFRASTRUCTURE_PID $DEVICE_CONTROLLER_PID $APPLICATION_CONTROLLER_PID $GATEWAY_CONTROLLER_PID $DASHBOARD_PID" > .wasmbed-pids

print_status "INFO" "Use './scripts/stop.sh' to stop all services"