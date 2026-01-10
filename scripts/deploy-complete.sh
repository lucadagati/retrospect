#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Complete deployment script for Wasmbed platform
# Deploys all components with proper RBAC from scratch

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

NAMESPACE="wasmbed"

print_header "WASMBED COMPLETE DEPLOYMENT"

# Check prerequisites
print_header "STEP 1: CHECK PREREQUISITES"

if ! command -v kubectl &> /dev/null; then
    print_status "ERROR" "kubectl is not installed"
    exit 1
fi
print_status "SUCCESS" "kubectl found"

if ! command -v docker &> /dev/null; then
    print_status "ERROR" "Docker is not installed"
    exit 1
fi
print_status "SUCCESS" "Docker found"

# Check Kubernetes cluster
if ! kubectl cluster-info &> /dev/null; then
    print_status "ERROR" "Kubernetes cluster is not accessible"
    exit 1
fi
print_status "SUCCESS" "Kubernetes cluster accessible"

# Check if using kind
IS_KIND=false
if kubectl cluster-info | grep -q "kind"; then
    IS_KIND=true
    print_status "INFO" "Detected kind cluster"
fi

# Cleanup existing resources (optional)
print_header "STEP 2: CLEANUP (OPTIONAL)"

read -p "Do you want to delete existing Wasmbed resources? (y/N): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    print_status "INFO" "Deleting existing resources..."
    kubectl delete namespace "$NAMESPACE" --ignore-not-found=true || true
    kubectl delete clusterrole,clusterrolebinding -l app=wasmbed --ignore-not-found=true || true
    sleep 5
    print_status "SUCCESS" "Cleanup complete"
else
    print_status "INFO" "Skipping cleanup"
fi

# Create namespace
print_header "STEP 3: CREATE NAMESPACE"

if kubectl get namespace "$NAMESPACE" &> /dev/null; then
    print_status "INFO" "Namespace $NAMESPACE already exists"
else
    kubectl apply -f k8s/namespace.yaml
    print_status "SUCCESS" "Namespace $NAMESPACE created"
fi

# Apply CRDs
print_header "STEP 4: APPLY CRDS"

print_status "INFO" "Applying Custom Resource Definitions..."
for crd in k8s/crds/*.yaml; do
    if [ -f "$crd" ]; then
        kubectl apply -f "$crd"
        print_status "SUCCESS" "Applied $(basename $crd)"
    fi
done

# Wait for CRDs to be ready
print_status "INFO" "Waiting for CRDs to be ready..."
kubectl wait --for condition=established --timeout=60s \
    crd/devices.wasmbed.io \
    crd/applications.wasmbed.io \
    crd/gateways.wasmbed.io 2>/dev/null || print_status "WARNING" "Some CRDs may not be ready yet"

# Apply RBAC
print_header "STEP 5: APPLY RBAC"

print_status "INFO" "Applying RBAC resources..."

# Gateway RBAC (needs secrets access)
if [ -f "k8s/rbac/gateway-rbac.yaml" ]; then
    kubectl apply -f k8s/rbac/gateway-rbac.yaml
    print_status "SUCCESS" "Gateway RBAC applied"
fi

# Gateway Controller RBAC
if [ -f "k8s/rbac/gateway-controller-rbac.yaml" ]; then
    kubectl apply -f k8s/rbac/gateway-controller-rbac.yaml
    print_status "SUCCESS" "Gateway Controller RBAC applied"
fi

# Device Controller RBAC
if [ -f "k8s/rbac/device-controller-rbac.yaml" ]; then
    kubectl apply -f k8s/rbac/device-controller-rbac.yaml
    print_status "SUCCESS" "Device Controller RBAC applied"
fi

# Application Controller RBAC
if [ -f "k8s/rbac/application-controller-rbac.yaml" ]; then
    kubectl apply -f k8s/rbac/application-controller-rbac.yaml
    print_status "SUCCESS" "Application Controller RBAC applied"
fi

# API Server RBAC
if [ -f "k8s/rbac/api-server-rbac.yaml" ]; then
    kubectl apply -f k8s/rbac/api-server-rbac.yaml
    print_status "SUCCESS" "API Server RBAC applied"
fi

# General Wasmbed RBAC (if exists)
if [ -f "k8s/rbac/wasmbed-rbac.yaml" ]; then
    kubectl apply -f k8s/rbac/wasmbed-rbac.yaml
    print_status "SUCCESS" "General Wasmbed RBAC applied"
fi

# Verify RBAC
print_status "INFO" "Verifying RBAC..."
kubectl get serviceaccounts -n "$NAMESPACE"
kubectl get role,clusterrole -n "$NAMESPACE" | grep wasmbed || true
kubectl get rolebinding,clusterrolebinding -n "$NAMESPACE" | grep wasmbed || true

# Build Docker images
print_header "STEP 6: BUILD DOCKER IMAGES"

print_status "INFO" "Building Docker images..."

# Gateway
if [ -f "Dockerfile.gateway" ]; then
    print_status "INFO" "Building gateway image..."
    docker build -f Dockerfile.gateway -t wasmbed/gateway:latest . || print_status "WARNING" "Gateway build failed"
    [ "$IS_KIND" = true ] && kind load docker-image wasmbed/gateway:latest || true
fi

# API Server
if [ -f "Dockerfile.api-server" ]; then
    print_status "INFO" "Building API server image..."
    docker build -f Dockerfile.api-server -t wasmbed/api-server:latest . || print_status "WARNING" "API server build failed"
    [ "$IS_KIND" = true ] && kind load docker-image wasmbed/api-server:latest || true
fi

# Dashboard
if [ -f "Dockerfile.dashboard" ]; then
    print_status "INFO" "Building dashboard image..."
    docker build -f Dockerfile.dashboard -t wasmbed/dashboard:latest . || print_status "WARNING" "Dashboard build failed"
    [ "$IS_KIND" = true ] && kind load docker-image wasmbed/dashboard:latest || true
fi

# Device Controller
if [ -f "Dockerfile.device-controller" ]; then
    print_status "INFO" "Building device controller image..."
    docker build -f Dockerfile.device-controller -t wasmbed/device-controller:latest . || print_status "WARNING" "Device controller build failed"
    [ "$IS_KIND" = true ] && kind load docker-image wasmbed/device-controller:latest || true
fi

# Application Controller
if [ -f "Dockerfile.application-controller" ]; then
    print_status "INFO" "Building application controller image..."
    docker build -f Dockerfile.application-controller -t wasmbed/application-controller:latest . || print_status "WARNING" "Application controller build failed"
    [ "$IS_KIND" = true ] && kind load docker-image wasmbed/application-controller:latest || true
fi

# Gateway Controller
if [ -f "Dockerfile.gateway-controller" ]; then
    print_status "INFO" "Building gateway controller image..."
    docker build -f Dockerfile.gateway-controller -t wasmbed/gateway-controller:latest . || print_status "WARNING" "Gateway controller build failed"
    [ "$IS_KIND" = true ] && kind load docker-image wasmbed/gateway-controller:latest || true
fi

# Create certificates secret (if needed)
print_header "STEP 7: SETUP CERTIFICATES"

if [ -d "certs" ] && [ -f "certs/ca-cert.pem" ] && [ -f "certs/server-cert.pem" ] && [ -f "certs/server-key.pem" ]; then
    print_status "INFO" "Creating gateway certificates secret..."
    kubectl create secret generic gateway-certificates \
        --from-file=ca-cert.pem=certs/ca-cert.pem \
        --from-file=server-cert.pem=certs/server-cert.pem \
        --from-file=server-key.pem=certs/server-key.pem \
        -n "$NAMESPACE" \
        --dry-run=client -o yaml | kubectl apply -f -
    print_status "SUCCESS" "Certificates secret created"
else
    print_status "WARNING" "Certificates not found, gateway may not work without TLS"
fi

# Apply deployments
print_header "STEP 8: APPLY DEPLOYMENTS"

print_status "INFO" "Applying deployments..."

# Main deployments (gateway, controllers)
if [ -f "k8s/deployments/wasmbed-deployments.yaml" ]; then
    kubectl apply -f k8s/deployments/wasmbed-deployments.yaml
    print_status "SUCCESS" "Main deployments applied"
fi

# API Server deployment
if [ -f "k8s/deployments/api-server-deployment.yaml" ]; then
    kubectl apply -f k8s/deployments/api-server-deployment.yaml
    print_status "SUCCESS" "API Server deployment applied"
fi

# Dashboard deployment
if [ -f "k8s/deployments/dashboard-deployment.yaml" ]; then
    kubectl apply -f k8s/deployments/dashboard-deployment.yaml
    print_status "SUCCESS" "Dashboard deployment applied"
fi

# Wait for deployments
print_header "STEP 9: WAIT FOR DEPLOYMENTS"

print_status "INFO" "Waiting for deployments to be ready..."

DEPLOYMENTS=(
    "wasmbed-gateway"
    "wasmbed-api-server"
    "wasmbed-dashboard"
    "wasmbed-device-controller"
    "wasmbed-application-controller"
    "wasmbed-gateway-controller"
)

for deployment in "${DEPLOYMENTS[@]}"; do
    if kubectl get deployment "$deployment" -n "$NAMESPACE" &> /dev/null; then
        print_status "INFO" "Waiting for $deployment..."
        if kubectl wait --for=condition=available --timeout=120s deployment/"$deployment" -n "$NAMESPACE" 2>/dev/null; then
            print_status "SUCCESS" "$deployment is ready"
        else
            print_status "WARNING" "$deployment may still be starting"
        fi
    else
        print_status "INFO" "$deployment not found, skipping"
    fi
done

# Verify deployment
print_header "STEP 10: VERIFY DEPLOYMENT"

print_status "INFO" "Deployment status:"
kubectl get deployments -n "$NAMESPACE"

print_status "INFO" "Service status:"
kubectl get services -n "$NAMESPACE"

print_status "INFO" "Pod status:"
kubectl get pods -n "$NAMESPACE"

print_status "INFO" "RBAC status:"
kubectl get serviceaccounts,role,rolebinding,clusterrole,clusterrolebinding -n "$NAMESPACE" | grep wasmbed || true

# Check for errors
print_header "STEP 11: CHECK FOR ERRORS"

ERRORS=0
for pod in $(kubectl get pods -n "$NAMESPACE" -o name); do
    if kubectl get "$pod" -n "$NAMESPACE" -o jsonpath='{.status.phase}' | grep -q "Error\|CrashLoopBackOff\|ImagePullBackOff"; then
        print_status "ERROR" "$pod is in error state"
        kubectl describe "$pod" -n "$NAMESPACE" | tail -20
        ERRORS=$((ERRORS + 1))
    fi
done

if [ $ERRORS -eq 0 ]; then
    print_status "SUCCESS" "No pod errors found"
else
    print_status "WARNING" "Found $ERRORS pod(s) with errors"
fi

# Show access information
print_header "DEPLOYMENT COMPLETE"

print_status "SUCCESS" "Wasmbed platform deployed successfully!"

echo ""
print_status "INFO" "Access information:"
echo "  Dashboard: kubectl port-forward -n $NAMESPACE svc/wasmbed-dashboard 3000:3000"
echo "  API Server: kubectl port-forward -n $NAMESPACE svc/wasmbed-api-server 3001:3001"
echo "  Gateway: kubectl port-forward -n $NAMESPACE svc/wasmbed-gateway 8080:8080"
echo ""
print_status "INFO" "Service endpoints (from within cluster):"
echo "  Dashboard: http://wasmbed-dashboard.$NAMESPACE.svc.cluster.local:3000"
echo "  API Server: http://wasmbed-api-server.$NAMESPACE.svc.cluster.local:3001"
echo "  Gateway: http://wasmbed-gateway.$NAMESPACE.svc.cluster.local:8080"
echo ""
print_status "INFO" "To view logs:"
echo "  kubectl logs -n $NAMESPACE -l app=wasmbed-gateway"
echo "  kubectl logs -n $NAMESPACE -l app=wasmbed-api-server"
echo "  kubectl logs -n $NAMESPACE -l app=wasmbed-dashboard"

