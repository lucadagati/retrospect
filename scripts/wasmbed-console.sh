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
    
    # Create logs directory if it doesn't exist
    mkdir -p logs
    
    # Start services using nohup + disown (completely non-blocking)
    nohup ./target/release/wasmbed-infrastructure --port 30460 > logs/infrastructure.log 2>&1 &
    echo $! > .infrastructure.pid
    disown
    
    nohup ./target/release/wasmbed-gateway --bind-addr 127.0.0.1:30452 --http-addr 127.0.0.1:30453 --private-key certs/server-key.pem --certificate certs/server-cert.pem --client-ca certs/ca-cert.pem --namespace wasmbed --pod-namespace wasmbed --pod-name gateway-1 > logs/gateway.log 2>&1 &
    echo $! > .gateway.pid
    disown
    
    nohup ./target/release/wasmbed-device-controller > logs/device-controller.log 2>&1 &
    echo $! > .device-controller.pid
    disown
    
    nohup ./target/release/wasmbed-application-controller > logs/application-controller.log 2>&1 &
    echo $! > .application-controller.pid
    disown
    
    nohup ./target/release/wasmbed-gateway-controller > logs/gateway-controller.log 2>&1 &
    echo $! > .gateway-controller.pid
    disown
    
    nohup ./target/release/wasmbed-gateway --bind-addr 127.0.0.1:30454 --http-addr 127.0.0.1:30455 --private-key certs/server-key.pem --certificate certs/server-cert.pem --client-ca certs/ca-cert.pem --namespace wasmbed --pod-namespace wasmbed --pod-name gateway-2 > logs/gateway-2.log 2>&1 &
    echo $! > .gateway-2.pid
    disown
    
    nohup ./target/release/wasmbed-gateway --bind-addr 127.0.0.1:30456 --http-addr 127.0.0.1:30457 --private-key certs/server-key.pem --certificate certs/server-cert.pem --client-ca certs/ca-cert.pem --namespace wasmbed --pod-namespace wasmbed --pod-name gateway-3 > logs/gateway-3.log 2>&1 &
    echo $! > .gateway-3.pid
    disown
    
    print_status "SUCCESS" "All services started!"
}

# Function to stop all services - NO BLOCKING COMMANDS
stop_all_services() {
    print_status "INFO" "Stopping all Wasmbed services..."
    
    # Stop services using individual PID files
    services=("infrastructure" "gateway" "device-controller" "application-controller" "gateway-controller" "gateway-2" "gateway-3")
    
    for service in "${services[@]}"; do
        pid_file=".${service}.pid"
        if [ -f "$pid_file" ]; then
            pid=$(cat "$pid_file")
            if kill -0 "$pid" 2>/dev/null; then
                print_status "INFO" "Stopping $service (PID: $pid)"
                kill "$pid" 2>/dev/null || true
            fi
            rm -f "$pid_file"
        fi
    done
    
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
    
    # Check services using PID files (non-blocking)
    services=("infrastructure" "gateway" "device-controller" "application-controller" "gateway-controller" "gateway-2" "gateway-3")
    
    for service in "${services[@]}"; do
        pid_file=".${service}.pid"
        if [ -f "$pid_file" ]; then
            pid=$(cat "$pid_file")
            if kill -0 "$pid" 2>/dev/null; then
                print_status "SUCCESS" "$service is running (PID: $pid)"
            else
                print_status "ERROR" "$service is not running (stale PID file)"
                rm -f "$pid_file"
            fi
        else
            print_status "ERROR" "$service is not running (no PID file)"
        fi
    done
    
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
    
    # Check if already running
    if [ -f ".react-dashboard.pid" ]; then
        pid=$(cat ".react-dashboard.pid")
        if kill -0 "$pid" 2>/dev/null; then
            print_status "WARNING" "React Dashboard is already running (PID: $pid)"
            return 0
        else
            rm -f ".react-dashboard.pid"
        fi
    fi
    
    cd dashboard-react
    nohup npm start > ../logs/react-dashboard.log 2>&1 &
    echo $! > ../.react-dashboard.pid
    disown
    cd ..
    
    print_status "SUCCESS" "React Dashboard started!"
}

# Function to stop React dashboard
stop_react_dashboard() {
    print_status "INFO" "Stopping React Dashboard..."
    
    # Stop using PID file
    if [ -f ".react-dashboard.pid" ]; then
        pid=$(cat ".react-dashboard.pid")
        if kill -0 "$pid" 2>/dev/null; then
            print_status "INFO" "Stopping React Dashboard (PID: $pid)"
            kill "$pid" 2>/dev/null || true
        fi
        rm -f ".react-dashboard.pid"
    fi
    
    # Also kill any remaining processes
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