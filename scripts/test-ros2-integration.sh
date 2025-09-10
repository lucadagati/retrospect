#!/bin/bash

# Test ROS 2 Integration in Kubernetes
set -e

echo "🧪 Testing ROS 2 Integration in Kubernetes..."

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

# Test 1: Check if ROS 2 namespaces exist
echo "📦 Testing ROS 2 namespaces..."
if kubectl get namespace ros2-system &> /dev/null; then
    echo "✅ ros2-system namespace exists"
else
    echo "❌ ros2-system namespace not found"
    exit 1
fi

if kubectl get namespace ros2-apps &> /dev/null; then
    echo "✅ ros2-apps namespace exists"
else
    echo "❌ ros2-apps namespace not found"
    exit 1
fi

# Test 2: Check if microROS agent is running
echo "🤖 Testing microROS agent..."
if kubectl get deployment microros-agent -n ros2-system &> /dev/null; then
    echo "✅ microROS agent deployment exists"
    
    # Check if pods are running
    if kubectl get pods -n ros2-system -l app=microros-agent --field-selector=status.phase=Running | grep -q microros-agent; then
        echo "✅ microROS agent pods are running"
    else
        echo "❌ microROS agent pods are not running"
        kubectl get pods -n ros2-system -l app=microros-agent
        exit 1
    fi
else
    echo "❌ microROS agent deployment not found"
    exit 1
fi

# Test 3: Check if microROS bridge is running
echo "🌉 Testing microROS bridge..."
if kubectl get deployment wasmbed-microros-bridge -n ros2-system &> /dev/null; then
    echo "✅ microROS bridge deployment exists"
    
    # Check if pods are running
    if kubectl get pods -n ros2-system -l app=wasmbed-microros-bridge --field-selector=status.phase=Running | grep -q wasmbed-microros-bridge; then
        echo "✅ microROS bridge pods are running"
    else
        echo "❌ microROS bridge pods are not running"
        kubectl get pods -n ros2-system -l app=wasmbed-microros-bridge
        exit 1
    fi
else
    echo "❌ microROS bridge deployment not found"
    exit 1
fi

# Test 4: Check if CRDs are installed
echo "📋 Testing ROS 2 CRDs..."
if kubectl get crd ros2topics.wasmbed.io &> /dev/null; then
    echo "✅ ROS2Topic CRD exists"
else
    echo "❌ ROS2Topic CRD not found"
    exit 1
fi

if kubectl get crd ros2services.wasmbed.io &> /dev/null; then
    echo "✅ ROS2Service CRD exists"
else
    echo "❌ ROS2Service CRD not found"
    exit 1
fi

# Test 5: Check if example ROS 2 application is deployed
echo "📱 Testing example ROS 2 application..."
if kubectl get ros2topic drone-telemetry -n ros2-apps &> /dev/null; then
    echo "✅ drone-telemetry ROS2Topic exists"
else
    echo "❌ drone-telemetry ROS2Topic not found"
    exit 1
fi

if kubectl get ros2topic drone-commands -n ros2-apps &> /dev/null; then
    echo "✅ drone-commands ROS2Topic exists"
else
    echo "❌ drone-commands ROS2Topic not found"
    exit 1
fi

if kubectl get ros2service drone-arm-service -n ros2-apps &> /dev/null; then
    echo "✅ drone-arm-service ROS2Service exists"
else
    echo "❌ drone-arm-service ROS2Service not found"
    exit 1
fi

# Test 6: Test HTTP API endpoints
echo "🌐 Testing HTTP API endpoints..."

# Start port forwarding for microROS bridge
echo "🔗 Starting port forwarding for microROS bridge..."
kubectl port-forward -n ros2-system svc/wasmbed-microros-bridge 8080:8080 &
PORT_FORWARD_PID=$!

# Wait for port forwarding to be ready
sleep 5

# Test health endpoint
echo "🏥 Testing health endpoint..."
if curl -s http://localhost:8080/health | grep -q "healthy"; then
    echo "✅ Health endpoint is working"
else
    echo "❌ Health endpoint is not working"
    kill $PORT_FORWARD_PID 2>/dev/null || true
    exit 1
fi

# Test status endpoint
echo "📊 Testing status endpoint..."
if curl -s http://localhost:8080/status | grep -q "initialized"; then
    echo "✅ Status endpoint is working"
else
    echo "❌ Status endpoint is not working"
    kill $PORT_FORWARD_PID 2>/dev/null || true
    exit 1
fi

# Test topics endpoint
echo "📝 Testing topics endpoint..."
if curl -s http://localhost:8080/topics | grep -q "input_topics"; then
    echo "✅ Topics endpoint is working"
else
    echo "❌ Topics endpoint is not working"
    kill $PORT_FORWARD_PID 2>/dev/null || true
    exit 1
fi

# Clean up port forwarding
kill $PORT_FORWARD_PID 2>/dev/null || true

# Test 7: Test ROS 2 message publishing (if possible)
echo "📤 Testing ROS 2 message publishing..."
# This would require a ROS 2 client, which we'll implement later
echo "⏳ ROS 2 message publishing test skipped (requires ROS 2 client)"

echo ""
echo "🎉 All ROS 2 integration tests passed!"
echo ""
echo "📊 Summary:"
echo "  ✅ ROS 2 namespaces created"
echo "  ✅ microROS agent running"
echo "  ✅ microROS bridge running"
echo "  ✅ ROS 2 CRDs installed"
echo "  ✅ Example ROS 2 application deployed"
echo "  ✅ HTTP API endpoints working"
echo ""
echo "🚀 ROS 2 integration is ready for use!"
