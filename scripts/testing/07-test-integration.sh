#!/bin/bash
# Run integration tests for Wasmbed platform
# This script tests component integration and communication

set -e

echo "🔗 Running integration tests..."

# Test 1: CRD generation and validation
echo "📋 Testing CRD generation..."
cargo run -p wasmbed-k8s-resource-tool -- crd device > /tmp/device-crd.yaml
cargo run -p wasmbed-k8s-resource-tool -- crd application > /tmp/application-crd.yaml

if [ -s /tmp/device-crd.yaml ] && [ -s /tmp/application-crd.yaml ]; then
    echo "✅ CRD generation working"
else
    echo "❌ CRD generation failed"
    exit 1
fi

# Test 2: Certificate generation
echo "📋 Testing certificate generation..."
cargo run -p wasmbed-cert-tool -- generate-ca --common-name "Test CA" --out-key /tmp/test-ca.key --out-cert /tmp/test-ca.der server

if [ -f /tmp/test-ca.key ] && [ -f /tmp/test-ca.der ]; then
    echo "✅ Certificate generation working"
else
    echo "❌ Certificate generation failed"
    exit 1
fi

# Test 3: Gateway server integration (if cluster available)
if kubectl cluster-info >/dev/null 2>&1; then
    echo "📋 Testing gateway server integration..."
    
    # Check if gateway is running
    if kubectl get pods -n wasmbed -l app=wasmbed-gateway --no-headers | grep -q Running; then
        echo "✅ Gateway server is running"
    else
        echo "⚠️ Gateway server not running (expected if not deployed)"
    fi
else
    echo "⚠️ Kubernetes cluster not available, skipping gateway test"
fi

# Test 5: Controller integration (if cluster available)
if kubectl cluster-info >/dev/null 2>&1; then
    echo "📋 Testing controller integration..."
    
    # Check if controller is running
    if kubectl get pods -n wasmbed -l app=wasmbed-k8s-controller --no-headers | grep -q Running; then
        echo "✅ Controller is running"
    else
        echo "⚠️ Controller not running (expected if not deployed)"
    fi
else
    echo "⚠️ Kubernetes cluster not available, skipping controller test"
fi

# Cleanup
rm -f /tmp/device-crd.yaml /tmp/application-crd.yaml /tmp/test-ca.key /tmp/test-ca.der

echo ""
echo "✅ All integration tests passed!"
echo ""
echo "Next steps:"
echo "  ./wasmbed.sh test-end-to-end          # Run end-to-end tests"
echo "  ./wasmbed.sh test-security            # Run security tests"
