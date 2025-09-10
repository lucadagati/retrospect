#!/bin/bash

# Deploy ROS 2 microROS Bridge to Kubernetes
set -e

echo "🚀 Deploying ROS 2 microROS Bridge to Kubernetes..."

# Check if kubectl is available
if ! command -v kubectl &> /dev/null; then
    echo "❌ kubectl is not installed or not in PATH"
    exit 1
fi

# Check if cluster is accessible
if ! kubectl cluster-info &> /dev/null; then
    echo "❌ Cannot access Kubernetes cluster"
    exit 1
fi

echo "✅ Kubernetes cluster is accessible"

# Apply ROS 2 namespaces
echo "📦 Creating ROS 2 namespaces..."
kubectl apply -f resources/k8s/ros2/namespace.yaml

# Apply ROS 2 RBAC
echo "🔐 Setting up ROS 2 RBAC..."
kubectl apply -f resources/k8s/ros2/rbac.yaml

# Apply ROS 2 configuration
echo "⚙️ Applying ROS 2 configuration..."
kubectl apply -f resources/k8s/ros2/configmap.yaml

# Apply microROS agent
echo "🤖 Deploying microROS agent..."
kubectl apply -f resources/k8s/ros2/microros-agent.yaml

# Wait for microROS agent to be ready
echo "⏳ Waiting for microROS agent to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment/microros-agent -n ros2-system

# Build and push the microROS bridge image (if needed)
echo "🔨 Building microROS bridge image..."
cd crates/wasmbed-microros-bridge
docker build -t wasmbed-microros-bridge:latest .
cd ../..

# Apply microROS bridge deployment
echo "🌉 Deploying microROS bridge..."
kubectl apply -f resources/k8s/ros2/wasmbed-microros-bridge.yaml

# Wait for bridge to be ready
echo "⏳ Waiting for microROS bridge to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment/wasmbed-microros-bridge -n ros2-system

# Apply example ROS 2 application
echo "📱 Deploying example ROS 2 application..."
kubectl apply -f resources/k8s/ros2/examples/drone-ros2-app.yaml

# Show deployment status
echo "📊 Deployment status:"
kubectl get pods -n ros2-system
kubectl get services -n ros2-system
kubectl get ros2topics -n ros2-apps
kubectl get ros2services -n ros2-apps

echo "✅ ROS 2 microROS Bridge deployment completed!"
echo ""
echo "🔗 Access points:"
echo "  - microROS Agent: kubectl port-forward -n ros2-system svc/microros-agent 8888:8888"
echo "  - microROS Bridge: kubectl port-forward -n ros2-system svc/wasmbed-microros-bridge 8080:8080"
echo "  - ROS Bridge WebSocket: kubectl port-forward -n ros2-system svc/microros-agent 9090:9090"
echo ""
echo "🧪 Test the deployment:"
echo "  curl http://localhost:8080/health"
echo "  curl http://localhost:8080/status"
echo "  curl http://localhost:8080/topics"
