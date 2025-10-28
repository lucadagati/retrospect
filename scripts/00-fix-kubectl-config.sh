#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

# Wasmbed Platform - Fix kubectl Configuration Script
# This script fixes kubectl configuration and installs CRDs

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

print_header() {
    local title=$1
    echo ""
    echo "========================================"
    echo "  $title"
    echo "========================================"
}

print_header "WASMBED PLATFORM - KUBECTL CONFIGURATION FIX"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_status "ERROR" "Please run this script from the project root directory"
    exit 1
fi

# Check prerequisites
print_status "INFO" "Checking prerequisites..."

if ! command -v kind >/dev/null 2>&1; then
    print_status "ERROR" "kind is not installed"
    exit 1
fi

if ! command -v kubectl >/dev/null 2>&1; then
    print_status "ERROR" "kubectl is not installed"
    exit 1
fi

print_status "SUCCESS" "All prerequisites are available"

# Check if Kind cluster exists
print_status "INFO" "Checking Kind cluster..."
if ! kind get clusters | grep -q "wasmbed"; then
    print_status "ERROR" "Kind cluster 'wasmbed' does not exist. Please create it first."
    exit 1
fi

print_status "SUCCESS" "Kind cluster 'wasmbed' exists"

# Configure kubectl context
print_status "INFO" "Configuring kubectl context..."
kubectl config use-context kind-wasmbed
print_status "SUCCESS" "kubectl context configured"

# Verify kubectl connection
print_status "INFO" "Verifying kubectl connection..."
if kubectl cluster-info >/dev/null 2>&1; then
    print_status "SUCCESS" "kubectl connection verified"
else
    print_status "ERROR" "kubectl connection failed"
    exit 1
fi

# Create namespace if it doesn't exist
print_status "INFO" "Creating namespace..."
kubectl create namespace wasmbed --dry-run=client -o yaml | kubectl apply -f -
print_status "SUCCESS" "Namespace created/verified"

# Install CRDs
print_status "INFO" "Installing CRDs..."
kubectl apply -f k8s/crds/
print_status "SUCCESS" "CRDs installed"

# Wait for CRDs to be established
print_status "INFO" "Waiting for CRDs to be established..."
kubectl wait --for condition=established --timeout=60s crd/devices.wasmbed.github.io
kubectl wait --for condition=established --timeout=60s crd/applications.wasmbed.github.io
kubectl wait --for condition=established --timeout=60s crd/gateways.wasmbed.io
print_status "SUCCESS" "CRDs established"

# Verify CRDs
print_status "INFO" "Verifying CRDs..."
kubectl get crd | grep -E "(devices|applications|gateways)"
print_status "SUCCESS" "CRDs verified"

# Test API calls
print_status "INFO" "Testing API calls..."
if kubectl get devices -n wasmbed >/dev/null 2>&1; then
    print_status "SUCCESS" "Devices CRD is working"
else
    print_status "WARNING" "Devices CRD may have issues"
fi

if kubectl get applications -n wasmbed >/dev/null 2>&1; then
    print_status "SUCCESS" "Applications CRD is working"
else
    print_status "WARNING" "Applications CRD may have issues"
fi

if kubectl get gateways -n wasmbed >/dev/null 2>&1; then
    print_status "SUCCESS" "Gateways CRD is working"
else
    print_status "WARNING" "Gateways CRD may have issues"
fi

print_header "CONFIGURATION COMPLETE"
print_status "SUCCESS" "kubectl configuration fixed successfully!"
print_status "INFO" "You can now restart the API server:"
print_status "INFO" "  pkill -f wasmbed-api-server"
print_status "INFO" "  nohup ./target/release/wasmbed-api-server > api-server.log 2>&1 &"
print_status "INFO" "Or run the full deployment: ./scripts/99-full-deployment.sh"
