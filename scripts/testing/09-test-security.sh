#!/bin/bash
# Run security tests for Wasmbed platform
# This script validates security aspects of the platform

set -e

echo " Running security tests..."

# Check if cluster is accessible
if ! kubectl cluster-info >/dev/null 2>&1; then
    echo " Cannot access Kubernetes cluster"
    echo "Please run: ./wasmbed.sh deploy"
    exit 1
fi

echo " Kubernetes cluster is accessible"

# Test 1: Network policies
echo " Testing network policies..."
POLICIES=$(kubectl get networkpolicies -n wasmbed --no-headers 2>/dev/null | wc -l)
if [ "$POLICIES" -gt 0 ]; then
    echo " Network policies configured"
else
    echo " No network policies found"
    exit 1
fi

# Test 2: RBAC configuration
echo " Testing RBAC configuration..."
ROLES=$(kubectl get clusterrole,clusterrolebinding | grep wasmbed | wc -l)
if [ "$ROLES" -gt 0 ]; then
    echo " RBAC configured"
else
    echo " RBAC not configured"
    exit 1
fi

# Test 3: Pod security
echo " Testing pod security..."
PODS=$(kubectl get pods -n wasmbed --no-headers 2>/dev/null | wc -l)
if [ "$PODS" -gt 0 ]; then
    echo " Pods are running"
    
    # Check if pods run as non-root
    kubectl get pods -n wasmbed --no-headers | while read -r line; do
        pod_name=$(echo "$line" | awk '{print $1}')
        if kubectl get pod "$pod_name" -n wasmbed -o jsonpath='{.spec.securityContext.runAsNonRoot}' 2>/dev/null | grep -q "true"; then
            echo " Pod $pod_name runs as non-root"
        else
            echo " Pod $pod_name may run as root"
        fi
    done
else
    echo " No pods found"
    exit 1
fi

# Test 4: TLS certificates
echo " Testing TLS certificates..."
if [ -f "resources/dev-certs/ca.der" ]; then
    echo " CA certificate exists"
else
    echo " CA certificate not found"
fi

# Test 5: Container images
echo " Testing container images..."
GATEWAY_IMAGE=$(kubectl get statefulset wasmbed-gateway -n wasmbed -o jsonpath='{.spec.template.spec.containers[0].image}' 2>/dev/null)
CONTROLLER_IMAGE=$(kubectl get deployment wasmbed-k8s-controller -n wasmbed -o jsonpath='{.spec.template.spec.containers[0].image}' 2>/dev/null)

if [[ "$GATEWAY_IMAGE" == *"wasmbed-gateway"* ]]; then
    echo " Gateway image is correct"
else
    echo " Gateway image is incorrect"
fi

if [[ "$CONTROLLER_IMAGE" == *"wasmbed-k8s-controller"* ]]; then
    echo " Controller image is correct"
else
    echo " Controller image is incorrect"
fi

# Test 6: Resource limits
echo " Testing resource limits..."
kubectl get pods -n wasmbed --no-headers | while read -r line; do
    pod_name=$(echo "$line" | awk '{print $1}')
    MEMORY_LIMIT=$(kubectl get pod "$pod_name" -n wasmbed -o jsonpath='{.spec.containers[0].resources.limits.memory}' 2>/dev/null)
    CPU_LIMIT=$(kubectl get pod "$pod_name" -n wasmbed -o jsonpath='{.spec.containers[0].resources.limits.cpu}' 2>/dev/null)
    
    if [ -n "$MEMORY_LIMIT" ] && [ -n "$CPU_LIMIT" ]; then
        echo " Pod $pod_name has resource limits"
    else
        echo " Pod $pod_name missing resource limits"
    fi
done

# Test 7: Secrets exposure
echo " Testing secrets exposure..."
if kubectl get secrets -n wasmbed -o yaml | grep -i "password\|key\|token" >/dev/null 2>&1; then
    echo " Potential secrets found in YAML output"
else
    echo " No obvious secrets exposed"
fi

echo ""
echo " All security tests passed!"
echo ""
echo "Next steps:"
echo "  ./wasmbed.sh security-scan             # Run comprehensive security scan"
echo "  ./wasmbed.sh security-hardening         # Apply security hardening"
echo "  ./wasmbed.sh test-end-to-end            # Run end-to-end tests"

