#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CLUSTER_NAME="wasmbed"
NAMESPACE="wasmbed"

echo " Wasmbed System Monitor"

check_environment() {
    # Check if we're in Nix shell
    if [ -z "${IN_NIX_SHELL:-}" ]; then
        echo -e "${YELLOW}  Not in Nix shell. Running 'nix develop' first...${NC}"
        exec nix develop --command "$0" "$@"
    fi
    
    # Set kubeconfig if not already set
    if [ -z "${KUBECONFIG:-}" ]; then
        export KUBECONFIG=$(k3d kubeconfig write "$CLUSTER_NAME" 2>/dev/null || echo "")
    fi
    
    if [ -z "$KUBECONFIG" ] || ! kubectl cluster-info &> /dev/null; then
        echo -e "${RED} Cannot connect to cluster. Please run './scripts/setup.sh' first${NC}"
        exit 1
    fi
}

show_cluster_overview() {
    echo -e "${BLUE}🏠 Cluster Overview${NC}"
    echo "=================="
    
    echo "📍 Cluster Info:"
    kubectl cluster-info | grep -E "(Kubernetes|CoreDNS)" || true
    echo ""
    
    echo "  Namespaces:"
    kubectl get namespaces | grep -E "(NAME|wasmbed|kube-system)" || true
    echo ""
}

show_gateway_status() {
    echo -e "${BLUE}🚪 Gateway Status${NC}"
    echo "================="
    
    echo "📦 StatefulSet:"
    kubectl -n "$NAMESPACE" get statefulset wasmbed-gateway 2>/dev/null || echo "   StatefulSet not found"
    echo ""
    
    echo "🏃 Pods:"
    kubectl -n "$NAMESPACE" get pods -l app=wasmbed-gateway 2>/dev/null || echo "   No gateway pods found"
    echo ""
    
    echo " Services:"
    kubectl -n "$NAMESPACE" get services wasmbed-gateway-service 2>/dev/null || echo "   Service not found"
    echo ""
    
    # Show pod details if pod exists
    if kubectl -n "$NAMESPACE" get pod wasmbed-gateway-0 &> /dev/null; then
        echo " Pod Details:"
        kubectl -n "$NAMESPACE" describe pod wasmbed-gateway-0 | grep -A 5 -E "(Status:|Conditions:|Events:)" || true
        echo ""
    fi
}

show_device_status() {
    echo -e "${BLUE} Device Status${NC}"
    echo "================"
    
    echo " All Devices:"
    if kubectl -n "$NAMESPACE" get devices &> /dev/null; then
        kubectl -n "$NAMESPACE" get devices
        echo ""
        
        # Show detailed status for each device
        for device in $(kubectl -n "$NAMESPACE" get devices -o name 2>/dev/null); do
            device_name=$(basename "$device")
            echo " Device $device_name Status:"
            kubectl -n "$NAMESPACE" get "$device" -o yaml | grep -A 20 "status:" | head -20 || echo "    No status available"
            echo ""
        done
    else
        echo "   No devices found or Device CRD not installed"
        echo ""
    fi
}

show_logs() {
    echo -e "${BLUE}📜 Recent Logs${NC}"
    echo "=============="
    
    if kubectl -n "$NAMESPACE" get pod wasmbed-gateway-0 &> /dev/null; then
        echo "🚪 Gateway Logs (last 20 lines):"
        kubectl -n "$NAMESPACE" logs wasmbed-gateway-0 --tail=20 2>/dev/null || echo "   Cannot retrieve logs"
        echo ""
    else
        echo "   Gateway pod not found"
        echo ""
    fi
}

show_events() {
    echo -e "${BLUE} Recent Events${NC}"
    echo "==============="
    
    echo "🔔 Namespace Events (last 10):"
    kubectl -n "$NAMESPACE" get events --sort-by='.lastTimestamp' --field-selector type!=Normal 2>/dev/null | tail -10 || echo "    No notable events"
    echo ""
}

show_network_status() {
    echo -e "${BLUE} Network Status${NC}"
    echo "=================="
    
    echo "🔌 Port Forwards:"
    if pgrep -f "kubectl.*port-forward.*wasmbed-gateway-service" > /dev/null; then
        echo "   Port forward is active"
        ps aux | grep "kubectl.*port-forward.*wasmbed-gateway-service" | grep -v grep || true
    else
        echo "   No active port forward"
        echo "   To start: kubectl -n $NAMESPACE port-forward service/wasmbed-gateway-service 4423:4423"
    fi
    echo ""
    
    echo "🔗 Service Endpoints:"
    kubectl -n "$NAMESPACE" get endpoints wasmbed-gateway-service 2>/dev/null || echo "   Service endpoints not found"
    echo ""
}

interactive_mode() {
    while true; do
        echo ""
        echo -e "${YELLOW}  Interactive Monitor${NC}"
        echo "====================="
        echo "1) Refresh all status"
        echo "2) Show gateway logs (follow)"
        echo "3) Show events (follow)" 
        echo "4) Restart gateway"
        echo "5) Scale gateway (up/down)"
        echo "6) Test connection"
        echo "7) Cleanup and exit"
        echo "q) Quit"
        echo ""
        read -p "Choose an option: " choice
        
        case $choice in
            1)
                clear
                main_display
                ;;
            2)
                echo -e "${BLUE}📜 Following gateway logs (Ctrl+C to stop)...${NC}"
                kubectl -n "$NAMESPACE" logs -f wasmbed-gateway-0 2>/dev/null || echo "Cannot follow logs"
                ;;
            3)
                echo -e "${BLUE} Following events (Ctrl+C to stop)...${NC}"
                kubectl -n "$NAMESPACE" get events --watch 2>/dev/null || echo "Cannot follow events"
                ;;
            4)
                echo -e "${BLUE}🔄 Restarting gateway...${NC}"
                kubectl -n "$NAMESPACE" rollout restart statefulset/wasmbed-gateway
                kubectl -n "$NAMESPACE" rollout status statefulset/wasmbed-gateway
                ;;
            5)
                echo "Current replicas:"
                kubectl -n "$NAMESPACE" get statefulset wasmbed-gateway -o jsonpath='{.spec.replicas}'
                echo ""
                read -p "Enter new replica count: " replicas
                kubectl -n "$NAMESPACE" scale statefulset wasmbed-gateway --replicas="$replicas"
                ;;
            6)
                echo -e "${BLUE} Running connection test...${NC}"
                ./scripts/test.sh
                ;;
            7)
                echo -e "${BLUE} Running cleanup...${NC}"
                ./scripts/cleanup.sh
                exit 0
                ;;
            q|Q)
                echo -e "${GREEN}👋 Goodbye!${NC}"
                exit 0
                ;;
            *)
                echo -e "${RED} Invalid option${NC}"
                ;;
        esac
    done
}

main_display() {
    show_cluster_overview
    show_gateway_status
    show_device_status
    show_network_status
    show_logs
    show_events
}

main() {
    check_environment
    
    if [ "${1:-}" = "--interactive" ] || [ "${1:-}" = "-i" ]; then
        main_display
        interactive_mode
    else
        main_display
        echo -e "${YELLOW} Tip: Run '$0 --interactive' for interactive monitoring${NC}"
    fi
}

main "$@"
