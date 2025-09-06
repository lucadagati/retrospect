#!/bin/bash
# Build Docker images for Wasmbed platform
# This script builds and tags all required Docker images

set -e

echo "🐳 Building Wasmbed Docker images..."

# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    echo " Docker is not running"
    echo "Please start Docker and try again"
    exit 1
fi

echo " Docker is running"

# Build gateway image
echo " Building wasmbed-gateway image..."
docker build -f crates/wasmbed-gateway/Dockerfile -t wasmbed-gateway:latest .

if [ $? -eq 0 ]; then
    echo " wasmbed-gateway image built successfully"
else
    echo " Failed to build wasmbed-gateway image"
    exit 1
fi

# Build controller image
echo " Building wasmbed-k8s-controller image..."
docker build -f crates/wasmbed-k8s-controller/Dockerfile -t wasmbed-k8s-controller:latest .

if [ $? -eq 0 ]; then
    echo " wasmbed-k8s-controller image built successfully"
else
    echo " Failed to build wasmbed-k8s-controller image"
    exit 1
fi

# Show built images
echo " Built images:"
docker images | grep wasmbed

echo ""
echo " All Docker images built successfully!"
echo ""
echo "Next steps:"
echo "  ./wasmbed.sh deploy                    # Deploy to Kubernetes"
echo "  k3d image import wasmbed-gateway:latest -c wasmbed      # Import to k3d"
echo "  k3d image import wasmbed-k8s-controller:latest -c wasmbed # Import to k3d"

