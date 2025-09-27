#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

# Wasmbed Platform - Status Script
# This script checks the status of all Wasmbed services

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

print_status "INFO" "Wasmbed Platform Status Check"

echo ""

# Check k3d cluster
print_status "INFO" "Checking k3d cluster..."
if k3d cluster list | grep -q "wasmbed-test"; then
    print_status "SUCCESS" "k3d cluster 'wasmbed-test' is running"
else
    print_status "ERROR" "k3d cluster 'wasmbed-test' is not running"
fi

# Check Kubernetes resources
print_status "INFO" "Checking Kubernetes resources..."
if kubectl get namespace wasmbed >/dev/null 2>&1; then
    print_status "SUCCESS" "Namespace 'wasmbed' exists"
else
    print_status "ERROR" "Namespace 'wasmbed' does not exist"
fi

# Check CRDs
print_status "INFO" "Checking Custom Resource Definitions..."
if kubectl get crd devices.wasmbed.github.io >/dev/null 2>&1; then
    print_status "SUCCESS" "Device CRD is installed"
else
    print_status "ERROR" "Device CRD is not installed"
fi

if kubectl get crd applications.wasmbed.github.io >/dev/null 2>&1; then
    print_status "SUCCESS" "Application CRD is installed"
else
    print_status "ERROR" "Application CRD is not installed"
fi

if kubectl get crd gateways.wasmbed.io >/dev/null 2>&1; then
    print_status "SUCCESS" "Gateway CRD is installed"
else
    print_status "ERROR" "Gateway CRD is not installed"
fi

# Check Multi-Gateway System
print_status "INFO" "Checking Multi-Gateway System..."
if kubectl get gateways -n wasmbed --no-headers | wc -l | grep -q "3"; then
    print_status "SUCCESS" "All 3 gateways are deployed"
else
    print_status "ERROR" "Not all gateways are deployed"
fi

if kubectl get devices -n wasmbed --no-headers | wc -l | grep -q "7"; then
    print_status "SUCCESS" "All 6 boards + 1 test device are deployed"
else
    print_status "ERROR" "Not all devices are deployed"
fi

if kubectl get application test-app-1 -n wasmbed >/dev/null 2>&1; then
    APP_STATUS=$(kubectl get application test-app-1 -n wasmbed -o jsonpath='{.status.phase}' 2>/dev/null || echo "Unknown")
    print_status "SUCCESS" "Test application exists (Status: $APP_STATUS)"
else
    print_status "ERROR" "Test application does not exist"
fi

if kubectl get gateway gateway-1 -n wasmbed >/dev/null 2>&1; then
    GATEWAY_STATUS=$(kubectl get gateway gateway-1 -n wasmbed -o jsonpath='{.status.phase}' 2>/dev/null || echo "Unknown")
    print_status "SUCCESS" "Test gateway exists (Status: $GATEWAY_STATUS)"
else
    print_status "ERROR" "Test gateway does not exist"
fi

# Check services
print_status "INFO" "Checking services..."

# Check Infrastructure
if curl -s "http://localhost:30460/api/v1/status" >/dev/null 2>&1; then
    print_status "SUCCESS" "Infrastructure service is responding on port 30460"
else
    print_status "ERROR" "Infrastructure service is not responding on port 30460"
fi

# Check Gateway
if curl -s "http://localhost:30453/api/v1/devices" >/dev/null 2>&1; then
    print_status "SUCCESS" "Gateway service is responding on port 30453"
else
    print_status "ERROR" "Gateway service is not responding on port 30453"
fi

# Check Dashboard
if curl -s "http://localhost:30470/api/status" >/dev/null 2>&1; then
    print_status "SUCCESS" "Dashboard service is responding on port 30470"
else
    print_status "ERROR" "Dashboard service is not responding on port 30470"
fi

# Check running processes
print_status "INFO" "Checking running processes..."
if pgrep -f "wasmbed-device-controller" >/dev/null; then
    print_status "SUCCESS" "Device Controller is running"
else
    print_status "ERROR" "Device Controller is not running"
fi

if pgrep -f "wasmbed-application-controller" >/dev/null; then
    print_status "SUCCESS" "Application Controller is running"
else
    print_status "ERROR" "Application Controller is not running"
fi

if pgrep -f "wasmbed-gateway-controller" >/dev/null; then
    print_status "SUCCESS" "Gateway Controller is running"
else
    print_status "ERROR" "Gateway Controller is not running"
fi

if pgrep -f "wasmbed-gateway" >/dev/null; then
    print_status "SUCCESS" "Gateway is running"
else
    print_status "ERROR" "Gateway is not running"
fi

if pgrep -f "wasmbed-infrastructure" >/dev/null; then
    print_status "SUCCESS" "Infrastructure is running"
else
    print_status "ERROR" "Infrastructure is not running"
fi

if pgrep -f "wasmbed-dashboard" >/dev/null; then
    print_status "SUCCESS" "Dashboard is running"
else
    print_status "ERROR" "Dashboard is not running"
fi

echo ""
print_status "INFO" "=== SERVICE ENDPOINTS ==="
echo "  Infrastructure API: http://localhost:30460"
echo "  Gateway 1: http://localhost:30453"
echo "  Gateway 2: http://localhost:30455"
echo "  Gateway 3: http://localhost:30457"
echo "  Dashboard UI: http://localhost:30470"
echo ""

print_status "INFO" "Status check completed"
