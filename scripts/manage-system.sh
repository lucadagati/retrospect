#!/bin/bash
# Wasmbed Complete System Management
# Unified script for clean, deploy, and ROS 2 integration

set -euo pipefail

# Source logging library
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/logging.sh"

# Configuration
CLUSTER_NAME="wasmbed"
NAMESPACE="wasmbed"
ROS2_NAMESPACE="ros2-system"
ROS2_APPS_NAMESPACE="ros2-apps"
LOG_LEVEL=${LOG_LEVEL:-3}

# Initialize logging
init_logging "$@"

log_header "Wasmbed Complete System Management"

# Function to check prerequisites
check_prerequisites() {
    log_step "Checking prerequisites"
    
    # Check required tools
    local tools=("kubectl" "k3d" "docker" "cargo" "curl" "openssl")
    for tool in "${tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            log_error "Required tool not found: $tool"
            return 1
        fi
    done
    
    # Check Docker daemon
    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running"
        return 1
    fi
    
    # Check k3d
    if ! k3d version &> /dev/null; then
        log_error "k3d is not properly installed"
        return 1
    fi
    
    log_success "All prerequisites satisfied"
    return 0
}

# Function to clean everything
clean_all() {
    log_header "🧹 Complete System Cleanup"
    
    # Clean Kubernetes resources
    log_step "Cleaning Kubernetes resources"
    
    # Clean ROS 2 resources first
    log_info "Cleaning ROS 2 resources"
    kubectl delete ros2topics --all -n "$ROS2_APPS_NAMESPACE" 2>/dev/null || true
    kubectl delete ros2services --all -n "$ROS2_APPS_NAMESPACE" 2>/dev/null || true
    kubectl delete namespace "$ROS2_APPS_NAMESPACE" 2>/dev/null || true
    kubectl delete namespace "$ROS2_NAMESPACE" 2>/dev/null || true
    
    # Clean main Wasmbed resources
    log_info "Cleaning main Wasmbed resources"
    kubectl delete applications --all -n "$NAMESPACE" 2>/dev/null || true
    kubectl delete devices --all -n "$NAMESPACE" 2>/dev/null || true
    kubectl delete statefulset wasmbed-gateway -n "$NAMESPACE" 2>/dev/null || true
    kubectl delete deployment wasmbed-controller -n "$NAMESPACE" 2>/dev/null || true
    kubectl delete deployment microros-agent -n "$ROS2_NAMESPACE" 2>/dev/null || true
    kubectl delete deployment wasmbed-microros-bridge -n "$ROS2_NAMESPACE" 2>/dev/null || true
    
    # Clean services
    kubectl delete service wasmbed-gateway-service -n "$NAMESPACE" 2>/dev/null || true
    kubectl delete service microros-agent -n "$ROS2_NAMESPACE" 2>/dev/null || true
    kubectl delete service wasmbed-microros-bridge -n "$ROS2_NAMESPACE" 2>/dev/null || true
    
    # Clean secrets
    kubectl delete secret wasmbed-tls-secret-rsa wasmbed-ca-secret-rsa -n "$NAMESPACE" 2>/dev/null || true
    
    # Clean RBAC
    kubectl delete clusterrole wasmbed-controller wasmbed-controller-role wasmbed-device-access ros2-cluster-role 2>/dev/null || true
    kubectl delete clusterrolebinding wasmbed-controller wasmbed-controller-binding wasmbed-gateway-device-access-binding ros2-cluster-rolebinding 2>/dev/null || true
    kubectl delete serviceaccount wasmbed-controller wasmbed-controller-sa wasmbed-gateway ros2-service-account -n "$NAMESPACE" 2>/dev/null || true
    
    # Clean CRDs
    kubectl delete crd applications.wasmbed.github.io devices.wasmbed.github.io ros2topics.wasmbed.io ros2services.wasmbed.io 2>/dev/null || true
    
    # Clean namespaces
    kubectl delete namespace "$NAMESPACE" 2>/dev/null || true
    
    # Clean Docker images
    log_step "Cleaning Docker images"
    docker rmi wasmbed-gateway:latest 2>/dev/null || true
    docker rmi wasmbed-controller:latest 2>/dev/null || true
    docker rmi wasmbed-microros-bridge:latest 2>/dev/null || true
    docker image prune -f
    
    # Clean k3d cluster
    log_step "Cleaning k3d cluster"
    if k3d cluster get "$CLUSTER_NAME" >/dev/null 2>&1; then
        execute_cmd "k3d cluster delete $CLUSTER_NAME" "Deleting k3d cluster"
    fi
    
    # Clean certificates
    log_step "Cleaning TLS certificates"
    rm -f *.pem 2>/dev/null || true
    rm -rf certs/ 2>/dev/null || true
    
    # Clean QEMU files
    log_step "Cleaning QEMU files"
    pkill -f qemu-system 2>/dev/null || true
    rm -rf qemu/ 2>/dev/null || true
    
    # Clean build files
    log_step "Cleaning build files"
    execute_cmd "cargo clean" "Cleaning Cargo build"
    rm -rf target/ 2>/dev/null || true
    
    # Clean temporary files
    log_step "Cleaning temporary files"
    rm -f test-report-*.txt *.log 2>/dev/null || true
    rm -rf backups/ tmp/ 2>/dev/null || true
    
    log_success "Complete system cleanup finished!"
}

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
    
    execute_cmd "k3d cluster create $CLUSTER_NAME --port 8080:30080@loadbalancer --port 4423:30423@loadbalancer --port 8888:30888@loadbalancer --port 9090:30990@loadbalancer" "Creating k3d cluster"
}

# Function to build Docker images
build_images() {
    log_step "Building Docker images"
    
    # Build gateway
    execute_cmd "docker build -t wasmbed-gateway:latest -f Dockerfile.gateway ." "Building gateway image"
    
    # Build controller
    execute_cmd "docker build -t wasmbed-controller:latest -f Dockerfile.controller ." "Building controller image"
    
    # Build microROS bridge
    execute_cmd "docker build -t wasmbed-microros-bridge:latest -f crates/wasmbed-microros-bridge/Dockerfile ." "Building microROS bridge image"
    
    # Load images into k3d
    execute_cmd "k3d image import wasmbed-gateway:latest wasmbed-controller:latest wasmbed-microros-bridge:latest -c $CLUSTER_NAME" "Loading images into cluster"
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

# Function to deploy main Wasmbed resources
deploy_main_resources() {
    log_step "Deploying main Wasmbed resources"
    
    # Create namespace
    execute_cmd "kubectl create namespace $NAMESPACE --dry-run=client -o yaml | kubectl apply -f -" "Creating main namespace"
    
    # Deploy CRDs
    execute_cmd "kubectl apply -f resources/k8s/crds/" "Deploying main CRDs"
    
    # Deploy RBAC
    execute_cmd "kubectl apply -f resources/k8s/rbac/" "Deploying main RBAC"
    
    # Deploy gateway config
    execute_cmd "kubectl apply -f resources/k8s/wasmbed-gateway-configmap.yaml" "Deploying gateway config"
    
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
}

# Function to deploy ROS 2 resources
deploy_ros2_resources() {
    log_step "Deploying ROS 2 resources"
    
    # Deploy ROS 2 namespaces
    execute_cmd "kubectl apply -f resources/k8s/ros2/namespace.yaml" "Creating ROS 2 namespaces"
    
    # Deploy ROS 2 RBAC
    execute_cmd "kubectl apply -f resources/k8s/ros2/rbac.yaml" "Deploying ROS 2 RBAC"
    
    # Deploy ROS 2 configuration
    execute_cmd "kubectl apply -f resources/k8s/ros2/configmap.yaml" "Deploying ROS 2 configuration"
    
    # Deploy microROS agent
    execute_cmd "kubectl apply -f resources/k8s/ros2/microros-agent.yaml" "Deploying microROS agent"
    
    # Deploy microROS bridge
    execute_cmd "kubectl apply -f resources/k8s/ros2/wasmbed-microros-bridge.yaml" "Deploying microROS bridge"
    
    # Deploy ROS 2 CRDs
    execute_cmd "kubectl apply -f resources/k8s/ros2/crds/" "Deploying ROS 2 CRDs"
    
    # Deploy example ROS 2 application
    execute_cmd "kubectl apply -f resources/k8s/ros2/examples/drone-ros2-app.yaml" "Deploying example ROS 2 application"
}

# Function to wait for deployments
wait_for_deployments() {
    log_step "Waiting for deployments to be ready"
    
    # Wait for main deployments
    execute_cmd "kubectl wait --for=condition=available --timeout=300s deployment/wasmbed-controller -n $NAMESPACE" "Waiting for controller"
    execute_cmd "kubectl wait --for=condition=ready --timeout=300s pod -l app=wasmbed-gateway -n $NAMESPACE" "Waiting for gateway"
    
    # Wait for ROS 2 deployments
    execute_cmd "kubectl wait --for=condition=available --timeout=300s deployment/microros-agent -n $ROS2_NAMESPACE" "Waiting for microROS agent"
    execute_cmd "kubectl wait --for=condition=available --timeout=300s deployment/wasmbed-microros-bridge -n $ROS2_NAMESPACE" "Waiting for microROS bridge"
}

# Function to verify deployment
verify_deployment() {
    log_step "Verifying deployment"
    
    # Check main pods
    log_info "Checking main pod status"
    kubectl get pods -n "$NAMESPACE"
    
    # Check ROS 2 pods
    log_info "Checking ROS 2 pod status"
    kubectl get pods -n "$ROS2_NAMESPACE"
    kubectl get pods -n "$ROS2_APPS_NAMESPACE"
    
    # Check services
    log_info "Checking services"
    kubectl get services -n "$NAMESPACE"
    kubectl get services -n "$ROS2_NAMESPACE"
    
    # Check CRDs
    log_info "Checking CRDs"
    kubectl get crd | grep wasmbed
    
    # Check ROS 2 resources
    log_info "Checking ROS 2 resources"
    kubectl get ros2topics -n "$ROS2_APPS_NAMESPACE" 2>/dev/null || log_info "No ROS2Topics found"
    kubectl get ros2services -n "$ROS2_APPS_NAMESPACE" 2>/dev/null || log_info "No ROS2Services found"
    
    # Test HTTP endpoints
    log_info "Testing HTTP endpoints"
    if curl -s -f "http://localhost:8080/health" >/dev/null; then
        log_success "Main HTTP endpoint accessible"
    else
        log_warn "Main HTTP endpoint not accessible (may need port forwarding)"
    fi
    
    if curl -s -f "http://localhost:8888/health" >/dev/null; then
        log_success "microROS agent HTTP endpoint accessible"
    else
        log_warn "microROS agent HTTP endpoint not accessible (may need port forwarding)"
    fi
}

# Function to show deployment info
show_deployment_info() {
    log_header "Deployment Information"
    
    echo -e "${CYAN}Cluster:${NC} $CLUSTER_NAME"
    echo -e "${CYAN}Main Namespace:${NC} $NAMESPACE"
    echo -e "${CYAN}ROS 2 System Namespace:${NC} $ROS2_NAMESPACE"
    echo -e "${CYAN}ROS 2 Apps Namespace:${NC} $ROS2_APPS_NAMESPACE"
    echo ""
    echo -e "${CYAN}Endpoints:${NC}"
    echo "  Main HTTP API: http://localhost:8080"
    echo "  Main TLS API: https://localhost:4423"
    echo "  microROS Agent: http://localhost:8888"
    echo "  ROS Bridge WebSocket: http://localhost:9090"
    echo ""
    echo -e "${CYAN}Useful Commands:${NC}"
    echo "  kubectl get pods -A | grep wasmbed"
    echo "  kubectl get ros2topics -n $ROS2_APPS_NAMESPACE"
    echo "  kubectl get ros2services -n $ROS2_APPS_NAMESPACE"
    echo "  kubectl logs -f -l app=wasmbed-gateway -n $NAMESPACE"
    echo "  kubectl logs -f -l app=wasmbed-microros-bridge -n $ROS2_NAMESPACE"
    echo ""
    echo -e "${CYAN}To test the deployment:${NC}"
    echo "  ./scripts/test-complete-system.sh"
    echo "  curl http://localhost:8080/health"
    echo "  curl http://localhost:8888/health"
}

# Function to run tests
run_tests() {
    log_header "Running Complete System Tests"
    
    # Run the complete test suite
    execute_cmd "./scripts/test-complete-system.sh" "Running complete system tests"
}

# Main deployment function
deploy_all() {
    log_header "🚀 Starting Complete Wasmbed Platform Deployment"
    
    # Check prerequisites
    check_prerequisites || exit 1
    
    # Deploy steps
    create_cluster
    build_images
    generate_certificates
    deploy_main_resources
    deploy_ros2_resources
    wait_for_deployments
    verify_deployment
    show_deployment_info
    
    log_success "Complete deployment finished successfully!"
    log_info "Platform is ready for use with ROS 2 integration"
}

# Function to clean and deploy
clean_and_deploy() {
    log_header "🧹🔄 Clean and Deploy Complete System"
    
    clean_all
    deploy_all
    
    log_success "Clean and deploy completed successfully!"
}

# Handle script arguments
case "${1:-help}" in
    "clean")
        clean_all
        ;;
    "deploy")
        deploy_all
        ;;
    "clean-deploy"|"clean-and-deploy")
        clean_and_deploy
        ;;
    "test")
        run_tests
        ;;
    "verify")
        verify_deployment
        ;;
    "info")
        show_deployment_info
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
    "main")
        deploy_main_resources
        ;;
    "ros2")
        deploy_ros2_resources
        ;;
    "help"|"-h"|"--help")
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  clean-deploy  - Clean everything and deploy complete system (default)"
        echo "  clean         - Clean everything"
        echo "  deploy        - Deploy complete system"
        echo "  test          - Run complete system tests"
        echo "  verify        - Verify deployment"
        echo "  info          - Show deployment information"
        echo ""
        echo "Sub-commands:"
        echo "  cluster       - Create k3d cluster only"
        echo "  images        - Build Docker images only"
        echo "  certs         - Generate TLS certificates only"
        echo "  main          - Deploy main Wasmbed resources only"
        echo "  ros2          - Deploy ROS 2 resources only"
        echo "  help          - Show this help"
        echo ""
        echo "Environment variables:"
        echo "  LOG_LEVEL     - Set logging level (1=error, 2=warn, 3=info, 4=debug)"
        ;;
    *)
        log_error "Unknown command: $1"
        echo "Use '$0 help' for usage information"
        exit 1
        ;;
esac
