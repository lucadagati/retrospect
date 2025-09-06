#!/bin/bash

# Wasmbed Complete Cleanup Script
# Removes all Wasmbed components and resets the environment

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CLUSTER_NAME="wasmbed-platform"
NAMESPACE="wasmbed"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Confirm cleanup
confirm_cleanup() {
    if [ "${1:-}" != "--force" ]; then
        log_warning "This will completely remove the Wasmbed platform and all data!"
        log_warning "This action cannot be undone."
        echo ""
        read -p "Are you sure you want to continue? (yes/no): " -r
        if [[ ! $REPLY =~ ^[Yy][Ee][Ss]$ ]]; then
            log_info "Cleanup cancelled"
            exit 0
        fi
    fi
}

# Clean up Kubernetes resources
cleanup_kubernetes() {
    log_info "Cleaning up Kubernetes resources..."
    
    # Delete applications
    log_info "Deleting applications..."
    kubectl delete applications --all -n "$NAMESPACE" --ignore-not-found=true || true
    
    # Delete devices
    log_info "Deleting devices..."
    kubectl delete devices --all -n "$NAMESPACE" --ignore-not-found=true || true
    
    # Delete Gateway resources
    log_info "Deleting Gateway resources..."
    kubectl delete statefulset wasmbed-gateway -n "$NAMESPACE" --ignore-not-found=true || true
    kubectl delete service wasmbed-gateway-service -n "$NAMESPACE" --ignore-not-found=true || true
    kubectl delete serviceaccount wasmbed-gateway -n "$NAMESPACE" --ignore-not-found=true || true
    
    # Delete RBAC
    log_info "Deleting RBAC resources..."
    kubectl delete clusterrolebinding wasmbed-gateway-device-access-binding --ignore-not-found=true || true
    kubectl delete clusterrole wasmbed-device-access --ignore-not-found=true || true
    
    # Delete secrets
    log_info "Deleting secrets..."
    kubectl delete secret wasmbed-tls-secret -n "$NAMESPACE" --ignore-not-found=true || true
    kubectl delete secret wasmbed-ca-secret -n "$NAMESPACE" --ignore-not-found=true || true
    kubectl delete secret wasmbed-tls-secret-pkcs8 -n "$NAMESPACE" --ignore-not-found=true || true
    kubectl delete secret wasmbed-tls-secret-rsa -n "$NAMESPACE" --ignore-not-found=true || true
    kubectl delete secret wasmbed-ca-secret-rsa -n "$NAMESPACE" --ignore-not-found=true || true
    
    # Delete configmaps
    log_info "Deleting configmaps..."
    kubectl delete configmap dds-test-message -n "$NAMESPACE" --ignore-not-found=true || true
    
    # Delete CRDs
    log_info "Deleting Custom Resource Definitions..."
    kubectl delete crd applications.wasmbed.github.io --ignore-not-found=true || true
    kubectl delete crd devices.wasmbed.github.io --ignore-not-found=true || true
    
    # Delete namespace
    log_info "Deleting namespace..."
    kubectl delete namespace "$NAMESPACE" --ignore-not-found=true || true
    
    log_success "Kubernetes resources cleaned up"
}

# Clean up k3d cluster
cleanup_cluster() {
    log_info "Cleaning up k3d cluster..."
    
    if k3d cluster list | grep -q "$CLUSTER_NAME"; then
        log_info "Deleting k3d cluster: $CLUSTER_NAME"
        k3d cluster delete "$CLUSTER_NAME" || true
    else
        log_info "Cluster $CLUSTER_NAME not found"
    fi
    
    log_success "k3d cluster cleaned up"
}

# Clean up Docker resources
cleanup_docker() {
    log_info "Cleaning up Docker resources..."
    
    # Remove Wasmbed images
    log_info "Removing Wasmbed Docker images..."
    docker images | grep wasmbed | awk '{print $3}' | xargs -r docker rmi -f || true
    
    # Clean up containers
    log_info "Cleaning up containers..."
    docker container prune -f || true
    
    # Clean up images
    log_info "Cleaning up unused images..."
    docker image prune -f || true
    
    # Clean up volumes
    log_info "Cleaning up volumes..."
    docker volume prune -f || true
    
    # Clean up networks
    log_info "Cleaning up networks..."
    docker network prune -f || true
    
    log_success "Docker resources cleaned up"
}

# Clean up certificates
cleanup_certificates() {
    log_info "Cleaning up certificates..."
    
    if [ -d "certs" ]; then
        rm -rf certs
        log_success "Certificate directory removed"
    else
        log_info "No certificate directory found"
    fi
}

# Clean up build artifacts
cleanup_build_artifacts() {
    log_info "Cleaning up build artifacts..."
    
    # Clean Rust build artifacts
    if [ -d "target" ]; then
        cargo clean
        log_success "Rust build artifacts cleaned"
    fi
    
    # Clean up temporary files
    rm -f microros-bridge.wat microros-bridge.wasm || true
    rm -f test-device.yaml test-device-correct.yaml test-application.yaml test-application-simple.yaml || true
    
    log_success "Build artifacts cleaned up"
}

# Clean up QEMU processes
cleanup_qemu() {
    log_info "Cleaning up QEMU processes..."
    
    # Kill any running QEMU processes
    pkill -f qemu-system-riscv64 || true
    pkill -f qemu-system-arm || true
    
    log_success "QEMU processes cleaned up"
}

# Reset kubectl context
reset_kubectl() {
    log_info "Resetting kubectl context..."
    
    # Remove k3d contexts
    kubectl config delete-context "k3d-$CLUSTER_NAME" --ignore-not-found=true || true
    
    # Reset to default context if available
    if kubectl config get-contexts | grep -q "default"; then
        kubectl config use-context default || true
    fi
    
    log_success "kubectl context reset"
}

# Clean up logs
cleanup_logs() {
    log_info "Cleaning up log files..."
    
    # Clean up system logs related to Wasmbed
    sudo journalctl --vacuum-time=1d || true
    
    log_success "Log files cleaned up"
}

# Verify cleanup
verify_cleanup() {
    log_info "Verifying cleanup..."
    
    # Check if cluster exists
    if k3d cluster list | grep -q "$CLUSTER_NAME"; then
        log_warning "Cluster $CLUSTER_NAME still exists"
    else
        log_success "Cluster $CLUSTER_NAME removed"
    fi
    
    # Check if namespace exists
    if kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
        log_warning "Namespace $NAMESPACE still exists"
    else
        log_success "Namespace $NAMESPACE removed"
    fi
    
    # Check if CRDs exist
    if kubectl get crd | grep -q wasmbed; then
        log_warning "Wasmbed CRDs still exist"
    else
        log_success "Wasmbed CRDs removed"
    fi
    
    # Check Docker images
    if docker images | grep -q wasmbed; then
        log_warning "Wasmbed Docker images still exist"
    else
        log_success "Wasmbed Docker images removed"
    fi
    
    log_success "Cleanup verification completed"
}

# Show cleanup summary
show_summary() {
    log_info "Cleanup Summary:"
    echo ""
    echo "Removed:"
    echo "  - k3d cluster: $CLUSTER_NAME"
    echo "  - Kubernetes namespace: $NAMESPACE"
    echo "  - All Wasmbed CRDs and resources"
    echo "  - All Docker images and containers"
    echo "  - All certificates and build artifacts"
    echo "  - All QEMU processes"
    echo ""
    log_success "Environment completely reset!"
    log_info "You can now run deploy-complete.sh to start fresh"
}

# Main cleanup function
main() {
    log_info "Starting Wasmbed complete cleanup..."
    
    confirm_cleanup "$@"
    
    cleanup_kubernetes
    cleanup_cluster
    cleanup_docker
    cleanup_certificates
    cleanup_build_artifacts
    cleanup_qemu
    reset_kubectl
    cleanup_logs
    verify_cleanup
    show_summary
    
    log_success "Wasmbed platform completely cleaned up!"
}

# Run main function
main "$@"
