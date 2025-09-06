#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CLUSTER_NAME="wasmbed"
NAMESPACE="wasmbed"
GATEWAY_IMAGE_TAG=""

echo "🚀 Setting up Wasmbed development environment..."

check_prerequisites() {
    echo -e "${BLUE}📋 Checking prerequisites...${NC}"
    
    local missing_tools=()
    
    if ! command -v nix &> /dev/null; then
        missing_tools+=("nix")
    fi
    
    if ! command -v docker &> /dev/null; then
        missing_tools+=("docker")
    fi
    
    if ! command -v k3d &> /dev/null; then
        missing_tools+=("k3d")
    fi
    
    if ! command -v kubectl &> /dev/null; then
        missing_tools+=("kubectl")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        echo -e "${RED}❌ Missing required tools: ${missing_tools[*]}${NC}"
        echo "Please install missing tools and run the setup again."
        exit 1
    fi
    
    # Check if we're in Nix shell
    if [ -z "${IN_NIX_SHELL:-}" ]; then
        echo -e "${YELLOW}⚠️  Not in Nix shell. Running 'nix develop' first...${NC}"
        exec nix develop --command "$0" "$@"
    fi
    
    echo -e "${GREEN}✅ Prerequisites check passed${NC}"
}

build_gateway_image() {
    echo -e "${BLUE}🔨 Building gateway Docker image...${NC}"
    
    # Build the Docker image
    echo "  📦 Building image with Nix..."
    nix build '.#dockerImages.x86_64-linux.wasmbed-gateway'
    
    # Load the image and capture the tag
    echo "  📥 Loading image into Docker..."
    local load_output
    load_output=$(docker load -i "$(readlink result)")
    
    # Extract the image tag from Docker load output
    GATEWAY_IMAGE_TAG=$(echo "$load_output" | grep "Loaded image:" | sed 's/Loaded image: //')
    
    if [ -z "$GATEWAY_IMAGE_TAG" ]; then
        echo -e "${RED}❌ Failed to extract image tag${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Gateway image built: ${GATEWAY_IMAGE_TAG}${NC}"
}

setup_k3d_cluster() {
    echo -e "${BLUE}🏗️  Setting up k3d cluster...${NC}"
    
    # Check if cluster already exists
    if k3d cluster list | grep -q "$CLUSTER_NAME"; then
        echo "  ⚠️  Cluster $CLUSTER_NAME already exists, deleting..."
        k3d cluster delete "$CLUSTER_NAME"
    fi
    
    # Create the cluster
    echo "  🆕 Creating k3d cluster..."
    k3d cluster create --config resources/k3d/config.yaml
    
    # Configure kubectl
    echo "  ⚙️  Configuring kubectl..."
    export KUBECONFIG=$(k3d kubeconfig write "$CLUSTER_NAME")
    
    # Wait for cluster to be ready
    echo "  ⏳ Waiting for cluster to be ready..."
    kubectl wait --for=condition=Ready nodes --all --timeout=60s
    
    echo -e "${GREEN}✅ k3d cluster is ready${NC}"
    echo -e "${BLUE}💡 KUBECONFIG is set to: $KUBECONFIG${NC}"
}

import_images() {
    echo -e "${BLUE}📦 Importing images into k3d cluster...${NC}"
    
    echo "  📥 Importing gateway image: $GATEWAY_IMAGE_TAG"
    k3d image import -c "$CLUSTER_NAME" "$GATEWAY_IMAGE_TAG"
    
    echo -e "${GREEN}✅ Images imported successfully${NC}"
}

deploy_kubernetes_resources() {
    echo -e "${BLUE}🚢 Deploying Kubernetes resources...${NC}"
    
    # Create namespace
    echo "  📁 Creating namespace..."
    kubectl apply -f resources/k8s/000-namespace.yaml
    
    # Deploy CRDs
    echo "  📜 Deploying Device CRD..."
    cargo run -p wasmbed-k8s-resource-tool crd device | kubectl -n "$NAMESPACE" apply -f -
    
    echo "  📜 Deploying Application CRD..."
    cargo run -p wasmbed-k8s-resource-tool crd application | kubectl -n "$NAMESPACE" apply -f -
    
    # Deploy RBAC resources
    echo "  🔐 Deploying RBAC resources..."
    kubectl apply -f resources/k8s/100-service-account-gateway.yaml
    kubectl apply -f resources/k8s/101-cluster-role-gateway-device-access.yaml
    kubectl apply -f resources/k8s/102-cluster-rolebinding-gateway.yaml
    
    # Deploy service
    echo "  🌐 Deploying gateway service..."
    kubectl apply -f resources/k8s/110-service-gateway.yaml
    
    # Update StatefulSet with correct image tag and deploy
    echo "  🎯 Deploying gateway StatefulSet with image: $GATEWAY_IMAGE_TAG"
    sed "s|image: wasmbed-gateway:.*|image: $GATEWAY_IMAGE_TAG|" \
        resources/k8s/111-statefulset-gateway.yaml | \
        kubectl apply -f -
    
    # Scale to 1 replica for simplicity
    echo "  📏 Scaling to 1 replica..."
    kubectl -n "$NAMESPACE" scale statefulset wasmbed-gateway --replicas=1
    
    echo -e "${GREEN}✅ Kubernetes resources deployed${NC}"
}

create_test_device() {
    echo -e "${BLUE}📱 Creating test device...${NC}"
    
    cargo run -p wasmbed-k8s-resource-tool manifest device \
        --name device-0 \
        --cert resources/dev-certs/client-0.der \
    | kubectl -n "$NAMESPACE" apply -f -
    
    echo -e "${GREEN}✅ Test device created${NC}"
}

wait_for_gateway() {
    echo -e "${BLUE}⏳ Waiting for gateway to be ready...${NC}"
    
    # Wait for StatefulSet to be ready
    kubectl -n "$NAMESPACE" rollout status statefulset/wasmbed-gateway --timeout=300s
    
    # Wait for pod to be ready
    kubectl -n "$NAMESPACE" wait --for=condition=ready pod/wasmbed-gateway-0 --timeout=300s
    
    echo -e "${GREEN}✅ Gateway is ready${NC}"
}

display_status() {
    echo ""
    echo -e "${GREEN}🎉 Wasmbed environment setup completed successfully!${NC}"
    echo ""
    echo -e "${BLUE}📊 Environment Status:${NC}"
    echo "  🏠 Cluster: $CLUSTER_NAME"
    echo "  📁 Namespace: $NAMESPACE"
    echo "  🐳 Gateway Image: $GATEWAY_IMAGE_TAG"
    echo "  ⚙️  KUBECONFIG: $KUBECONFIG"
    echo ""
    echo -e "${BLUE}🔍 Quick status check:${NC}"
    kubectl -n "$NAMESPACE" get pods,services,devices
    echo ""
    echo -e "${BLUE}💡 Next steps:${NC}"
    echo "  1. Run './scripts/test.sh' to test the connection"
    echo "  2. Run './scripts/monitor.sh' to monitor the system"
    echo "  3. To cleanup: './scripts/cleanup.sh'"
    echo ""
    echo -e "${YELLOW}📝 Remember to export KUBECONFIG in new terminal sessions:${NC}"
    echo "  export KUBECONFIG=$KUBECONFIG"
}

main() {
    echo -e "${YELLOW}This will set up a complete Wasmbed development environment.${NC}"
    echo ""
    
    check_prerequisites
    build_gateway_image
    setup_k3d_cluster
    import_images
    deploy_kubernetes_resources
    create_test_device
    wait_for_gateway
    display_status
}

main "$@"
