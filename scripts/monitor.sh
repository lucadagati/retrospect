#!/bin/bash
# Wasmbed System Monitoring
# Handles monitoring, logging, and system management

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

log_header "Wasmbed System Monitoring"

# Function to show system status
show_status() {
    log_step "System Status Overview"
    
    echo -e "${CYAN}Cluster Information:${NC}"
    kubectl cluster-info
    
    echo -e "\n${CYAN}Namespace Resources:${NC}"
    kubectl get all -n "$NAMESPACE"
    
    echo -e "\n${CYAN}CRDs:${NC}"
    kubectl get crd | grep wasmbed || echo "No wasmbed CRDs found"
    
    echo -e "\n${CYAN}Pods Status:${NC}"
    kubectl get pods -n "$NAMESPACE" -o wide
    
    echo -e "\n${CYAN}Services:${NC}"
    kubectl get services -n "$NAMESPACE"
    
    echo -e "\n${CYAN}Node Resources:${NC}"
    kubectl top nodes 2>/dev/null || echo "Metrics server not available"
    
    echo -e "\n${CYAN}Pod Resources:${NC}"
    kubectl top pods -n "$NAMESPACE" 2>/dev/null || echo "Metrics server not available"
}

# Function to show logs
show_logs() {
    local component="${1:-all}"
    local lines="${2:-50}"
    
    log_step "Showing logs for $component"
    
    case "$component" in
        "gateway"|"gw")
            kubectl logs -l app=wasmbed-gateway -n "$NAMESPACE" --tail="$lines" -f
            ;;
        "controller"|"ctrl")
            kubectl logs -l app=wasmbed-controller -n "$NAMESPACE" --tail="$lines" -f
            ;;
        "all")
            echo -e "${CYAN}Gateway Logs:${NC}"
            kubectl logs -l app=wasmbed-gateway -n "$NAMESPACE" --tail="$lines"
            echo -e "\n${CYAN}Controller Logs:${NC}"
            kubectl logs -l app=wasmbed-controller -n "$NAMESPACE" --tail="$lines"
            ;;
        *)
            log_error "Unknown component: $component"
            echo "Available components: gateway, controller, all"
            return 1
            ;;
    esac
}

# Function to perform health check
health_check() {
    log_step "Performing health check"
    
    local health_status=0
    
    # Check cluster connectivity
    if kubectl cluster-info >/dev/null 2>&1; then
        log_success "Cluster connectivity: OK"
    else
        log_error "Cluster connectivity: FAILED"
        health_status=1
    fi
    
    # Check namespace
    if kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
        log_success "Namespace: OK"
    else
        log_error "Namespace: FAILED"
        health_status=1
    fi
    
    # Check pods
    local gateway_pods=$(kubectl get pods -n "$NAMESPACE" -l app=wasmbed-gateway --no-headers | wc -l)
    local controller_pods=$(kubectl get pods -n "$NAMESPACE" -l app=wasmbed-controller --no-headers | wc -l)
    
    if [ "$gateway_pods" -gt 0 ]; then
        log_success "Gateway pods: OK ($gateway_pods running)"
    else
        log_error "Gateway pods: FAILED"
        health_status=1
    fi
    
    if [ "$controller_pods" -gt 0 ]; then
        log_success "Controller pods: OK ($controller_pods running)"
    else
        log_error "Controller pods: FAILED"
        health_status=1
    fi
    
    # Check services
    if kubectl get service wasmbed-gateway-service -n "$NAMESPACE" >/dev/null 2>&1; then
        log_success "Gateway service: OK"
    else
        log_error "Gateway service: FAILED"
        health_status=1
    fi
    
    # Check API endpoints
    if curl -s -f "http://localhost:8080/health" >/dev/null 2>&1; then
        log_success "HTTP API: OK"
    else
        log_warn "HTTP API: Not accessible (may need port forwarding)"
    fi
    
    if timeout 3 openssl s_client -connect localhost:4423 -servername wasmbed-gateway < /dev/null >/dev/null 2>&1; then
        log_success "TLS API: OK"
    else
        log_warn "TLS API: Not accessible (may need port forwarding)"
    fi
    
    # Check CRDs
    local crd_count=$(kubectl get crd | grep wasmbed | wc -l)
    if [ "$crd_count" -ge 2 ]; then
        log_success "CRDs: OK ($crd_count found)"
    else
        log_error "CRDs: FAILED ($crd_count found, expected 2+)"
        health_status=1
    fi
    
    if [ $health_status -eq 0 ]; then
        log_success "Overall health: HEALTHY"
    else
        log_error "Overall health: UNHEALTHY"
    fi
    
    return $health_status
}

# Function to monitor resources
monitor_resources() {
    log_step "Monitoring resource usage"
    
    while true; do
        clear
        log_header "Resource Monitoring - $(date)"
        
        echo -e "${CYAN}Node Resources:${NC}"
        kubectl top nodes 2>/dev/null || echo "Metrics server not available"
        
        echo -e "\n${CYAN}Pod Resources:${NC}"
        kubectl top pods -n "$NAMESPACE" 2>/dev/null || echo "Metrics server not available"
        
        echo -e "\n${CYAN}Pod Status:${NC}"
        kubectl get pods -n "$NAMESPACE" -o wide
        
        echo -e "\n${CYAN}Press Ctrl+C to stop monitoring${NC}"
        sleep 5
    done
}

# Function to restart components
restart_component() {
    local component="$1"
    
    log_step "Restarting $component"
    
    case "$component" in
        "gateway"|"gw")
            execute_cmd "kubectl rollout restart statefulset/wasmbed-gateway -n $NAMESPACE" "Restarting gateway"
            execute_cmd "kubectl rollout status statefulset/wasmbed-gateway -n $NAMESPACE" "Waiting for gateway restart"
            ;;
        "controller"|"ctrl")
            execute_cmd "kubectl rollout restart deployment/wasmbed-controller -n $NAMESPACE" "Restarting controller"
            execute_cmd "kubectl rollout status deployment/wasmbed-controller -n $NAMESPACE" "Waiting for controller restart"
            ;;
        "all")
            execute_cmd "kubectl rollout restart deployment/wasmbed-controller -n $NAMESPACE" "Restarting controller"
            execute_cmd "kubectl rollout restart statefulset/wasmbed-gateway -n $NAMESPACE" "Restarting gateway"
            execute_cmd "kubectl rollout status deployment/wasmbed-controller -n $NAMESPACE" "Waiting for controller restart"
            execute_cmd "kubectl rollout status statefulset/wasmbed-gateway -n $NAMESPACE" "Waiting for gateway restart"
            ;;
        *)
            log_error "Unknown component: $component"
            echo "Available components: gateway, controller, all"
            return 1
            ;;
    esac
    
    log_success "$component restarted successfully"
}

# Function to scale components
scale_component() {
    local component="$1"
    local replicas="$2"
    
    log_step "Scaling $component to $replicas replicas"
    
    case "$component" in
        "gateway"|"gw")
            execute_cmd "kubectl scale statefulset/wasmbed-gateway --replicas=$replicas -n $NAMESPACE" "Scaling gateway"
            ;;
        "controller"|"ctrl")
            execute_cmd "kubectl scale deployment/wasmbed-controller --replicas=$replicas -n $NAMESPACE" "Scaling controller"
            ;;
        *)
            log_error "Unknown component: $component"
            echo "Available components: gateway, controller"
            return 1
            ;;
    esac
    
    log_success "$component scaled to $replicas replicas"
}

# Function to backup platform
backup_platform() {
    local backup_dir="backups/$(date +%Y%m%d-%H%M%S)"
    
    log_step "Creating platform backup"
    
    execute_cmd "mkdir -p $backup_dir" "Creating backup directory"
    
    # Backup Kubernetes resources
    execute_cmd "kubectl get all -n $NAMESPACE -o yaml > $backup_dir/k8s-resources.yaml" "Backing up Kubernetes resources"
    execute_cmd "kubectl get crd -o yaml > $backup_dir/crds.yaml" "Backing up CRDs"
    execute_cmd "kubectl get secrets -n $NAMESPACE -o yaml > $backup_dir/secrets.yaml" "Backing up secrets"
    
    # Backup configuration files
    execute_cmd "cp -r resources/ $backup_dir/" "Backing up configuration files"
    
    log_success "Platform backup created: $backup_dir"
}

# Function to show QEMU status
show_qemu_status() {
    log_step "QEMU Device Status"
    
    if pgrep -f qemu-system >/dev/null; then
        log_info "QEMU processes running:"
        ps aux | grep qemu-system | grep -v grep
    else
        log_info "No QEMU processes running"
    fi
}

# Main monitoring function
main() {
    log_header "Wasmbed System Monitoring"
    
    show_status
    health_check
    show_qemu_status
}

# Handle script arguments
case "${1:-status}" in
    "status"|"")
        show_status
        ;;
    "logs")
        show_logs "${2:-all}" "${3:-50}"
        ;;
    "health")
        health_check
        ;;
    "monitor")
        monitor_resources
        ;;
    "restart")
        restart_component "${2:-all}"
        ;;
    "scale")
        scale_component "${2:-controller}" "${3:-1}"
        ;;
    "backup")
        backup_platform
        ;;
    "qemu")
        show_qemu_status
        ;;
    "help"|"-h"|"--help")
        echo "Usage: $0 [command] [options]"
        echo ""
        echo "Commands:"
        echo "  status           - Show system status (default)"
        echo "  logs [comp] [n]  - Show logs (gateway|controller|all, lines)"
        echo "  health           - Perform health check"
        echo "  monitor          - Monitor resources in real-time"
        echo "  restart [comp]   - Restart component (gateway|controller|all)"
        echo "  scale [comp] [n] - Scale component to n replicas"
        echo "  backup           - Create platform backup"
        echo "  qemu             - Show QEMU device status"
        echo "  help             - Show this help"
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