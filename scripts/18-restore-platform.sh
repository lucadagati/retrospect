#!/bin/bash
# Restore Wasmbed platform from backup
# This script restores platform state from a previous backup

set -e

echo "🔄 Restoring Wasmbed platform from backup..."

# Configuration
NAMESPACE="wasmbed"
BACKUP_DIR="./backups"

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

# Parse command line arguments
BACKUP_NAME=""
if [ "$1" = "--backup" ] && [ -n "$2" ]; then
    BACKUP_NAME="$2"
else
    echo "Usage: $0 --backup <backup-name>"
    echo ""
    echo "Available backups:"
    ls -1 "$BACKUP_DIR"/*.tar.gz 2>/dev/null | sed 's/.*\///' | sed 's/\.tar\.gz$//' || echo "No backups found"
    exit 1
fi

# Check if cluster is accessible
if ! kubectl cluster-info >/dev/null 2>&1; then
    print_status "ERROR" "Cannot access Kubernetes cluster"
    exit 1
fi

print_status "SUCCESS" "Kubernetes cluster is accessible"

# Check if backup exists
BACKUP_FILE="$BACKUP_DIR/$BACKUP_NAME.tar.gz"
if [ ! -f "$BACKUP_FILE" ]; then
    print_status "ERROR" "Backup file not found: $BACKUP_FILE"
    exit 1
fi

print_status "SUCCESS" "Backup file found: $BACKUP_FILE"

# Extract backup
echo "📋 Extracting backup..."
cd "$BACKUP_DIR"
tar -xzf "$BACKUP_NAME.tar.gz"
cd - > /dev/null

# Find backup files
TIMESTAMP=$(echo "$BACKUP_NAME" | sed 's/wasmbed-backup-//')

# Step 1: Restore CRDs
echo "📋 Step 1: Restoring CRDs..."
if [ -f "$BACKUP_DIR/crds_$TIMESTAMP.txt" ]; then
    cargo run -p wasmbed-k8s-resource-tool -- crd device | kubectl apply -f - 2>/dev/null || true
    cargo run -p wasmbed-k8s-resource-tool -- crd application | kubectl apply -f - 2>/dev/null || true
    print_status "SUCCESS" "CRDs restored"
else
    print_status "WARN" "CRD backup not found"
fi

# Step 2: Restore namespace
echo "📋 Step 2: Restoring namespace..."
kubectl create namespace "$NAMESPACE" 2>/dev/null || true
print_status "SUCCESS" "Namespace restored"

# Step 3: Restore RBAC
echo "📋 Step 3: Restoring RBAC..."
kubectl apply -f resources/k8s/020-service-accounts.yaml 2>/dev/null || true
kubectl apply -f resources/k8s/030-roles.yaml 2>/dev/null || true
kubectl apply -f resources/k8s/040-role-bindings.yaml 2>/dev/null || true
print_status "SUCCESS" "RBAC restored"

# Step 4: Restore resources
echo "📋 Step 4: Restoring resources..."
if [ -f "$BACKUP_DIR/resources_$TIMESTAMP.yaml" ]; then
    kubectl apply -f "$BACKUP_DIR/resources_$TIMESTAMP.yaml" 2>/dev/null || true
    print_status "SUCCESS" "Resources restored"
else
    print_status "WARN" "Resources backup not found"
fi

# Step 5: Restore secrets
echo "📋 Step 5: Restoring secrets..."
if [ -f "$BACKUP_DIR/secrets_$TIMESTAMP.yaml" ]; then
    kubectl apply -f "$BACKUP_DIR/secrets_$TIMESTAMP.yaml" 2>/dev/null || true
    print_status "SUCCESS" "Secrets restored"
else
    print_status "WARN" "Secrets backup not found"
fi

# Step 6: Restore configmaps
echo "📋 Step 6: Restoring configmaps..."
if [ -f "$BACKUP_DIR/configmaps_$TIMESTAMP.yaml" ]; then
    kubectl apply -f "$BACKUP_DIR/configmaps_$TIMESTAMP.yaml" 2>/dev/null || true
    print_status "SUCCESS" "Configmaps restored"
else
    print_status "WARN" "Configmaps backup not found"
fi

# Step 7: Restore custom resources
echo "📋 Step 7: Restoring custom resources..."
if [ -f "$BACKUP_DIR/devices_$TIMESTAMP.yaml" ]; then
    kubectl apply -f "$BACKUP_DIR/devices_$TIMESTAMP.yaml" 2>/dev/null || true
    print_status "SUCCESS" "Devices restored"
else
    print_status "WARN" "Devices backup not found"
fi

if [ -f "$BACKUP_DIR/applications_$TIMESTAMP.yaml" ]; then
    kubectl apply -f "$BACKUP_DIR/applications_$TIMESTAMP.yaml" 2>/dev/null || true
    print_status "SUCCESS" "Applications restored"
else
    print_status "WARN" "Applications backup not found"
fi

# Step 8: Restore network policies
echo "📋 Step 8: Restoring network policies..."
if [ -f "$BACKUP_DIR/networkpolicies_$TIMESTAMP.yaml" ]; then
    kubectl apply -f "$BACKUP_DIR/networkpolicies_$TIMESTAMP.yaml" 2>/dev/null || true
    print_status "SUCCESS" "Network policies restored"
else
    print_status "WARN" "Network policies backup not found"
fi

# Step 9: Restore certificates
echo "📋 Step 9: Restoring certificates..."
if [ -d "$BACKUP_DIR/certs_$TIMESTAMP" ]; then
    cp -r "$BACKUP_DIR/certs_$TIMESTAMP"/* resources/dev-certs/ 2>/dev/null || true
    print_status "SUCCESS" "Certificates restored"
else
    print_status "WARN" "Certificates backup not found"
fi

# Step 10: Wait for pods to be ready
echo "📋 Step 10: Waiting for pods to be ready..."
kubectl rollout status statefulset/wasmbed-gateway -n "$NAMESPACE" --timeout=300s 2>/dev/null || true
kubectl rollout status deployment/wasmbed-k8s-controller -n "$NAMESPACE" --timeout=300s 2>/dev/null || true
print_status "SUCCESS" "Pods are ready"

# Step 11: Verify restoration
echo "📋 Step 11: Verifying restoration..."
PODS=$(kubectl get pods -n "$NAMESPACE" --no-headers 2>/dev/null | wc -l)
RUNNING_PODS=$(kubectl get pods -n "$NAMESPACE" --no-headers 2>/dev/null | grep -c "Running" || echo "0")

if [ "$PODS" -gt 0 ] && [ "$RUNNING_PODS" -eq "$PODS" ]; then
    print_status "SUCCESS" "All $PODS pods are running"
else
    print_status "WARN" "Only $RUNNING_PODS/$PODS pods are running"
fi

# Cleanup extracted files
echo "📋 Cleaning up extracted files..."
rm -rf "$BACKUP_DIR"/*"$TIMESTAMP"* 2>/dev/null || true

echo ""
echo "🎉 Platform restoration completed!"
echo ""
echo "📊 Restoration Summary:"
echo "======================="
echo "Backup: $BACKUP_NAME"
echo "Namespace: $NAMESPACE"
echo "Pods running: $RUNNING_PODS/$PODS"
echo ""
echo "Next steps:"
echo "  ./wasmbed.sh status                   # Check platform status"
echo "  ./wasmbed.sh health-check             # Run health checks"
echo "  ./wasmbed.sh test                     # Run complete test suite"

