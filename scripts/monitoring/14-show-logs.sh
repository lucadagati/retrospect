#!/bin/bash
# Show platform logs for Wasmbed
# This script displays logs from all platform components

set -e

echo " Showing Wasmbed platform logs..."

# Check if cluster is accessible
if ! kubectl cluster-info >/dev/null 2>&1; then
    echo " Cannot access Kubernetes cluster"
    echo "Please ensure k3d cluster is running: k3d cluster start wasmbed"
    exit 1
fi

echo " Kubernetes cluster is accessible"

# Function to show logs for a component
show_component_logs() {
    local component=$1
    local selector=$2
    local namespace=${3:-wasmbed}
    
    echo ""
    echo " $component Logs:"
    echo "=================="
    
    if kubectl get pods -n "$namespace" -l "$selector" --no-headers >/dev/null 2>&1; then
        kubectl logs -n "$namespace" -l "$selector" --tail=50 --timestamps
    else
        echo " No $component pods found"
    fi
}

# Show gateway logs
show_component_logs "Gateway" "app=wasmbed-gateway"

# Show controller logs
show_component_logs "Controller" "app=wasmbed-k8s-controller"

# Show recent events
echo ""
echo " Recent Events:"
echo "=================="
kubectl get events -n wasmbed --sort-by='.lastTimestamp' | tail -10

# Show pod status
echo ""
echo " Pod Status:"
echo "=============="
kubectl get pods -n wasmbed

echo ""
echo " Log Options:"
echo "==============="
echo "  kubectl logs -n wasmbed -l app=wasmbed-gateway -f     # Follow gateway logs"
echo "  kubectl logs -n wasmbed -l app=wasmbed-k8s-controller -f # Follow controller logs"
echo "  kubectl logs -n wasmbed <pod-name> -f                 # Follow specific pod logs"
echo ""
echo "Next steps:"
echo "  ./wasmbed.sh status                   # Show platform status"
echo "  ./wasmbed.sh health-check             # Run health checks"
echo "  ./wasmbed.sh monitor                  # Start monitoring"

