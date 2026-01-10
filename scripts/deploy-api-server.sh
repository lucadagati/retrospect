#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Deploy API Server to Kubernetes

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    local status=$1
    local message=$2
    case $status in
        "SUCCESS") echo -e "${GREEN}✓${NC} $message" ;;
        "ERROR") echo -e "${RED}✗${NC} $message" ;;
        "WARNING") echo -e "${YELLOW}⚠${NC} $message" ;;
        "INFO") echo -e "${BLUE}ℹ${NC} $message" ;;
    esac
}

print_header() {
    echo ""
    echo "========================================"
    echo "  $1"
    echo "========================================"
}

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

print_header "DEPLOY API SERVER TO KUBERNETES"

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    print_status "ERROR" "Docker is not installed"
    exit 1
fi

# Check if kubectl is available
if ! command -v kubectl &> /dev/null; then
    print_status "ERROR" "kubectl is not installed"
    exit 1
fi

# Build Docker image
print_header "STEP 1: BUILD DOCKER IMAGE"

print_status "INFO" "Building API server Docker image..."
if docker build -f Dockerfile.api-server -t wasmbed/api-server:latest .; then
    print_status "SUCCESS" "Docker image built successfully"
else
    print_status "ERROR" "Failed to build Docker image"
    exit 1
fi

# Load image into kind (if using kind)
if kubectl cluster-info | grep -q "kind"; then
    print_status "INFO" "Loading image into kind cluster..."
    kind load docker-image wasmbed/api-server:latest || print_status "WARNING" "Failed to load image into kind (may not be using kind)"
fi

# Apply RBAC
print_header "STEP 2: APPLY RBAC"

print_status "INFO" "Applying API server RBAC..."
if kubectl apply -f k8s/rbac/api-server-rbac.yaml; then
    print_status "SUCCESS" "RBAC applied"
else
    print_status "ERROR" "Failed to apply RBAC"
    exit 1
fi

# Apply deployment
print_header "STEP 3: APPLY DEPLOYMENT"

print_status "INFO" "Applying API server deployment..."
if kubectl apply -f k8s/deployments/api-server-deployment.yaml; then
    print_status "SUCCESS" "Deployment applied"
else
    print_status "ERROR" "Failed to apply deployment"
    exit 1
fi

# Wait for deployment
print_header "STEP 4: WAIT FOR DEPLOYMENT"

print_status "INFO" "Waiting for API server to be ready..."
if kubectl wait --for=condition=available --timeout=120s deployment/wasmbed-api-server -n wasmbed; then
    print_status "SUCCESS" "API server is ready"
else
    print_status "WARNING" "Deployment may still be starting"
fi

# Show status
print_header "STEP 5: VERIFY DEPLOYMENT"

print_status "INFO" "API server status:"
kubectl get deployment wasmbed-api-server -n wasmbed
kubectl get svc wasmbed-api-server -n wasmbed

print_status "INFO" "API server pods:"
kubectl get pods -n wasmbed -l app=wasmbed-api-server

print_status "INFO" "API server logs:"
kubectl logs -n wasmbed -l app=wasmbed-api-server --tail=20

print_header "DEPLOYMENT COMPLETE"

print_status "SUCCESS" "API server deployed to Kubernetes"
print_status "INFO" "Service: wasmbed-api-server.wasmbed.svc.cluster.local:3001"
print_status "INFO" "To access from outside cluster, use port-forward:"
print_status "INFO" "  kubectl port-forward -n wasmbed svc/wasmbed-api-server 3001:3001"

