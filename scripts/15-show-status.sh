#!/bin/bash
# Show platform status and health
# This script displays comprehensive platform status

set -e

echo "📊 Wasmbed Platform Status Report"
echo "=================================="
echo ""

# Check if cluster is accessible
if ! kubectl cluster-info >/dev/null 2>&1; then
    echo "❌ Cannot access Kubernetes cluster"
    echo "Please ensure k3d cluster is running: k3d cluster start wasmbed"
    exit 1
fi

echo "✅ Kubernetes cluster is accessible"
echo ""

# Cluster info
echo "🏠 Cluster Information:"
kubectl cluster-info | head -2
echo ""

# Namespace status
echo "📋 Namespace Status:"
if kubectl get namespace wasmbed >/dev/null 2>&1; then
    echo "✅ wasmbed namespace exists"
else
    echo "❌ wasmbed namespace missing"
fi
echo ""

# CRD status
echo "📋 Custom Resource Definitions:"
CRDS=$(kubectl get crd | grep wasmbed | wc -l)
if [ "$CRDS" -gt 0 ]; then
    echo "✅ $CRDS CRDs installed:"
    kubectl get crd | grep wasmbed
else
    echo "❌ No CRDs found"
fi
echo ""

# Pod status
echo "🚀 Pod Status:"
PODS=$(kubectl get pods -n wasmbed --no-headers 2>/dev/null | wc -l)
if [ "$PODS" -gt 0 ]; then
    echo "📊 Found $PODS pods:"
    kubectl get pods -n wasmbed
else
    echo "❌ No pods found in wasmbed namespace"
fi
echo ""

# Service status
echo "🔌 Service Status:"
SERVICES=$(kubectl get services -n wasmbed --no-headers 2>/dev/null | wc -l)
if [ "$SERVICES" -gt 0 ]; then
    echo "📊 Found $SERVICES services:"
    kubectl get services -n wasmbed
else
    echo "❌ No services found in wasmbed namespace"
fi
echo ""

# Resource status
echo "📋 Custom Resources:"
DEVICES=$(kubectl get devices -n wasmbed --no-headers 2>/dev/null | wc -l)
APPLICATIONS=$(kubectl get applications -n wasmbed --no-headers 2>/dev/null | wc -l)

echo "📱 Devices: $DEVICES"
if [ "$DEVICES" -gt 0 ]; then
    kubectl get devices -n wasmbed
fi

echo "📦 Applications: $APPLICATIONS"
if [ "$APPLICATIONS" -gt 0 ]; then
    kubectl get applications -n wasmbed
fi
echo ""

# Network policies
echo "🛡️ Network Policies:"
POLICIES=$(kubectl get networkpolicies -n wasmbed --no-headers 2>/dev/null | wc -l)
if [ "$POLICIES" -gt 0 ]; then
    echo "✅ $POLICIES network policies configured:"
    kubectl get networkpolicies -n wasmbed
else
    echo "⚠️ No network policies found"
fi
echo ""

# RBAC status
echo "🔐 RBAC Status:"
ROLES=$(kubectl get clusterrole,clusterrolebinding | grep wasmbed | wc -l)
if [ "$ROLES" -gt 0 ]; then
    echo "✅ RBAC configured:"
    kubectl get clusterrole,clusterrolebinding | grep wasmbed
else
    echo "❌ RBAC not configured"
fi
echo ""

# Health summary
echo "🏥 Health Summary:"
RUNNING_PODS=$(kubectl get pods -n wasmbed --no-headers 2>/dev/null | grep -c "Running" || echo "0")
TOTAL_PODS=$(kubectl get pods -n wasmbed --no-headers 2>/dev/null | wc -l || echo "0")

if [ "$TOTAL_PODS" -gt 0 ]; then
    HEALTH_PERCENT=$((RUNNING_PODS * 100 / TOTAL_PODS))
    echo "📊 Pod Health: $RUNNING_PODS/$TOTAL_PODS running ($HEALTH_PERCENT%)"
    
    if [ "$HEALTH_PERCENT" -eq 100 ]; then
        echo "✅ All pods are healthy"
    elif [ "$HEALTH_PERCENT" -gt 50 ]; then
        echo "⚠️ Some pods are not running"
    else
        echo "❌ Most pods are not running"
    fi
else
    echo "❌ No pods found"
fi
echo ""

# Recommendations
echo "💡 Recommendations:"
if [ "$TOTAL_PODS" -eq 0 ]; then
    echo "  - Run: ./wasmbed.sh deploy"
elif [ "$HEALTH_PERCENT" -lt 100 ]; then
    echo "  - Run: ./wasmbed.sh health-check"
    echo "  - Check logs: ./wasmbed.sh logs"
fi

if [ "$CRDS" -eq 0 ]; then
    echo "  - Run: ./wasmbed.sh deploy"
fi

if [ "$POLICIES" -eq 0 ]; then
    echo "  - Run: ./wasmbed.sh security-hardening"
fi

echo ""
echo "🔍 For detailed information:"
echo "  ./wasmbed.sh logs                     # Show platform logs"
echo "  ./wasmbed.sh health-check             # Run health checks"
echo "  ./wasmbed.sh security-scan             # Security validation"
echo "  ./wasmbed.sh monitor                  # Start monitoring dashboard"

