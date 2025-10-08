#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

# Wasmbed Platform - Stop Script
# This script stops all Wasmbed services

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

print_status "INFO" "Stopping Wasmbed Platform services..."

# Stop services using individual PID files
print_status "INFO" "Stopping services using PID files..."

# List of services to stop
services=("infrastructure" "gateway" "device-controller" "application-controller" "gateway-controller" "api-server" "dashboard" "multi-gateway")

for service in "${services[@]}"; do
    pid_file=".${service}.pid"
    if [ -f "$pid_file" ]; then
        pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            print_status "INFO" "Stopping $service (PID: $pid)"
            kill "$pid" 2>/dev/null || true
        fi
        rm -f "$pid_file"
    fi
done

print_status "SUCCESS" "Services stopped using PID files"

# Stop any remaining Wasmbed processes
print_status "INFO" "Stopping any remaining Wasmbed processes..."
pkill -f "wasmbed-" 2>/dev/null || true
pkill -f "wasmbed_" 2>/dev/null || true

# Force kill processes using Wasmbed ports
print_status "INFO" "Force killing processes using Wasmbed ports..."
sudo fuser -k 30460/tcp 2>/dev/null || true
sudo fuser -k 30470/tcp 2>/dev/null || true
sudo fuser -k 30450/tcp 2>/dev/null || true
sudo fuser -k 30451/tcp 2>/dev/null || true
sudo fuser -k 30453/tcp 2>/dev/null || true
sleep 2

# Stop k3d cluster
print_status "INFO" "Stopping k3d cluster..."
k3d cluster delete wasmbed-test 2>/dev/null || true

# Clean up PID files
print_status "INFO" "Cleaning up PID files..."
rm -f .*.pid .wasmbed-pids 2>/dev/null || true

# Verify ports are free
print_status "INFO" "Verifying ports are free..."
if netstat -tlnp 2>/dev/null | grep -E "30460|30470|30450|30451|30453" >/dev/null; then
    print_status "WARNING" "Some Wasmbed ports are still in use"
    netstat -tlnp 2>/dev/null | grep -E "30460|30470|30450|30451|30453" || true
else
    print_status "SUCCESS" "All Wasmbed ports are free"
fi

print_status "SUCCESS" "All Wasmbed services stopped successfully!"
print_status "INFO" "System is now clean"
