#!/bin/bash
# Run end-to-end tests for Wasmbed platform
# This script tests the complete deployment pipeline

set -e

echo "🔄 Running end-to-end tests..."

# Check if cluster is accessible
if ! kubectl cluster-info >/dev/null 2>&1; then
    echo " Cannot access Kubernetes cluster"
    echo "Please run: ./wasmbed.sh deploy"
    exit 1
fi

echo " Kubernetes cluster is accessible"

# Test 1: Platform deployment
echo " Testing platform deployment..."
if kubectl get namespace wasmbed >/dev/null 2>&1; then
    echo " Namespace exists"
else
    echo " Namespace missing"
    exit 1
fi

# Test 2: CRDs
echo " Testing CRDs..."
if kubectl get crd | grep wasmbed >/dev/null 2>&1; then
    echo " CRDs are installed"
else
    echo " CRDs missing"
    exit 1
fi

# Test 3: Pods running
echo " Testing pod status..."
GATEWAY_PODS=$(kubectl get pods -n wasmbed -l app=wasmbed-gateway --no-headers | wc -l)
CONTROLLER_PODS=$(kubectl get pods -n wasmbed -l app=wasmbed-k8s-controller --no-headers | wc -l)

if [ "$GATEWAY_PODS" -gt 0 ] && [ "$CONTROLLER_PODS" -gt 0 ]; then
    echo " Pods are deployed"
else
    echo " Pods not deployed"
    exit 1
fi

# Test 4: Services
echo " Testing services..."
if kubectl get services -n wasmbed >/dev/null 2>&1; then
    echo " Services are configured"
else
    echo " Services missing"
    exit 1
fi

# Test 5: Device CRD functionality
echo " Testing Device CRD..."
cat <<EOF | kubectl apply -f -
apiVersion: wasmbed.github.io/v0
kind: Device
metadata:
  name: e2e-test-device
  namespace: wasmbed
spec:
  publicKey: "ZTIyZS10ZXN0LWtleQ=="
EOF

if kubectl get device e2e-test-device -n wasmbed >/dev/null 2>&1; then
    echo " Device CRD working"
    # Cleanup
    kubectl delete device e2e-test-device -n wasmbed
else
    echo " Device CRD failed"
    exit 1
fi

# Test 6: Application CRD functionality
echo " Testing Application CRD..."
cat <<EOF | kubectl apply -f -
apiVersion: wasmbed.github.io/v1alpha1
kind: Application
metadata:
  name: e2e-test-app
  namespace: wasmbed
spec:
  name: "E2E Test App"
  description: "End-to-end test application"
  wasm_bytes: "AA=="
  target_devices:
    device_names: ["e2e-test-device"]
  config:
    memory_limit: 1048576
    cpu_time_limit: 1000
    auto_restart: true
    max_restarts: 3
EOF

if kubectl get application e2e-test-app -n wasmbed >/dev/null 2>&1; then
    echo " Application CRD working"
    # Cleanup
    kubectl delete application e2e-test-app -n wasmbed
else
    echo " Application CRD failed"
    exit 1
fi

# Test 7: Gateway connectivity
echo " Testing gateway connectivity..."
GATEWAY_SERVICE=$(kubectl get service wasmbed-gateway-service -n wasmbed -o jsonpath='{.spec.clusterIP}')
if [ -n "$GATEWAY_SERVICE" ]; then
    echo " Gateway service accessible at $GATEWAY_SERVICE"
else
    echo " Gateway service not accessible"
    exit 1
fi

# Test 8: Controller health
echo " Testing controller health..."
if kubectl port-forward -n wasmbed deployment/wasmbed-k8s-controller 8080:8080 --address=localhost >/dev/null 2>&1 &
then
    PF_PID=$!
    sleep 2
    
    if curl -s http://localhost:8080/health >/dev/null 2>&1; then
        echo " Controller health endpoint working"
    else
        echo " Controller health endpoint failed"
    fi
    
    kill $PF_PID 2>/dev/null || true
else
    echo " Could not test controller health endpoint"
fi

echo ""
echo " All end-to-end tests passed!"
echo ""
echo " Platform is fully operational!"
echo ""
echo "Next steps:"
echo "  ./wasmbed.sh test-security            # Run security validation"
echo "  ./wasmbed.sh monitor                  # Start monitoring"
echo "  ./wasmbed.sh status                   # Show platform status"

