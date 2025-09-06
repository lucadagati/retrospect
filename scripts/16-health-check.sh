#!/bin/bash
# Run health checks for Wasmbed platform
# This script performs comprehensive health validation

set -e

echo "🏥 Running Wasmbed platform health checks..."

# Check if cluster is accessible
if ! kubectl cluster-info >/dev/null 2>&1; then
    echo "❌ Cannot access Kubernetes cluster"
    echo "Please ensure k3d cluster is running: k3d cluster start wasmbed"
    exit 1
fi

echo "✅ Kubernetes cluster is accessible"

# Function to check pod health
check_pod_health() {
    local pod_name=$1
    local namespace=${2:-wasmbed}
    
    echo "📋 Checking pod: $pod_name"
    
    # Check if pod exists
    if ! kubectl get pod "$pod_name" -n "$namespace" >/dev/null 2>&1; then
        echo "❌ Pod $pod_name not found"
        return 1
    fi
    
    # Check pod status
    local status=$(kubectl get pod "$pod_name" -n "$namespace" -o jsonpath='{.status.phase}')
    if [ "$status" = "Running" ]; then
        echo "✅ Pod $pod_name is running"
    else
        echo "❌ Pod $pod_name status: $status"
        return 1
    fi
    
    # Check readiness
    local ready=$(kubectl get pod "$pod_name" -n "$namespace" -o jsonpath='{.status.containerStatuses[0].ready}')
    if [ "$ready" = "true" ]; then
        echo "✅ Pod $pod_name is ready"
    else
        echo "❌ Pod $pod_name is not ready"
        return 1
    fi
    
    # Check restart count
    local restarts=$(kubectl get pod "$pod_name" -n "$namespace" -o jsonpath='{.status.containerStatuses[0].restartCount}')
    if [ "$restarts" -eq 0 ]; then
        echo "✅ Pod $pod_name has no restarts"
    else
        echo "⚠️ Pod $pod_name has $restarts restarts"
    fi
    
    return 0
}

# Function to check service health
check_service_health() {
    local service_name=$1
    local namespace=${2:-wasmbed}
    
    echo "📋 Checking service: $service_name"
    
    # Check if service exists
    if ! kubectl get service "$service_name" -n "$namespace" >/dev/null 2>&1; then
        echo "❌ Service $service_name not found"
        return 1
    fi
    
    # Check service endpoints
    local endpoints=$(kubectl get endpoints "$service_name" -n "$namespace" -o jsonpath='{.subsets[0].addresses[*].ip}' 2>/dev/null | wc -w)
    if [ "$endpoints" -gt 0 ]; then
        echo "✅ Service $service_name has $endpoints endpoints"
    else
        echo "❌ Service $service_name has no endpoints"
        return 1
    fi
    
    return 0
}

# Check namespace
echo "📋 Checking namespace..."
if kubectl get namespace wasmbed >/dev/null 2>&1; then
    echo "✅ wasmbed namespace exists"
else
    echo "❌ wasmbed namespace missing"
    exit 1
fi

# Check CRDs
echo "📋 Checking CRDs..."
CRDS=$(kubectl get crd | grep wasmbed | wc -l)
if [ "$CRDS" -gt 0 ]; then
    echo "✅ $CRDS CRDs installed"
else
    echo "❌ No CRDs found"
    exit 1
fi

# Check gateway pods
echo "📋 Checking gateway pods..."
GATEWAY_PODS=$(kubectl get pods -n wasmbed -l app=wasmbed-gateway --no-headers | wc -l)
if [ "$GATEWAY_PODS" -gt 0 ]; then
    echo "✅ Found $GATEWAY_PODS gateway pods"
    
    # Check each gateway pod
    kubectl get pods -n wasmbed -l app=wasmbed-gateway --no-headers | while read -r line; do
        pod_name=$(echo "$line" | awk '{print $1}')
        check_pod_health "$pod_name"
    done
else
    echo "❌ No gateway pods found"
    exit 1
fi

# Check controller pod
echo "📋 Checking controller pod..."
CONTROLLER_PODS=$(kubectl get pods -n wasmbed -l app=wasmbed-k8s-controller --no-headers | wc -l)
if [ "$CONTROLLER_PODS" -gt 0 ]; then
    echo "✅ Found $CONTROLLER_PODS controller pods"
    
    # Check controller pod
    kubectl get pods -n wasmbed -l app=wasmbed-k8s-controller --no-headers | while read -r line; do
        pod_name=$(echo "$line" | awk '{print $1}')
        check_pod_health "$pod_name"
    done
else
    echo "❌ No controller pods found"
    exit 1
fi

# Check services
echo "📋 Checking services..."
check_service_health "wasmbed-gateway-service"

# Check health endpoints
echo "📋 Checking health endpoints..."

# Test controller health endpoint
if kubectl port-forward -n wasmbed deployment/wasmbed-k8s-controller 8080:8080 --address=localhost >/dev/null 2>&1 &
then
    PF_PID=$!
    sleep 2
    
    if curl -s http://localhost:8080/health >/dev/null 2>&1; then
        echo "✅ Controller health endpoint accessible"
    else
        echo "❌ Controller health endpoint not accessible"
    fi
    
    if curl -s http://localhost:8080/ready >/dev/null 2>&1; then
        echo "✅ Controller ready endpoint accessible"
    else
        echo "❌ Controller ready endpoint not accessible"
    fi
    
    kill $PF_PID 2>/dev/null || true
else
    echo "⚠️ Could not test controller health endpoints"
fi

# Check resource usage
echo "📋 Checking resource usage..."
kubectl top pods -n wasmbed 2>/dev/null || echo "⚠️ Resource metrics not available"

# Check events
echo "📋 Checking recent events..."
kubectl get events -n wasmbed --sort-by='.lastTimestamp' | tail -5

# Health summary
echo ""
echo "🏥 Health Check Summary:"
RUNNING_PODS=$(kubectl get pods -n wasmbed --no-headers 2>/dev/null | grep -c "Running" || echo "0")
TOTAL_PODS=$(kubectl get pods -n wasmbed --no-headers 2>/dev/null | wc -l || echo "0")

if [ "$TOTAL_PODS" -gt 0 ]; then
    HEALTH_PERCENT=$((RUNNING_PODS * 100 / TOTAL_PODS))
    echo "📊 Overall Health: $RUNNING_PODS/$TOTAL_PODS pods running ($HEALTH_PERCENT%)"
    
    if [ "$HEALTH_PERCENT" -eq 100 ]; then
        echo "✅ Platform is healthy"
    elif [ "$HEALTH_PERCENT" -gt 50 ]; then
        echo "⚠️ Platform has issues"
    else
        echo "❌ Platform is unhealthy"
        exit 1
    fi
else
    echo "❌ No pods found"
    exit 1
fi

echo ""
echo "🎉 Health checks completed!"
echo ""
echo "Next steps:"
echo "  ./wasmbed.sh logs                     # Show detailed logs"
echo "  ./wasmbed.sh monitor                  # Start monitoring"
echo "  ./wasmbed.sh status                   # Show platform status"

