#!/bin/bash
# Security scanning script for Wasmbed production hardening
set -e

echo "🔒 Starting Wasmbed security scan..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    if [ "$status" = "PASS" ]; then
        echo -e "${GREEN}✅ PASS${NC}: $message"
    elif [ "$status" = "FAIL" ]; then
        echo -e "${RED}❌ FAIL${NC}: $message"
    else
        echo -e "${YELLOW}⚠️  WARN${NC}: $message"
    fi
}

# 1. Check if cluster is accessible
echo "📋 Checking cluster accessibility..."
if kubectl cluster-info >/dev/null 2>&1; then
    print_status "PASS" "Kubernetes cluster is accessible"
else
    print_status "FAIL" "Cannot access Kubernetes cluster"
    exit 1
fi

# 2. Check namespace security
echo "📋 Checking namespace security..."
if kubectl get namespace wasmbed >/dev/null 2>&1; then
    print_status "PASS" "Wasmbed namespace exists"
else
    print_status "FAIL" "Wasmbed namespace not found"
fi

# 3. Check RBAC configuration
echo "📋 Checking RBAC configuration..."
if kubectl get clusterrole,clusterrolebinding | grep wasmbed >/dev/null 2>&1; then
    print_status "PASS" "RBAC roles and bindings configured"
else
    print_status "FAIL" "RBAC configuration missing"
fi

# 4. Check network policies
echo "📋 Checking network policies..."
if kubectl get networkpolicies -n wasmbed >/dev/null 2>&1; then
    print_status "PASS" "Network policies configured"
else
    print_status "WARN" "Network policies not found - applying default policies"
    kubectl apply -f resources/k8s/network-policies.yaml
fi

# 5. Check pod security
echo "📋 Checking pod security..."
PODS=$(kubectl get pods -n wasmbed -o jsonpath='{.items[*].metadata.name}')
for pod in $PODS; do
    if kubectl get pod $pod -n wasmbed -o jsonpath='{.spec.securityContext.runAsNonRoot}' | grep -q "true"; then
        print_status "PASS" "Pod $pod runs as non-root"
    else
        print_status "WARN" "Pod $pod may run as root"
    fi
done

# 6. Check secrets
echo "📋 Checking secrets..."
if kubectl get secrets -n wasmbed >/dev/null 2>&1; then
    print_status "PASS" "Secrets are properly configured"
else
    print_status "WARN" "No secrets found in wasmbed namespace"
fi

# 7. Check TLS certificates
echo "📋 Checking TLS certificates..."
if [ -f "resources/dev-certs/ca.der" ]; then
    print_status "PASS" "CA certificate exists"
else
    print_status "WARN" "CA certificate not found"
fi

# 8. Check container images
echo "📋 Checking container images..."
GATEWAY_IMAGE=$(kubectl get statefulset wasmbed-gateway -n wasmbed -o jsonpath='{.spec.template.spec.containers[0].image}')
CONTROLLER_IMAGE=$(kubectl get deployment wasmbed-k8s-controller -n wasmbed -o jsonpath='{.spec.template.spec.containers[0].image}')

if [[ "$GATEWAY_IMAGE" == *"wasmbed-gateway"* ]]; then
    print_status "PASS" "Gateway image is correct: $GATEWAY_IMAGE"
else
    print_status "FAIL" "Gateway image is incorrect: $GATEWAY_IMAGE"
fi

if [[ "$CONTROLLER_IMAGE" == *"wasmbed-k8s-controller"* ]]; then
    print_status "PASS" "Controller image is correct: $CONTROLLER_IMAGE"
else
    print_status "FAIL" "Controller image is incorrect: $CONTROLLER_IMAGE"
fi

# 9. Check resource limits
echo "📋 Checking resource limits..."
PODS=$(kubectl get pods -n wasmbed -o jsonpath='{.items[*].metadata.name}')
for pod in $PODS; do
    MEMORY_LIMIT=$(kubectl get pod $pod -n wasmbed -o jsonpath='{.spec.containers[0].resources.limits.memory}')
    CPU_LIMIT=$(kubectl get pod $pod -n wasmbed -o jsonpath='{.spec.containers[0].resources.limits.cpu}')
    
    if [ -n "$MEMORY_LIMIT" ] && [ -n "$CPU_LIMIT" ]; then
        print_status "PASS" "Pod $pod has resource limits: $MEMORY_LIMIT, $CPU_LIMIT"
    else
        print_status "WARN" "Pod $pod missing resource limits"
    fi
done

# 10. Check health endpoints
echo "📋 Checking health endpoints..."
if kubectl port-forward -n wasmbed deployment/wasmbed-k8s-controller 8080:8080 --address=localhost >/dev/null 2>&1 &
then
    PF_PID=$!
    sleep 2
    
    if curl -s http://localhost:8080/health >/dev/null 2>&1; then
        print_status "PASS" "Controller health endpoint accessible"
    else
        print_status "FAIL" "Controller health endpoint not accessible"
    fi
    
    kill $PF_PID 2>/dev/null || true
else
    print_status "WARN" "Could not test health endpoints"
fi

# 11. Check for exposed secrets
echo "📋 Checking for exposed secrets..."
if kubectl get secrets -n wasmbed -o yaml | grep -i "password\|key\|token" >/dev/null 2>&1; then
    print_status "WARN" "Potential secrets found in YAML output"
else
    print_status "PASS" "No obvious secrets exposed"
fi

# 12. Check pod status
echo "📋 Checking pod status..."
PODS=$(kubectl get pods -n wasmbed -o jsonpath='{.items[*].status.phase}')
for pod_status in $PODS; do
    if [ "$pod_status" = "Running" ]; then
        print_status "PASS" "All pods are running"
    else
        print_status "FAIL" "Pod status: $pod_status"
    fi
done

echo ""
echo "🔒 Security scan completed!"
echo ""
echo "📊 Summary:"
echo "- Check the output above for any FAIL or WARN items"
echo "- Address any security issues before production deployment"
echo "- Consider running additional security tools like:"
echo "  - trivy (container vulnerability scanning)"
echo "  - kube-bench (Kubernetes security benchmarking)"
echo "  - falco (runtime security monitoring)"

# Exit with error if any FAIL items were found
if grep -q "❌ FAIL" <<< "$(cat /dev/stdin)"; then
    exit 1
fi
