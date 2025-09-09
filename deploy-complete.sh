#!/bin/bash

# Wasmbed Complete Deployment Script
# Deploys the entire Wasmbed platform with MCU devices for testing

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CLUSTER_NAME="wasmbed-platform"
NAMESPACE="wasmbed"
MCU_DEVICES_COUNT=4
GATEWAY_REPLICAS=3

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    local missing_deps=()
    
    # Check required tools
    command -v docker >/dev/null 2>&1 || missing_deps+=("docker")
    command -v k3d >/dev/null 2>&1 || missing_deps+=("k3d")
    command -v kubectl >/dev/null 2>&1 || missing_deps+=("kubectl")
    command -v cargo >/dev/null 2>&1 || missing_deps+=("cargo")
    command -v openssl >/dev/null 2>&1 || missing_deps+=("openssl")
    command -v python3 >/dev/null 2>&1 || missing_deps+=("python3")
    command -v qemu-system-riscv64 >/dev/null 2>&1 || missing_deps+=("qemu-system-riscv64")
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_error "Please install missing dependencies and try again"
        exit 1
    fi
    
    log_success "All prerequisites satisfied"
}

# Clean up existing environment
cleanup_existing() {
    log_info "Cleaning up existing environment..."
    
    # Delete existing k3d cluster
    if k3d cluster list | grep -q "$CLUSTER_NAME"; then
        log_info "Deleting existing k3d cluster: $CLUSTER_NAME"
        k3d cluster delete "$CLUSTER_NAME" || true
    fi
    
    # Clean up Docker images
    log_info "Cleaning up Docker images..."
    docker image prune -f || true
    
    log_success "Cleanup completed"
}

# Create k3d cluster
create_cluster() {
    log_info "Creating k3d cluster: $CLUSTER_NAME"
    
    k3d cluster create "$CLUSTER_NAME" \
        --port "8080:80@loadbalancer" \
        --port "8443:443@loadbalancer" \
        --port "30000-30010:30000-30010@server:0" \
        --agents 2 \
        --wait
    
    # Configure kubectl with external certificates
    log_info "Configuring kubectl with external certificates..."
    fix_kubeconfig_certificates
    
    log_success "Cluster created and configured"
}

# Fix kubeconfig certificates to use external files
fix_kubeconfig_certificates() {
    log_info "Fixing kubeconfig certificates..."
    
    # Get the kubeconfig
    k3d kubeconfig get "$CLUSTER_NAME" > kubeconfig-temp.yaml
    
    # Extract certificates to external files
    python3 -c "
import yaml
import base64
import os

with open('kubeconfig-temp.yaml', 'r') as f:
    config = yaml.safe_load(f)

# Extract certificates
cluster = config['clusters'][0]['cluster']
user = config['users'][0]['user']

ca_cert = base64.b64decode(cluster['certificate-authority-data']).decode()
client_cert = base64.b64decode(user['client-certificate-data']).decode()
client_key = base64.b64decode(user['client-key-data']).decode()

# Write to files
os.makedirs('certs', exist_ok=True)
with open('certs/k3d-ca.crt', 'w') as f:
    f.write(ca_cert)

with open('certs/k3d-client.crt', 'w') as f:
    f.write(client_cert)

with open('certs/k3d-client.key', 'w') as f:
    f.write(client_key)

print('Certificates extracted successfully')
" || {
        log_error "Failed to extract certificates"
        exit 1
    }
    
    # Update kubeconfig to use external files
    python3 -c "
import yaml
import os

with open('kubeconfig-temp.yaml', 'r') as f:
    config = yaml.safe_load(f)

# Update cluster config to use external CA file
config['clusters'][0]['cluster']['certificate-authority'] = os.path.abspath('certs/k3d-ca.crt')
del config['clusters'][0]['cluster']['certificate-authority-data']

# Update user config to use external cert/key files
config['users'][0]['user']['client-certificate'] = os.path.abspath('certs/k3d-client.crt')
config['users'][0]['user']['client-key'] = os.path.abspath('certs/k3d-client.key')
del config['users'][0]['user']['client-certificate-data']
del config['users'][0]['user']['client-key-data']

# Write updated config
kubeconfig_path = os.path.expanduser('~/.kube/config')
os.makedirs(os.path.dirname(kubeconfig_path), exist_ok=True)
with open(kubeconfig_path, 'w') as f:
    yaml.dump(config, f, default_flow_style=False)

print('Kubeconfig updated successfully')
" || {
        log_error "Failed to update kubeconfig"
        exit 1
    }
    
    # Clean up temp file
    rm -f kubeconfig-temp.yaml
    
    # Set KUBECONFIG environment variable
    export KUBECONFIG="$HOME/.kube/config"
    
    log_success "Kubeconfig certificates fixed"
}

# Generate certificates
generate_certificates() {
    log_info "Generating TLS certificates..."
    
    # Create certs directory
    mkdir -p certs
    
    # Generate CA
    openssl genrsa -out certs/ca-key.pem 4096
    openssl req -new -x509 -days 365 -key certs/ca-key.pem -out certs/ca-cert.pem \
        -subj "/C=US/ST=CA/L=San Francisco/O=Wasmbed/OU=Development/CN=wasmbed-ca"
    
    # Generate server certificate
    openssl genrsa -out certs/server-key.pem 4096
    openssl req -new -key certs/server-key.pem -out certs/server.csr \
        -subj "/C=US/ST=CA/L=San Francisco/O=Wasmbed/OU=Development/CN=wasmbed-gateway"
    openssl x509 -req -in certs/server.csr -CA certs/ca-cert.pem -CAkey certs/ca-key.pem \
        -out certs/server-cert.pem -days 365 -CAcreateserial
    
    # Generate client certificates for MCU devices
    for i in $(seq 1 $MCU_DEVICES_COUNT); do
        openssl genrsa -out "certs/client-${i}-key.pem" 4096
        openssl req -new -key "certs/client-${i}-key.pem" -out "certs/client-${i}.csr" \
            -subj "/C=US/ST=CA/L=San Francisco/O=Wasmbed/OU=Development/CN=mcu-device-${i}"
        openssl x509 -req -in "certs/client-${i}.csr" -CA certs/ca-cert.pem -CAkey certs/ca-key.pem \
            -out "certs/client-${i}-cert.pem" -days 365 -CAcreateserial
    done
    
    # Clean up CSR files
    rm -f certs/*.csr certs/*.srl
    
    log_success "Certificates generated successfully"
}

# Build and deploy components
build_and_deploy() {
    log_info "Building and deploying components..."
    
    # Build Gateway Docker image
    log_info "Building Gateway Docker image..."
    docker build -f crates/wasmbed-gateway/Dockerfile -t wasmbed-gateway:latest .
    
    # Build Controller Docker image
    log_info "Building Controller Docker image..."
    docker build -f crates/wasmbed-k8s-controller/Dockerfile -t wasmbed-k8s-controller:latest .
    
    # Import images to k3d
    log_info "Importing images to k3d..."
    k3d image import wasmbed-gateway:latest -c "$CLUSTER_NAME"
    k3d image import wasmbed-k8s-controller:latest -c "$CLUSTER_NAME"
    
    # Create namespace
    log_info "Creating namespace: $NAMESPACE"
    kubectl create namespace "$NAMESPACE" || true
    
    # Create TLS secrets with correct names
    log_info "Creating TLS secrets..."
    kubectl create secret tls wasmbed-tls-secret-rsa \
        --cert=certs/server-cert.pem \
        --key=certs/server-key.pem \
        -n "$NAMESPACE" || true
    
    kubectl create secret generic wasmbed-ca-secret-rsa \
        --from-file=ca-cert.pem=certs/ca-cert.pem \
        -n "$NAMESPACE" || true
    
    # Deploy CRDs
    log_info "Deploying Custom Resource Definitions..."
    kubectl apply -f resources/k8s/crds/ -n "$NAMESPACE"
    
    # Deploy RBAC
    log_info "Deploying RBAC configuration..."
    kubectl apply -f resources/k8s/100-service-account-gateway.yaml -n "$NAMESPACE"
    kubectl apply -f resources/k8s/101-cluster-role-gateway-device-access.yaml -n "$NAMESPACE"
    kubectl apply -f resources/k8s/102-cluster-rolebinding-gateway.yaml -n "$NAMESPACE"
    
    # Deploy Controller
    log_info "Deploying Controller..."
    kubectl apply -f resources/k8s/controller-rbac.yaml -n "$NAMESPACE"
    kubectl apply -f resources/k8s/controller-deployment.yaml -n "$NAMESPACE"
    
    # Deploy Gateway
    log_info "Deploying Gateway..."
    kubectl apply -f resources/k8s/110-service-gateway.yaml -n "$NAMESPACE"
    kubectl apply -f resources/k8s/111-statefulset-gateway.yaml -n "$NAMESPACE"
    
    log_success "Components deployed successfully"
}

# Create MCU devices
create_mcu_devices() {
    log_info "Creating $MCU_DEVICES_COUNT MCU devices..."
    
    for i in $(seq 1 $MCU_DEVICES_COUNT); do
        cat <<EOF | kubectl apply -f -
apiVersion: wasmbed.github.io/v0
kind: Device
metadata:
  name: mcu-device-${i}
  namespace: $NAMESPACE
spec:
  deviceId: "mcu-device-${i}"
  publicKey: "$(openssl rsa -in "certs/client-${i}-key.pem" -pubout -outform PEM | base64 -w 0 | tr '+/' '-_' | tr -d '=')"
  deviceType: "riscv-hifive1"
  capabilities:
    - "wasm-execution"
    - "tls-client"
    - "microROS"
    - "DDS-communication"
EOF
    done
    
    log_success "MCU devices created successfully"
}

# Wait for deployment
wait_for_deployment() {
    log_info "Waiting for deployment to be ready..."
    
    # Wait for CRDs first
    kubectl wait --for=condition=established crd/devices.wasmbed.github.io --timeout=60s
    kubectl wait --for=condition=established crd/applications.wasmbed.github.io --timeout=60s
    
    # Wait for Controller pods
    kubectl wait --for=condition=ready pod -l app=wasmbed-k8s-controller -n "$NAMESPACE" --timeout=300s || {
        log_warning "Controller pods not ready, checking logs..."
        kubectl logs -l app=wasmbed-k8s-controller -n "$NAMESPACE" --tail=20
    }
    
    # Wait for Gateway pods
    kubectl wait --for=condition=ready pod -l app=wasmbed-gateway -n "$NAMESPACE" --timeout=300s || {
        log_warning "Gateway pods not ready, checking logs..."
        kubectl logs -l app=wasmbed-gateway -n "$NAMESPACE" --tail=20
    }
    
    log_success "Deployment is ready"
}

# Verify deployment
verify_deployment() {
    log_info "Verifying deployment..."
    
    # Check cluster status
    kubectl cluster-info
    
    # Check pods
    log_info "Pod status:"
    kubectl get pods -n "$NAMESPACE"
    
    # Check services
    log_info "Service status:"
    kubectl get services -n "$NAMESPACE"
    
    # Check devices
    log_info "Device status:"
    kubectl get devices -n "$NAMESPACE"
    
    # Check CRDs
    log_info "CRD status:"
    kubectl get crd | grep wasmbed
    
    log_success "Deployment verification completed"
}

# Main deployment function
main() {
    log_info "Starting Wasmbed complete deployment..."
    log_info "Cluster: $CLUSTER_NAME"
    log_info "Namespace: $NAMESPACE"
    log_info "MCU Devices: $MCU_DEVICES_COUNT"
    log_info "Gateway Replicas: $GATEWAY_REPLICAS"
    
    check_prerequisites
    cleanup_existing
    create_cluster
    generate_certificates
    build_and_deploy
    create_mcu_devices
    wait_for_deployment
    verify_deployment
    
    log_success "Wasmbed platform deployed successfully!"
    log_info "Gateway available at: http://localhost:8080"
    log_info "Gateway TLS available at: https://localhost:8443"
    log_info "Gateway NodePort: 30423 (external access)"
    log_info "MCU devices ready for microROS applications"
    
    echo ""
    log_info "Deployment Summary:"
    log_info "- Cluster: $CLUSTER_NAME (k3d)"
    log_info "- Namespace: $NAMESPACE"
    log_info "- Gateway Replicas: $GATEWAY_REPLICAS"
    log_info "- MCU Devices: $MCU_DEVICES_COUNT"
    log_info "- Custom TLS Library: Active"
    log_info "- Controller: Running"
    
    echo ""
    log_info "Next steps:"
    log_info "1. Run: ./run-microROS-app.sh"
    log_info "2. Check logs: kubectl logs -l app=wasmbed-gateway -n $NAMESPACE"
    log_info "3. Monitor devices: kubectl get devices -n $NAMESPACE"
    log_info "4. Or run: ./cleanup-all.sh to clean up"
}

# Run main function
main "$@"
