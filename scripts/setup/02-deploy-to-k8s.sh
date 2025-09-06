#!/bin/bash
# Deploy Wasmbed platform to Kubernetes
# This script deploys all components to a running Kubernetes cluster

set -e

echo " Deploying Wasmbed platform to Kubernetes..."

# Check if cluster is accessible
if ! kubectl cluster-info >/dev/null 2>&1; then
    echo " Cannot access Kubernetes cluster"
    echo "Please ensure k3d cluster is running: k3d cluster start wasmbed"
    exit 1
fi

echo " Kubernetes cluster is accessible"

# Create namespace
echo " Creating namespace..."
kubectl apply -f resources/k8s/000-namespace.yaml

# Apply RBAC
echo " Applying RBAC configuration..."
kubectl apply -f resources/k8s/100-service-account-gateway.yaml
kubectl apply -f resources/k8s/101-cluster-role-gateway-device-access.yaml
kubectl apply -f resources/k8s/102-cluster-rolebinding-gateway.yaml
kubectl apply -f resources/k8s/controller-rbac.yaml

# Apply CRDs
echo " Applying Custom Resource Definitions..."
cargo run -p wasmbed-k8s-resource-tool -- crd device | kubectl apply -f -
cargo run -p wasmbed-k8s-resource-tool -- crd application | kubectl apply -f -

# Apply services
echo "🔌 Applying services..."
kubectl apply -f resources/k8s/110-service-gateway.yaml

# Apply deployments
echo " Applying deployments..."
kubectl apply -f resources/k8s/111-statefulset-gateway.yaml
kubectl apply -f resources/k8s/controller-deployment.yaml

# Apply network policies
echo " Applying network policies..."
kubectl apply -f resources/k8s/network-policies.yaml

# Wait for pods to be ready
echo "⏳ Waiting for pods to be ready..."
kubectl rollout status statefulset/wasmbed-gateway -n wasmbed --timeout=300s
kubectl rollout status deployment/wasmbed-k8s-controller -n wasmbed --timeout=300s

echo " Deployment completed successfully!"

# Show status
echo " Platform status:"
kubectl get pods -n wasmbed
kubectl get services -n wasmbed

echo ""
echo " Wasmbed platform deployed successfully!"
echo ""
echo "Next steps:"
echo "  ./wasmbed.sh test                    # Run complete test suite"
echo "  ./wasmbed.sh security-scan           # Run security validation"
echo "  ./wasmbed.sh monitor                 # Start monitoring"
