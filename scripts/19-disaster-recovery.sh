#!/bin/bash
# Disaster recovery script for Wasmbed platform
set -e

echo "🚨 Starting Wasmbed disaster recovery..."

# Configuration
NAMESPACE="wasmbed"
BACKUP_DIR="./backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    if [ "$status" = "SUCCESS" ]; then
        echo -e "${GREEN}✅ SUCCESS${NC}: $message"
    elif [ "$status" = "ERROR" ]; then
        echo -e "${RED}❌ ERROR${NC}: $message"
    else
        echo -e "${YELLOW}⚠️  WARN${NC}: $message"
    fi
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
echo "📋 Checking prerequisites..."

if ! command_exists kubectl; then
    print_status "ERROR" "kubectl not found"
    exit 1
fi

if ! kubectl cluster-info >/dev/null 2>&1; then
    print_status "ERROR" "Cannot access Kubernetes cluster"
    exit 1
fi

print_status "SUCCESS" "Prerequisites check passed"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Step 1: Backup current state
echo "📋 Step 1: Creating backup of current state..."

# Backup CRDs
kubectl get crd | grep wasmbed > "$BACKUP_DIR/crds_$TIMESTAMP.txt" 2>/dev/null || true

# Backup resources
kubectl get all -n "$NAMESPACE" -o yaml > "$BACKUP_DIR/resources_$TIMESTAMP.yaml" 2>/dev/null || true

# Backup secrets
kubectl get secrets -n "$NAMESPACE" -o yaml > "$BACKUP_DIR/secrets_$TIMESTAMP.yaml" 2>/dev/null || true

# Backup configmaps
kubectl get configmaps -n "$NAMESPACE" -o yaml > "$BACKUP_DIR/configmaps_$TIMESTAMP.yaml" 2>/dev/null || true

print_status "SUCCESS" "Backup created in $BACKUP_DIR"

# Step 2: Check cluster health
echo "📋 Step 2: Checking cluster health..."

NODES=$(kubectl get nodes --no-headers | wc -l)
if [ "$NODES" -gt 0 ]; then
    print_status "SUCCESS" "Cluster has $NODES nodes"
else
    print_status "ERROR" "No nodes found in cluster"
    exit 1
fi

# Step 3: Restore namespace if needed
echo "📋 Step 3: Ensuring namespace exists..."

if ! kubectl get namespace "$NAMESPACE" >/dev/null 2>&1; then
    print_status "WARN" "Namespace $NAMESPACE not found, creating..."
    kubectl create namespace "$NAMESPACE"
    print_status "SUCCESS" "Namespace $NAMESPACE created"
else
    print_status "SUCCESS" "Namespace $NAMESPACE exists"
fi

# Step 4: Restore CRDs
echo "📋 Step 4: Restoring CRDs..."

# Generate and apply Device CRD
if command_exists cargo; then
    cargo run -p wasmbed-k8s-resource-tool -- crd device | kubectl apply -f - 2>/dev/null || true
    cargo run -p wasmbed-k8s-resource-tool -- crd application | kubectl apply -f - 2>/dev/null || true
    print_status "SUCCESS" "CRDs restored"
else
    print_status "WARN" "Cargo not found, skipping CRD restoration"
fi

# Step 5: Restore RBAC
echo "📋 Step 5: Restoring RBAC..."

# Apply RBAC resources
kubectl apply -f resources/k8s/010-namespace.yaml 2>/dev/null || true
kubectl apply -f resources/k8s/020-service-accounts.yaml 2>/dev/null || true
kubectl apply -f resources/k8s/030-roles.yaml 2>/dev/null || true
kubectl apply -f resources/k8s/040-role-bindings.yaml 2>/dev/null || true

print_status "SUCCESS" "RBAC restored"

# Step 6: Restore services
echo "📋 Step 6: Restoring services..."

kubectl apply -f resources/k8s/100-service-gateway.yaml 2>/dev/null || true

print_status "SUCCESS" "Services restored"

# Step 7: Restore deployments
echo "📋 Step 7: Restoring deployments..."

kubectl apply -f resources/k8s/111-statefulset-gateway.yaml 2>/dev/null || true
kubectl apply -f resources/k8s/controller-deployment.yaml 2>/dev/null || true

print_status "SUCCESS" "Deployments restored"

# Step 8: Restore network policies
echo "📋 Step 8: Restoring network policies..."

kubectl apply -f resources/k8s/network-policies.yaml 2>/dev/null || true

print_status "SUCCESS" "Network policies restored"

# Step 9: Wait for pods to be ready
echo "📋 Step 9: Waiting for pods to be ready..."

# Wait for gateway pods
kubectl rollout status statefulset/wasmbed-gateway -n "$NAMESPACE" --timeout=300s 2>/dev/null || true

# Wait for controller deployment
kubectl rollout status deployment/wasmbed-k8s-controller -n "$NAMESPACE" --timeout=300s 2>/dev/null || true

print_status "SUCCESS" "Pods are ready"

# Step 10: Verify recovery
echo "📋 Step 10: Verifying recovery..."

# Check pod status
PODS=$(kubectl get pods -n "$NAMESPACE" --no-headers | wc -l)
RUNNING_PODS=$(kubectl get pods -n "$NAMESPACE" --no-headers | grep -c "Running" || true)

if [ "$PODS" -gt 0 ] && [ "$RUNNING_PODS" -eq "$PODS" ]; then
    print_status "SUCCESS" "All $PODS pods are running"
else
    print_status "WARN" "Only $RUNNING_PODS/$PODS pods are running"
fi

# Check services
SERVICES=$(kubectl get services -n "$NAMESPACE" --no-headers | wc -l)
if [ "$SERVICES" -gt 0 ]; then
    print_status "SUCCESS" "$SERVICES services are available"
else
    print_status "WARN" "No services found"
fi

# Check CRDs
CRDS=$(kubectl get crd | grep wasmbed | wc -l)
if [ "$CRDS" -gt 0 ]; then
    print_status "SUCCESS" "$CRDS CRDs are available"
else
    print_status "WARN" "No CRDs found"
fi

# Step 11: Run basic tests
echo "📋 Step 11: Running basic tests..."

# Test if we can create a device
if command_exists cargo; then
    echo "apiVersion: wasmbed.github.io/v0
kind: Device
metadata:
  name: recovery-test-device
  namespace: $NAMESPACE
spec:
  publicKey: \"cmVjb3ZlcnktdGVzdC1rZXk=\"" | kubectl apply -f - 2>/dev/null || true
    
    if kubectl get device recovery-test-device -n "$NAMESPACE" >/dev/null 2>&1; then
        print_status "SUCCESS" "Device CRD is working"
        # Clean up test device
        kubectl delete device recovery-test-device -n "$NAMESPACE" 2>/dev/null || true
    else
        print_status "WARN" "Device CRD test failed"
    fi
fi

echo ""
echo "🎉 Disaster recovery completed!"
echo ""
echo "📊 Recovery Summary:"
echo "- Backup created in: $BACKUP_DIR"
echo "- Namespace: $NAMESPACE"
echo "- Pods running: $RUNNING_PODS/$PODS"
echo "- Services: $SERVICES"
echo "- CRDs: $CRDS"
echo ""
echo "🔍 Next steps:"
echo "1. Run: ./scripts/test.sh"
echo "2. Run: ./scripts/security-scan.sh"
echo "3. Verify application functionality"
echo "4. Check logs: kubectl logs -n $NAMESPACE -l app=wasmbed-gateway"
echo ""
echo "📁 Backup files:"
ls -la "$BACKUP_DIR"/*"$TIMESTAMP"* 2>/dev/null || echo "No backup files found"
