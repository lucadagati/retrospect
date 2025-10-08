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

# Check Kind cluster
print_status "INFO" "Checking Kind cluster..."
if kind get clusters | grep -q "wasmbed"; then
    print_status "SUCCESS" "Kind cluster 'wasmbed' is running"
else
    print_status "ERROR" "Kind cluster 'wasmbed' is not running"
fi

# Check kubectl context
print_status "INFO" "Checking kubectl context..."
if kubectl config current-context | grep -q "kind-wasmbed"; then
    print_status "SUCCESS" "kubectl context is correctly set to 'kind-wasmbed'"
else
    print_status "WARNING" "kubectl context is not set to 'kind-wasmbed'"
    print_status "INFO" "Run: kubectl config use-context kind-wasmbed"
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
if curl -4 -s "http://localhost:30460/api/v1/status" >/dev/null 2>&1; then
    print_status "SUCCESS" "Infrastructure service is responding on port 30460"
else
    print_status "ERROR" "Infrastructure service is not responding on port 30460"
fi

# Check Gateway (not running standalone - requires Kubernetes deployment)
print_status "INFO" "Gateway service requires Kubernetes deployment with certificates"
print_status "WARNING" "Gateway service is not running standalone"

# Check API Server (backend)
if curl -4 -s "http://localhost:3001/health" >/dev/null 2>&1; then
    print_status "SUCCESS" "API Server is responding on port 3001"
else
    print_status "ERROR" "API Server is not responding on port 3001"
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

# Gateway not running standalone - requires Kubernetes deployment
print_status "INFO" "Gateway requires Kubernetes deployment with certificates"

if pgrep -f "wasmbed-infrastructure" >/dev/null; then
    print_status "SUCCESS" "Infrastructure is running"
else
    print_status "ERROR" "Infrastructure is not running"
fi

if pgrep -f "wasmbed-api-server" >/dev/null; then
    print_status "SUCCESS" "API Server is running"
else
    print_status "ERROR" "API Server is not running"
fi

echo ""
print_status "INFO" "=== SERVICE ENDPOINTS ==="
echo "  Infrastructure API: http://localhost:30460"
echo "  API Server (Backend): http://localhost:3001"
echo "  Dashboard UI (Frontend): http://localhost:3000"
echo "  Gateway: Will be deployed via Kubernetes with certificates"
echo ""

print_status "INFO" "Status check completed"
