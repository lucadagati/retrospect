#!/bin/bash
# Wasmbed Complete Platform Deployment
# Deploys entire platform: k3d cluster, Docker images, Kubernetes resources, TLS certificates

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

log_header "Wasmbed Complete Platform Deployment"

# Function to create k3d cluster
create_cluster() {
    log_step "Creating k3d cluster"
    
    if k3d cluster get "$CLUSTER_NAME" >/dev/null 2>&1; then
        log_warn "Cluster $CLUSTER_NAME already exists"
        read -p "Delete existing cluster? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            execute_cmd "k3d cluster delete $CLUSTER_NAME" "Deleting existing cluster"
        else
            log_info "Using existing cluster"
            return 0
        fi
    fi
    
    execute_cmd "k3d cluster create $CLUSTER_NAME --port 8080:30080@loadbalancer --port 4423:30423@loadbalancer" "Creating k3d cluster"
}

# Function to build Docker images
build_images() {
    log_step "Building Docker images"
    
    # Build gateway
    execute_cmd "docker build -t wasmbed-gateway:latest -f Dockerfile.gateway ." "Building gateway image"
    
    # Build controller
    execute_cmd "docker build -t wasmbed-controller:latest -f Dockerfile.controller ." "Building controller image"
    
    # Load images into k3d
    execute_cmd "k3d image import wasmbed-gateway:latest wasmbed-controller:latest -c $CLUSTER_NAME" "Loading images into cluster"
}

# Function to generate TLS certificates
generate_certificates() {
    log_step "Generating TLS certificates"
    
    # Create certs directory
    mkdir -p certs/
    
    # Generate CA certificate
    log_info "Generating CA certificate"
    execute_cmd "openssl genrsa -out certs/ca-key.pem 4096" "Generating CA private key"
    execute_cmd "openssl req -new -x509 -days 365 -key certs/ca-key.pem -out certs/ca-cert.pem -subj '/CN=wasmbed-ca'" "Generating CA certificate"
    
    # Generate server certificate
    log_info "Generating server certificate"
    execute_cmd "openssl genrsa -out certs/server-key.pem 4096" "Generating server private key"
    execute_cmd "openssl req -new -key certs/server-key.pem -out certs/server.csr -subj '/CN=wasmbed-gateway'" "Generating server certificate request"
    execute_cmd "openssl x509 -req -days 365 -in certs/server.csr -CA certs/ca-cert.pem -CAkey certs/ca-key.pem -out certs/server-cert.pem" "Signing server certificate"
    
    # Clean up CSR
    rm -f certs/server.csr
    
    log_success "TLS certificates generated"
}

# Function to deploy Kubernetes resources
deploy_k8s_resources() {
    log_step "Deploying Kubernetes resources"
    
    # Create namespace
    execute_cmd "kubectl create namespace $NAMESPACE --dry-run=client -o yaml | kubectl apply -f -" "Creating namespace"
    
    # Deploy CRDs
    execute_cmd "kubectl apply -f resources/k8s/crds/" "Deploying CRDs"
    
    # Deploy RBAC
    execute_cmd "kubectl apply -f resources/k8s/rbac/" "Deploying RBAC"
    
    # Create TLS secrets
    log_info "Creating TLS secrets"
    kubectl create secret tls wasmbed-tls-secret-rsa \
        --cert=certs/server-cert.pem \
        --key=certs/server-key.pem \
        -n "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -
    
    kubectl create secret generic wasmbed-ca-secret-rsa \
        --from-file=ca-cert.pem=certs/ca-cert.pem \
        -n "$NAMESPACE" --dry-run=client -o yaml | kubectl apply -f -
    
    # Deploy controller
    execute_cmd "kubectl apply -f resources/k8s/controller-deployment.yaml" "Deploying controller"
    
    # Deploy gateway
    execute_cmd "kubectl apply -f resources/k8s/111-statefulset-gateway.yaml" "Deploying gateway"
    
    # Wait for deployments
    log_info "Waiting for deployments to be ready"
    execute_cmd "kubectl wait --for=condition=available --timeout=300s deployment/wasmbed-controller -n $NAMESPACE" "Waiting for controller"
    execute_cmd "kubectl wait --for=condition=ready --timeout=300s pod -l app=wasmbed-gateway -n $NAMESPACE" "Waiting for gateway"
}

# Function to verify deployment
verify_deployment() {
    log_step "Verifying deployment"
    
    # Check pods
    log_info "Checking pod status"
    kubectl get pods -n "$NAMESPACE"
    
    # Check services
    log_info "Checking services"
    kubectl get services -n "$NAMESPACE"
    
    # Test HTTP endpoint
    log_info "Testing HTTP endpoint"
    if curl -s -f "http://localhost:8080/health" >/dev/null; then
        log_success "HTTP endpoint accessible"
    else
        log_warn "HTTP endpoint not accessible (may need port forwarding)"
    fi
    
    # Test TLS endpoint
    log_info "Testing TLS endpoint"
    if timeout 3 openssl s_client -connect localhost:4423 -servername wasmbed-gateway < /dev/null >/dev/null 2>&1; then
        log_success "TLS endpoint accessible"
    else
        log_warn "TLS endpoint not accessible (may need port forwarding)"
    fi
}

# Function to show deployment info
show_deployment_info() {
    log_header "Deployment Information"
    
    echo -e "${CYAN}Cluster:${NC} $CLUSTER_NAME"
    echo -e "${CYAN}Namespace:${NC} $NAMESPACE"
    echo -e "${CYAN}HTTP API:${NC} http://localhost:8080"
    echo -e "${CYAN}TLS API:${NC} https://localhost:4423"
    echo ""
    echo -e "${CYAN}Useful Commands:${NC}"
    echo "  kubectl get pods -n $NAMESPACE"
    echo "  kubectl logs -f -l app=wasmbed-gateway -n $NAMESPACE"
    echo "  kubectl logs -f -l app=wasmbed-controller -n $NAMESPACE"
    echo ""
    echo -e "${CYAN}To test the deployment:${NC}"
    echo "  ./scripts/app.sh test"
}

# Main deployment function
main() {
    log_header "Starting Wasmbed Platform Deployment"
    
    # Check prerequisites
    check_prerequisites || exit 1
    
    # Deploy steps
    create_cluster
    build_images
    generate_certificates
    deploy_k8s_resources
    verify_deployment
    show_deployment_info
    
    log_success "Deployment completed successfully!"
    log_info "Platform is ready for use"
}

# Handle script arguments
case "${1:-deploy}" in
    "deploy"|"")
        main
        ;;
    "cluster")
        create_cluster
        ;;
    "images")
        build_images
        ;;
    "certs")
        generate_certificates
        ;;
    "k8s")
        deploy_k8s_resources
        ;;
    "verify")
        verify_deployment
        ;;
    "info")
        show_deployment_info
        ;;
    "help"|"-h"|"--help")
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  deploy    - Full deployment (default)"
        echo "  cluster   - Create k3d cluster only"
        echo "  images    - Build Docker images only"
        echo "  certs     - Generate TLS certificates only"
        echo "  k8s       - Deploy Kubernetes resources only"
        echo "  verify    - Verify deployment"
        echo "  info      - Show deployment information"
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