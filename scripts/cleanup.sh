#!/bin/bash
# Wasmbed Complete System Cleanup
# Cleans everything: Kubernetes, Docker, k3d, certificates, build files, QEMU

set -euo pipefail

# Source logging library
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/logging.sh"

# Configuration
CLUSTER_NAME="wasmbed"
NAMESPACE="wasmbed"
LOG_LEVEL=${LOG_LEVEL:-3}

# Initialize logging
init_logging "$@"

log_header "Wasmbed Complete System Cleanup"

# Function to clean Kubernetes resources
clean_kubernetes() {
    log_step "Cleaning Kubernetes resources"
    
    # Delete applications and devices
    log_info "Deleting applications and devices"
    kubectl delete applications --all -n "$NAMESPACE" 2>/dev/null || true
    kubectl delete devices --all -n "$NAMESPACE" 2>/dev/null || true
    
    # Delete deployments
    log_info "Deleting deployments"
    kubectl delete statefulset wasmbed-gateway -n "$NAMESPACE" 2>/dev/null || true
    kubectl delete deployment wasmbed-controller -n "$NAMESPACE" 2>/dev/null || true
    
    # Delete services
    log_info "Deleting services"
    kubectl delete service wasmbed-gateway-service -n "$NAMESPACE" 2>/dev/null || true
    
    # Delete secrets
    log_info "Deleting secrets"
    kubectl delete secret wasmbed-tls-secret-rsa wasmbed-ca-secret-rsa -n "$NAMESPACE" 2>/dev/null || true
    
    # Delete RBAC
    log_info "Deleting RBAC resources"
    kubectl delete clusterrole wasmbed-controller wasmbed-controller-role wasmbed-device-access 2>/dev/null || true
    kubectl delete clusterrolebinding wasmbed-controller wasmbed-controller-binding wasmbed-gateway-device-access-binding 2>/dev/null || true
    kubectl delete serviceaccount wasmbed-controller wasmbed-controller-sa wasmbed-gateway -n "$NAMESPACE" 2>/dev/null || true
    
    # Delete CRDs
    log_info "Deleting CRDs"
    kubectl delete crd applications.wasmbed.github.io devices.wasmbed.github.io 2>/dev/null || true
    
    # Delete namespace
    log_info "Deleting namespace"
    kubectl delete namespace "$NAMESPACE" 2>/dev/null || true
    
    log_success "Kubernetes resources cleaned"
}

# Function to clean Docker images
clean_docker() {
    log_step "Cleaning Docker images"
    
    # Remove wasmbed images
    log_info "Removing wasmbed Docker images"
    docker rmi wasmbed-gateway:latest 2>/dev/null || true
    docker rmi wasmbed-controller:latest 2>/dev/null || true
    
    # Clean up dangling images
    log_info "Cleaning dangling images"
    docker image prune -f
    
    log_success "Docker images cleaned"
}

# Function to clean k3d cluster
clean_k3d() {
    log_step "Cleaning k3d cluster"
    
    if k3d cluster get "$CLUSTER_NAME" >/dev/null 2>&1; then
        log_info "Deleting k3d cluster: $CLUSTER_NAME"
        execute_cmd "k3d cluster delete $CLUSTER_NAME" "Deleting k3d cluster"
    else
        log_info "k3d cluster $CLUSTER_NAME not found"
    fi
    
    log_success "k3d cluster cleaned"
}

# Function to clean TLS certificates
clean_certificates() {
    log_step "Cleaning TLS certificates"
    
    # Remove certificate files
    log_info "Removing certificate files"
    rm -f *.pem 2>/dev/null || true
    rm -rf certs/ 2>/dev/null || true
    
    log_success "TLS certificates cleaned"
}

# Function to clean QEMU files
clean_qemu() {
    log_step "Cleaning QEMU files"
    
    # Stop QEMU processes
    log_info "Stopping QEMU processes"
    pkill -f qemu-system 2>/dev/null || true
    
    # Remove QEMU directory
    log_info "Removing QEMU directory"
    rm -rf qemu/ 2>/dev/null || true
    
    log_success "QEMU files cleaned"
}

# Function to clean build files
clean_build() {
    log_step "Cleaning build files"
    
    # Clean Cargo build
    log_info "Cleaning Cargo build files"
    execute_cmd "cargo clean" "Cleaning Cargo build"
    
    # Clean target directory
    log_info "Removing target directory"
    rm -rf target/ 2>/dev/null || true
    
    log_success "Build files cleaned"
}

# Function to clean temporary files
clean_temp() {
    log_step "Cleaning temporary files"
    
    # Remove test reports
    log_info "Removing test reports"
    rm -f test-report-*.txt 2>/dev/null || true
    
    # Remove backup files
    log_info "Removing backup files"
    rm -rf backups/ 2>/dev/null || true
    
    # Remove log files
    log_info "Removing log files"
    rm -f *.log 2>/dev/null || true
    
    # Remove temporary directories
    log_info "Removing temporary directories"
    rm -rf tmp/ 2>/dev/null || true
    
    log_success "Temporary files cleaned"
}

# Function to clean all
clean_all() {
    log_header "Starting Complete System Cleanup"
    
    clean_kubernetes
    clean_docker
    clean_k3d
    clean_certificates
    clean_qemu
    clean_build
    clean_temp
    
    log_success "Complete system cleanup finished!"
    log_info "System is now in a clean state"
}

# Function to verify cleanup
verify_cleanup() {
    log_step "Verifying cleanup"
    
    # Check Kubernetes resources
    log_info "Checking Kubernetes resources"
    if kubectl get all -A | grep wasmbed >/dev/null 2>&1; then
        log_warn "Some wasmbed resources still exist"
        kubectl get all -A | grep wasmbed
    else
        log_success "No wasmbed Kubernetes resources found"
    fi
    
    # Check CRDs
    log_info "Checking CRDs"
    if kubectl get crd | grep wasmbed >/dev/null 2>&1; then
        log_warn "Some wasmbed CRDs still exist"
        kubectl get crd | grep wasmbed
    else
        log_success "No wasmbed CRDs found"
    fi
    
    # Check Docker images
    log_info "Checking Docker images"
    if docker images | grep wasmbed >/dev/null 2>&1; then
        log_warn "Some wasmbed Docker images still exist"
        docker images | grep wasmbed
    else
        log_success "No wasmbed Docker images found"
    fi
    
    # Check TLS certificates
    log_info "Checking TLS certificates"
    if ls *.pem >/dev/null 2>&1; then
        log_warn "Some TLS certificates still exist"
        ls -la *.pem
    else
        log_success "No TLS certificates found"
    fi
    
    # Check QEMU files
    log_info "Checking QEMU files"
    if [ -d "qemu" ]; then
        log_warn "QEMU directory still exists"
        ls -la qemu/
    else
        log_success "No QEMU directory found"
    fi
    
    log_success "Cleanup verification completed"
}

# Main cleanup function
main() {
    clean_all
    verify_cleanup
}

# Handle script arguments
case "${1:-clean}" in
    "clean"|"")
        main
        ;;
    "k8s")
        clean_kubernetes
        ;;
    "docker")
        clean_docker
        ;;
    "k3d")
        clean_k3d
        ;;
    "certs")
        clean_certificates
        ;;
    "qemu")
        clean_qemu
        ;;
    "build")
        clean_build
        ;;
    "temp")
        clean_temp
        ;;
    "verify")
        verify_cleanup
        ;;
    "help"|"-h"|"--help")
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  clean     - Complete system cleanup (default)"
        echo "  k8s       - Clean Kubernetes resources only"
        echo "  docker    - Clean Docker images only"
        echo "  k3d       - Clean k3d cluster only"
        echo "  certs     - Clean TLS certificates only"
        echo "  qemu      - Clean QEMU files only"
        echo "  build     - Clean build files only"
        echo "  temp      - Clean temporary files only"
        echo "  verify    - Verify cleanup"
        echo "  help      - Show this help"
        echo ""
        echo "Environment variables:"
        echo "  LOG_LEVEL - Set logging level (1=error, 2=warn, 3=info, 4=debug)"
        ;;
    *)
        log_error "Unknown command: $1"
        echo "Use '$0 help' for usage information"
        exit 1
        ;;
esac