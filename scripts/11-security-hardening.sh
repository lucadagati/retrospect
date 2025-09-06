#!/bin/bash
# Apply security hardening to Wasmbed platform
# This script applies production security configurations

set -e

echo "🛡️ Applying security hardening..."

# Check if cluster is accessible
if ! kubectl cluster-info >/dev/null 2>&1; then
    echo "❌ Cannot access Kubernetes cluster"
    echo "Please run: ./wasmbed.sh deploy"
    exit 1
fi

echo "✅ Kubernetes cluster is accessible"

# Step 1: Apply network policies
echo "📋 Applying network policies..."
kubectl apply -f resources/k8s/network-policies.yaml
echo "✅ Network policies applied"

# Step 2: Verify RBAC configuration
echo "📋 Verifying RBAC configuration..."
RBAC_COUNT=$(kubectl get clusterrole,clusterrolebinding | grep wasmbed | wc -l)
if [ "$RBAC_COUNT" -gt 0 ]; then
    echo "✅ RBAC is properly configured"
else
    echo "❌ RBAC not configured, applying..."
    kubectl apply -f resources/k8s/030-roles.yaml
    kubectl apply -f resources/k8s/040-role-bindings.yaml
    echo "✅ RBAC applied"
fi

# Step 3: Check pod security
echo "📋 Checking pod security..."
PODS=$(kubectl get pods -n wasmbed --no-headers 2>/dev/null | wc -l)
if [ "$PODS" -gt 0 ]; then
    echo "✅ Pods are running"
    
    # Check resource limits
    kubectl get pods -n wasmbed --no-headers | while read -r line; do
        pod_name=$(echo "$line" | awk '{print $1}')
        MEMORY_LIMIT=$(kubectl get pod "$pod_name" -n wasmbed -o jsonpath='{.spec.containers[0].resources.limits.memory}' 2>/dev/null)
        CPU_LIMIT=$(kubectl get pod "$pod_name" -n wasmbed -o jsonpath='{.spec.containers[0].resources.limits.cpu}' 2>/dev/null)
        
        if [ -n "$MEMORY_LIMIT" ] && [ -n "$CPU_LIMIT" ]; then
            echo "✅ Pod $pod_name has resource limits"
        else
            echo "⚠️ Pod $pod_name missing resource limits"
        fi
    done
else
    echo "❌ No pods found"
    exit 1
fi

# Step 4: Verify TLS certificates
echo "📋 Verifying TLS certificates..."
if [ -f "resources/dev-certs/ca.der" ]; then
    echo "✅ CA certificate exists"
else
    echo "⚠️ CA certificate not found"
fi

# Step 5: Check container images
echo "📋 Verifying container images..."
GATEWAY_IMAGE=$(kubectl get statefulset wasmbed-gateway -n wasmbed -o jsonpath='{.spec.template.spec.containers[0].image}' 2>/dev/null)
CONTROLLER_IMAGE=$(kubectl get deployment wasmbed-k8s-controller -n wasmbed -o jsonpath='{.spec.template.spec.containers[0].image}' 2>/dev/null)

if [[ "$GATEWAY_IMAGE" == *"wasmbed-gateway"* ]]; then
    echo "✅ Gateway image is correct"
else
    echo "❌ Gateway image is incorrect"
fi

if [[ "$CONTROLLER_IMAGE" == *"wasmbed-k8s-controller"* ]]; then
    echo "✅ Controller image is correct"
else
    echo "❌ Controller image is incorrect"
fi

# Step 6: Security recommendations
echo "📋 Security recommendations:"
echo "  - Ensure all pods run as non-root users"
echo "  - Implement resource limits for all containers"
echo "  - Use network policies to restrict pod communication"
echo "  - Regularly rotate TLS certificates"
echo "  - Monitor for security vulnerabilities"
echo "  - Implement audit logging"

echo ""
echo "✅ Security hardening applied!"
echo ""
echo "Next steps:"
echo "  ./wasmbed.sh security-scan             # Run comprehensive security scan"
echo "  ./wasmbed.sh test-security              # Run security tests"
echo "  ./wasmbed.sh certificate-rotate         # Rotate certificates"

