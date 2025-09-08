#!/bin/bash

# Wasmbed Troubleshooting Script
# Comprehensive diagnostic and troubleshooting tool for the Wasmbed platform

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

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    local missing_deps=()
    
    # Check required tools
    command -v docker >/dev/null 2>&1 || missing_deps+=("docker")
    command -v k3d >/dev/null 2>&1 || missing_deps+=("k3d")
    command -v kubectl >/dev/null 2>&1 || missing_deps+=("kubectl")
    command -v cargo >/dev/null 2>&1 || missing_deps+=("cargo")
    command -v openssl >/dev/null 2>&1 || missing_deps+=("openssl")
    command -v python3 >/dev/null 2>&1 || missing_deps+=("python3")
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_error "Please install missing dependencies and try again"
        return 1
    fi
    
    log_success "All prerequisites satisfied"
    return 0
}

# Check cluster status
check_cluster_status() {
    log_info "Checking cluster status..."
    
    # Check if k3d cluster exists
    if ! k3d cluster list | grep -q "$CLUSTER_NAME"; then
        log_error "k3d cluster '$CLUSTER_NAME' not found"
        log_info "Run: k3d cluster create $CLUSTER_NAME"
        return 1
    fi
    
    # Check cluster health
    if ! kubectl cluster-info >/dev/null 2>&1; then
        log_error "Cannot connect to cluster"
        log_info "Check kubeconfig: kubectl config current-context"
        return 1
    fi
    
    log_success "Cluster is healthy"
    return 0
}

# Check namespace and resources
check_namespace_resources() {
    log_info "Checking namespace and resources..."
    
    # Check if namespace exists
    if ! kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
        log_error "Namespace '$NAMESPACE' not found"
        log_info "Run: kubectl create namespace $NAMESPACE"
        return 1
    fi
    
    # Check pods
    log_info "Pod status:"
    kubectl get pods -n "$NAMESPACE" || {
        log_error "Failed to get pods"
        return 1
    }
    
    # Check services
    log_info "Service status:"
    kubectl get services -n "$NAMESPACE" || {
        log_error "Failed to get services"
        return 1
    }
    
    # Check CRDs
    log_info "CRD status:"
    kubectl get crd | grep wasmbed || {
        log_warning "No Wasmbed CRDs found"
    }
    
    # Check devices
    log_info "Device status:"
    kubectl get devices -n "$NAMESPACE" || {
        log_warning "No devices found"
    }
    
    # Check applications
    log_info "Application status:"
    kubectl get applications -n "$NAMESPACE" || {
        log_warning "No applications found"
    }
    
    log_success "Namespace resources checked"
    return 0
}

# Check pod health
check_pod_health() {
    log_info "Checking pod health..."
    
    local failed_pods=()
    
    # Get all pods in namespace
    local pods
    pods=$(kubectl get pods -n "$NAMESPACE" --no-headers -o custom-columns=":metadata.name" 2>/dev/null || true)
    
    if [ -z "$pods" ]; then
        log_warning "No pods found in namespace '$NAMESPACE'"
        return 0
    fi
    
    # Check each pod
    while IFS= read -r pod; do
        if [ -n "$pod" ]; then
            local status
            status=$(kubectl get pod "$pod" -n "$NAMESPACE" --no-headers -o custom-columns=":status.phase" 2>/dev/null || echo "Unknown")
            
            case "$status" in
                "Running")
                    log_success "Pod '$pod' is running"
                    ;;
                "Pending")
                    log_warning "Pod '$pod' is pending"
                    ;;
                "Failed"|"CrashLoopBackOff"|"Error")
                    log_error "Pod '$pod' has status: $status"
                    failed_pods+=("$pod")
                    ;;
                *)
                    log_warning "Pod '$pod' has status: $status"
                    ;;
            esac
        fi
    done <<< "$pods"
    
    # Show logs for failed pods
    if [ ${#failed_pods[@]} -ne 0 ]; then
        log_info "Checking logs for failed pods..."
        for pod in "${failed_pods[@]}"; do
            log_info "Logs for pod '$pod':"
            kubectl logs "$pod" -n "$NAMESPACE" --tail=20 || true
        done
    fi
    
    return 0
}

# Check certificates and secrets
check_certificates() {
    log_info "Checking certificates and secrets..."
    
    # Check TLS secrets
    log_info "TLS secrets:"
    kubectl get secrets -n "$NAMESPACE" | grep tls || {
        log_warning "No TLS secrets found"
    }
    
    # Check CA secrets
    log_info "CA secrets:"
    kubectl get secrets -n "$NAMESPACE" | grep ca || {
        log_warning "No CA secrets found"
    }
    
    # Check certificate files
    if [ -d "certs" ]; then
        log_info "Certificate files:"
        ls -la certs/ || true
    else
        log_warning "No certs directory found"
    fi
    
    # Validate certificate files
    if [ -f "certs/server-cert.pem" ] && [ -f "certs/server-key.pem" ]; then
        log_info "Validating server certificate..."
        if openssl x509 -in certs/server-cert.pem -text -noout >/dev/null 2>&1; then
            log_success "Server certificate is valid"
        else
            log_error "Server certificate is invalid"
        fi
        
        if openssl rsa -in certs/server-key.pem -check >/dev/null 2>&1; then
            log_success "Server private key is valid"
        else
            log_error "Server private key is invalid"
        fi
    else
        log_warning "Server certificate files not found"
    fi
    
    return 0
}

# Check network connectivity
check_network_connectivity() {
    log_info "Checking network connectivity..."
    
    # Check if gateway service is accessible
    local gateway_service
    gateway_service=$(kubectl get service wasmbed-gateway-service -n "$NAMESPACE" --no-headers -o custom-columns=":metadata.name" 2>/dev/null || echo "")
    
    if [ -n "$gateway_service" ]; then
        log_info "Testing gateway service connectivity..."
        
        # Get service details
        kubectl get service "$gateway_service" -n "$NAMESPACE" -o wide
        
        # Test internal connectivity
        log_info "Testing internal connectivity..."
        kubectl run test-connectivity --image=busybox --rm -it --restart=Never --timeout=10s -- nc -zv "$gateway_service.$NAMESPACE.svc.cluster.local" 4423 || {
            log_warning "Internal connectivity test failed"
        }
        
        # Test NodePort if available
        local nodeport
        nodeport=$(kubectl get service "$gateway_service" -n "$NAMESPACE" --no-headers -o custom-columns=":spec.ports[0].nodePort" 2>/dev/null || echo "")
        
        if [ -n "$nodeport" ] && [ "$nodeport" != "<none>" ]; then
            log_info "Testing NodePort connectivity on port $nodeport..."
            timeout 5 telnet localhost "$nodeport" || {
                log_warning "NodePort connectivity test failed"
            }
        fi
    else
        log_warning "Gateway service not found"
    fi
    
    return 0
}

# Check Docker images
check_docker_images() {
    log_info "Checking Docker images..."
    
    # Check if images exist locally
    log_info "Local Docker images:"
    docker images | grep wasmbed || {
        log_warning "No Wasmbed Docker images found locally"
    }
    
    # Check if images are imported to k3d
    log_info "k3d imported images:"
    k3d image list -c "$CLUSTER_NAME" | grep wasmbed || {
        log_warning "No Wasmbed images imported to k3d cluster"
    }
    
    return 0
}

# Check logs
check_logs() {
    log_info "Checking application logs..."
    
    # Check gateway logs
    log_info "Gateway logs (last 20 lines):"
    kubectl logs -l app=wasmbed-gateway -n "$NAMESPACE" --tail=20 || {
        log_warning "No gateway logs found"
    }
    
    # Check controller logs
    log_info "Controller logs (last 20 lines):"
    kubectl logs -l app=wasmbed-k8s-controller -n "$NAMESPACE" --tail=20 || {
        log_warning "No controller logs found"
    }
    
    return 0
}

# Generate diagnostic report
generate_diagnostic_report() {
    log_info "Generating diagnostic report..."
    
    local report_file="wasmbed-diagnostic-$(date +%Y%m%d-%H%M%S).txt"
    
    {
        echo "Wasmbed Platform Diagnostic Report"
        echo "Generated: $(date)"
        echo "=================================="
        echo ""
        
        echo "Cluster Information:"
        kubectl cluster-info || echo "Failed to get cluster info"
        echo ""
        
        echo "Node Information:"
        kubectl get nodes -o wide || echo "Failed to get node info"
        echo ""
        
        echo "Namespace Resources:"
        kubectl get all -n "$NAMESPACE" || echo "Failed to get namespace resources"
        echo ""
        
        echo "CRDs:"
        kubectl get crd | grep wasmbed || echo "No Wasmbed CRDs found"
        echo ""
        
        echo "Secrets:"
        kubectl get secrets -n "$NAMESPACE" || echo "Failed to get secrets"
        echo ""
        
        echo "Events:"
        kubectl get events -n "$NAMESPACE" --sort-by='.lastTimestamp' || echo "Failed to get events"
        echo ""
        
    } > "$report_file"
    
    log_success "Diagnostic report saved to: $report_file"
    return 0
}

# Fix common issues
fix_common_issues() {
    log_info "Attempting to fix common issues..."
    
    # Fix kubeconfig certificates
    if kubectl cluster-info >/dev/null 2>&1; then
        log_info "Kubeconfig is working"
    else
        log_info "Attempting to fix kubeconfig..."
        if [ -f "certs/k3d-ca.crt" ] && [ -f "certs/k3d-client.crt" ] && [ -f "certs/k3d-client.key" ]; then
            log_info "External certificate files found, updating kubeconfig..."
            # This would require the same Python script from deploy-complete.sh
            log_warning "Manual kubeconfig fix required - run deploy-complete.sh"
        else
            log_error "External certificate files not found"
        fi
    fi
    
    # Restart failed pods
    local failed_pods
    failed_pods=$(kubectl get pods -n "$NAMESPACE" --field-selector=status.phase=Failed --no-headers -o custom-columns=":metadata.name" 2>/dev/null || true)
    
    if [ -n "$failed_pods" ]; then
        log_info "Restarting failed pods..."
        while IFS= read -r pod; do
            if [ -n "$pod" ]; then
                log_info "Deleting failed pod: $pod"
                kubectl delete pod "$pod" -n "$NAMESPACE" || true
            fi
        done <<< "$failed_pods"
    fi
    
    # Recreate secrets if missing
    if ! kubectl get secret wasmbed-tls-secret-rsa -n "$NAMESPACE" >/dev/null 2>&1; then
        log_info "Recreating TLS secret..."
        if [ -f "certs/server-cert.pem" ] && [ -f "certs/server-key.pem" ]; then
            kubectl create secret tls wasmbed-tls-secret-rsa \
                --cert=certs/server-cert.pem \
                --key=certs/server-key.pem \
                -n "$NAMESPACE" || true
        else
            log_warning "Certificate files not found for secret recreation"
        fi
    fi
    
    if ! kubectl get secret wasmbed-ca-secret-rsa -n "$NAMESPACE" >/dev/null 2>&1; then
        log_info "Recreating CA secret..."
        if [ -f "certs/ca-cert.pem" ]; then
            kubectl create secret generic wasmbed-ca-secret-rsa \
                --from-file=ca-cert.pem=certs/ca-cert.pem \
                -n "$NAMESPACE" || true
        else
            log_warning "CA certificate file not found for secret recreation"
        fi
    fi
    
    return 0
}

# Main troubleshooting function
main() {
    log_info "Starting Wasmbed troubleshooting..."
    
    local all_checks_passed=true
    
    # Run all checks
    check_prerequisites || all_checks_passed=false
    check_cluster_status || all_checks_passed=false
    check_namespace_resources || all_checks_passed=false
    check_pod_health || all_checks_passed=false
    check_certificates || all_checks_passed=false
    check_network_connectivity || all_checks_passed=false
    check_docker_images || all_checks_passed=false
    check_logs || all_checks_passed=false
    
    # Generate diagnostic report
    generate_diagnostic_report
    
    # Attempt to fix common issues
    fix_common_issues
    
    echo ""
    if [ "$all_checks_passed" = true ]; then
        log_success "All checks passed! Platform appears to be healthy."
    else
        log_warning "Some checks failed. Review the output above for issues."
        log_info "Common solutions:"
        log_info "1. Run: ./deploy-complete.sh (for complete redeployment)"
        log_info "2. Run: kubectl delete pod <pod-name> -n $NAMESPACE (to restart specific pods)"
        log_info "3. Check logs: kubectl logs <pod-name> -n $NAMESPACE"
        log_info "4. Verify certificates: openssl x509 -in certs/server-cert.pem -text -noout"
    fi
    
    echo ""
    log_info "Troubleshooting completed. Check the diagnostic report for detailed information."
}

# Show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  --prereqs      Check prerequisites only"
    echo "  --cluster      Check cluster status only"
    echo "  --pods         Check pod health only"
    echo "  --certs        Check certificates only"
    echo "  --network      Check network connectivity only"
    echo "  --logs         Check logs only"
    echo "  --fix          Attempt to fix common issues"
    echo "  --report       Generate diagnostic report only"
    echo ""
    echo "Examples:"
    echo "  $0                    # Run all checks"
    echo "  $0 --pods            # Check pod health only"
    echo "  $0 --fix             # Fix common issues"
    echo "  $0 --report          # Generate diagnostic report"
}

# Parse command line arguments
case "${1:-}" in
    -h|--help)
        show_usage
        exit 0
        ;;
    --prereqs)
        check_prerequisites
        ;;
    --cluster)
        check_cluster_status
        ;;
    --pods)
        check_pod_health
        ;;
    --certs)
        check_certificates
        ;;
    --network)
        check_network_connectivity
        ;;
    --logs)
        check_logs
        ;;
    --fix)
        fix_common_issues
        ;;
    --report)
        generate_diagnostic_report
        ;;
    "")
        main
        ;;
    *)
        log_error "Unknown option: $1"
        show_usage
        exit 1
        ;;
esac
