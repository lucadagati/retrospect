#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

# Wasmbed Platform - Clean System Script
# This script cleans up all Wasmbed components and resources

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

print_status "INFO" "Starting Wasmbed Platform Cleanup..."

# Stop all running Wasmbed processes
print_status "INFO" "Stopping all Wasmbed processes..."
pkill -f "wasmbed-" 2>/dev/null || true
pkill -f "wasmbed_" 2>/dev/null || true

# Clean up k3d clusters
print_status "INFO" "Cleaning up k3d clusters..."
k3d cluster list --no-headers | awk '{print $1}' | while read cluster; do
    if [[ "$cluster" == *"wasmbed"* ]]; then
        print_status "INFO" "Deleting k3d cluster: $cluster"
        k3d cluster delete "$cluster" 2>/dev/null || true
    fi
done

# Clean up Docker containers and images
print_status "INFO" "Cleaning up Docker resources..."
docker ps -a --filter "name=k3d-wasmbed" --format "{{.ID}}" | xargs -r docker rm -f 2>/dev/null || true
docker images --filter "reference=k3d-wasmbed*" --format "{{.ID}}" | xargs -r docker rmi -f 2>/dev/null || true

# Clean up build artifacts
print_status "INFO" "Cleaning up build artifacts..."
if [ -d "target" ]; then
    rm -rf target/
    print_status "SUCCESS" "Build artifacts cleaned"
fi

# Clean up certificates
print_status "INFO" "Cleaning up certificates..."
if [ -d "certs" ]; then
    rm -rf certs/
    print_status "SUCCESS" "Certificates cleaned"
fi

# Clean up logs
print_status "INFO" "Cleaning up logs..."
find . -name "*.log" -type f -delete 2>/dev/null || true
print_status "SUCCESS" "Log files cleaned"

# Clean up temporary files
print_status "INFO" "Cleaning up temporary files..."
find . -name "*.tmp" -type f -delete 2>/dev/null || true
find . -name ".DS_Store" -type f -delete 2>/dev/null || true
print_status "SUCCESS" "Temporary files cleaned"

# Clean up node_modules if they exist
print_status "INFO" "Cleaning up Node.js dependencies..."
if [ -d "dashboard-react/node_modules" ]; then
    rm -rf dashboard-react/node_modules/
    print_status "SUCCESS" "Node.js dependencies cleaned"
fi

# Reset kubectl context
print_status "INFO" "Resetting kubectl context..."
kubectl config unset current-context 2>/dev/null || true

print_status "SUCCESS" "Wasmbed Platform cleanup completed!"
print_status "INFO" "System is now clean and ready for fresh deployment"
