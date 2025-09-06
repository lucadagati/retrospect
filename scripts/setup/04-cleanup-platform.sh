#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -euo pipefail

echo "🧹 Cleaning up Wasmbed environment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

cleanup_k3d() {
    echo -e "${BLUE}🔧 Cleaning up k3d cluster...${NC}"
    if k3d cluster list | grep -q wasmbed; then
        echo "  ⏹️  Stopping wasmbed cluster..."
        k3d cluster stop wasmbed || true
        echo "  🗑️  Deleting wasmbed cluster..."
        k3d cluster delete wasmbed || true
        echo -e "${GREEN}  ✅ k3d cluster cleaned up${NC}"
    else
        echo "  ℹ️  No wasmbed cluster found"
    fi
}

cleanup_docker() {
    echo -e "${BLUE}🐳 Cleaning up Docker images...${NC}"
    
    # Remove wasmbed-gateway images
    if docker images | grep -q wasmbed-gateway; then
        echo "  🗑️  Removing wasmbed-gateway images..."
        docker images --format "table {{.Repository}}:{{.Tag}}" | grep wasmbed-gateway | xargs -r docker rmi -f || true
        echo -e "${GREEN}  ✅ Docker images cleaned up${NC}"
    else
        echo "  ℹ️  No wasmbed-gateway images found"
    fi
}

cleanup_kubeconfig() {
    echo -e "${BLUE}🔧 Cleaning up kubeconfig...${NC}"
    if [ -n "${KUBECONFIG:-}" ]; then
        echo "  ⚠️  Unsetting KUBECONFIG environment variable"
        unset KUBECONFIG || true
    fi
    echo -e "${GREEN}  ✅ Kubeconfig cleaned up${NC}"
}

cleanup_build_artifacts() {
    echo -e "${BLUE}🧹 Cleaning up build artifacts...${NC}"
    
    # Remove Nix build results
    if [ -L "result" ]; then
        echo "  🗑️  Removing Nix build result symlink..."
        rm -f result
    fi
    
    # Clean Cargo build cache
    echo "  🗑️  Cleaning Cargo build cache..."
    cargo clean || true
    
    echo -e "${GREEN}  ✅ Build artifacts cleaned up${NC}"
}

main() {
    echo -e "${YELLOW}This will completely clean up the Wasmbed development environment.${NC}"
    echo -e "${YELLOW}This includes: k3d cluster, Docker images, kubeconfig, and build artifacts.${NC}"
    echo ""
    read -p "Are you sure you want to continue? (y/N): " -n 1 -r
    echo ""
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}⏹️  Cleanup cancelled${NC}"
        exit 0
    fi
    
    echo ""
    cleanup_k3d
    cleanup_docker
    cleanup_kubeconfig
    cleanup_build_artifacts
    
    echo ""
    echo -e "${GREEN}🎉 Cleanup completed successfully!${NC}"
    echo -e "${BLUE}You can now run ./scripts/setup.sh to rebuild the environment.${NC}"
}

main "$@"
