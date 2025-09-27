#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

# Wasmbed Platform - Multi-Gateway Deployment Script
# This script deploys 3 gateways and 6 boards (3 MCU + 3 RISC-V)

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

print_status "INFO" "Deploying Wasmbed Multi-Gateway System..."

# Create directories if they don't exist
mkdir -p k8s/gateways k8s/devices logs

# Deploy Gateways
print_status "INFO" "Deploying 3 Gateways..."
kubectl apply -f k8s/gateways/gateway-1.yaml
kubectl apply -f k8s/gateways/gateway-2.yaml
kubectl apply -f k8s/gateways/gateway-3.yaml
print_status "SUCCESS" "All 3 gateways deployed"

# Deploy MCU Boards
print_status "INFO" "Deploying 3 MCU Boards..."
kubectl apply -f k8s/devices/mcu-board-1.yaml
kubectl apply -f k8s/devices/mcu-board-2.yaml
kubectl apply -f k8s/devices/mcu-board-3.yaml
print_status "SUCCESS" "All 3 MCU boards deployed"

# Deploy RISC-V Boards
print_status "INFO" "Deploying 3 RISC-V Boards..."
kubectl apply -f k8s/devices/riscv-board-1.yaml
kubectl apply -f k8s/devices/riscv-board-2.yaml
kubectl apply -f k8s/devices/riscv-board-3.yaml
print_status "SUCCESS" "All 3 RISC-V boards deployed"

# Start additional Gateway instances
print_status "INFO" "Starting additional Gateway instances..."

# Gateway 2
if ! pgrep -f "wasmbed-gateway.*30454" >/dev/null; then
    print_status "INFO" "Starting Gateway 2 on ports 30454/30455..."
    nohup ./target/release/wasmbed-gateway \
        --bind-addr 127.0.0.1:30454 \
        --http-addr 127.0.0.1:30455 \
        --private-key certs/server-key.pem \
        --certificate certs/server-cert.pem \
        --client-ca certs/ca-cert.pem \
        --namespace wasmbed \
        --pod-namespace wasmbed \
        --pod-name gateway-2 >/dev/null 2>&1 &
    disown $!
    print_status "SUCCESS" "Gateway 2 started"
fi

# Gateway 3
if ! pgrep -f "wasmbed-gateway.*30456" >/dev/null; then
    print_status "INFO" "Starting Gateway 3 on ports 30456/30457..."
    nohup ./target/release/wasmbed-gateway \
        --bind-addr 127.0.0.1:30456 \
        --http-addr 127.0.0.1:30457 \
        --private-key certs/server-key.pem \
        --certificate certs/server-cert.pem \
        --client-ca certs/ca-cert.pem \
        --namespace wasmbed \
        --pod-namespace wasmbed \
        --pod-name gateway-3 >/dev/null 2>&1 &
    disown $!
    print_status "SUCCESS" "Gateway 3 started"
fi

# Wait for services to initialize
print_status "INFO" "Waiting for services to initialize..."
sleep 5

# Show deployment summary
print_status "INFO" "=== DEPLOYMENT SUMMARY ==="
print_status "INFO" "Gateways: 3 (ports 30452/30453, 30454/30455, 30456/30457)"
print_status "INFO" "MCU Boards: 3 (mcu-board-1, mcu-board-2, mcu-board-3)"
print_status "INFO" "RISC-V Boards: 3 (riscv-board-1, riscv-board-2, riscv-board-3)"
print_status "INFO" "Total Devices: 6"

echo ""
print_status "INFO" "=== SERVICE ENDPOINTS ==="
print_status "INFO" "Gateway 1: http://localhost:30453"
print_status "INFO" "Gateway 2: http://localhost:30455"
print_status "INFO" "Gateway 3: http://localhost:30457"
print_status "INFO" "Dashboard: http://localhost:30470"

echo ""
print_status "SUCCESS" "Multi-Gateway deployment completed successfully!"
print_status "INFO" "Use 'kubectl get devices -n wasmbed' to see all devices"
print_status "INFO" "Use 'kubectl get gateways -n wasmbed' to see all gateways"
