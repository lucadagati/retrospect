#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Fix Gateway Connection - Ensure port-forward is active for device creation

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() {
    local status=$1
    local message=$2
    case $status in
        "SUCCESS") echo -e "${GREEN}✓${NC} $message" ;;
        "ERROR") echo -e "${RED}✗${NC} $message" ;;
        "WARNING") echo -e "${YELLOW}⚠${NC} $message" ;;
        "INFO") echo -e "${BLUE}ℹ${NC} $message" ;;
    esac
}

print_header() {
    echo ""
    echo "========================================"
    echo "  $1"
    echo "========================================"
}

print_header "GATEWAY CONNECTION FIX"

# Check if gateway service exists
if ! kubectl get svc -n wasmbed gateway-1-service >/dev/null 2>&1; then
    print_status "ERROR" "Gateway service 'gateway-1-service' not found in namespace 'wasmbed'"
    exit 1
fi

# Check if port-forward is already running
if ps aux | grep -q "[k]ubectl port-forward.*gateway.*8080" || lsof -i :8080 >/dev/null 2>&1; then
    print_status "INFO" "Port-forward already active on port 8080"
    
    # Test connection
    if curl -s http://localhost:8080/health >/dev/null 2>&1; then
        print_status "SUCCESS" "Gateway is reachable on http://localhost:8080"
        exit 0
    else
        print_status "WARNING" "Port-forward exists but gateway is not responding"
        print_status "INFO" "Killing existing port-forward..."
        pkill -f "kubectl port-forward.*gateway.*8080" || true
        sleep 1
    fi
fi

# Start port-forward
print_status "INFO" "Starting port-forward for gateway-1-service..."
kubectl port-forward -n wasmbed svc/gateway-1-service 8080:8080 > /tmp/gateway-portforward.log 2>&1 &
PORT_FORWARD_PID=$!

# Wait for port-forward to be ready
sleep 2

# Test connection
if curl -s http://localhost:8080/health >/dev/null 2>&1; then
    print_status "SUCCESS" "Gateway is now reachable on http://localhost:8080"
    print_status "INFO" "Port-forward PID: $PORT_FORWARD_PID"
    print_status "INFO" "Logs: /tmp/gateway-portforward.log"
    echo ""
    print_status "INFO" "To stop port-forward: kill $PORT_FORWARD_PID"
    exit 0
else
    print_status "ERROR" "Failed to connect to gateway after starting port-forward"
    print_status "INFO" "Check logs: /tmp/gateway-portforward.log"
    kill $PORT_FORWARD_PID 2>/dev/null || true
    exit 1
fi

