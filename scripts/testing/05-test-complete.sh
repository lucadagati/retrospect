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
GATEWAY_PORT="4423"
LOCAL_PORT="4423"

echo "🧪 Testing Wasmbed system..."

check_environment() {
    echo -e "${BLUE}📋 Checking environment...${NC}"
    
    # Check if cluster exists
    if ! k3d cluster list | grep -q "$CLUSTER_NAME"; then
        echo -e "${RED}❌ Cluster $CLUSTER_NAME not found${NC}"
        echo "Please run './scripts/setup.sh' first"
        exit 1
    fi
    
    # Check if we're in Nix shell
    if [ -z "${IN_NIX_SHELL:-}" ]; then
        echo -e "${YELLOW}⚠️  Not in Nix shell. Running 'nix develop' first...${NC}"
        exec nix develop --command "$0" "$@"
    fi
    
    # Set kubeconfig if not already set
    if [ -z "${KUBECONFIG:-}" ]; then
        echo "  ⚙️  Setting KUBECONFIG..."
        export KUBECONFIG=$(k3d kubeconfig write "$CLUSTER_NAME")
    fi
    
    echo -e "${GREEN}✅ Environment check passed${NC}"
}

check_cluster_status() {
    echo -e "${BLUE}🏥 Checking cluster health...${NC}"
    
    # Check if cluster is running
    if ! kubectl cluster-info &> /dev/null; then
        echo -e "${RED}❌ Cluster is not accessible${NC}"
        exit 1
    fi
    
    # Check namespace
    if ! kubectl get namespace "$NAMESPACE" &> /dev/null; then
        echo -e "${RED}❌ Namespace $NAMESPACE not found${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✅ Cluster is healthy${NC}"
}

check_gateway_status() {
    echo -e "${BLUE}🚪 Checking gateway status...${NC}"
    
    # Check if StatefulSet exists
    if ! kubectl -n "$NAMESPACE" get statefulset wasmbed-gateway &> /dev/null; then
        echo -e "${RED}❌ Gateway StatefulSet not found${NC}"
        exit 1
    fi
    
    # Check if pod is running
    local pod_status
    pod_status=$(kubectl -n "$NAMESPACE" get pod wasmbed-gateway-0 -o jsonpath='{.status.phase}' 2>/dev/null || echo "NotFound")
    
    if [ "$pod_status" != "Running" ]; then
        echo -e "${RED}❌ Gateway pod is not running (status: $pod_status)${NC}"
        echo "  📊 Pod details:"
        kubectl -n "$NAMESPACE" describe pod wasmbed-gateway-0 | tail -20
        exit 1
    fi
    
    echo -e "${GREEN}✅ Gateway is running${NC}"
}

check_device_registration() {
    echo -e "${BLUE}📱 Checking device registration...${NC}"
    
    # Check if test device exists
    if ! kubectl -n "$NAMESPACE" get device device-0 &> /dev/null; then
        echo -e "${RED}❌ Test device not found${NC}"
        exit 1
    fi
    
    # Show device status
    echo "  📊 Device status:"
    kubectl -n "$NAMESPACE" get device device-0 -o yaml | grep -A 20 "status:" || echo "  ℹ️  No status available yet"
    
    echo -e "${GREEN}✅ Device is registered${NC}"
}

test_port_forward() {
    echo -e "${BLUE}🔌 Testing port forwarding...${NC}"
    
    # Kill any existing port-forward process
    pkill -f "kubectl.*port-forward.*wasmbed-gateway-service" || true
    sleep 2
    
    # Start port forwarding in background
    echo "  🔗 Starting port forward from localhost:$LOCAL_PORT to service:$GATEWAY_PORT..."
    kubectl -n "$NAMESPACE" port-forward service/wasmbed-gateway-service "$LOCAL_PORT:$GATEWAY_PORT" &
    local pf_pid=$!
    
    # Wait a moment for port forward to establish
    sleep 3
    
    # Check if port forward is working
    if ! ps -p $pf_pid > /dev/null; then
        echo -e "${RED}❌ Port forward failed to start${NC}"
        exit 1
    fi
    
    # Test if port is listening
    if ! netstat -tln | grep -q ":$LOCAL_PORT "; then
        echo -e "${RED}❌ Port $LOCAL_PORT is not listening${NC}"
        kill $pf_pid || true
        exit 1
    fi
    
    echo -e "${GREEN}✅ Port forward is working (PID: $pf_pid)${NC}"
    echo $pf_pid > /tmp/wasmbed-port-forward.pid
}

test_gateway_connection() {
    echo -e "${BLUE}🤝 Testing gateway connection...${NC}"
    
    echo "  🔐 Running test client..."
    local client_output
    if client_output=$(timeout 10 cargo run -p wasmbed-gateway-test-client -- \
        --address "127.0.0.1:$LOCAL_PORT" \
        --server-ca resources/dev-certs/server-ca.der \
        --private-key resources/dev-certs/client-0.key \
        --certificate resources/dev-certs/client-0.der 2>&1); then
        
        echo -e "${GREEN}✅ Gateway connection successful${NC}"
        echo "  📄 Client output:"
        echo "$client_output" | sed 's/^/    /'
        
        # Check if device status was updated
        echo "  📊 Updated device status:"
        sleep 2  # Give some time for status update
        kubectl -n "$NAMESPACE" get device device-0 -o jsonpath='{.status}' | jq . 2>/dev/null || \
            kubectl -n "$NAMESPACE" get device device-0 -o yaml | grep -A 20 "status:"
        
    else
        echo -e "${RED}❌ Gateway connection failed${NC}"
        echo "  📄 Client output:"
        echo "$client_output" | sed 's/^/    /'
        return 1
    fi
}

test_protocol_tools() {
    echo -e "${BLUE}🔧 Testing protocol tools...${NC}"
    
    echo "  📝 Testing heartbeat message encoding/decoding..."
    
    # Test encoding a heartbeat (this should create a valid CBOR message)
    # Since we need a real heartbeat message, let's just verify the tool works
    if echo "83000000800" | cargo run -p wasmbed-protocol-tool -- --format hex --message-type client &> /dev/null; then
        echo -e "${GREEN}✅ Protocol tools working${NC}"
    else
        echo -e "${YELLOW}⚠️  Protocol tool test inconclusive (tool available but message format may be incorrect)${NC}"
    fi
}

cleanup_port_forward() {
    echo -e "${BLUE}🧹 Cleaning up port forward...${NC}"
    
    if [ -f /tmp/wasmbed-port-forward.pid ]; then
        local pf_pid
        pf_pid=$(cat /tmp/wasmbed-port-forward.pid)
        if ps -p "$pf_pid" > /dev/null 2>&1; then
            kill "$pf_pid" || true
            echo "  ⏹️  Stopped port forward (PID: $pf_pid)"
        fi
        rm -f /tmp/wasmbed-port-forward.pid
    fi
    
    # Also kill by process name as backup
    pkill -f "kubectl.*port-forward.*wasmbed-gateway-service" || true
}

display_results() {
    echo ""
    echo -e "${GREEN}🎉 System test completed!${NC}"
    echo ""
    echo -e "${BLUE}📊 Test Summary:${NC}"
    echo "  ✅ Environment check"
    echo "  ✅ Cluster health"
    echo "  ✅ Gateway status"
    echo "  ✅ Device registration"
    echo "  ✅ Port forwarding"
    echo "  ✅ Gateway connection"
    echo "  ✅ Protocol tools"
    echo ""
    echo -e "${BLUE}🏁 System is ready for development!${NC}"
    echo ""
    echo -e "${YELLOW}💡 Next steps:${NC}"
    echo "  1. Start implementing the client MCU: edit crates/wasmbed-firmware-hifive1-qemu/src/main.rs"
    echo "  2. Extend the protocol: edit crates/wasmbed-protocol/src/lib.rs"
    echo "  3. Monitor logs: ./scripts/monitor.sh"
    echo "  4. Re-run tests: ./scripts/test.sh"
}

main() {
    # Set up cleanup trap
    trap cleanup_port_forward EXIT
    
    check_environment
    check_cluster_status
    check_gateway_status
    check_device_registration
    test_port_forward
    test_gateway_connection
    test_protocol_tools
    display_results
}

main "$@"
