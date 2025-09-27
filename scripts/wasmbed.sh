#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Wasmbed Platform - Working Management Script
# This script NEVER blocks - uses only non-blocking commands

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    local status=$1
    local message=$2
    case $status in
        "SUCCESS")
            echo -e "${GREEN}✓ $message${NC}"
            ;;
        "ERROR")
            echo -e "${RED}✗ $message${NC}"
            ;;
        "WARNING")
            echo -e "${YELLOW}⚠ $message${NC}"
            ;;
        "INFO")
            echo -e "${BLUE}ℹ $message${NC}"
            ;;
    esac
}

# Function to start all services - NO BLOCKING COMMANDS
start_all_services() {
    print_status "INFO" "Starting all Wasmbed services..."
    
    # Start services using screen sessions (completely non-blocking)
    screen -dmS wasmbed-infrastructure ./target/release/wasmbed-infrastructure --port 30460
    screen -dmS wasmbed-gateway ./target/release/wasmbed-gateway --bind-addr 127.0.0.1:30452 --http-addr 127.0.0.1:30453 --private-key certs/server-key.pem --certificate certs/server-cert.pem --client-ca certs/ca-cert.pem --namespace wasmbed --pod-namespace wasmbed --pod-name gateway-1
    # Dashboard React will be started separately on port 3000
    screen -dmS wasmbed-device-controller ./target/release/wasmbed-device-controller
    screen -dmS wasmbed-application-controller ./target/release/wasmbed-application-controller
    screen -dmS wasmbed-gateway-controller ./target/release/wasmbed-gateway-controller
    screen -dmS wasmbed-gateway-2 ./target/release/wasmbed-gateway --bind-addr 127.0.0.1:30454 --http-addr 127.0.0.1:30455 --private-key certs/server-key.pem --certificate certs/server-cert.pem --client-ca certs/ca-cert.pem --namespace wasmbed --pod-namespace wasmbed --pod-name gateway-2
    screen -dmS wasmbed-gateway-3 ./target/release/wasmbed-gateway --bind-addr 127.0.0.1:30456 --http-addr 127.0.0.1:30457 --private-key certs/server-key.pem --certificate certs/server-cert.pem --client-ca certs/ca-cert.pem --namespace wasmbed --pod-namespace wasmbed --pod-name gateway-3
    
    print_status "SUCCESS" "All services started!"
}

# Function to stop all services - NO BLOCKING COMMANDS
stop_all_services() {
    print_status "INFO" "Stopping all Wasmbed services..."
    
    # Kill screen sessions (non-blocking)
    screen -S wasmbed-infrastructure -X quit 2>/dev/null || true
    screen -S wasmbed-gateway -X quit 2>/dev/null || true
    screen -S wasmbed-device-controller -X quit 2>/dev/null || true
    screen -S wasmbed-application-controller -X quit 2>/dev/null || true
    screen -S wasmbed-gateway-controller -X quit 2>/dev/null || true
    screen -S wasmbed-gateway-2 -X quit 2>/dev/null || true
    screen -S wasmbed-gateway-3 -X quit 2>/dev/null || true
    
    # Also kill any remaining processes
    pkill -f "wasmbed-infrastructure" 2>/dev/null || true
    pkill -f "wasmbed-gateway" 2>/dev/null || true
    pkill -f "wasmbed-dashboard" 2>/dev/null || true
    pkill -f "wasmbed-device-controller" 2>/dev/null || true
    pkill -f "wasmbed-application-controller" 2>/dev/null || true
    pkill -f "wasmbed-gateway-controller" 2>/dev/null || true
    
    print_status "SUCCESS" "All services stopped!"
}

# Function to show service status - NO BLOCKING COMMANDS
show_all_status() {
    print_status "INFO" "=== WASMBED SERVICES STATUS ==="
    
    # Check screen sessions (non-blocking)
    if screen -list | grep -q "wasmbed-infrastructure"; then
        print_status "SUCCESS" "infrastructure is running"
    else
        print_status "ERROR" "infrastructure is not running"
    fi
    
    if screen -list | grep -q "wasmbed-gateway"; then
        print_status "SUCCESS" "gateway is running"
    else
        print_status "ERROR" "gateway is not running"
    fi
    
    # Dashboard React runs separately on port 3000
    
    if screen -list | grep -q "wasmbed-device-controller"; then
        print_status "SUCCESS" "device-controller is running"
    else
        print_status "ERROR" "device-controller is not running"
    fi
    
    if screen -list | grep -q "wasmbed-application-controller"; then
        print_status "SUCCESS" "application-controller is running"
    else
        print_status "ERROR" "application-controller is not running"
    fi
    
    if screen -list | grep -q "wasmbed-gateway-controller"; then
        print_status "SUCCESS" "gateway-controller is running"
    else
        print_status "ERROR" "gateway-controller is not running"
    fi
    
    if screen -list | grep -q "wasmbed-gateway-2"; then
        print_status "SUCCESS" "gateway-2 is running"
    else
        print_status "ERROR" "gateway-2 is not running"
    fi
    
    if screen -list | grep -q "wasmbed-gateway-3"; then
        print_status "SUCCESS" "gateway-3 is running"
    else
        print_status "ERROR" "gateway-3 is not running"
    fi
    
    echo ""
    print_status "INFO" "=== SERVICE ENDPOINTS ==="
    print_status "INFO" "Infrastructure: http://localhost:30460"
    print_status "INFO" "Gateway 1: http://localhost:30453"
    print_status "INFO" "Gateway 2: http://localhost:30455"
    print_status "INFO" "Gateway 3: http://localhost:30457"
    print_status "INFO" "Dashboard React: http://localhost:3000"
}

# Function to test endpoints - NO BLOCKING COMMANDS
test_endpoints() {
    print_status "INFO" "Testing service endpoints..."
    
    for port in 30453 30455 30457; do
        if timeout 1 curl -s http://localhost:$port/api/v1/devices >/dev/null 2>&1; then
            print_status "SUCCESS" "Gateway on port $port is responding"
        else
            print_status "ERROR" "Gateway on port $port is not responding"
        fi
    done
    
    if timeout 1 curl -s http://localhost:3000 >/dev/null 2>&1; then
        print_status "SUCCESS" "Dashboard React is responding"
    else
        print_status "ERROR" "Dashboard React is not responding"
    fi
}

# Function to start React dashboard
start_react_dashboard() {
    print_status "INFO" "Starting React Dashboard..."
    
    if screen -list | grep -q "wasmbed-react-dashboard"; then
        print_status "WARNING" "React Dashboard is already running"
        return 0
    fi
    
    cd dashboard-react
    screen -dmS wasmbed-react-dashboard npm start
    cd ..
    
    print_status "SUCCESS" "React Dashboard started!"
}

# Function to stop React dashboard
stop_react_dashboard() {
    print_status "INFO" "Stopping React Dashboard..."
    
    screen -S wasmbed-react-dashboard -X quit 2>/dev/null || true
    pkill -f "react-scripts" 2>/dev/null || true
    
    print_status "SUCCESS" "React Dashboard stopped!"
}

# Function to deploy Kubernetes resources - NO BLOCKING COMMANDS
deploy_k8s_resources() {
    print_status "INFO" "Deploying Kubernetes resources..."
    
    mkdir -p k8s/gateways k8s/devices logs
    
    kubectl apply -f k8s/gateways/gateway-1.yaml &
    kubectl apply -f k8s/gateways/gateway-2.yaml &
    kubectl apply -f k8s/gateways/gateway-3.yaml &
    kubectl apply -f k8s/devices/mcu-board-1.yaml &
    kubectl apply -f k8s/devices/mcu-board-2.yaml &
    kubectl apply -f k8s/devices/mcu-board-3.yaml &
    kubectl apply -f k8s/devices/riscv-board-1.yaml &
    kubectl apply -f k8s/devices/riscv-board-2.yaml &
    kubectl apply -f k8s/devices/riscv-board-3.yaml &
    
    print_status "SUCCESS" "Kubernetes resources deployment initiated!"
}

# Main command handling
case "${1:-}" in
    "start")
        start_all_services
        ;;
    "stop")
        stop_all_services
        ;;
    "restart")
        stop_all_services
        start_all_services
        ;;
    "dashboard")
        start_react_dashboard
        ;;
    "stop-dashboard")
        stop_react_dashboard
        ;;
    "status")
        show_all_status
        ;;
    "test")
        test_endpoints
        ;;
    "k8s")
        deploy_k8s_resources
        ;;
    *)
        echo "Wasmbed Platform - Working Management Script"
        echo ""
        echo "Usage: $0 [COMMAND]"
        echo ""
        echo "Commands:"
        echo "  start                     Start all services (using screen)"
        echo "  stop                      Stop all services (using screen)"
        echo "  restart                   Restart all services"
        echo "  dashboard                 Start React Dashboard (port 3000)"
        echo "  stop-dashboard            Stop React Dashboard"
        echo "  status                    Show status of all services"
        echo "  test                      Test all service endpoints"
        echo "  k8s                       Deploy Kubernetes resources only"
        echo ""
        echo "Examples:"
        echo "  $0 start                  # Start services only"
        echo "  $0 dashboard              # Start React Dashboard"
        echo "  $0 status                 # Check status"
        echo "  $0 test                   # Test endpoints"
        echo ""
        echo "Note: This script uses 'screen' sessions to avoid blocking"
        ;;
esac