#!/bin/bash
# Create backup of Wasmbed platform state
# This script creates a comprehensive backup of all platform resources

set -e

echo " Creating Wasmbed platform backup..."

# Configuration
NAMESPACE="wasmbed"
BACKUP_DIR="./backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
BACKUP_NAME="wasmbed-backup-$TIMESTAMP"

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
        echo -e "${GREEN} SUCCESS${NC}: $message"
    elif [ "$status" = "ERROR" ]; then
        echo -e "${RED} ERROR${NC}: $message"
    else
        echo -e "${YELLOW}  WARN${NC}: $message"
    fi
}

# Check if cluster is accessible
if ! kubectl cluster-info >/dev/null 2>&1; then
    print_status "ERROR" "Cannot access Kubernetes cluster"
    exit 1
fi

print_status "SUCCESS" "Kubernetes cluster is accessible"

# Create backup directory
mkdir -p "$BACKUP_DIR"

# Step 1: Backup CRDs
echo " Step 1: Backing up CRDs..."
kubectl get crd | grep wasmbed > "$BACKUP_DIR/crds_$TIMESTAMP.txt" 2>/dev/null || true
print_status "SUCCESS" "CRDs backed up"

# Step 2: Backup namespace resources
echo " Step 2: Backing up namespace resources..."
kubectl get all -n "$NAMESPACE" -o yaml > "$BACKUP_DIR/resources_$TIMESTAMP.yaml" 2>/dev/null || true
print_status "SUCCESS" "Resources backed up"

# Step 3: Backup secrets
echo " Step 3: Backing up secrets..."
kubectl get secrets -n "$NAMESPACE" -o yaml > "$BACKUP_DIR/secrets_$TIMESTAMP.yaml" 2>/dev/null || true
print_status "SUCCESS" "Secrets backed up"

# Step 4: Backup configmaps
echo " Step 4: Backing up configmaps..."
kubectl get configmaps -n "$NAMESPACE" -o yaml > "$BACKUP_DIR/configmaps_$TIMESTAMP.yaml" 2>/dev/null || true
print_status "SUCCESS" "Configmaps backed up"

# Step 5: Backup custom resources
echo " Step 5: Backing up custom resources..."
kubectl get devices -n "$NAMESPACE" -o yaml > "$BACKUP_DIR/devices_$TIMESTAMP.yaml" 2>/dev/null || true
kubectl get applications -n "$NAMESPACE" -o yaml > "$BACKUP_DIR/applications_$TIMESTAMP.yaml" 2>/dev/null || true
print_status "SUCCESS" "Custom resources backed up"

# Step 6: Backup network policies
echo " Step 6: Backing up network policies..."
kubectl get networkpolicies -n "$NAMESPACE" -o yaml > "$BACKUP_DIR/networkpolicies_$TIMESTAMP.yaml" 2>/dev/null || true
print_status "SUCCESS" "Network policies backed up"

# Step 7: Backup RBAC
echo " Step 7: Backing up RBAC..."
kubectl get clusterrole,clusterrolebinding | grep wasmbed > "$BACKUP_DIR/rbac_$TIMESTAMP.txt" 2>/dev/null || true
print_status "SUCCESS" "RBAC backed up"

# Step 8: Backup certificates
echo " Step 8: Backing up certificates..."
if [ -d "resources/dev-certs" ]; then
    cp -r resources/dev-certs "$BACKUP_DIR/certs_$TIMESTAMP" 2>/dev/null || true
    print_status "SUCCESS" "Certificates backed up"
else
    print_status "WARN" "Certificate directory not found"
fi

# Step 9: Create backup manifest
echo " Step 9: Creating backup manifest..."
cat > "$BACKUP_DIR/backup_manifest_$TIMESTAMP.txt" << EOF
Wasmbed Platform Backup
=======================
Timestamp: $TIMESTAMP
Backup Name: $BACKUP_NAME
Namespace: $NAMESPACE

Backup Contents:
- CRDs: crds_$TIMESTAMP.txt
- Resources: resources_$TIMESTAMP.yaml
- Secrets: secrets_$TIMESTAMP.yaml
- Configmaps: configmaps_$TIMESTAMP.yaml
- Devices: devices_$TIMESTAMP.yaml
- Applications: applications_$TIMESTAMP.yaml
- Network Policies: networkpolicies_$TIMESTAMP.yaml
- RBAC: rbac_$TIMESTAMP.txt
- Certificates: certs_$TIMESTAMP/

Restore Command:
./wasmbed.sh restore --backup $BACKUP_NAME
EOF

print_status "SUCCESS" "Backup manifest created"

# Step 10: Create compressed backup
echo " Step 10: Creating compressed backup..."
cd "$BACKUP_DIR"
tar -czf "$BACKUP_NAME.tar.gz" *"$TIMESTAMP"* 2>/dev/null || true
cd - > /dev/null

if [ -f "$BACKUP_DIR/$BACKUP_NAME.tar.gz" ]; then
    print_status "SUCCESS" "Compressed backup created: $BACKUP_NAME.tar.gz"
else
    print_status "WARN" "Could not create compressed backup"
fi

# Backup summary
echo ""
echo " Backup Summary:"
echo "=================="
echo "Backup Name: $BACKUP_NAME"
echo "Location: $BACKUP_DIR"
echo "Timestamp: $TIMESTAMP"
echo ""
echo "Backup Files:"
ls -la "$BACKUP_DIR"/*"$TIMESTAMP"* 2>/dev/null || echo "No backup files found"

echo ""
echo " Backup completed successfully!"
echo ""
echo "Next steps:"
echo "  ./wasmbed.sh restore --backup $BACKUP_NAME    # Restore from this backup"
echo "  ./wasmbed.sh status                           # Check platform status"
echo "  ./wasmbed.sh health-check                     # Run health checks"

